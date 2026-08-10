use super::{Command, CommandContext};
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

    fn run(&self, ctx: &mut CommandContext, args: &[String]) -> String {
        if args.is_empty() {
            "cat: not enough arguments\r\n".to_owned()
        } else {
            let path = args.first().unwrap();
            let reader = FILESYSTEM.read().unwrap();
            let content = reader.read_file(path, Some(ctx.pwd));
            content.map_or_else(
                |err| format!("cat: {}\r\n", err),
                |content| format!("{}\r\n", String::from_utf8_lossy(content).into_owned()),
            )
        }
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}