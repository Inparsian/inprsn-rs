use super::{Command, CommandContext, CommandResult};
use crate::sys;

pub struct Kill;
pub const KILL: Kill = Kill;

impl Command for Kill {
    fn name(&self) -> &'static str {
        "kill"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, _ctx: &mut CommandContext, args: &[String], _stdin: Option<&str>) -> CommandResult {
        if args.is_empty() {
            CommandResult::err("kill: not enough arguments\r\n")
        } else {
            // kill usually can take a exit signal code as an argument,
            // however our process manager can only kill processes (sig 9) at the moment.
            // as such we'll only take pids
            args[0].parse::<u32>().map_or_else(
                |_| CommandResult::err(format!("kill: cannot find process \"{}\"\r\n", args[0])),
                |pid| if sys::has_pid(pid) {
                    sys::kill_process(pid);
                    CommandResult::ok(String::new())
                } else {
                    CommandResult::err(format!("kill: sending signal to {} failed: No such process\r\n", pid))
                }
            )
        }
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}