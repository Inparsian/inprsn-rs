pub mod icon;
use icon::DesktopIcon;

use dioxus::prelude::*;
use dioxus_free_icons::icons::ld_icons::{LdBomb, LdCat, LdCircleX, LdGrid3x3};

use crate::{windows, crazyerror};

#[component]
pub fn Desktop() -> Element {
    let mut selected_id: Signal<Option<u32>> = use_signal(|| None);
    
    rsx! {
        div { 
            class: "desktop",
            
            div {
                class: "desktop-empty",
                onmousedown: move |_| {
                    selected_id.set(None);
                },
            },
            
            div {
                class: "desktop-icons",
                
                DesktopIcon {
                    id: 0,
                    selected_id,
                    label: "about",
                    icon: LdCat,
                    on_open: move |()| {
                        windows::spawn_window(windows::about::new_about_instance());
                    }
                }
                
                DesktopIcon {
                    id: 1,
                    selected_id,
                    label: "crazy error",
                    icon: LdCircleX,
                    on_open: move |()| async move {
                        crazyerror::run().await;
                    }
                }
                
                div {
                    class: "desktop-icons-row",
                    
                    DesktopIcon {
                        id: 2,
                        selected_id,
                        label: "tictactoe",
                        icon: LdGrid3x3,
                        on_open: move |()| async move {
                            windows::spawn_window(windows::tictactoe::new_tictactoe_instance());
                        }
                    }
                    
                    DesktopIcon {
                        id: 3,
                        selected_id,
                        label: "minesweeper",
                        icon: LdBomb,
                        on_open: move |()| async move {
                            windows::spawn_window(windows::minesweeper::new_minesweeper_instance());
                        }
                    }
                }
            }
        }
    }
}