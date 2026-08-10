use std::sync::LazyLock;

pub mod echo;
pub mod clear;
pub mod pwd;
pub mod whoami;
pub mod neofetch;
pub mod ls;
pub mod cat;
pub mod rm;
pub mod kill;
pub mod ps;

pub static BINS: LazyLock<Vec<&'static dyn Command>> = LazyLock::new(|| {
    vec![
        &echo::ECHO,
        &clear::CLEAR,
        &pwd::PWD,
        &whoami::WHOAMI,
        &neofetch::NEOFETCH,
        &ls::LS,
        &cat::CAT,
        &rm::RM,
        &kill::KILL,
        &ps::PS,
    ]
});

pub fn bins() -> &'static [&'static dyn Command] {
    BINS.as_slice()
}

pub fn find(name: &str) -> Option<&'static dyn Command> {
    bins().iter().copied().find(|cmd| {
        cmd.name() == name || cmd.aliases().contains(&name)
    })
}

pub trait Command: Sync {
    fn name(&self) -> &'static str;
    fn aliases(&self) -> &'static [&'static str];
    fn run(&self, ctx: &mut CommandContext, args: &[String]) -> String;
    fn complete(&self, ctx: &mut CommandContext, args: &[String], cursor: usize) -> Vec<String>;
}

pub struct CommandContext<'a> {
    pub pwd: &'a mut String,
}

