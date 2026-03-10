use std::rc::Rc;
use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::os::{self, Process, WindowInstance, WindowInstanceProps};

pub fn new_minesweeper_instance() -> Process {
    let mut process = Process::new("minesweeper");
    let pid = process.id;
    process.add_window(WindowInstance::new(WindowInstanceProps {
        title: "minesweeper".to_owned(),
        size: ScreenCoordinates::Absolute { x: 332, y: 396 },
        on_close: Some(Rc::new(move |_| {
            os::kill_process(pid);
        })),
        ..Default::default()
    }, move |_| rsx! {
        WindowMinesweeper {}
    }));
    
    process
}

#[component]
fn WindowMinesweeper() -> Element {
    rsx! {
        div {}
    }
}