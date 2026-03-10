pub(super) mod window;
use window::WindowInstance;

use std::time::Duration;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::{apps::about::about_window, os::PROCESSES};

pub static WINDOWS: GlobalSignal<Vec<WindowInstance>> = GlobalSignal::new(|| vec![
    // pushing to WINDOWS during DOM init is unsafe, so i do it here instead
    about_window(0)
]);

pub fn with_window<F>(window_id: Uuid, f: F)
where
    F: FnOnce(&WindowInstance),
{
    WINDOWS.with(|windows| {
        let window = windows.iter().find(|w| w.id == window_id);
        if let Some(window) = window {
            f(window);
        }
    });
}

pub fn with_window_mut<F>(window_id: Uuid, f: F)
where
    F: FnOnce(&mut WindowInstance),
{
    WINDOWS.with_mut(|windows| {
        let window = windows.iter_mut().find(|w| w.id == window_id);
        if let Some(window) = window {
            f(window);
        }
    });
}

pub fn call_on_close(window_id: Uuid) {
    let mut on_close = None;
    with_window_mut(window_id, |window| {
        on_close = window.props.on_close.clone();
    });
    
    if let Some(on_close) = on_close {
        on_close(window_id);
    }
}

// prefer this setter instead as it uses the global mut to unfocus
// other windows
pub fn set_window_focused(window_id: Uuid, focused: bool) {
    WINDOWS.with_mut(|windows| {
        if let Some(window) = windows.iter_mut().find(|w| w.id == window_id) {
            window.focused = focused;
            
            if focused {
                for other in windows.iter_mut().filter(|w| w.id != window_id) {
                    other.focused = false;
                }
            }
        }
    });
}

pub fn force_close_window(window_id: Uuid) {
    WINDOWS.with_mut(|windows| if let Some(pos) = windows.iter().position(|w| w.id == window_id) {
        windows.remove(pos);
        
        // processes shouldn't reference this wid at this point
        PROCESSES.with_mut(|processes| {
            for process in processes.iter_mut() {
                process.windows.retain(|w| *w != window_id);
            }
        });
    });
}

pub fn close_window(window_id: Uuid) {
    with_window_mut(window_id, |window| {
        window.closing = true;
        
        let id = window.id;
        spawn(async move {
            gloo_timers::future::sleep(Duration::from_millis(500)).await;
            force_close_window(id);
        });
    });
}

pub fn retain_windows(mut predicate: impl FnMut(&WindowInstance) -> bool) {
    WINDOWS.with_mut(|windows| {
        let mut to_close: Vec<Uuid> = Vec::new();

        for window in windows.iter_mut() {
            if !predicate(window) && !window.closing {
                window.closing = true;
                to_close.push(window.id);
            }
        }

        spawn(async move {
            gloo_timers::future::sleep(Duration::from_millis(500)).await;
            
            for id in to_close {
                force_close_window(id);
            }
        });
    });
}