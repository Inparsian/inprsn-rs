use dioxus::prelude::*;
use dioxus_free_icons::{Icon, IconShape};

#[derive(Props, Clone, PartialEq)]
pub struct DesktopIconProps<I: IconShape + Clone + PartialEq + 'static> {
    pub id: u32,
    pub selected_id: Signal<Option<u32>>,
    pub label: String,
    pub icon: I,
    #[props(default = None)]
    pub on_open: Option<EventHandler>,
}

#[component]
pub fn DesktopIcon<I: IconShape + Clone + PartialEq + 'static>(mut props: DesktopIconProps<I>) -> Element {
    let mut opened = use_signal(|| false);
    let selected = use_memo(move || props.selected_id.read().is_some_and(|id| id == props.id));

    rsx! {
        div {
            class: if *selected.read() {
                "desktop-icon selected"
            } else {
                "desktop-icon"
            },
            
            onmouseup: move |_| {
                if *selected.read() {
                    if !*opened.read() {
                        if let Some(f) = props.on_open.as_ref() {
                            f.call(());
                        }
                        opened.set(true);
                    } else {
                        opened.set(false);
                    }
                } else {
                    props.selected_id.set(Some(props.id));
                }
            },
            
            Icon {
                class: "desktop-icon-icon",
                fill: "#e2e2e2",
                icon: props.icon,
            }
            
            span {
                class: "desktop-icon-label",
                {props.label}
            }
        }
    }
}
