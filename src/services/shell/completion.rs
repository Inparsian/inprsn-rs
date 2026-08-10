use std::fmt::Write as _;

use super::{
    parser::{completion_segment, tokenize_shell, ShellOp, ShellToken},
    CompletionState, Shell,
};
use crate::{
    consts::{ANSI_BOLD, ANSI_BRIGHT_RED, ANSI_RESET},
    services::{
        bins::{self, CommandContext},
        fs::FILESYSTEM,
    },
};

const MAX_COMPLETION_MENU_ITEMS: usize = 6;

pub(super) fn handle_tab_dir(shell: &mut Shell, reverse: bool) -> String {
    let cycled = if reverse {
        prev_completion_candidate(shell)
    } else {
        next_completion_candidate(shell)
    };

    if let Some(next) = cycled {
        let mut out = replace_current_token(shell, &next);
        out.push_str(&render_completion_menu(shell));
        return out;
    }

    let mut candidates = handle_complete(shell, &shell.buffer.clone());
    if candidates.is_empty() {
        shell.completion_state = None;
        return clear_completion_menu(shell);
    }

    candidates.sort_unstable();
    candidates.dedup();

    if candidates.len() == 1 {
        shell.completion_state = None;
        let mut out = replace_current_token(shell, &candidates[0]);
        out.push_str(&clear_completion_menu(shell));
        return out;
    }

    let (start, _) = current_token_bounds(shell);

    // start at first for Tab, last for Shift+Tab
    let index = if reverse { candidates.len() - 1 } else { 0 };

    shell.completion_state = Some(CompletionState {
        start,
        candidates,
        index,
    });

    let selected = shell.completion_state.as_ref().unwrap().candidates[index].clone();

    let mut out = replace_current_token(shell, &selected);
    out.push_str(&render_completion_menu(shell));
    out
}

pub(super) fn reset_completion(shell: &mut Shell) -> String {
    shell.completion_state = None;
    clear_completion_menu(shell)
}

pub(super) fn handle_complete(shell: &mut Shell, buffer: &str) -> Vec<String> {
    let before_cursor = &buffer[..shell.pos];
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
            ShellToken::Op(
                ShellOp::RedirectIn
                | ShellOp::RedirectOut
                | ShellOp::RedirectAppend
                | ShellOp::RedirectErrOut
                | ShellOp::RedirectErrAppend,
            ) => {
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
        let ctx = CommandContext { pwd: &mut shell.pwd };
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
        let ctx = CommandContext { pwd: &mut shell.pwd };
        return bins::complete_path(&ctx, params, 0, true);
    }

    if let Some(bin) = bins::find(command) && bin_exists(bin.name()) {
        let mut ctx = CommandContext { pwd: &mut shell.pwd };
        let mut cands = bin.complete(&mut ctx, params, shell.pos);
        cands.sort_unstable();
        cands.dedup();
        return cands;
    }

    Vec::new()
}

fn clear_completion_menu(shell: &mut Shell) -> String {
    if shell.completion_menu_lines == 0 {
        return String::new();
    }

    shell.completion_menu_lines = 0;
    shell.completion_state = None;

    let mut out = shell.redraw_line();
    out.push_str("\x1B[J"); // clear everything below prompt line
    out.push_str(&shell.redraw_line());
    out
}

fn render_completion_menu(shell: &mut Shell) -> String {
    let mut out = shell.redraw_line();
    out.push_str("\x1B[J"); // clear stale menu region below prompt

    let Some(state) = shell.completion_state.as_ref() else {
        shell.completion_menu_lines = 0;
        return out;
    };

    let total = state.candidates.len();
    if total == 0 {
        shell.completion_menu_lines = 0;
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

    shell.completion_menu_lines = printed_lines;

    if printed_lines > 0 {
        let _ = write!(&mut out, "\x1B[{}A\r", printed_lines);
    }

    out.push_str(&shell.redraw_line());
    out
}

fn next_completion_candidate(shell: &mut Shell) -> Option<String> {
    let (start, end) = current_token_bounds(shell);
    let current_token = shell.buffer[start..end].to_owned();

    let state = shell.completion_state.as_mut()?;
    if state.start != start {
        return None;
    }

    if current_token != state.candidates[state.index] {
        return None;
    }

    state.index = (state.index + 1) % state.candidates.len();
    Some(state.candidates[state.index].clone())
}

fn prev_completion_candidate(shell: &mut Shell) -> Option<String> {
    let (start, end) = current_token_bounds(shell);
    let current_token = shell.buffer[start..end].to_owned();

    let state = shell.completion_state.as_mut()?;
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

fn current_token_bounds(shell: &Shell) -> (usize, usize) {
    let before = &shell.buffer[..shell.pos];
    let start = before
        .rfind(|c: char| c.is_whitespace() || matches!(c, '|' | '&' | ';' | '<' | '>'))
        .map_or(0, |i| i + 1);

    let after = &shell.buffer[shell.pos..];
    let rel_end = after
        .find(|c: char| c.is_whitespace() || matches!(c, '|' | '&' | ';' | '<' | '>'))
        .unwrap_or(after.len());
    let end = shell.pos + rel_end;

    (start, end)
}

fn replace_current_token(shell: &mut Shell, token: &str) -> String {
    let (start, end) = current_token_bounds(shell);
    shell.buffer.replace_range(start..end, token);
    shell.pos = start + token.len();
    shell.redraw_line()
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
