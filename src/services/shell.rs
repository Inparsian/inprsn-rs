use std::fmt::Write as _;
use dioxus::signals::ReadableExt as _;

use crate::{consts::{ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RED, ANSI_RESET}, sys};

pub struct Shell {
    pub pwd: String,
    pub buffer: String,
    pub pos: usize,
    // This is temporary until I establish a persistent VFS with a .sheesh_history file
    pub history: Vec<String>,
    pub history_pos: Option<usize>,
    // Stores the current line whilst browsing history
    pub temp_buffer: Option<String>,
    pub tx: async_channel::Sender<String>,
    pub rx: async_channel::Receiver<String>,
}

impl Default for Shell {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded();
        let sh = Self {
            pwd: "/home/inparsian".to_owned(),
            buffer: String::new(),
            history: Vec::new(),
            history_pos: None,
            temp_buffer: None,
            pos: 0,
            tx,
            rx,
        };
        
        let _ = sh.tx.try_send(sh.prompt());
        sh
    }
}

impl Shell {
    fn prompt(&self) -> String {
        let path = if self.pwd == "/home/inparsian" {
            "~"
        } else {
            &self.pwd
        };
        
        format!("{path} {ANSI_RED}›{ANSI_RESET} ")
    }
    
    fn move_cursor(&mut self, delta: i32) -> String {
        let new_pos = (self.pos as i32 + delta).clamp(0, self.buffer.len() as i32) as usize;
        if new_pos != self.pos {
            let out = if delta > 0 {
                "\x1B[C"
            } else {
                "\x1B[D"
            };
            self.pos = new_pos;
            out.to_owned()
        } else {
            String::new()
        }
    }
    
    fn backspace(&mut self) -> String {
        if self.pos == 0 {
            return String::new();
        }
    
        let prev_char_idx = self.buffer[..self.pos]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i);
    
        self.buffer.remove(prev_char_idx);
        self.pos = prev_char_idx;
    
        let rest = &self.buffer[self.pos..];
        let rest_char_count = rest.chars().count();
    
        if rest.is_empty() {
            "\x1B[D\x1B[K".to_owned()
        } else {
            format!("\x1B[D\x1B[K{}\x1B[{}D", rest, rest_char_count)
        }
    }
    
    fn history_up(&mut self) -> String {
       if self.history.is_empty() {
           return String::new();
       }

       if let Some(idx) = self.history_pos && idx > 0 {
           self.history_pos = Some(idx - 1);
       } else {
           self.temp_buffer = Some(self.buffer.clone());
           self.history_pos = Some(self.history.len() - 1);
       }

       self.update_line_from_history()
    }
    
    fn history_down(&mut self) -> String {
        let Some(idx) = self.history_pos else {
            return String::new();
        };
        
        if idx + 1 >= self.history.len() {
            self.history_pos = None;
            self.buffer = self.temp_buffer.clone().unwrap_or_default();
            self.temp_buffer = None;
        } else {
            self.history_pos = Some(idx + 1);
        }
    
        self.update_line_from_history()
    }
    
    fn update_line_from_history(&mut self) -> String {
        if let Some(idx) = self.history_pos {
            self.buffer = self.history[idx].clone();
        }
        self.pos = self.buffer.len();
        format!("\r{}{}\x1B[K{}", self.prompt(), self.buffer, "")
    }
    
    pub fn handle_stdin(&mut self, key: &str) {
        match key {
            "\x1B[A" => { // up
                let res = self.history_up();
                let _ = self.tx.try_send(res);
            },
            "\x1B[B" => { // down
                let res = self.history_down();
                let _ = self.tx.try_send(res);
            },
            "\x1B[C" => { // right
                let res = self.move_cursor(1);
                let _ = self.tx.try_send(res);
            },
            "\x1B[D" => { // left
                let res = self.move_cursor(-1);
                let _ = self.tx.try_send(res);
            },
            "\r" => {
                let cmd = self.buffer.clone();
                if !cmd.is_empty() {
                    self.history.push(cmd.clone());
                }
                self.history_pos = None;
                self.buffer.clear();
                self.pos = 0;
                let _ = self.tx.try_send("\r\n".to_owned());
                let result = self.handle_cmd(&cmd);
                let _ = self.tx.try_send(result);
                let _ = self.tx.try_send(self.prompt());
            },
            "\u{7f}" => {
                let res = self.backspace();
                let _ = self.tx.try_send(res);
            },
            _ => if key.chars().all(|c| !c.is_control()) {
                self.buffer.insert_str(self.pos, key);
                let output = format!("{}\x1B[K{}", key, &self.buffer[self.pos + key.len()..]);
                let shift = self.buffer.len() - self.pos - key.len();
                self.pos += key.len();
                let out = if shift > 0 { format!("{}\x1B[{}D", output, shift) } else { output };
                let _ = self.tx.try_send(out);
            },
        }
    }
    
    pub fn handle_cmd(&mut self, cmd: &str) -> String {
        let Some(args) = shlex::split(cmd) else {
            return "sheesh: Invalid quotes or escape sequence\r\n".to_owned();
        };
        
        if args.is_empty() {
            return String::new();
        }
        
        let command = &args[0];
        let params = &args[1..];
        
        match command.as_str() {
            "echo" => format!("{}\r\n", params.join(" ")),
            "clear" => format!("{}{}{}", ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET),
            "pwd" => format!("{}\r\n", self.pwd),
            "whoami" => "inparsian\r\n".to_owned(),
            "neofetch" | "fastfetch" => "ok\r\n".to_owned(),
            "kill" => if params.is_empty() {
                "kill: not enough arguments\r\n".to_owned()
            } else {
                // kill usually can take a exit signal code as an argument,
                // however our process manager can only kill processes (sig 9) at the moment.
                // as such we'll only take pids
                params[0].parse::<u32>().map_or_else(
                    |_| format!("kill: cannot find process \"{}\"\r\n", params[0]),
                    |pid| if sys::has_pid(pid) {
                        sys::kill_process(pid);
                        String::new()
                    } else {
                        format!("kill: sending signal to {} failed: No such process\r\n", pid)
                    }
                )
            },
            "ps" => {
                // simple ahh implementation until these commands are put into their
                // own separate modules
                let mut out = format!("{:<5} {:<10}\r\n", "PID", "COMMAND");
                for proc in sys::PROCESSES.read().iter() {
                    let _ = write!(&mut out, "{:<5} {:<10}\r\n", proc.id, proc.name);
                }
                out
            },
            _ => {
                let mut unknown = "sheesh: Unknown command: ".to_owned();
                unknown.push_str(command);
                unknown.push_str("\r\n");
                unknown
            },
        }
    }
}