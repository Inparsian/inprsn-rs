use std::rc::Rc;
use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::sys::{self, Process, WindowInstance, WindowInstanceProps};

pub fn new_filething_instance() -> Process {
    let mut process = Process::new("filething");
    let pid = process.id;
    process.add_window(WindowInstance::new(WindowInstanceProps {
        title: "file thing".to_owned(),
        size: ScreenCoordinates::Absolute { x: 600, y: 350 },
        on_close: Some(Rc::new(move |_| {
            sys::kill_process(pid);
        })),
        ..Default::default()
    }, move |_| rsx! {
        WindowFileThing {}
    }));
    
    process
}

#[component]
fn WindowFileThing() -> Element {
    rsx! {
        div {
            class: "filething",
        },
    }
}