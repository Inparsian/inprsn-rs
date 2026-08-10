use super::{Command, CommandContext};

pub struct Uname;
pub const UNAME: Uname = Uname;

const KERNEL_NAME: &str = "Linux";
const NODE_NAME: &str = "I-USE-AWCH-UWU";
const KERNEL_RELEASE_NAME: &str = "7.1.5-arch1-2";
const KERNEL_VERSION_NAME: &str = "#1 SMP PREEMPT_DYNAMIC Tue, 28 Jul 2026 13:49:51 +0000";
const MACHINE_NAME: &str = "x86_64";
const PROCESSOR_NAME: &str = "unknown";
const HARDWARE_PLATFORM_NAME: &str = "x86_64";
const OPERATING_SYSTEM_NAME: &str = "GNU/Linux";

// long option abbreviations we allow to resolve if unique
const LONGS: &[&str] = &[
    "--all",
    "--kernel-name",
    "--nodename",
    "--kernel-release",
    "--kernel-version",
    "--machine",
    "--processor",
    "--hardware-platform",
    "--operating-system",
    "--help",
    "--version",
];

const SHORTS: &[&str] = &["-a", "-s", "-n", "-r", "-v", "-m", "-p", "-i", "-o"];

// texts
const HELP_TEXT: &str = "Usage: uname [OPTION]...\r\nPrint certain system information.  With no OPTION, same as -s.\r\n\r\n  -a, --all                print all information, in the following order,\r\n                             except omit -p and -i if unknown\r\n  -s, --kernel-name        print the kernel name\r\n  -n, --nodename           print the network node hostname\r\n  -r, --kernel-release     print the kernel release\r\n  -v, --kernel-version     print the kernel version\r\n  -m, --machine            print the machine hardware name\r\n  -p, --processor          print the processor type (non-portable)\r\n  -i, --hardware-platform  print the hardware platform (non-portable)\r\n  -o, --operating-system   print the operating system\r\n      --help\r\n         display this help and exit\r\n      --version\r\n         output version information and exit\r\n\r\nReport bugs to: bug-coreutils@gnu.org\r\nGNU coreutils home page: <https://www.gnu.org/software/coreutils/>\r\nGeneral help using GNU software: <https://www.gnu.org/gethelp/>\r\nFull documentation <https://www.gnu.org/software/coreutils/uname>\r\nor available locally via: info '(coreutils) uname invocation'";
const VERSION_TEXT: &str = "uname (GNU coreutils) 9.11\r\nCopyright (C) 2026 Free Software Foundation, Inc.\r\nLicense GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.\r\nThis is free software: you are free to change and redistribute it.\r\nThere is NO WARRANTY, to the extent permitted by law.\r\n\r\nWritten by David MacKenzie.";

