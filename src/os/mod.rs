mod process;
pub use process::Process;

mod wm;
pub use wm::{WINDOWS, with_window, with_window_mut, set_window_focused, call_on_close, force_close_window, close_window, retain_windows};
pub use wm::window::{WindowInstance, WindowInstanceProps};

use std::sync::atomic::AtomicU32;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::KERNEL_PANIC;

// Global os state
pub static PID_COUNTER: AtomicU32 = AtomicU32::new(3);
pub static PROCESSES: GlobalSignal<Vec<Process>> = Signal::global(|| {
    // get the first window id, should be the about window
    let first_wid = WINDOWS.with(|windows| windows.first().map(|w| w.id).unwrap_or_default());
    vec![
        Process {
            id: 1,
            name: "systemd".to_owned(),
            windows: vec![],
        },
        Process {
            id: 2,
            name: "about".to_owned(),
            windows: vec![first_wid],
        }
    ]
});

pub fn spawn_process(process: Process) {
    PROCESSES.with_mut(|processes| {
        processes.push(process);
    });
}

pub fn with_process<F>(process_id: u32, f: F)
where
    F: FnOnce(&Process),
{
    PROCESSES.with(|processes| {
        let process = processes.iter().find(|p| p.id == process_id);
        if let Some(process) = process {
            f(process);
        }
    });
}

pub fn with_process_mut<F>(process_id: u32, f: F)
where
    F: FnOnce(&mut Process),
{
    PROCESSES.with_mut(|processes| {
        let process = processes.iter_mut().find(|p| p.id == process_id);
        if let Some(process) = process {
            f(process);
        }
    });
}

pub fn with_process_with_window<F>(window_id: Uuid, f: F)
where
    F: FnOnce(&Process),
{
    PROCESSES.with(|processes| {
        let process = processes.iter().find(|p| p.windows.contains(&window_id));
        if let Some(process) = process {
            f(process);
        }
    });
}

pub fn with_process_with_window_mut<F>(window_id: Uuid, f: F)
where
    F: FnOnce(&mut Process),
{
    PROCESSES.with_mut(|processes| {
        let process = processes.iter_mut().find(|p| p.windows.contains(&window_id));
        if let Some(process) = process {
            f(process);
        }
    });
}

pub fn kill_process(pid: u32) {
    PROCESSES.with_mut(|processes| {
        if let Some(pos) = processes.iter().position(|p| p.id == pid) {
            processes[pos].close_all_windows();
            processes.remove(pos);
        }
    });
    
    // special pids
    if pid == 1 {
        *KERNEL_PANIC.write() = true;
    }
}