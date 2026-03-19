use crate::consts::{ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET};

pub struct Shell {
    pub pwd: String,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            pwd: "/home/inparsian".to_owned(),
        }
    }
}

impl Shell {
    pub fn pwd(&self) -> &str {
        if self.pwd == "/home/inparsian" {
            "~"
        } else {
            &self.pwd
        }
    }
    
    pub fn handle_input(&mut self, input: &str) -> String {
        let Some(command) = input.split_whitespace().next() else {
            return String::new();
        };
        
        let input = input.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
        
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