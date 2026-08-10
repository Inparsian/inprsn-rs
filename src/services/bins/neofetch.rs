use super::{Command, CommandContext};

pub struct Neofetch;
pub const NEOFETCH: Neofetch = Neofetch;

impl Command for Neofetch {
    fn name(&self) -> &'static str {
        "neofetch"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["fastfetch"]
    }

    fn run(&self, _ctx: &mut CommandContext, _args: &[String]) -> String {
        format!("{}\r\n", "inparsian")
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}