use std::rc::Rc;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::enums::{Corner, ScreenCoordinates};

#[derive(Clone)]
pub struct WindowInstanceProps {
    pub title: String,
    pub icon: String,
    pub resizable: bool,
    pub position: ScreenCoordinates,
    pub size: ScreenCoordinates,
    pub on_close: Option<Rc<dyn Fn(Uuid)>>,
}

impl Default for WindowInstanceProps {
    fn default() -> Self {
        Self {
            title: "Untitled Window".to_owned(),
            icon: "mdi:window-restore".to_owned(),
            resizable: true,
            position: ScreenCoordinates::Percent { x: 50.0, y: 50.0 },
            size: ScreenCoordinates::Absolute { x: 800, y: 400 },
            on_close: None,
        }
    }
}

impl PartialEq for WindowInstanceProps {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title &&
        self.icon == other.icon &&
        self.resizable == other.resizable &&
        self.position == other.position &&
        self.size == other.size
    }
}

#[derive(Clone)]
pub struct WindowInstance {
    pub id: Uuid,
    pub props: WindowInstanceProps,
    pub no_transition: bool,
    pub focused: bool,
    pub dragging: bool,
    pub resize_corner: Option<Corner>,
    pub maximized: bool,
    pub iconified: bool,
    pub closing: bool,
    pub render: Rc<dyn Fn(Uuid) -> Element>,
}

impl PartialEq for WindowInstance {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.props == other.props
    }
}

impl WindowInstance {
    pub fn new(props: WindowInstanceProps, render: impl Fn(Uuid) -> Element + 'static) -> Self {
        WindowInstance {
            id: Uuid::new_v4(),
            props,
            no_transition: false,
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