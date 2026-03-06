pub mod about;
pub mod tictactoe;
pub mod minesweeper;

use std::rc::Rc;
use std::time::Duration;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::enums::{Corner, ScreenCoordinates};
use self::about::new_about_instance;

// Global wm state
pub static WINDOWS: GlobalSignal<Vec<WindowInstance>> = Signal::global(|| vec![
    new_about_instance()
]);

#[derive(Clone, PartialEq)]
pub struct WindowInstanceProps {
    pub title: String,
    pub icon: String,
    pub position: ScreenCoordinates,
    pub size: ScreenCoordinates,
}

impl Default for WindowInstanceProps {
    fn default() -> Self {
        Self {
            title: "Untitled Window".to_owned(),
            icon: "mdi:window-restore".to_owned(),
            position: ScreenCoordinates::Percent { x: 50.0, y: 50.0 },
            size: ScreenCoordinates::Absolute { x: 800, y: 400 },
        }
    }
}

#[derive(Clone)]
pub struct WindowInstance {
    pub id: Uuid,
    pub props: WindowInstanceProps,
    pub focused: bool,
    pub dragging: bool,
    pub resize_corner: Option<Corner>,
    pub maximized: bool,
    pub iconified: bool,
    pub closing: bool,
    pub render: Rc<dyn Fn() -> Element>,
}

impl PartialEq for WindowInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.props == other.props
    }
}

impl WindowInstance {
    pub fn new(props: WindowInstanceProps, render: impl Fn() -> Element + 'static) -> Self {
        WindowInstance {
            id: Uuid::new_v4(),
            props,
            focused: true,
            dragging: false,
            resize_corner: None,
            maximized: false,
            iconified: false,
            closing: false,
            render: Rc::new(render)
        }
    }
}

pub fn spawn_window(mut instance: WindowInstance) -> Uuid {
    let id = instance.id;
    
    WINDOWS.with_mut(|windows| {
        // If a window other than this one is currently being dragged or resized,
        // this shouldn't be put in focus
        instance.focused = !windows.iter()
            .any(|window| window.dragging || window.resize_corner.is_some());
        
        for window in windows.iter_mut() {
            window.focused = window.dragging || window.resize_corner.is_some();
        }
        
        windows.push(instance);
    });
    
    id
}

pub fn resize_window(id: Uuid, new_size: ScreenCoordinates) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.props.size = new_size;
        }
    });
}

pub fn move_window(id: Uuid, new_position: ScreenCoordinates) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.props.position = new_position;
        }
    });
}

pub fn set_window_focused(id: Uuid, focused: bool) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.focused = focused;
            
            if focused {
                for other_instance in windows.iter_mut().filter(|window| window.id != id) {
                    other_instance.focused = false;
                }
            }
        }
    });
}

pub fn set_window_dragging(id: Uuid, dragging: bool) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.dragging = dragging;
        }
    });
}

pub fn set_window_resize_corner(id: Uuid, corner: Option<Corner>) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.resize_corner = corner;
        }
    });
}

pub fn set_window_maximized(id: Uuid, maximized: bool) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.maximized = maximized;
        }
    });
}

pub fn set_window_iconified(id: Uuid, iconified: bool) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.iconified = iconified;
        }
    });
}

pub fn force_close_window(id: Uuid) {
    WINDOWS.with_mut(|windows| {
        if let Some(index) = windows.iter().position(|window| window.id == id) {
            windows.remove(index);
        }
    });
}

pub fn close_window(id: Uuid) {
    WINDOWS.with_mut(|windows| {
        if let Some(window) = windows.iter_mut().find(|window| window.id == id) {
            window.closing = true;
            
            let id = window.id;
            spawn(async move {
               gloo_timers::future::sleep(Duration::from_millis(500)).await;
               force_close_window(id);
            });
        }
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