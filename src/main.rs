pub mod enums;
pub mod consts;
pub mod sys;
pub mod crazyerror;
pub mod info;
pub mod services;

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
pub const ALT_PRESSED: GlobalSignal<bool> = GlobalSignal::new(|| false);

// Router
#[derive(Routable, Clone, PartialEq, Eq)]
pub enum Route {
    #[route("/")]
    Full,
    #[route("/simple")]
    Simple,
}

fn main() {
    dioxus_cookie::init();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // global key states
    use_effect(move || {
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::JsCast as _;

        let Some(window) = web_sys::window() else {
            return;
        };

        // keydown
        let keydown_cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Alt" || e.key() == "AltGraph" {
                *ALT_PRESSED.write() = true;
            }
        }));
        let _ = window.add_event_listener_with_callback(
            "keydown",
            keydown_cb.as_ref().unchecked_ref(),
        );

        // keyup
        let keyup_cb = Closure::<dyn FnMut(web_sys::KeyboardEvent)>::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Alt" || e.key() == "AltGraph" {
                *ALT_PRESSED.write() = false;
            }
        }));
        let _ = window.add_event_listener_with_callback("keyup", keyup_cb.as_ref().unchecked_ref());

        // in case of alt+tab / focus loss
        let blur_cb = Closure::<dyn FnMut(web_sys::FocusEvent)>::wrap(Box::new(move |_e: web_sys::FocusEvent| {
            *ALT_PRESSED.write() = false;
        }));
        let _ = window.add_event_listener_with_callback("blur", blur_cb.as_ref().unchecked_ref());

        keydown_cb.forget();
        keyup_cb.forget();
        blur_cb.forget();
    });

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        Router::<Route> {}
    }
}

#[component]
fn KernelPanic() -> Element {
    let uptime = web_sys::window()
        .and_then(|w| w.performance())
        .map_or(0.0, |p| p.now() / 1000.0);
    
    let mut rng = rand::rng();
    
    let mut lines: Vec<(f64, &str)> = vec![
        (0.0, "Linux version 6.7.0 (inpr@inprsn) #1 SMP PREEMPT_DYNAMIC"),
        (0.0, "Command line: BOOT_IMAGE=/vmlinuz root=/dev/sda1 ro quiet"),
        (0.0, "x86/fpu: Supporting XSAVE feature 0x001: 'x87 floating point registers'"),
        (0.0, "x86/fpu: Supporting XSAVE feature 0x002: 'SSE registers'"),
        (0.0, "x86/fpu: Enabled xstate features 0x3, context size is 832 bytes, using 'compacted' format."),
        // We will append to these stamps later onward
        (uptime, "systemd[1]: Caught <SIGTERM>, shutting down."),
        (uptime, "systemd[1]: Freezing execution."),
        (uptime, "Kernel panic - not syncing: Attempted to kill init! exitcode=0x0000000f"),
        (uptime, "CPU: 0 PID: 1 Comm: systemd Not tainted 6.7.0-arch1-1 #1"),
        (uptime, "Hardware name: DioxusBox Virtual Machine"),
        (uptime, "Call Trace:"),
        (uptime, " <TASK>"),
        (uptime, " dump_stack_lvl+0x47/0x60"),
        (uptime, " panic+0x103/0x2b0"),
        (uptime, " do_exit+0x9e6/0xad0"),
        (uptime, " do_group_exit+0x33/0xa0"),
        (uptime, " get_signal+0x8c0/0x910"),
        (uptime, " arch_do_signal_or_restart+0x3e/0x230"),
        (uptime, " exit_to_user_mode_prepare+0x10f/0x1c0"),
        (uptime, " syscall_exit_to_user_mode+0x16/0x40"),
        (uptime, " do_syscall_64+0x44/0x90"),
        (uptime, " entry_SYSCALL_64_after_hwframe+0x6e/0xd8"),
        (uptime, " </TASK>"),
        (uptime, "---[ end Kernel panic - not syncing: Attempted to kill init! exitcode=0x0000000f ]---"),
    ];
    
    for i in 5..lines.len() {
        let prev = lines[i - 1].0;
        let inc = rng.random_range(0.001_f64..=0.003_f64);
        lines[i].0 = if i <= 5 { uptime } else { prev } + inc;
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
        document::Link { rel: "stylesheet", href: "https://cdn.jsdelivr.net/npm/xterm@5.3.0/css/xterm.min.css" }
        document::Script { src: "https://cdn.jsdelivr.net/npm/xterm@5.3.0/lib/xterm.min.js" }
        document::Script { src: "https://cdn.jsdelivr.net/npm/xterm-addon-fit@0.8.0/lib/xterm-addon-fit.min.js" }
        
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
                for window in sys::WINDOWS.read().iter() {
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
