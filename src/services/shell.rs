use std::fmt::Write as _;

use super::bins::{self, CommandContext};
use crate::{consts::{ANSI_RED, ANSI_BOLD, ANSI_BRIGHT_RED, ANSI_RESET}, services::fs::{FILESYSTEM, FilesystemData}};

const MAX_COMPLETION_MENU_ITEMS: usize = 6;

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
            "\x1B[A" => { // up
                let out = self.history_up();
                self.emit(out);
            }
            "\x1B[B" => { // down
                let out = self.history_down();
                self.emit(out);
            }
            "\x1B[C" => { // right
                let out = self.move_cursor(1);
                self.emit(out);
            }
            "\x1B[D" => { // left
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
            "\x1B[Z" => { // Shift+Tab
                let out = self.handle_tab_dir(true);
                self.emit(out);
            }
            _ => if key.chars().all(|c| !c.is_control()) {
                self.buffer.insert_str(self.pos, key);
                let output = format!("{}\x1B[K{}", key, &self.buffer[self.pos + key.len()..]);
                let shift = self.buffer.len() - self.pos - key.len();
                self.pos += key.len();
                let out = if shift > 0 { format!("{}\x1B[{}D", output, shift) } else { output };
                self.emit(out);
            },
        }
    }

    fn handle_tab_dir(&mut self, reverse: bool) -> String {
        let cycled = if reverse {
            self.prev_completion_candidate()
        } else {
            self.next_completion_candidate()
        };
    
        if let Some(next) = cycled {
            let mut out = self.replace_current_token(&next);
            out.push_str(&self.render_completion_menu());
            return out;
        }
    
        let mut candidates = self.handle_complete(&self.buffer.clone());
        if candidates.is_empty() {
            self.completion_state = None;
            return self.clear_completion_menu();
        }
    
        candidates.sort_unstable();
        candidates.dedup();
    
        if candidates.len() == 1 {
            self.completion_state = None;
            let mut out = self.replace_current_token(&candidates[0]);
            out.push_str(&self.clear_completion_menu());
            return out;
        }
    
        let (start, _) = self.current_token_bounds();
    
        // start at first for Tab, last for Shift+Tab
        let index = if reverse { candidates.len() - 1 } else { 0 };
    
        self.completion_state = Some(CompletionState {
            start,
            candidates,
            index,
        });
    
        let selected = self
            .completion_state
            .as_ref()
            .unwrap()
            .candidates[index]
            .clone();
    
        let mut out = self.replace_current_token(&selected);
        out.push_str(&self.render_completion_menu());
        out
    }

    fn reset_completion(&mut self) -> String {
        self.completion_state = None;
        self.clear_completion_menu()
    }
    
    fn clear_completion_menu(&mut self) -> String {
        if self.completion_menu_lines == 0 {
            return String::new();
        }
    
        self.completion_menu_lines = 0;
        self.completion_state = None;
    
        let mut out = self.redraw_line();
        out.push_str("\x1B[J"); // clear everything below prompt line
        out.push_str(&self.redraw_line());
        out
    }
    
    fn render_completion_menu(&mut self) -> String {
        let mut out = self.redraw_line();
        out.push_str("\x1B[J"); // clear stale menu region below prompt
    
        let Some(state) = self.completion_state.as_ref() else {
            self.completion_menu_lines = 0;
            return out;
        };
    
        let total = state.candidates.len();
        if total == 0 {
            self.completion_menu_lines = 0;
            return out;
        }
    
        let visible = total.min(MAX_COMPLETION_MENU_ITEMS);
    
        // keep selected item inside visible window (roughly centered)
        let half = visible / 2;
        let mut start = state.index.saturating_sub(half);
        let max_start = total.saturating_sub(visible);
        if start > max_start {
            start = max_start;
        }
        let end = start + visible;
    
        let hidden_above = start;
        let hidden_below = total.saturating_sub(end);
    
        // optional summary line when truncated
        let mut printed_lines = 0_usize;
        if hidden_above > 0 || hidden_below > 0 {
            out.push_str("\r\n\x1B[2K");
            let _ = write!(
                &mut out,
                "  … {} above, {} below",
                hidden_above,
                hidden_below
            );
            printed_lines += 1;
        }
    
        for i in start..end {
            let candidate = &state.candidates[i];
            out.push_str("\r\n\x1B[2K");
            if i == state.index {
                let _ = write!(
                    &mut out,
                    "{ANSI_BOLD}{ANSI_BRIGHT_RED}> {candidate}{ANSI_RESET}"
                );
            } else {
                let _ = write!(&mut out, "  {candidate}");
            }
            printed_lines += 1;
        }
    
        self.completion_menu_lines = printed_lines;
    
        if printed_lines > 0 {
            let _ = write!(&mut out, "\x1B[{}A\r", printed_lines);
        }
    
        out.push_str(&self.redraw_line());
        out
    }
    
    fn next_completion_candidate(&mut self) -> Option<String> {
        let (start, end) = self.current_token_bounds();
        let current_token = self.buffer[start..end].to_owned();
    
        let state = self.completion_state.as_mut()?;
        if state.start != start {
            return None;
        }
    
        if current_token != state.candidates[state.index] {
            return None;
        }
    
        state.index = (state.index + 1) % state.candidates.len();
        Some(state.candidates[state.index].clone())
    }

    fn prev_completion_candidate(&mut self) -> Option<String> {
        let (start, end) = self.current_token_bounds();
        let current_token = self.buffer[start..end].to_owned();
    
        let state = self.completion_state.as_mut()?;
        if state.start != start {
            return None;
        }
    
        if current_token != state.candidates[state.index] {
            return None;
        }
    
        if state.index == 0 {
            state.index = state.candidates.len().saturating_sub(1);
        } else {
            state.index -= 1;
        }
    
        Some(state.candidates[state.index].clone())
    }
    
    fn current_token_bounds(&self) -> (usize, usize) {
        let before = &self.buffer[..self.pos];
        let start = before.rfind(char::is_whitespace).map_or(0, |i| i + 1);
    
        let after = &self.buffer[self.pos..];
        let rel_end = after.find(char::is_whitespace).unwrap_or(after.len());
        let end = self.pos + rel_end;
    
        (start, end)
    }
    
    fn replace_current_token(&mut self, token: &str) -> String {
        let (start, end) = self.current_token_bounds();
        self.buffer.replace_range(start..end, token);
        self.pos = start + token.len();
        self.redraw_line()
    }
    
    fn redraw_line(&self) -> String {
        let mut out = format!("\r{}{}\x1B[K", self.prompt(), self.buffer);
    
        let tail_chars = self.buffer[self.pos..].chars().count();
        if tail_chars > 0 {
            let _ = write!(out, "\x1B[{}D", tail_chars);
        }
    
        out
    }

    pub fn handle_complete(&mut self, buffer: &str) -> Vec<String> {
        let before_cursor = &buffer[..self.pos];
        let ends_ws = before_cursor.chars().last().is_some_and(|c| c.is_whitespace());
    
        let Some(args) = shlex::split(before_cursor) else {
            return Vec::new();
        };
    
        if args.is_empty() {
            return command_candidates("");
        }
    
        if args.len() == 1 && !ends_ws {
            return command_candidates(&args[0]);
        }
    
        let command = &args[0];
        let params = &args[1..];
        
        // builtin completion: cd (directories only)
        if command == "cd" {
            let ctx = CommandContext {
                pwd: &mut self.pwd
            };
            return bins::complete_path(&ctx, params, 0, true);
        }
        
        if let Some(bin) = bins::find(command) && bin_exists(bin.name()) {
            let mut ctx = CommandContext {
                pwd: &mut self.pwd
            };
            let mut cands = bin.complete(&mut ctx, params, self.pos);
            cands.sort_unstable();
            cands.dedup();
            return cands;
        }
    
        if let Some(bin) = bins::find(command) && bin_exists(bin.name()) {
            let mut ctx = CommandContext { pwd: &mut self.pwd };
            let mut cands = bin.complete(&mut ctx, params, self.pos);
            cands.sort_unstable();
            cands.dedup();
            return cands;
        }
    
        Vec::new()
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
        
        if command.as_str() == "cd" {
            if params.is_empty() {
                self.pwd = "/home/inparsian".to_owned();
                return String::new();
            }

            let path = params[0].clone();
            let reader = FILESYSTEM.read().unwrap();
            if let Some(query) = reader.resolve_read(&path, Some(&self.pwd)) {
                match &query.data {
                    FilesystemData::Directory { .. } => {
                        self.pwd = reader.resolve_path(&path, Some(&self.pwd)).unwrap();
                        String::new()
                    },
                    FilesystemData::File { .. } |
                    FilesystemData::SymbolicLink { .. } => {
                        format!("cd: {}: Not a directory\r\n", path)
                    },
                }
            } else {
                format!("cd: {}: No such file or directory\r\n", path)
            }
        } else {
            // see if it's in our static bins, then find it in the fs to see if it can be run
            if let Some(bin) = bins::find(command) && bin_exists(bin.name()) {
                let mut ctx = CommandContext {
                    pwd: &mut self.pwd,
                };

                return bin.run(&mut ctx, params);
            }

            let mut unknown = "sheesh: Unknown command: ".to_owned();
            unknown.push_str(command);
            unknown.push_str("\r\n");
            unknown
        }
    }
}

fn command_candidates(partial: &str) -> Vec<String> {
    let mut out = Vec::new();

    // builtins
    if "cd".starts_with(partial) {
        out.push("cd".to_owned());
    }

    // external bins that actually exist in FS
    for bin in bins::bins() {
        let exists = bin_exists(bin.name());
        if !exists {
            continue;
        }

        if bin.name().starts_with(partial) {
            out.push(bin.name().to_owned());
        }

        for alias in bin.aliases() {
            if alias.starts_with(partial) {
                out.push((*alias).to_owned());
            }
        }
    }

    out.sort_unstable();
    out.dedup();
    out
}

fn bin_exists(name: &str) -> bool {
    let bin_candidates = [
        format!("/home/inparsian/.local/bin/{name}"),
        format!("/usr/local/bin/{name}"),
        format!("/usr/bin/{name}"),
        format!("/bin/{name}"),
    ];

    let reader = FILESYSTEM.read().unwrap();
    bin_candidates.iter().any(|c| reader.resolve_read(c, None).is_some())
}