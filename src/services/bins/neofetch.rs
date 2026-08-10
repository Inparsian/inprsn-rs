use std::fmt::Write as _;

use crate::consts::{ANSI_BOLD, ANSI_BRIGHT_CYAN, ANSI_CYAN, ANSI_DIM, ANSI_RESET};
use super::{Command, CommandContext, CommandResult};

pub struct Neofetch;
pub const NEOFETCH: Neofetch = Neofetch;

const NEOFETCH_LOGO: &[&str] = &[
    "      /\\      ",
    "     /  \\     ",
    "    /\\   \\    ",
    "   /      \\   ",
    "  /   ,,   \\  ",
    " /   |  |  -\\ ",
    "/_-''    ''-_\\",
];

impl Command for Neofetch {
    fn name(&self) -> &'static str {
        "neofetch"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["fastfetch"]
    }

    fn run(&self, _ctx: &mut CommandContext, _args: &[String], _stdin: Option<&str>) -> CommandResult {
        let fields = [
            ("OS", "Arch Linux x86_64"),
            ("Host", "Samsung Galaxy Note 7"),
            ("Kernel", "7.1.5-arch1-2"),
            ("Uptime", "9783 days, 12 hours, 69 mins"),
            ("Packages", "69 (pacman)"),
            ("Shell", "/bin/sheesh 4.2.0"),
            ("Terminal", "1337 hax0r terminal"),
            ("CPU", "AMD Ryzen 7 5800X (16) @ 3.800GHz"),
            ("GPU", "NVIDIA GeForce RTX 3060 Ti"),
            ("Memory", "3161MiB / 32016MiB"),
            ("Disk", "69G / 420G"),
        ];

        let mut neofetch_lines = vec![
            format!(
                "{}{}inparsian{}@{}{}I-USE-AWCH-UWU",
                ANSI_BOLD, ANSI_CYAN, ANSI_RESET, ANSI_BOLD, ANSI_CYAN
            ),
            format!("{}{}---------------", ANSI_RESET, ANSI_DIM),
        ];

        neofetch_lines.extend(
            fields
                .iter()
                .map(|(label, value)| format!("{}{}{}: {}", ANSI_CYAN, label, ANSI_RESET, value)),
        );

        let mut output = String::new();
        let max_lines = NEOFETCH_LOGO.len().max(neofetch_lines.len());

        for i in 0..max_lines {
            let logo = NEOFETCH_LOGO.get(i).copied().unwrap_or("              ");
            let line = neofetch_lines.get(i).map_or("", String::as_str);
            let _ = write!(output, "{}{}{}{}     {}{}\r\n", ANSI_RESET, ANSI_BOLD, ANSI_BRIGHT_CYAN, logo, ANSI_RESET, line);
        }

        CommandResult::ok(output)
    }

    fn complete(&self, _ctx: &mut CommandContext, _args: &[String], _cursor: usize) -> Vec<String> {
        Vec::new()
    }
}