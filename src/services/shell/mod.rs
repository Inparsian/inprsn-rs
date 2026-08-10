use std::fmt::Write as _;

use crate::consts::{ANSI_RED, ANSI_RESET};

use self::{completion as completion_logic, execution as execution_logic};

mod completion;
mod execution;
mod parser;
mod redirect;

struct CompletionState {
    start: usize,
    candidates: Vec<String>,
    index: usize,
}

pub struct Shell {
    pub pwd: String,
    pub buffer: String,
    pub pos: usize,
    completion_state: Option<CompletionState>,
    completion_menu_lines: usize,
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
            completion_state: None,
            completion_menu_lines: 0,
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
            let out = if delta > 0 { "\x1B[C" } else { "\x1B[D" };
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

    fn emit(&self, out: String) {
        if !out.is_empty() {
            let _ = self.tx.try_send(out);
        }
    }

    pub fn handle_stdin(&mut self, key: &str) {
        if key != "\t" && key != "\x1B[Z" {
            let out = self.reset_completion();
            self.emit(out);
        }

        match key {
            "\x1B[A" => {
                // up
                let out = self.history_up();
                self.emit(out);
            }
            "\x1B[B" => {
                // down
                let out = self.history_down();
                self.emit(out);
            }
            "\x1B[C" => {
                // right
                let out = self.move_cursor(1);
                self.emit(out);
            }
            "\x1B[D" => {
                // left
                let out = self.move_cursor(-1);
                self.emit(out);
            }

            "\r" => {
                let cmd = self.buffer.clone();
                if !cmd.is_empty() {
                    self.history.push(cmd.clone());
                }

                self.history_pos = None;
                self.buffer.clear();
                self.pos = 0;

                self.emit("\r\n".to_owned());
                let out = self.handle_cmd(&cmd);
                self.emit(out);
                self.emit(self.prompt());
            }

            "\u{7f}" => {
                let out = self.backspace();
                self.emit(out);
            }
            "\t" => {
                let out = self.handle_tab_dir(false);
                self.emit(out);
            }
            "\x1B[Z" => {
                // Shift+Tab
                let out = self.handle_tab_dir(true);
                self.emit(out);
            }
            _ if key.chars().all(|c| !c.is_control()) => {
                self.buffer.insert_str(self.pos, key);
                let output = format!("{}\x1B[K{}", key, &self.buffer[self.pos + key.len()..]);
                let shift = self.buffer.len() - self.pos - key.len();
                self.pos += key.len();
                let out = if shift > 0 {
                    format!("{}\x1B[{}D", output, shift)
                } else {
                    output
                };
                self.emit(out);
            }
            _ => {}
        }
    }

    fn redraw_line(&self) -> String {
        let mut out = format!("\r{}{}\x1B[K", self.prompt(), self.buffer);

        let tail_chars = self.buffer[self.pos..].chars().count();
        if tail_chars > 0 {
            let _ = write!(out, "\x1B[{}D", tail_chars);
        }

        out
    }

    fn handle_tab_dir(&mut self, reverse: bool) -> String {
        completion_logic::handle_tab_dir(self, reverse)
    }

    fn reset_completion(&mut self) -> String {
        completion_logic::reset_completion(self)
    }

    pub fn handle_complete(&mut self, buffer: &str) -> Vec<String> {
        completion_logic::handle_complete(self, buffer)
    }

    pub fn handle_cmd(&mut self, cmd: &str) -> String {
        execution_logic::handle_cmd(self, cmd)
    }
}
