use std::fmt::Write as _;

use super::bins::{self, CommandContext, CommandResult};
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
        let start = before
            .rfind(|c: char| c.is_whitespace() || matches!(c, '|' | '&' | ';' | '<' | '>'))
            .map_or(0, |i| i + 1);

        let after = &self.buffer[self.pos..];
        let rel_end = after
            .find(|c: char| c.is_whitespace() || matches!(c, '|' | '&' | ';' | '<' | '>'))
            .unwrap_or(after.len());
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
        let segment = completion_segment(before_cursor);

        let Ok(tokens) = tokenize_shell(&segment) else {
            return Vec::new();
        };

        let mut command_words: Vec<String> = Vec::new();
        let mut expecting_redir_target = false;
        let mut last_redir_target: Option<String> = None;
        let mut last_was_redir_target = false;

        for token in &tokens {
            match token {
                ShellToken::Word(word) => {
                    if expecting_redir_target {
                        last_redir_target = Some(word.clone());
                        expecting_redir_target = false;
                        last_was_redir_target = true;
                    } else {
                        command_words.push(word.clone());
                        last_was_redir_target = false;
                    }
                }
                ShellToken::Op(ShellOp::RedirectIn | ShellOp::RedirectOut | ShellOp::RedirectAppend | ShellOp::RedirectErrOut | ShellOp::RedirectErrAppend)
                => {
                    expecting_redir_target = true;
                    last_redir_target = Some(String::new());
                    last_was_redir_target = false;
                }
                ShellToken::Op(ShellOp::Seq | ShellOp::AndIf | ShellOp::OrIf | ShellOp::Pipe) => {
                    command_words.clear();
                    expecting_redir_target = false;
                    last_redir_target = None;
                    last_was_redir_target = false;
                }
            }
        }

        if expecting_redir_target || (last_was_redir_target && !ends_ws) {
            let partial = last_redir_target.unwrap_or_default();
            let ctx = CommandContext { pwd: &mut self.pwd };
            return bins::complete_path(&ctx, &[partial], 0, false);
        }

        if command_words.is_empty() {
            return command_candidates("");
        }

        if command_words.len() == 1 && !ends_ws {
            return command_candidates(&command_words[0]);
        }

        let command = &command_words[0];
        let params = &command_words[1..];

        // builtin completion: cd (directories only)
        if command == "cd" {
            let ctx = CommandContext { pwd: &mut self.pwd };
            return bins::complete_path(&ctx, params, 0, true);
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
        self.handle_cmd_result(cmd).render()
    }

    fn handle_cmd_result(&mut self, cmd: &str) -> CommandResult {
        let tokens = match tokenize_shell(cmd) {
            Ok(tokens) => tokens,
            Err(err) => return CommandResult::err(format!("sheesh: {}\r\n", err)),
        };

        if tokens.is_empty() {
            return CommandResult::ok(String::new());
        }

        let chains = match parse_command_chains(&tokens) {
            Ok(chains) => chains,
            Err(err) => return CommandResult::err(format!("sheesh: {}\r\n", err)),
        };

        let mut aggregate = String::new();
        let mut last_status = 0;

        for (idx, (maybe_cond, pipeline)) in chains.into_iter().enumerate() {
            let should_run = if idx == 0 {
                true
            } else {
                match maybe_cond.unwrap_or(ChainCondition::Always) {
                    ChainCondition::Always => true,
                    ChainCondition::AndIf => last_status == 0,
                    ChainCondition::OrIf => last_status != 0,
                }
            };

            if !should_run {
                continue;
            }

            let result = self.execute_pipeline(pipeline);
            last_status = result.status;
            aggregate.push_str(&result.render());
        }

        CommandResult {
            output: aggregate,
            error: String::new(),
            status: last_status,
        }
    }

    fn execute_pipeline(&mut self, pipeline: Pipeline) -> CommandResult {
        let mut piped_input: Option<String> = None;
        let mut last = CommandResult::ok(String::new());

        let last_index = pipeline.commands.len().saturating_sub(1);
        for (i, cmd) in pipeline.commands.into_iter().enumerate() {
            let result = self.execute_simple_command(cmd, piped_input.as_deref());
            if i < last_index {
                piped_input = Some(result.output.clone());
            }
            last = result;
        }

        last
    }

    fn execute_simple_command(&mut self, cmd: SimpleCommand, piped_input: Option<&str>) -> CommandResult {
        let stdin_input = if let Some(path) = cmd.stdin.as_ref() {
            let reader = FILESYSTEM.read().unwrap();
            match reader.read_file(path, Some(&self.pwd)) {
                Ok(content) => Some(String::from_utf8_lossy(content).into_owned()),
                Err(err) => return CommandResult::err(format!("{}\r\n", err)),
            }
        } else {
            piped_input.map(str::to_owned)
        };

        if let Some((path, append)) = cmd.stdout.as_ref()
            && let Err(err) = prepare_redirect(path, *append, Some(&self.pwd))
        {
            return CommandResult::err(format!("{}\r\n", err));
        }

        if let Some((path, append)) = cmd.stderr.as_ref()
            && let Err(err) = prepare_redirect(path, *append, Some(&self.pwd))
        {
            return CommandResult::err(format!("{}\r\n", err));
        }

        if cmd.argv.is_empty() {
            return CommandResult::err("syntax error near unexpected token\r\n");
        }

        let command = &cmd.argv[0];
        let params = &cmd.argv[1..];

        let mut result = if command == "cd" {
            self.run_builtin_cd(params)
        } else if let Some(bin) = bins::find(command) {
            if bin_exists(bin.name()) {
                let mut ctx = CommandContext { pwd: &mut self.pwd };
                bin.run(&mut ctx, params, stdin_input.as_deref())
            } else {
                CommandResult {
                    output: String::new(),
                    error: format!("sheesh: Unknown command: {}\r\n", command),
                    status: 127,
                }
            }
        } else {
            CommandResult {
                output: String::new(),
                error: format!("sheesh: Unknown command: {}\r\n", command),
                status: 127,
            }
        };

        if let Some((path, append)) = cmd.stdout {
            if let Err(err) = write_redirect(&path, append, &result.output, Some(&self.pwd)) {
                let _ = write!(result.error, "{}\r\n", err);
                if result.status == 0 {
                    result.status = 1;
                }
            } else {
                result.output.clear();
            }
        }

        if let Some((path, append)) = cmd.stderr {
            if let Err(err) = write_redirect(&path, append, &result.error, Some(&self.pwd)) {
                let _ = write!(result.error, "{}\r\n", err);
                if result.status == 0 {
                    result.status = 1;
                }
            } else {
                result.error.clear();
            }
        }

        result
    }

    fn run_builtin_cd(&mut self, params: &[String]) -> CommandResult {
        if params.is_empty() {
            self.pwd = "/home/inparsian".to_owned();
            return CommandResult::ok(String::new());
        }

        let path = params[0].clone();
        let reader = FILESYSTEM.read().unwrap();
        if let Some(query) = reader.resolve_read(&path, Some(&self.pwd)) {
            match &query.data {
                FilesystemData::Directory { .. } => {
                    self.pwd = reader.resolve_path(&path, Some(&self.pwd)).unwrap();
                    CommandResult::ok(String::new())
                },
                FilesystemData::File { .. } | FilesystemData::SymbolicLink { .. } => {
                    CommandResult::err(format!("cd: {}: Not a directory\r\n", path))
                },
            }
        } else {
            CommandResult::err(format!("cd: {}: No such file or directory\r\n", path))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShellOp {
    AndIf,
    OrIf,
    Seq,
    Pipe,
    RedirectIn,
    RedirectOut,
    RedirectAppend,
    RedirectErrOut,
    RedirectErrAppend,
}

#[derive(Clone, Debug)]
enum ShellToken {
    Word(String),
    Op(ShellOp),
}

#[derive(Clone, Copy, Debug)]
enum ChainCondition {
    Always,
    AndIf,
    OrIf,
}

#[derive(Clone, Debug)]
struct SimpleCommand {
    argv: Vec<String>,
    stdin: Option<String>,
    stdout: Option<(String, bool)>, // (path, append)
    stderr: Option<(String, bool)>, // (path, append)
}

#[derive(Clone, Debug)]
struct Pipeline {
    commands: Vec<SimpleCommand>,
}

fn completion_segment(before_cursor: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let mut escape = false;
    let mut last_start = 0_usize;
    let chars: Vec<char> = before_cursor.chars().collect();
    let mut i = 0_usize;

    while i < chars.len() {
        let ch = chars[i];

        if escape {
            escape = false;
            i += 1;
            continue;
        }

        if in_single {
            if ch == '\'' {
                in_single = false;
            }
            i += 1;
            continue;
        }

        if in_double {
            match ch {
                '\\' => escape = true,
                '"' => in_double = false,
                _ => {}
            }
            i += 1;
            continue;
        }

        match ch {
            '\\' => escape = true,
            '\'' => in_single = true,
            '"' => in_double = true,
            '|' if i + 1 < chars.len() && chars[i + 1] == '|' => {
                last_start = i + 2;
                i += 1;
            }
            '|' | ';' => last_start = i + 1,
            '&' if i + 1 < chars.len() && chars[i + 1] == '&' => {
                last_start = i + 2;
                i += 1;
            }
            _ => {}
        }

        i += 1;
    }

    before_cursor[last_start..].to_owned()
}

fn tokenize_shell(input: &str) -> Result<Vec<ShellToken>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0_usize;

    while i < chars.len() {
        let ch = chars[i];

        if ch.is_whitespace() {
            i += 1;
            continue;
        }

        if i + 2 < chars.len() && chars[i] == '2' && chars[i + 1] == '>' && chars[i + 2] == '>' {
            tokens.push(ShellToken::Op(ShellOp::RedirectErrAppend));
            i += 3;
            continue;
        }

        if i + 1 < chars.len() {
            match (chars[i], chars[i + 1]) {
                ('&', '&') => {
                    tokens.push(ShellToken::Op(ShellOp::AndIf));
                    i += 2;
                    continue;
                }
                ('|', '|') => {
                    tokens.push(ShellToken::Op(ShellOp::OrIf));
                    i += 2;
                    continue;
                }
                ('>', '>') => {
                    tokens.push(ShellToken::Op(ShellOp::RedirectAppend));
                    i += 2;
                    continue;
                }
                ('2', '>') => {
                    tokens.push(ShellToken::Op(ShellOp::RedirectErrOut));
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }

        match ch {
            ';' => {
                tokens.push(ShellToken::Op(ShellOp::Seq));
                i += 1;
            }
            '|' => {
                tokens.push(ShellToken::Op(ShellOp::Pipe));
                i += 1;
            }
            '<' => {
                tokens.push(ShellToken::Op(ShellOp::RedirectIn));
                i += 1;
            }
            '>' => {
                tokens.push(ShellToken::Op(ShellOp::RedirectOut));
                i += 1;
            }
            '&' => return Err("syntax error near unexpected token '&'".to_owned()),
            _ => {
                let mut word = String::new();
                let mut in_single = false;
                let mut in_double = false;
                let mut escape = false;

                while i < chars.len() {
                    let c = chars[i];

                    if escape {
                        word.push(c);
                        escape = false;
                        i += 1;
                        continue;
                    }

                    if in_single {
                        if c == '\'' {
                            in_single = false;
                        } else {
                            word.push(c);
                        }
                        i += 1;
                        continue;
                    }

                    if in_double {
                        match c {
                            '\\' => escape = true,
                            '"' => in_double = false,
                            _ => word.push(c),
                        }
                        i += 1;
                        continue;
                    }

                    match c {
                        '\\' => {
                            escape = true;
                            i += 1;
                        }
                        '\'' => {
                            in_single = true;
                            i += 1;
                        }
                        '"' => {
                            in_double = true;
                            i += 1;
                        }
                        delim if delim.is_whitespace() || matches!(delim, ';' | '|' | '<' | '>' | '&') => {
                            break;
                        }
                        _ => {
                            word.push(c);
                            i += 1;
                        }
                    }
                }

                if escape || in_single || in_double {
                    return Err("Invalid quotes or escape sequence".to_owned());
                }

                if !word.is_empty() {
                    tokens.push(ShellToken::Word(word));
                }
            }
        }
    }

    Ok(tokens)
}

fn parse_command_chains(tokens: &[ShellToken]) -> Result<Vec<(Option<ChainCondition>, Pipeline)>, String> {
    let mut i = 0_usize;
    let mut out = Vec::new();
    let mut pending_cond: Option<ChainCondition> = None;

    while i < tokens.len() {
        let (pipeline, next_i) = parse_pipeline(tokens, i)?;
        out.push((pending_cond, pipeline));
        pending_cond = None;
        i = next_i;

        if i >= tokens.len() {
            break;
        }

        match tokens.get(i) {
            Some(ShellToken::Op(ShellOp::Seq)) => {
                pending_cond = Some(ChainCondition::Always);
                i += 1;
            }
            Some(ShellToken::Op(ShellOp::AndIf)) => {
                pending_cond = Some(ChainCondition::AndIf);
                i += 1;
            }
            Some(ShellToken::Op(ShellOp::OrIf)) => {
                pending_cond = Some(ChainCondition::OrIf);
                i += 1;
            }
            Some(ShellToken::Op(ShellOp::Pipe)) => {
                return Err("syntax error near unexpected token '|'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectIn)) => {
                return Err("syntax error near unexpected token '<'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectOut)) => {
                return Err("syntax error near unexpected token '>'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectAppend)) => {
                return Err("syntax error near unexpected token '>>'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectErrOut)) => {
                return Err("syntax error near unexpected token '2>'".to_owned());
            }
            Some(ShellToken::Op(ShellOp::RedirectErrAppend)) => {
                return Err("syntax error near unexpected token '2>>'".to_owned());
            }
            Some(ShellToken::Word(w)) => {
                return Err(format!("syntax error near unexpected token '{}'", w));
            }
            None => break,
        }
    }

    if pending_cond.is_some() {
        return Err("syntax error near unexpected token 'newline'".to_owned());
    }

    Ok(out)
}

fn parse_pipeline(tokens: &[ShellToken], mut i: usize) -> Result<(Pipeline, usize), String> {
    let mut commands = Vec::new();

    loop {
        let (cmd, next_i) = parse_simple_command(tokens, i)?;
        commands.push(cmd);
        i = next_i;

        match tokens.get(i) {
            Some(ShellToken::Op(ShellOp::Pipe)) => {
                i += 1;
            }
            _ => break,
        }
    }

    if commands.is_empty() {
        return Err("syntax error near unexpected token '|'".to_owned());
    }

    Ok((Pipeline { commands }, i))
}

fn parse_simple_command(tokens: &[ShellToken], mut i: usize) -> Result<(SimpleCommand, usize), String> {
    let mut argv = Vec::new();
    let mut stdin = None;
    let mut stdout = None;
    let mut stderr = None;

    while let Some(token) = tokens.get(i) {
        match token {
            ShellToken::Word(w) => {
                argv.push(w.clone());
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectIn) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '<'".to_owned());
                };
                stdin = Some(path.clone());
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectOut) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '>'".to_owned());
                };
                stdout = Some((path.clone(), false));
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectAppend) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '>>'".to_owned());
                };
                stdout = Some((path.clone(), true));
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectErrOut) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '2>'".to_owned());
                };
                stderr = Some((path.clone(), false));
                i += 1;
            }
            ShellToken::Op(ShellOp::RedirectErrAppend) => {
                i += 1;
                let Some(ShellToken::Word(path)) = tokens.get(i) else {
                    return Err("syntax error near unexpected token '2>>'".to_owned());
                };
                stderr = Some((path.clone(), true));
                i += 1;
            }
            ShellToken::Op(ShellOp::Pipe | ShellOp::AndIf | ShellOp::OrIf | ShellOp::Seq) => break,
        }
    }

    if argv.is_empty() {
        return Err("syntax error near unexpected token".to_owned());
    }

    Ok((SimpleCommand { argv, stdin, stdout, stderr }, i))
}

