use std::rc::Rc;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::enums::ScreenCoordinates;
use crate::windows::{self, WindowInstance, WindowInstanceProps};

pub fn new_hydra_instance() -> WindowInstance {
    let x = rand::random::<f32>() * 100.0;
    let y = rand::random::<f32>() * 100.0;
    
    WindowInstance::new(WindowInstanceProps {
        title: "hydra".to_owned(),
        resizable: false,
        position: ScreenCoordinates::Percent { x, y },
        size: ScreenCoordinates::Absolute { x: 344, y: 160 },
        on_close: Some(Rc::new(move |_| {
            windows::spawn_window(new_hydra_instance());
            windows::spawn_window(new_hydra_instance());
        })),
        ..Default::default()
    }, move |id| rsx! {
        WindowHydra { id }
    })
}

#[component]
fn WindowHydra(id: Uuid) -> Element {
    rsx! {
        div {
            class: "flex flex-col p-4 items-center text-center",
            h2 { "hydra" },
            span { "cut off a head, two more will take its place." },
            a {
                onclick: move |_| {
                    windows::close_window(id);
                },
                "ok"
            }
        }
    }
}