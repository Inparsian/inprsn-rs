use super::{Command, CommandContext, CommandResult};

pub struct Echo;
pub const ECHO: Echo = Echo;

impl Command for Echo {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, _ctx: &mut CommandContext, args: &[String], _stdin: Option<&str>) -> CommandResult {
        let mut newline = true;
        let mut interpret_escapes = false;
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "-n" => newline = false,
                "-e" => interpret_escapes = true,
                "-ne" | "-en" => {
                    newline = false;
                    interpret_escapes = true;
                }
                _ => break,
            }
            i += 1;
        }

        let mut output = args[i..].join(" ");

        if interpret_escapes {
            output = output
                .replace(r"\\", "\\")
                .replace(r"\n", "\n")
                .replace(r"\r", "\r")
                .replace(r"\t", "\t")
                .replace(r"\0", "\0");
        }

        if newline {
            output.push_str("\r\n");
        }

        CommandResult::ok(output)
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}