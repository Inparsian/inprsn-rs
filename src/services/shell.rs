use crate::consts::{ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET, ANSI_RED};

pub struct Shell {
    pub pwd: String,
    pub buffer: String,
    pub pos: usize,
    pub tx: async_channel::Sender<String>,
    pub rx: async_channel::Receiver<String>,
}

impl Default for Shell {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded();
        let sh = Self {
            pwd: "/home/inparsian".to_owned(),
            buffer: String::new(),
            pos: 0,
            tx,
            rx,
        };
        
        let _ = sh.tx.try_send(sh.prompt());
        sh
    }
}

impl Shell {
    pub fn prompt(&self) -> String {
        let path = if self.pwd == "/home/inparsian" {
            "~"
        } else {
            &self.pwd
        };
        
        format!("{path} {ANSI_RED}›{ANSI_RESET} ")
    }
    
    pub fn move_cursor(&mut self, delta: i32) -> String {
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
    
    pub fn handle_stdin(&mut self, key: &str) {
        match key {
            "\x1B[A" | "\x1B[B" => {}, // up | down
            "\x1B[C" => { // right
                let res = self.move_cursor(1);
                let _ = self.tx.try_send(res);
            },
            "\x1B[D" => { // left
                let res = self.move_cursor(-1);
                let _ = self.tx.try_send(res);
            },
            "\r" => {
                let _ = self.tx.try_send("\r\n".to_owned());
                let result = self.handle_cmd();
                self.buffer.clear();
                self.pos = 0;
                let _ = self.tx.try_send(result);
                let _ = self.tx.try_send(self.prompt());
            },
            "\u{7f}" => {
                self.buffer.pop();
                let _ = self.tx.try_send("\u{0008} \u{0008}".to_owned());
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
    
    pub fn handle_cmd(&mut self) -> String {
        let Some(command) = self.buffer.split_whitespace().next() else {
            return String::new();
        };
        
        let input = self.buffer.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        
        match command {
            "echo" => format!("{}\r\n", input),
            "clear" => format!("{}{}{}", ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET),
            "pwd" => format!("{}\r\n", self.pwd),
            "whoami" => "inparsian\r\n".to_owned(),
            "neofetch" | "fastfetch" => "ok\r\n".to_owned(),
            _ => {
                let mut unknown = "sheesh: Unknown command: ".to_owned();
                unknown.push_str(command);
                unknown.push_str("\r\n");
                unknown
            },
        }
    }
}