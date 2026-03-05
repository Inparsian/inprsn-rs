use dioxus::html::geometry::{PageSpace, euclid::Point2D};
use dioxus::prelude::*;

use crate::enums::{Corner, ScreenCoordinates};
use crate::components::BAR_HEIGHT_PX;
use crate::windows::{self, WindowInstance};

#[derive(Props, Clone, PartialEq)]
pub struct WindowProps {
    instance: WindowInstance,
    children: Element,
}

#[component]
pub fn Window(props: WindowProps) -> Element {
    // Dragging & resizing state
    let mut drag_offset = use_signal(|| (0, 0));
    let mut resize_offset = use_signal(|| (0, 0));
    
    let (left, top, transform) = if props.instance.maximized {
        ("0px".to_owned(), "0px".to_owned(), "none".to_owned())
    } else {
        match props.instance.props.position {
            ScreenCoordinates::Absolute { x, y } => (format!("{x}px"), format!("{y}px"), "none".to_owned()),
            // Percent is an initial state for the window position until we can handle the math
            // later, this is just so we can use X% of the screen immediately without JS wizardry
            // and as a fallback if JS is disabled
            ScreenCoordinates::Percent { x, y } => {
                (
                    format!("{x}vw"),
                    format!("calc({y}vh - {BAR_HEIGHT_PX}px)"),
                    format!("translate(-{x}%, -{y}%)"),
                )
            },
        }
    };
    
    // We can perform the percent math on the client-side if JS is enabled
    // Window managers don't typically 'pin' windows to a percentage of the screen,
    // this is for realism purposes
    use_effect(move || {
        if let ScreenCoordinates::Percent { x, y } = props.instance.props.position
            && let Some(window) = web_sys::window()
        {
            let window_width = window.inner_width().unwrap().as_f64().unwrap();
            let window_height = window.inner_height().unwrap().as_f64().unwrap() - BAR_HEIGHT_PX as f64;
            let (width, height) = match props.instance.props.size {
                ScreenCoordinates::Absolute { x, y } => (x, y),
                ScreenCoordinates::Percent { x, y } => {(
                    (x * (window_width as f32 / 100.0)) as i32,
                    (y * (window_height as f32 / 100.0)) as i32,
                )},
            };

            let x_offset = (width as f32) * (x / 100.0);
            let y_offset = (height as f32) * (y / 100.0);

            windows::move_window(props.instance.id, ScreenCoordinates::Absolute {
                x: x.mul_add(window_width as f32 / 100.0, -x_offset).ceil() as i32,
                y: y.mul_add(window_height as f32 / 100.0, -y_offset).ceil() as i32,
            });
        }
    });
    
    let mut resize = move |page_coordinates: Point2D<f64, PageSpace>| {
        let (off_x, off_y) = *resize_offset.read();
        let (pos_x, pos_y) = match props.instance.props.position {
            ScreenCoordinates::Absolute { x, y } => (x, y),
            ScreenCoordinates::Percent { .. } => (0, 0), // TODO: handle percent math
        };

        if let Some(corner) = props.instance.resize_corner {
            let dx = page_coordinates.x as i32 - off_x;
            let dy = page_coordinates.y as i32 - off_y;

            let (cur_w, cur_h) = match props.instance.props.size {
                ScreenCoordinates::Absolute { x, y } => (x, y),
                ScreenCoordinates::Percent { .. } => (0, 0), // TODO: handle percent math
            };

            // Minimum size to prevent collapsing/negative sizing
            let min_w: i32 = 120;
            let min_h: i32 = 80;

            match corner {
                Corner::TopLeft => {
                    let new_w = (cur_w - dx).max(min_w);
                    let new_h = (cur_h - dy).max(min_h);
                    let new_x = if cur_w != new_w { pos_x + dx } else { pos_x };
                    let new_y = if cur_h != new_h { pos_y + dy } else { pos_y };
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: new_w, y: new_h });
                    windows::move_window(props.instance.id, ScreenCoordinates::Absolute { x: new_x, y: new_y });
                },
                Corner::TopCenter => {
                    let new_h = (cur_h - dy).max(min_h);
                    let new_y = if cur_h != new_h { pos_y + dy } else { pos_y };
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: cur_w, y: new_h });
                    windows::move_window(props.instance.id, ScreenCoordinates::Absolute { x: pos_x, y: new_y });
                },
                Corner::TopRight => {
                    let new_w = (cur_w + dx).max(min_w);
                    let new_h = (cur_h - dy).max(min_h);
                    let new_y = if cur_h != new_h { pos_y + dy } else { pos_y };
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: new_w, y: new_h });
                    windows::move_window(props.instance.id, ScreenCoordinates::Absolute { x: pos_x, y: new_y });
                },
                Corner::CenterRight => {
                    let new_w = (cur_w + dx).max(min_w);
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: new_w, y: cur_h });
                },
                Corner::BottomRight => {
                    let new_w = (cur_w + dx).max(min_w);
                    let new_h = (cur_h + dy).max(min_h);
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: new_w, y: new_h });
                },
                Corner::BottomCenter => {
                    let new_h = (cur_h + dy).max(min_h);
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: cur_w, y: new_h });
                },
                Corner::BottomLeft => {
                    let new_w = (cur_w - dx).max(min_w);
                    let new_h = (cur_h + dy).max(min_h);
                    let new_x = if cur_w != new_w { pos_x + dx } else { pos_x };
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: new_w, y: new_h });
                    windows::move_window(props.instance.id, ScreenCoordinates::Absolute { x: new_x, y: pos_y });
                },
                Corner::CenterLeft => {
                    let new_w = (cur_w - dx).max(min_w);
                    let new_x = if cur_w != new_w { pos_x + dx } else { pos_x };
                    windows::resize_window(props.instance.id, ScreenCoordinates::Absolute { x: new_w, y: cur_h });
                    windows::move_window(props.instance.id, ScreenCoordinates::Absolute { x: new_x, y: pos_y });
                },
            }

            resize_offset.set((page_coordinates.x as i32, page_coordinates.y as i32));
        }
    };
    
    rsx! {
        if props.instance.dragging {
            div {
                style: "position: fixed; inset: 0; z-index: 9999; cursor: grabbing;",
                onmousemove: move |evt| {
                    let window = web_sys::window()
                        .expect("Failed to get window");
                    
                    let (window_width, window_height) = {
                        let width = window.inner_width().unwrap().as_f64().unwrap() as i32;
                        let height = window.inner_height().unwrap().as_f64().unwrap() as i32 - BAR_HEIGHT_PX as i32;
                        (width, height)
                    };
                    
                    let (width, height) = match props.instance.props.size {
                        ScreenCoordinates::Absolute { x, y } => (x, y),
                        ScreenCoordinates::Percent { x, y } => {
                            ((window_width as f32 * x / 100.0) as i32, (window_height as f32 * y / 100.0) as i32)
                        },
                    };
                    
                    let (off_x, off_y) = *drag_offset.read();
                    let coordinates = evt.page_coordinates();
                    windows::move_window(props.instance.id, ScreenCoordinates::Absolute {
                        x: (coordinates.x as i32 - off_x).clamp(0, window_width - width),
                        y: (coordinates.y as i32 - off_y).clamp(0, window_height - height),
                    });
                },
                onmouseup: move |_| windows::set_window_dragging(props.instance.id, false),
            }
        }
        
        if props.instance.resize_corner.is_some() {
            div {
                style: {
                    let cursor = match props.instance.resize_corner {
                        Some(Corner::TopLeft) => "nw-resize",
                        Some(Corner::TopRight) => "ne-resize",
                        Some(Corner::TopCenter) => "n-resize",
                        Some(Corner::CenterRight) => "e-resize",
                        Some(Corner::BottomRight) => "se-resize",
                        Some(Corner::BottomCenter) => "s-resize",
                        Some(Corner::BottomLeft) => "sw-resize",
                        Some(Corner::CenterLeft) => "w-resize",
                        _ => "nwse-resize",
                    };
                    
                    format!("position: fixed; inset: 0; z-index: 9999; cursor: {cursor}")
                },
                onmousemove: move |evt| {
                    resize(evt.page_coordinates());
                },
                onmouseup: move |_| windows::set_window_resize_corner(props.instance.id, None)
            }
        }
        
        div {
            class: {
                let mut classes = vec!["window"];
                
                if props.instance.focused {
                    classes.push("focused");
                }
                
                if props.instance.closing {
                    classes.push("closing");
                }
                
                classes.join(" ")
            },
            style: {
                let (width, height) = if props.instance.maximized {
                    ("100%".to_owned(), format!("calc(100% - {BAR_HEIGHT_PX}px)"))
                } else {
                    match props.instance.props.size {
                        ScreenCoordinates::Absolute { x, y } => (format!("{}px", x as u32), format!("{}px", y as u32)),
                        ScreenCoordinates::Percent { .. } => ("0px".to_owned(), "0px".to_owned()) // TODO: handle percent math
                    }
                };
                
                format!("position: absolute; left: {left}; top: {top}; transform: {transform}; width: {width}; height: {height};")
            },
            onmousedown: move |_| {
                windows::set_window_focused(props.instance.id, true);
            },
            
            // inner div for animation & styling purposes so transform does not break % math
            div {
                class: "window-inner",
                if !props.instance.maximized {
                    for corner in Corner::all() {
                        div {
                            class: {
                                let corner = match corner {
                                    Corner::TopLeft => "top-left",
                                    Corner::TopCenter => "top-center",
                                    Corner::TopRight => "top-right",
                                    Corner::CenterRight => "center-right",
                                    Corner::BottomRight => "bottom-right",
                                    Corner::BottomCenter => "bottom-center",
                                    Corner::BottomLeft => "bottom-left",
                                    Corner::CenterLeft => "center-left",
                                };
                                
                                format!("window-corner {}", corner)
                            },
                            onmousedown: move |evt| {
                                evt.prevent_default();
                                let page_coordinates = evt.page_coordinates();
                                resize_offset.set((page_coordinates.x as i32, page_coordinates.y as i32));
                                windows::set_window_resize_corner(props.instance.id, Some(corner));
                            },
                        }
                    },
                }
                
                div {
                    class: "window-title-bar",
                    div {
                        class: "window-title-bar-draggable",
                        style: {
                            format!("cursor: {};", if props.instance.dragging {
                                "grabbing"
                            } else {
                                "grab"
                            })
                        },
                        onmousedown: move |evt| {
                            evt.prevent_default();
                            if !props.instance.maximized {
                                let element_coordinates = evt.element_coordinates();
                                drag_offset.set((element_coordinates.x as i32, element_coordinates.y as i32));
                                windows::set_window_dragging(props.instance.id, true);
                            }
                        },
                        span {
                            {props.instance.props.title}
                        }
                    }
                    div {
                        class: "window-title-bar-buttons",
                        button {
                            id: "maximize",
                            class: if props.instance.maximized {
                                "on"
                            } else {
                                ""
                            },
                            onclick: move |_| {
                                windows::set_window_maximized(props.instance.id, !props.instance.maximized);
                            },
                        }
                        button {
                            id: "close",
                            onclick: move |_| windows::close_window(props.instance.id),
                        }
                    }
                }
                
                div {
                    class: "window-content",
                    {props.children}
                }
            }
        }
    }
}