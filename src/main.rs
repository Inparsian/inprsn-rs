pub mod os;
pub mod enums;
pub mod crazyerror;
pub mod info;

mod apps;
mod components;
use components::{Window, Desktop, Bar};

use dioxus::prelude::*;
use rand::RngExt as _;

// Images
const FAVICON: Asset = asset!("/assets/favicon.ico");
const BG_SVG: Asset = asset!("/assets/bg.svg");
const WALLPAPER_IMG: Asset = asset!("/assets/wallpaper.jpg");

// Styles
const SIMPLE_CSS: Asset = asset!("/assets/styles/simple.css");
const MAIN_CSS: Asset = asset!("/assets/styles/main.css");
const FONTS_CSS: Asset = asset!("/assets/styles/fonts.css");
const TAILWIND_CSS: Asset = asset!("/assets/styles/tailwind.css");

// State
pub const KERNEL_PANIC: GlobalSignal<bool> = GlobalSignal::new(|| false);

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
fn KernelPanic() -> Element {
    let mut rng = rand::rng();
    
    let mut lines: Vec<(f32, &str)> = vec![
        (0.0, "Linux version 6.7.0 (inpr@inprsn) #1 SMP PREEMPT_DYNAMIC"),
        (0.0, "Command line: BOOT_IMAGE=/vmlinuz root=/dev/sda1 ro quiet"),
        (0.0, "x86/fpu: Supporting XSAVE feature 0x001: 'x87 floating point registers'"),
        (0.0, "x86/fpu: Supporting XSAVE feature 0x002: 'SSE registers'"),
        (0.0, "x86/fpu: Enabled xstate features 0x3, context size is 832 bytes, using 'compacted' format."),
        // We will append to these stamps later onward
        (1.2, "Initramfs unpacking failed: invalid magic at start of compressed archive"),
        (1.2, "VFS: Cannot open root device \"sda1\" or unknown-block(0,0): error -6"),
        (1.2, "Please append a correct \"root=\" boot option; here are the available partitions:"),
        (1.2, "0100            65536 ram0"),
        (1.2, "0101            65536 ram1"),
        (1.2, "0800        976773168 sda"),
        (1.2, "0801        976760832 sda1"),
        (1.2, "Kernel panic - not syncing: VFS: Unable to mount root fs on unknown-block(0,0)"),
        (1.2, "CPU: 0 PID: 1 Comm: swapper/0 Not tainted 6.7.0 #1"),
        (1.2, "Hardware name: DioxusBox Virtual Machine"),
        (1.2, "Call Trace:"),
        (1.2, " dump_stack_lvl+0x3a/0x50"),
        (1.2, " panic+0x103/0x2b0"),
        (1.2, " mount_root_generic+0x1f0/0x2b0"),
        (1.2, " prepare_namespace+0x16b/0x1a0"),
        (1.2, " kernel_init_freeable+0x1f0/0x240"),
        (1.2, " kernel_init+0x1a/0x140"),
        (1.2, " ret_from_fork+0x2f/0x50"),
        (1.2, "---[ end Kernel panic - not syncing: VFS: Unable to mount root fs on unknown-block(0,0) ]---"),
    ];
    
    for i in 5..lines.len() {
        let prev = lines[i - 1].0;
        let inc = rng.random_range(0.001_f32..=0.003_f32);
        lines[i].0 = if i <= 5 { 1.2 } else { prev } + inc;
    }
    
    rsx! {
        div {
            class: "panic",
            for (i, (stamp, line)) in lines.iter().enumerate() {
                span {
                    class: "panic-line",
                    style: format!("--i: {}", i + 1),
                    {format!("[    {:<6.6}]: {}", stamp, line)}
                }
            }
        }
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
        
        if *KERNEL_PANIC.read() {
            KernelPanic {}
        } else {
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
            Bar {}
            
            div {
                class: "windows",
                for window in os::WINDOWS.read().iter() {
                    Window {
                        key: "{window.id}",
                        instance: window.clone(),
                        
                        {(window.render)(window.id)}
                    }
                }
            }
        }
    }
}
