use dioxus::{html::input_data::MouseButton, prelude::*};

use crate::windows::{self, WINDOWS};

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
                            
                            if window.focused {
                                classes.push("focused");
                            }
                            
                            classes.join(" ")
                        },
                        
                        onmousedown: {
                            let id = window.id;
                            let focused = window.focused;
                            move |evt| {
                                if evt.trigger_button() == Some(MouseButton::Auxiliary) {
                                    windows::close_window(id);
                                } else {
                                    windows::set_window_focused(id, !focused);
                                }
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
