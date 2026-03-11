mod objects;

use std::time::Duration;
use dioxus::prelude::*;

use crate::KERNEL_PANIC;
use crate::enums::{ScreenCoordinates};
use crate::os::{self, Process, WindowInstance, WindowInstanceProps};

static MARISA_ACTIVE: GlobalSignal<bool> = GlobalSignal::new(|| false);

#[derive(Clone)]
enum MarisaEvent {
    SpanOne {
        position: ScreenCoordinates,
    },
    SpanTwo {
        position: ScreenCoordinates,
    },
    Clear,
}

impl MarisaEvent {
    fn spawn(&self) -> Option<WindowInstance> {
        match self {
            MarisaEvent::SpanOne { position } => Some(WindowInstance::new(WindowInstanceProps {
                title: "HALLO :D".to_owned(),
                position: *position,
                resizable: false,
                size: ScreenCoordinates::Absolute { x: 200, y: 100 },
                ..Default::default()
            }, move |_| rsx! {
                div {
                    class: "flex w-full h-full justify-center items-center text-center",
                    "HALLO :D"
                }
            })),
            
            MarisaEvent::SpanTwo { position } => Some(WindowInstance::new(WindowInstanceProps {
                title: "iswm has stopped responding".to_owned(),
                resizable: false,
                position: *position,
                size: ScreenCoordinates::Absolute { x: 300, y: 150 },
                ..Default::default()
            }, move |_| rsx! {
                div {
                    class: "flex flex-col space-y-4 w-full h-full justify-center items-center text-center text-wrap",
                    
                    span {
                        "iswm has stopped responding. Do you wish to restart it?"
                    }
                    
                    div {
                        class: "flex flex-row space-x-6 items-center",
                        a {
                            class: "text-blue-500 hover:underline",
                            "Yes"
                        }
                        a {
                            class: "text-blue-500 hover:underline",
                            "No"
                        }
                    }
                }
            })),
            
            MarisaEvent::Clear => None,
        }
    }
}

pub fn credits_window() -> WindowInstance {
    WindowInstance::new(WindowInstanceProps {
        title: "crazyerror".to_owned(),
        resizable: false,
        size: ScreenCoordinates::Absolute { x: 300, y: 150 },
        ..Default::default()
    }, move |_| rsx! {
        div {
            class: "flex flex-col w-full h-full justify-center items-center text-center",
            span {
                "CrazyError by inpr.sn"
            }
            span {
                "created with osu! beatmap editor"
            }
            span {
                "music rights go to IOSYS"
            }
        }
    })
}

pub async fn run() {
    let mut process = Process::new("crazyerror");
    if *MARISA_ACTIVE.read() {
        return;
    }
    let pid = process.id;
    let credits = credits_window();
    let id = credits.id;
    process.add_window(credits);
    os::spawn_process(process);
    *MARISA_ACTIVE.write() = true;
    gloo_timers::future::sleep(Duration::from_millis(2000)).await;
    os::close_window(id);
    hallo(pid).await;
}

async fn hallo(pid: u32) {
    // stop this immediately if one of the following is true:
    // 1. this pid does not exist anymore
    // 2. kernel panic
    if !os::has_pid(pid) || *KERNEL_PANIC.read() {
        *MARISA_ACTIVE.write() = false;
        return;
    }
    
    let mut events = objects::EVENTS.to_vec();
    events.sort_by_key(|(time, _)| std::cmp::Reverse(*time));
    
    // :D
    let audio = web_sys::HtmlAudioElement::new_with_src("marisa_stole_the_precious_thing.mp3").unwrap();
    if audio.play().is_err() {
        error!("Could not play audio");
        *MARISA_ACTIVE.write() = false;
        return;
    }
    
    // DEBUG: skip & pop every event before time threshold
    //audio.set_current_time(204.593);
    //events.retain(|(time, _)| *time > 204_593);
    
    loop {
        let current_ms = audio.current_time() * 1000.0;
        
        // Pop all events that are due or overdue
        while events.last().is_some_and(|(time, _)| (*time as f64 - current_ms) < 5.0) {
            if !os::has_pid(pid) || *KERNEL_PANIC.read() {
                break;
            }
            
            if let Some((_, event)) = events.pop() {
                match event {
                    MarisaEvent::SpanOne { .. }
                   | MarisaEvent::SpanTwo { .. } => if let Some(window) = event.spawn() {
                       os::with_process_mut(pid, |process| process.add_window(window));
                    },
                    _ => os::with_process_mut(pid, |process| process.close_all_windows()),
                }
            }
        }
        
        if events.is_empty() || !os::has_pid(pid) || *KERNEL_PANIC.read() {
            break;
        }
        
        gloo_timers::future::sleep(Duration::from_millis(5)).await;
    }
    
    if os::has_pid(pid) && !*KERNEL_PANIC.read() {
        os::with_process_mut(pid, |process| {
            process.close_all_windows();
            process.add_window(credits_window());
        });
        let remaining_duration = audio.duration() - audio.current_time();
        gloo_timers::future::sleep(Duration::from_millis((remaining_duration * 1000.0) as u64)).await;
    } else if let Err(e) = audio.pause() {
        error!("Failed to pause audio: {:?}", e);
    }
    *MARISA_ACTIVE.write() = false;
}