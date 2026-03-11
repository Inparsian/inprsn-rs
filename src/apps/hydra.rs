use std::rc::Rc;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::enums::ScreenCoordinates;
use crate::sys::{self, Process, WindowInstance, WindowInstanceProps};

pub fn new_hydra_instance() -> Process {
    let mut process = Process::new("hydra");
    process.add_window(new_hydra_window(process.id));
    process
}

pub fn new_hydra_window(pid: u32) -> WindowInstance {
    let x = rand::random::<f32>() * 100.0;
    let y = rand::random::<f32>() * 100.0;
    
    WindowInstance::new(WindowInstanceProps {
        title: "hydra".to_owned(),
        resizable: false,
        position: ScreenCoordinates::Percent { x, y },
        size: ScreenCoordinates::Absolute { x: 344, y: 160 },
        on_close: Some(Rc::new(move |_| sys::with_process_mut(pid, |process| {
            process.add_window(new_hydra_window(pid));
            process.add_window(new_hydra_window(pid));
        }))),
        ..Default::default()
    }, move |id| rsx! {
        WindowHydra { pid, id }
    })
}

#[component]
fn WindowHydra(pid: u32, id: Uuid) -> Element {
    rsx! {
        div {
            class: "flex flex-col p-4 items-center text-center",
            h2 { "hydra" },
            span { "cut off a head, two more will take its place." },
            a {
                onclick: move |_| {
                    sys::call_on_close(id);
                    sys::close_window(id);
                },
                "ok"
            }
        }
    }
}