pub mod enums;
pub mod marisa;

mod components;
use components::Window;

mod windows;
use windows::WINDOWS;

use dioxus::prelude::*;

// Images
const FAVICON: Asset = asset!("/assets/favicon.ico");
const BG_SVG: Asset = asset!("/assets/bg.svg");
const WALLPAPER_IMG: Asset = asset!("/assets/wallpaper.jpg");

// Styles
const MAIN_CSS: Asset = asset!("/assets/styles/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/styles/tailwind.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        
        div { id: "background", class: "absolute z-0 left-0 top-0 w-full h-full",
            div {
                class: "absolute z-0 opacity-65 w-[calc(100%+1px)] h-[calc(100%+1px)] -top-px -left-px",
                style: format!("background-image: url('{}'); background-size: cover; background-position: center;", WALLPAPER_IMG),
            }
    
            svg {
                class: "absolute z-1 opacity-30 top-0 left-0",
                xmlns: "http://www.w3.org/2000/svg",
                width: "100%",
                height: "100%",
    
                defs {
                    pattern { id: "matrix-bg", x: 0, y: 0, width: 512, height: 512, pattern_units: "userSpaceOnUse",
                        image { href: BG_SVG, x: 0, y: 0, width: 512, height: 512, }
                    }
                },
    
                rect { width: "100%", height: "100%", fill: "url(#matrix-bg)", }
            }
        }
        
        div { class: "desktop",
            button {
                onclick: move |_| {
                    windows::spawn(windows::about::new_about_instance());
                },
                "About"
            }
        }
        
        for window in WINDOWS.read().iter() {
            Window {
                key: "{window.id}",
                instance: window.clone(),
                
                {(window.render)()}
            }
        }
    }
}
