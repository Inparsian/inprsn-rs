use std::fmt::Write as _;

use super::{Command, CommandContext, CommandResult};
use crate::services::fs::FILESYSTEM;

pub struct Cat;
pub const CAT: Cat = Cat;

impl Command for Cat {
    fn name(&self) -> &'static str {
        "cat"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, ctx: &mut CommandContext, args: &[String], stdin: Option<&str>) -> CommandResult {
        if args.is_empty() {
            if let Some(input) = stdin {
                return CommandResult::ok(input.to_owned());
            }
            return CommandResult::err("cat: not enough arguments\r\n");
        }

        let mut out = String::new();
        let mut failed = false;

        for path in args {
            let reader = FILESYSTEM.read().unwrap();
            let content = reader.read_file(path, Some(ctx.pwd));
            match content {
                Ok(content) => out.push_str(&String::from_utf8_lossy(content)),
                Err(err) => {
                    failed = true;
                    let _ = write!(out, "cat: {}\r\n", err);
                }
            }
        }

        if !out.ends_with("\r\n") {
            out.push_str("\r\n");
        }

        if failed {
            CommandResult::err(out)
        } else {
            CommandResult::ok(out)
        }
    }

    fn complete(&self, ctx: &mut CommandContext, args: &[String], _cursor: usize) -> Vec<String> {
        super::complete_path(ctx, args, 0, false)
    }
}