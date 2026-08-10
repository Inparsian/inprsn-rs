use super::{Command, CommandContext, CommandResult};
use crate::consts::{ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET};

pub struct Clear;
pub const CLEAR: Clear = Clear;

impl Command for Clear {
    fn name(&self) -> &'static str {
        "clear"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, _ctx: &mut CommandContext, _args: &[String], _stdin: Option<&str>) -> CommandResult {
        CommandResult::ok(format!("{}{}{}", ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET))
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}