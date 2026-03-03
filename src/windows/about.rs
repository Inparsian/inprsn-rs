use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::windows::{WindowInstance, WindowInstanceProps};

pub fn new_about_instance() -> WindowInstance {
    WindowInstance::new(WindowInstanceProps {
        title: "inparsian".to_owned(),
        size: ScreenCoordinates::Absolute { x: 600, y: 200 },
        ..Default::default()
    }, move || rsx! {
        WindowAbout {}
    })
}

#[component]
pub fn WindowAbout() -> Element {
    rsx! {
        div {
            id: "about-container",
            class: "bg-[#131313] p-8 flex flex-row w-full h-full",

            div {
                id: "about-content",
                class: "flex flex-col gap-6",

                img {
                    src: "https://github.com/Inparsian.png",
                    alt: "Persona Image",
                    class: "about-pfp border border-dotted border-[#1B1B1B] p-px grayscale",
                }

                div {
                    id: "hero-text",

                    p {
                        "hi, i'm inparsian. i'm some dumb 20 y.o. american dude who makes software"
                    }
                    
                    a {
                        onclick: |_| async {
                            crate::marisa::hallo().await;
                        },
                        "click for a cool easter egg"
                    }
                }
            }
        }
    }
}