use super::{Command, CommandContext};

pub struct Echo;
pub const ECHO: Echo = Echo;

impl Command for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, _ctx: &mut CommandContext, args: &[String]) -> String {
        format!("{}\r\n", args.join(" "))
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}