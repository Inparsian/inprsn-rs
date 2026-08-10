use std::sync::LazyLock;

use crate::services::fs::{FILESYSTEM, FilesystemData};

pub mod echo;
pub mod clear;
pub mod pwd;
pub mod whoami;
pub mod neofetch;
pub mod uname;
pub mod ls;
pub mod cat;
pub mod rm;
pub mod touch;
pub mod mkdir;
pub mod kill;
pub mod ps;

pub static BINS: LazyLock<Vec<&'static dyn Command>> = LazyLock::new(|| {
    vec![
        &echo::ECHO,
        &clear::CLEAR,
        &pwd::PWD,
        &whoami::WHOAMI,
        &neofetch::NEOFETCH,
        &uname::UNAME,
        &ls::LS,
        &cat::CAT,
        &rm::RM,
        &touch::TOUCH,
        &mkdir::MKDIR,
        &kill::KILL,
        &ps::PS,
    ]
});

pub fn bins() -> &'static [&'static dyn Command] {
    BINS.as_slice()
}

pub fn find(name: &str) -> Option<&'static dyn Command> {
    bins().iter().copied().find(|cmd| {
        cmd.name() == name || cmd.aliases().contains(&name)
    })
}

pub trait Command: Sync {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> &'static [&'static str];
    fn run(&self, ctx: &mut CommandContext, args: &[String]) -> String;
    fn complete(&self, ctx: &mut CommandContext, args: &[String], cursor: usize) -> Vec<String>;
}

pub struct CommandContext<'a> {
    pub pwd: &'a mut String,
}

// complete helpers
pub fn complete_path(
    ctx: &CommandContext,
    args: &[String],
    arg_index: usize,
    only_dirs: bool,
) -> Vec<String> {
    if args.len() > arg_index + 1 {
        return Vec::new();
    }

    let partial = args.get(arg_index).map_or("", |s| s.as_str());
    let (base, prefix) = match partial.rsplit_once('/') {
        Some((dir, tail)) => (if dir.is_empty() { "/" } else { dir }, tail),
        None => (".", partial),
    };

    let reader = FILESYSTEM.read().unwrap();
    let Some(entries) = reader.resolve_read_dir(base, Some(ctx.pwd)) else {
        return Vec::new();
    };

    let mut out: Vec<String> = entries
        .into_iter()
        .filter(|e| e.name.starts_with(prefix))
        .filter(|e| {
            if only_dirs {
                matches!(e.data, FilesystemData::Directory { .. })
            } else {
                true
            }
        })
        .map(|e| if base == "." {
            e.name
        } else if base == "/" {
            format!("/{}", e.name)
        } else {
            format!("{}/{}", base, e.name)
        })
        .collect();

    out.sort_unstable();
    out.dedup();
    out
}