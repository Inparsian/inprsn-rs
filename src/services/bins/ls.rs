use std::fmt::Write as _;

use super::{Command, CommandContext};
use crate::services::fs::{FILESYSTEM, FilesystemData};

pub struct Ls;
pub const LS: Ls = Ls;

impl Command for Ls {
    fn name(&self) -> &'static str {
        "ls"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, ctx: &mut CommandContext, args: &[String]) -> String {
        let path = args.first().unwrap_or(ctx.pwd);
                    
        FILESYSTEM.read().unwrap().resolve_read(path, Some(ctx.pwd)).map_or_else(
            || format!("ls: cannot access '{}': No such file or directory\r\n", path),
            |query| match &query.data {
                FilesystemData::Directory { children } => {
                    children.iter().fold(String::new(), |mut out, child| {
                        let _ = write!(&mut out, "{}\r\n", child.name);
                        out
                    })
                },
                FilesystemData::File { .. } |
                FilesystemData::SymbolicLink { .. } => format!("{}\r\n", query.name),
            })
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}