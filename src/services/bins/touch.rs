use std::fmt::Write as _;

use super::{Command, CommandContext};
use crate::services::fs::FILESYSTEM;

pub struct Touch;
pub const TOUCH: Touch = Touch;

impl Command for Touch {
    fn name(&self) -> &'static str {
        "touch"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &[]
    }

    fn run(&self, ctx: &mut CommandContext, args: &[String]) -> String {
        if args.is_empty() {
            return "touch: not enough arguments\r\n".to_owned();
        }

        let mut out = String::new();

        for path in args {
            let exists = {
                let reader = FILESYSTEM.read().unwrap();
                reader.resolve_read(path, Some(ctx.pwd)).is_some()
            };

            // touch updates timestamp for existing files; our VFS has no timestamps,
            // so existing entries are a no-op.
            if exists {
                continue;
            }

            let result = FILESYSTEM.write().unwrap().create_file(path, Some(ctx.pwd), b"");
            if let Err(err) = result {
                let _ = write!(&mut out, "touch: {}\r\n", err);
            }
        }

        out
    }

    fn complete(&self, ctx: &mut CommandContext, args: &[String], _cursor: usize) -> Vec<String> {
        super::complete_path(ctx, args, 0, false)
    }
}