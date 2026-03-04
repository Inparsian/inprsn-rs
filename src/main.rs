pub mod enums;
pub mod marisa;
pub mod info;

mod components;
use components::{Window, Desktop};

mod windows;
use windows::WINDOWS;

use dioxus::prelude::*;

// Images
const FAVICON: Asset = asset!("/assets/favicon.ico");
const BG_SVG: Asset = asset!("/assets/bg.svg");
const WALLPAPER_IMG: Asset = asset!("/assets/wallpaper.jpg");

// Styles
const SIMPLE_CSS: Asset = asset!("/assets/styles/simple.css");
const MAIN_CSS: Asset = asset!("/assets/styles/main.css");
const FONTS_CSS: Asset = asset!("/assets/styles/fonts.css");
const TAILWIND_CSS: Asset = asset!("/assets/styles/tailwind.css");

// Router
#[derive(Routable, Clone, PartialEq, Eq)]
pub enum Route {
    #[route("/")]
    Full,
    #[route("/simple")]
    Simple,
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        Router::<Route> {}
    }
}

#[component]
fn Simple() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: SIMPLE_CSS }
        document::Link { rel: "stylesheet", href: FONTS_CSS }
        
        div {
            h1 {
                "inpr.sn"
            }
            
            img {
                id: "persona",
                src: "https://github.com/Inparsian.png",
                alt: "Persona Image",
                class: "about-pfp border border-dotted border-[#1B1B1B] p-px grayscale",
            }
            
            p {
                "hi, i'm inparsian. i'm some dumb 20 y.o. american dude who makes software"
            }
            
            h2 {
                "projects"
            }
            
            ul {
                for (name, desc, link) in info::PROJECTS {
                    li {
                        a {
                            href: link.to_owned(),
                            target: "_blank",
                            {name.to_owned()}
                        },
                        span {
                            {format!(" - {desc}")}
                        }
                    },
                }
            }
            
            h2 {
                "socials"
            }
            
            ul {
                for (name, _, link) in info::SOCIALS {
                    li {
                        a {
                            href: link.to_owned(),
                            target: "_blank",
                            {name.to_owned()}
                        },
                    },
                }
            }
            
            a {
                href: "/",
                "want the full experience? click here"
            },
        }
    }
}

#[component]
fn Full() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: FONTS_CSS }
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
        
        Desktop {}
        
        for window in WINDOWS.read().iter() {
            Window {
                key: "{window.id}",
                instance: window.clone(),
                
                {(window.render)()}
            }
        }
    }
}
