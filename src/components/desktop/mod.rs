pub mod icon;

use icon::DesktopIcon;

use dioxus::{core::spawn_forever, prelude::*};
use dioxus_free_icons::icons::ld_icons::{LdActivity, LdCat, LdCircleX, LdDrama, LdFolder, LdGrid3x3, LdTerminal};

use crate::{sys, apps, crazyerror};

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
                
                div {
                    class: "desktop-icons-row",
                    DesktopIcon {
                        id: 0,
                        selected_id,
                        label: "about",
                        icon: LdCat,
                        on_open: move |()| {
                            sys::spawn_process(apps::about::new_about_instance());
                        }
                    },
                    
                    DesktopIcon {
                        id: 1,
                        selected_id,
                        label: "tasks",
                        icon: LdActivity,
                        on_open: move |()| {
                            sys::spawn_process(apps::taskmanager::new_taskmanager_instance());
                        }
                    },
                    
                    DesktopIcon {
                        id: 2,
                        selected_id,
                        label: "terminal",
                        icon: LdTerminal,
                        on_open: move |()| {
                            sys::spawn_process(apps::terminal::new_terminal_instance());
                        }
                    },

                    DesktopIcon {
                        id: 3,
                        selected_id,
                        label: "file thing",
                        icon: LdFolder,
                        on_open: move |()| {
                            sys::spawn_process(apps::filething::new_filething_instance());
                        }
                    },
                },
                
                div {
                    class: "desktop-icons-row",
                    DesktopIcon {
                        id: 4,
                        selected_id,
                        label: "crazy error",
                        icon: LdCircleX,
                        on_open: move |()| {
                            spawn_forever(crazyerror::run());
                        },
                    },
                    
                    DesktopIcon {
                        id: 5,
                        selected_id,
                        label: "hydra",
                        icon: LdDrama,
                        on_open: move |()| {
                            sys::spawn_process(apps::hydra::new_hydra_instance());
                        }
                    },
                },
                
                div {
                    class: "desktop-icons-row",
                    DesktopIcon {
                        id: 6,
                        selected_id,
                        label: "tictactoe",
                        icon: LdGrid3x3,
                        on_open: move |()| async move {
                            sys::spawn_process(apps::tictactoe::new_tictactoe_instance());
                        }
                    },
                },
            }
        }
    }
}