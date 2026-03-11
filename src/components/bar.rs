use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::sys::{self, WINDOWS};

pub const BAR_HEIGHT_PX: u32 = 24;

#[component]
pub fn Bar() -> Element {
    rsx! {
        div {
            class: "bar",
            
            div {
                class: "bar-tasks",
                for window in WINDOWS.read().iter().filter(|w| !w.closing) {
                    div {
                        key: "{window.id}",
                        class: {
                            let mut classes = vec!["bar-task"];
                            
                            if window.focused && !window.iconified {
                                classes.push("focused");
                            }
                            
                            if window.iconified {
                                classes.push("iconified");
                            }
                            
                            classes.join(" ")
                        },
                        
                        onmousedown: {
                            let id = window.id;
                            let focused = window.focused;
                            let iconified = window.iconified;
                            move |evt| if evt.trigger_button() == Some(MouseButton::Auxiliary) {
                                sys::call_on_close(id);
                                sys::close_window(id);
                            } else if !iconified && focused {
                                sys::with_window_mut(id, |window| window.iconified = true);
                                sys::set_window_focused(id, false);
                            } else {
                                sys::with_window_mut(id, |window| window.iconified = false);
                                sys::set_window_focused(id, true);
                            }
                        },
                        
                        span {
                            {window.props.title.clone()}
                        }
                    }
                }
            }
        }
    }
}
