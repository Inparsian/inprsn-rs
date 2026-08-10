use std::fmt::Write as _;

use super::{Command, CommandContext, CommandResult};
use crate::services::fs::{FILESYSTEM, FilesystemData};

pub struct Rm;
pub const RM: Rm = Rm;

impl Command for Rm {
    fn name(&self) -> &'static str {
        "rm"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, ctx: &mut CommandContext, args: &[String], _stdin: Option<&str>) -> CommandResult {
        let mut recursive = false;
        let mut force = false;
        let mut paths: Vec<&String> = Vec::new();

        let mut parsing_flags = true;
        for arg in args {
            if parsing_flags && arg == "--" {
                parsing_flags = false;
                continue;
            }

            if parsing_flags && arg.starts_with('-') && arg.len() > 1 {
                for ch in arg.chars().skip(1) {
                    match ch {
                        'r' => recursive = true,
                        'f' => force = true,
                        _ => return CommandResult::err(format!("rm: invalid option -- '{}'\r\n", ch)),
                    }
                }
            } else {
                paths.push(arg);
            }
        }

        if paths.is_empty() {
            return if force {
                CommandResult::ok(String::new())
            } else {
                CommandResult::err("rm: not enough arguments\r\n")
            };
        }

        let mut out = String::new();

        for path in paths {
            let (exists, is_dir) = {
                let reader = FILESYSTEM.read().unwrap();
                reader.resolve_read(path, Some(ctx.pwd)).map_or(
                    (false, false),
                    |entry| (true, matches!(entry.data, FilesystemData::Directory { .. }))
                )
            };

            if !exists {
                if !force {
                    let _ = write!(out, "rm: cannot remove '{}': No such file or directory\r\n",
                        path);
                }
                continue;
            }

            if is_dir && !recursive {
                let _ = write!(out, "rm: cannot remove '{}': Is a directory\r\n", path);
                continue;
            }

            if let Err(err) = FILESYSTEM.write().unwrap().remove(path, Some(ctx.pwd)) && !force {
                let _ = write!(out, "rm: {}\r\n", err);
            }
        }

        if out.is_empty() {
            CommandResult::ok(out)
        } else {
            CommandResult::err(out)
        }
    }

    fn complete(&self, ctx: &mut CommandContext, args: &[String], _cursor: usize) -> Vec<String> {
        // Complete first non-flag token as path (keeps completion useful for: rm -r <tab>)
        let path_index = args
            .iter()
            .position(|a| !a.starts_with('-') || a == "-")
            .unwrap_or(args.len());

        if path_index == args.len() {
            return super::complete_path(ctx, &[String::new()], 0, false);
        }

        super::complete_path(ctx, args, path_index, false)
    }
}