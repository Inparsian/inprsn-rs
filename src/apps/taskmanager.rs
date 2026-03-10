use std::rc::Rc;
use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::os::{self, PROCESSES, Process, WindowInstance, WindowInstanceProps};

pub fn new_taskmanager_instance() -> Process {
    let mut process = Process::new("taskmanager");
    let pid = process.id;
    process.add_window(WindowInstance::new(WindowInstanceProps {
        title: "task manager".to_owned(),
        size: ScreenCoordinates::Absolute { x: 350, y: 250 },
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
            class: "taskmanager",
            
            table {
                tr {
                    th { "id" },
                    th { "name" },
                    th { "windows" },
                    th { "actions" },
                },
                
                for process in PROCESSES.read().iter() {
                    tr {
                        th { {process.id.to_string()} },
                        th { {process.name.clone()} },
                        th { {process.windows_len().to_string()} },
                        th {
                            a {
                                onclick: {
                                    let pid = process.id;
                                    move |_| os::kill_process(pid)
                                },
                                "kill"
                            }
                        },
                    },
                }
            },
        },
    }
}