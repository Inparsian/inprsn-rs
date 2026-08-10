use super::{Command, CommandContext};
use crate::services::fs::FILESYSTEM;

pub struct Rm;
pub const RM: Rm = Rm;

impl Command for Rm {
    fn name(&self) -> &'static str {
        "rm"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, ctx: &mut CommandContext, args: &[String]) -> String {
        if args.is_empty() {
            "rm: not enough arguments\r\n".to_owned()
        } else {
            // TODO: flags
            let path = args.first().unwrap();
            FILESYSTEM.write().unwrap().remove(path, Some(ctx.pwd)).map_or_else(
                |err| format!("rm: {}\r\n", err),
                |()| String::new(),
            )
        }
    }

    fn complete(&self, ctx: &mut CommandContext, args: &[String], _cursor: usize) -> Vec<String> {
        super::complete_path(ctx, args, 0, false)
    }
}