fn prepare_redirect(path: &str, append: bool, pwd: Option<&str>) -> Result<(), String> {
    let mut fs = FILESYSTEM.write().unwrap();

    if let Some(entry) = fs.resolve_read(path, pwd) {
        return match &entry.data {
            FilesystemData::File { .. } => {
                if append {
                    Ok(())
                } else {
                    // `>` truncates before command execution.
                    fs.write_file(path, pwd, b"")
                }
            }
            _ => Err(format!("Path is not a file: {}", path)),
        };
    }

    // Target doesn't exist: create it as an empty file before command execution.
    fs.create_file(path, pwd, b"")
}

fn write_redirect(path: &str, append: bool, data: &str, pwd: Option<&str>) -> Result<(), String> {
    let mut fs = FILESYSTEM.write().unwrap();

    if append {
        if let Some(entry) = fs.resolve_read(path, pwd) {
            match &entry.data {
                FilesystemData::File { .. } => {
                    let mut existing = fs.read_file(path, pwd)?.to_vec();
                    existing.extend_from_slice(data.as_bytes());
                    fs.write_file(path, pwd, &existing)
                }
                _ => Err(format!("Path is not a file: {}", path)),
            }
        } else {
            fs.create_file(path, pwd, data.as_bytes())
        }
    } else if fs.resolve_read(path, pwd).is_some() {
        fs.write_file(path, pwd, data.as_bytes())
    } else {
        fs.create_file(path, pwd, data.as_bytes())
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