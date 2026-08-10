use std::fmt::Write as _;

use super::{Command, CommandContext};
use crate::services::fs::FILESYSTEM;

pub struct Mkdir;
pub const MKDIR: Mkdir = Mkdir;

impl Command for Mkdir {
    fn name(&self) -> &'static str {
        "mkdir"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, ctx: &mut CommandContext, args: &[String]) -> String {
        let mut parents = false;
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
                        'p' => parents = true,
                        _ => return format!("mkdir: invalid option -- '{}'\r\n", ch),
                    }
                }
            } else {
                paths.push(arg);
            }
        }

        if paths.is_empty() {
            return "mkdir: not enough arguments\r\n".to_owned();
        }

        let mut out = String::new();

        for path in paths {
            let result = if parents {
                create_dir_all(path, Some(ctx.pwd))
            } else {
                FILESYSTEM.write().unwrap().create_directory(path, Some(ctx.pwd))
            };

            if let Err(err) = result {
                let _ = write!(&mut out, "mkdir: {}\r\n", err);
            }
        }

        out
    }

    fn complete(&self, ctx: &mut CommandContext, args: &[String], _cursor: usize) -> Vec<String> {
        let path_index = args
            .iter()
            .position(|a| !a.starts_with('-') || a == "-")
            .unwrap_or(args.len());

        if path_index == args.len() {
            return super::complete_path(ctx, &[String::new()], 0, true);
        }

        super::complete_path(ctx, args, path_index, true)
    }
}

fn create_dir_all(path: &str, pwd: Option<&str>) -> Result<(), String> {
    let resolved = {
        let reader = FILESYSTEM.read().unwrap();
        reader.resolve_path(path, pwd)
    };

    if let Some(resolved) = resolved {
        let reader = FILESYSTEM.read().unwrap();
        return match reader.resolve_read(&resolved, None) {
            Some(entry) if matches!(entry.data, crate::services::fs::FilesystemData::Directory { .. }) => Ok(()),
            Some(_) => Err(format!("Path is not a directory: {}", path)),
            None => Err("Parent directory does not exist".to_owned()),
        };
    }

    let absolute_target = if path.starts_with('/') {
        path.to_owned()
    } else {
        let base = pwd.unwrap_or("/").trim_end_matches('/');
        if base.is_empty() {
            format!("/{}", path)
        } else {
            format!("{}/{}", base, path)
        }
    };

    let parts = absolute_target
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return Ok(());
    }

    let mut prefix = String::new();
    for part in parts {
        if part == ".." {
            if let Some((head, _)) = prefix.rsplit_once('/') {
                prefix = head.to_owned();
            } else {
                prefix.clear();
            }
            continue;
        }

        prefix.push('/');
        prefix.push_str(part);

        let exists = {
            let reader = FILESYSTEM.read().unwrap();
            reader.resolve_read(&prefix, None).map(|entry| matches!(entry.data, crate::services::fs::FilesystemData::Directory { .. }))
        };

        match exists {
            Some(true) => {}
            Some(false) => return Err(format!("Path is not a directory: {}", prefix)),
            None => {
                FILESYSTEM.write().unwrap().create_directory(&prefix, None)?;
            }
        }
    }

    Ok(())
}
