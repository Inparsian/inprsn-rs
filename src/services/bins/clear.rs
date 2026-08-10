use super::{Command, CommandContext};
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

    fn run(&self, _ctx: &mut CommandContext, _args: &[String]) -> String {
        format!("{}{}{}", ANSI_CLEAR_SCREEN, ANSI_CURSOR_HOME, ANSI_RESET)
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}