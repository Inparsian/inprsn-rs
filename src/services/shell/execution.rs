use std::fmt::Write as _;

use super::{
    parser::{parse_command_chains, tokenize_shell, ChainCondition, Pipeline, SimpleCommand},
    redirect::{prepare_redirect, write_redirect},
    Shell,
};
use crate::services::{
    bins::{self, CommandContext, CommandResult},
    fs::{FilesystemData, FILESYSTEM},
};

pub(super) fn handle_cmd(shell: &mut Shell, cmd: &str) -> String {
    handle_cmd_result(shell, cmd).render()
}

fn handle_cmd_result(shell: &mut Shell, cmd: &str) -> CommandResult {
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

        let result = execute_pipeline(shell, pipeline);
        last_status = result.status;
        aggregate.push_str(&result.render());
    }

    CommandResult {
        output: aggregate,
        error: String::new(),
        status: last_status,
    }
}

fn execute_pipeline(shell: &mut Shell, pipeline: Pipeline) -> CommandResult {
    let mut piped_input: Option<String> = None;
    let mut last = CommandResult::ok(String::new());

    let last_index = pipeline.commands.len().saturating_sub(1);
    for (i, cmd) in pipeline.commands.into_iter().enumerate() {
        let result = execute_simple_command(shell, cmd, piped_input.as_deref());
        if i < last_index {
            piped_input = Some(result.output.clone());
        }
        last = result;
    }

    last
}

fn execute_simple_command(shell: &mut Shell, cmd: SimpleCommand, piped_input: Option<&str>) -> CommandResult {
    let stdin_input = if let Some(path) = cmd.stdin.as_ref() {
        let reader = FILESYSTEM.read().unwrap();
        match reader.read_file(path, Some(&shell.pwd)) {
            Ok(content) => Some(String::from_utf8_lossy(content).into_owned()),
            Err(err) => return CommandResult::err(format!("{}\r\n", err)),
        }
    } else {
        piped_input.map(str::to_owned)
    };

    if let Some((path, append)) = cmd.stdout.as_ref()
        && let Err(err) = prepare_redirect(path, *append, Some(&shell.pwd))
    {
        return CommandResult::err(format!("{}\r\n", err));
    }

    if let Some((path, append)) = cmd.stderr.as_ref()
        && let Err(err) = prepare_redirect(path, *append, Some(&shell.pwd))
    {
        return CommandResult::err(format!("{}\r\n", err));
    }

    if cmd.argv.is_empty() {
        return CommandResult::err("syntax error near unexpected token\r\n");
    }

    let command = &cmd.argv[0];
    let params = &cmd.argv[1..];

    let mut result = if command == "cd" {
        run_builtin_cd(shell, params)
    } else if let Some(bin) = bins::find(command) {
        if bin_exists(bin.name()) {
            let mut ctx = CommandContext { pwd: &mut shell.pwd };
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
        if let Err(err) = write_redirect(&path, append, &result.output, Some(&shell.pwd)) {
            let _ = write!(result.error, "{}\r\n", err);
            if result.status == 0 {
                result.status = 1;
            }
        } else {
            result.output.clear();
        }
    }

    if let Some((path, append)) = cmd.stderr {
        if let Err(err) = write_redirect(&path, append, &result.error, Some(&shell.pwd)) {
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

fn run_builtin_cd(shell: &mut Shell, params: &[String]) -> CommandResult {
    if params.is_empty() {
        shell.pwd = "/home/inparsian".to_owned();
        return CommandResult::ok(String::new());
    }

    let path = params[0].clone();
    let reader = FILESYSTEM.read().unwrap();
    if let Some(query) = reader.resolve_read(&path, Some(&shell.pwd)) {
        match &query.data {
            FilesystemData::Directory { .. } => {
                shell.pwd = reader.resolve_path(&path, Some(&shell.pwd)).unwrap();
                CommandResult::ok(String::new())
            }
            FilesystemData::File { .. } | FilesystemData::SymbolicLink { .. } => {
                CommandResult::err(format!("cd: {}: Not a directory\r\n", path))
            }
        }
    } else {
        CommandResult::err(format!("cd: {}: No such file or directory\r\n", path))
    }
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
