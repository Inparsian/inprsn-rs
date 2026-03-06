use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::windows::{WindowInstance, WindowInstanceProps};

pub fn new_minesweeper_instance() -> WindowInstance {
    WindowInstance::new(WindowInstanceProps {
        title: "minesweeper".to_owned(),
        size: ScreenCoordinates::Absolute { x: 332, y: 396 },
        ..Default::default()
    }, move || rsx! {
        WindowMinesweeper {}
    })
}

#[component]
pub fn WindowMinesweeper() -> Element {
    rsx! {
        div {}
    }
}