use std::sync::atomic::Ordering;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::sys::WINDOWS;

use super::WindowInstance;

pub struct Process {
    pub id: u32,
    pub name: String,
    pub(super) windows: Vec<Uuid>
}

impl Process {
    pub fn new(name: &str) -> Self {
        Self {
            id: super::PID_COUNTER.fetch_add(1, Ordering::SeqCst),
            name: name.to_owned(),
            windows: Vec::new(),
        }
    }
    
    pub fn windows_len(&self) -> usize {
        let wids = self.windows.clone();
        
        WINDOWS.with(|windows| windows.iter().filter(|w| wids.contains(&w.id) && !w.closing).count())
    }
    
    pub fn has_window(&self, window_id: Uuid) -> bool {
        self.windows.contains(&window_id)
    }
    
    pub fn add_window(&mut self, mut window: WindowInstance) {
        self.windows.push(window.id);
        
        super::WINDOWS.with_mut(|windows| {
            // If a window other than this one is currently being dragged or resized,
            // this shouldn't be put in focus
            window.focused = !windows.iter()
                .any(|window| window.dragging || window.resize_corner.is_some());
            
            for window in windows.iter_mut() {
                window.focused = window.dragging || window.resize_corner.is_some();
            }
            
            windows.push(window);
        });
    }
    
    pub fn with_window(&self, id: Uuid, callback: impl Fn(&WindowInstance)) {
        super::with_window(id, callback);
    }
    
    pub fn with_window_mut(&mut self, id: Uuid, callback: impl Fn(&mut WindowInstance)) {
        super::with_window_mut(id, callback);
    }
    
    pub fn close_all_windows(&mut self) {
        super::retain_windows(|window| !self.windows.contains(&window.id));
        self.windows.clear();
    }
}