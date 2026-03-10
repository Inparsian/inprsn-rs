use std::rc::Rc;
use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::os::{self, PROCESSES, Process, WindowInstance, WindowInstanceProps};

pub fn new_taskmanager_instance() -> Process {
    let mut process = Process::new("taskmanager");
    let pid = process.id;
    process.add_window(WindowInstance::new(WindowInstanceProps {
        title: "task manager".to_owned(),
        size: ScreenCoordinates::Absolute { x: 332, y: 396 },
        on_close: Some(Rc::new(move |_| {
            os::kill_process(pid);
        })),
        ..Default::default()
    }, move |_| rsx! {
        WindowTaskManager {}
    }));
    
    process
}

#[component]
fn WindowTaskManager() -> Element {
    rsx! {
        div {
            class: "taskmanager-header",
            span { "id" },
            span { "name" },
            span { "actions" },
        }
        div {
            class: "taskmanager-list",
            for process in PROCESSES.read().iter() {
                div {
                    class: "taskmanager-item",
                    
                    span { {process.id.to_string()} }
                    span { {process.name.clone()} }
                    div {
                        a {
                            onclick: {
                                let pid = process.id;
                                move |_| os::kill_process(pid)
                            },
                            "kill"
                        }
                    }
                }
            }
        }
    }
}