impl Command for Uname {
    fn name(&self) -> &'static str {
        "uname"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, _ctx: &mut CommandContext, args: &[String]) -> String {
        if args.is_empty() {
            return format!("{}\r\n", KERNEL_NAME);
        }

        let mut pieces: Vec<&'static str> = Vec::new();
        let mut end_of_options = false;

        for arg in args {
            if end_of_options {
                return format!(
                    "uname: extra operand '{}'\r\nTry 'uname --help' for more information.\r\n",
                    arg
                );
            }

            if arg == "--" {
                end_of_options = true;
                continue;
            }

            if arg == "--help" {
                return format!("{}\r\n", HELP_TEXT);
            }

            if arg == "--version" {
                return format!("{}\r\n", VERSION_TEXT);
            }

            if arg.starts_with("--") {
                let resolved = if let Some(opt) = full_option(arg) {
                    opt
                } else {
                    let possibilities: Vec<&str> = LONGS
                        .iter()
                        .copied()
                        .filter(|o| o.starts_with(arg))
                        .collect();

                    if possibilities.len() == 1 {
                        possibilities[0]
                    } else if !possibilities.is_empty() {
                        return format!(
                            "uname: option '{}' is ambiguous; possibilities: {}\r\nTry 'uname --help' for more information.\r\n",
                            arg,
                            possibilities
                                .iter()
                                .map(|p| format!("'{}'", p))
                                .collect::<Vec<_>>()
                                .join(" ")
                        );
                    } else {
                        return format!(
                            "uname: unrecognized option '{}'\r\nTry 'uname --help' for more information.\r\n",
                            arg
                        );
                    }
                };

                push_option_output(resolved, &mut pieces);
                continue;
            }

            // short flags: -a, -s, -sr, etc.
            if let Some(shorts) = arg.strip_prefix('-') {
                if shorts.is_empty() {
                    return "uname: unrecognized option '-'\r\nTry 'uname --help' for more information.\r\n".to_owned();
                }

                for ch in shorts.chars() {
                    match ch {
                        'a' => push_option_output("--all", &mut pieces),
                        's' => push_option_output("--kernel-name", &mut pieces),
                        'n' => push_option_output("--nodename", &mut pieces),
                        'r' => push_option_output("--kernel-release", &mut pieces),
                        'v' => push_option_output("--kernel-version", &mut pieces),
                        'm' => push_option_output("--machine", &mut pieces),
                        'p' => push_option_output("--processor", &mut pieces),
                        'i' => push_option_output("--hardware-platform", &mut pieces),
                        'o' => push_option_output("--operating-system", &mut pieces),
                        _ => {
                            return format!(
                                "uname: invalid option -- '{}'\r\nTry 'uname --help' for more information.\r\n",
                                ch
                            );
                        }
                    }
                }

                continue;
            }

            return format!(
                "uname: extra operand '{}'\r\nTry 'uname --help' for more information.\r\n",
                arg
            );
        }

        if pieces.is_empty() {
            pieces.push(KERNEL_NAME);
        }

        format!("{}\r\n", pieces.join(" "))
    }

    fn complete(&self, _ctx: &mut CommandContext, args: &[String], _cursor: usize) -> Vec<String> {
        if args.iter().any(|a| a == "--") {
            return Vec::new();
        }

        if args.is_empty() {
            let mut out = LONGS
                .iter()
                .copied()
                .chain(SHORTS.iter().copied())
                .map(str::to_owned)
                .collect::<Vec<_>>();
            out.sort_unstable();
            out.dedup();
            return out;
        }

        let current = args.last().map_or("", |s| s.as_str());

        let mut out = if current.starts_with("--") {
            LONGS
                .iter()
                .copied()
                .filter(|opt| opt.starts_with(current))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        } else if current.starts_with('-') {
            SHORTS
                .iter()
                .copied()
                .filter(|opt| opt.starts_with(current))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        out.sort_unstable();
        out.dedup();
        out
    }
}

fn full_option(input: &str) -> Option<&'static str> {
    LONGS.iter().find(|o| **o == input).copied()
}

fn push_option_output(opt: &str, out: &mut Vec<&'static str>) {
    match opt {
        "--all" => {
            out.push(KERNEL_NAME);
            out.push(NODE_NAME);
            out.push(KERNEL_RELEASE_NAME);
            out.push(KERNEL_VERSION_NAME);
            out.push(MACHINE_NAME);

            // GNU behavior: omit -p/-i in --all if "unknown"
            if PROCESSOR_NAME != "unknown" {
                out.push(PROCESSOR_NAME);
            }
            if HARDWARE_PLATFORM_NAME != "unknown" {
                out.push(HARDWARE_PLATFORM_NAME);
            }

            out.push(OPERATING_SYSTEM_NAME);
        }
        "--kernel-name" => out.push(KERNEL_NAME),
        "--nodename" => out.push(NODE_NAME),
        "--kernel-release" => out.push(KERNEL_RELEASE_NAME),
        "--kernel-version" => out.push(KERNEL_VERSION_NAME),
        "--machine" => out.push(MACHINE_NAME),
        "--processor" => out.push(PROCESSOR_NAME),
        "--hardware-platform" => out.push(HARDWARE_PLATFORM_NAME),
        "--operating-system" => out.push(OPERATING_SYSTEM_NAME),
        _ => {}
    }
}