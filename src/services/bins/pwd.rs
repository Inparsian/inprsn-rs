use super::{Command, CommandContext, CommandResult};

pub struct Pwd;
pub const PWD: Pwd = Pwd;

impl Command for Pwd {
    fn name(&self) -> &'static str {
        "pwd"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, ctx: &mut CommandContext, _args: &[String], _stdin: Option<&str>) -> CommandResult {
        CommandResult::ok(format!("{}\r\n", ctx.pwd))
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}