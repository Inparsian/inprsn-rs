use super::{Command, CommandContext, CommandResult};

pub struct WhoAmI;
pub const WHOAMI: WhoAmI = WhoAmI;

impl Command for WhoAmI {
    fn name(&self) -> &'static str {
        "whoami"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, _ctx: &mut CommandContext, _args: &[String], _stdin: Option<&str>) -> CommandResult {
        CommandResult::ok(format!("{}\r\n", "inparsian"))
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}