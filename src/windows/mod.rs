pub mod about;

use std::rc::Rc;
use std::time::Duration;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::enums::ScreenCoordinates;
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
            closing: false,
            render: Rc::new(render)
        }
    }
}

pub fn spawn(instance: WindowInstance) -> Uuid {
    let id = instance.id;
    
    WINDOWS.with_mut(|windows| {
        for window in windows.iter_mut() {
            window.focused = false;
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

pub fn focus_window(id: Uuid) {
    WINDOWS.with_mut(|windows| {
        if let Some(instance) = windows.iter_mut().find(|window| window.id == id) {
            instance.focused = true;
            
            for other_instance in windows.iter_mut().filter(|window| window.id != id) {
                other_instance.focused = false;
            }
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
            
            // Remove this *after* our timeout elapses
            let id = window.id;
            use_future(move || async move {
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

        use_future(move || {
            let to_close = to_close.clone();
            async move {
                gloo_timers::future::sleep(Duration::from_millis(500)).await;
                
                for id in to_close {
                    force_close_window(id);
                }
            }
        });
    });
}