use crate::consts::{ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET, ANSI_RED};

pub struct Shell {
    pub pwd: String,
    pub buffer: String,
    pub tx: async_channel::Sender<String>,
    pub rx: async_channel::Receiver<String>,
}

impl Default for Shell {
    fn default() -> Self {
        let (tx, rx) = async_channel::unbounded();
        let sh = Self {
            pwd: "/home/inparsian".to_owned(),
            buffer: String::new(),
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
    
    pub fn handle_stdin(&mut self, key: &str) {
        match key {
            "\r" => {
                let _ = self.tx.try_send("\r\n".to_owned());
                let result = self.handle_cmd();
                self.buffer.clear();
                let _ = self.tx.try_send(result);
                let _ = self.tx.try_send(self.prompt());
            },
            "\u{7f}" => {
                self.buffer.pop();
                let _ = self.tx.try_send("\u{0008} \u{0008}".to_owned());
            },
            _ => {
                self.buffer.push_str(key);
                let _ = self.tx.try_send(key.to_owned());
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