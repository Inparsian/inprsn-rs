use std::fmt::Write as _;
use dioxus::prelude::ReadableExt as _;

use super::{Command, CommandContext};
use crate::sys;

pub struct Ps;
pub const PS: Ps = Ps;

impl Command for Ps {
    fn name(&self) -> &'static str {
        "ps"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, _ctx: &mut CommandContext, _args: &[String]) -> String {
        // simple ahh implementation for now
        let mut out = format!("{:<5} {:<10}\r\n", "PID", "COMMAND");
        for proc in sys::PROCESSES.read().iter() {
            let _ = write!(&mut out, "{:<5} {:<10}\r\n", proc.id, proc.name);
        }
        out
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}