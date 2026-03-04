use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::windows::{WindowInstance, WindowInstanceProps};

enum AboutPage {
    Home,
    Projects,
}

pub fn new_about_instance() -> WindowInstance {
    WindowInstance::new(WindowInstanceProps {
        title: "inparsian".to_owned(),
        size: ScreenCoordinates::Absolute { x: 520, y: 280 },
        ..Default::default()
    }, move || rsx! {
        WindowAbout {}
    })
}

#[component]
fn HomePage() -> Element {
    rsx! {
        div {
            id: "about-home-page",
            
            img {
                src: "https://github.com/Inparsian.png",
                alt: "Persona Image",
                class: "about-pfp border border-dotted border-[#1B1B1B] p-px grayscale",
            }
    
            div {
                p {
                    "hi, i'm inparsian. i'm some dumb 20 y.o. american dude who makes software"
                }
                
                br {}
                
                a {
                    href: "/simple",
                    "nojs version"
                },
                
                br {}
                
                a {
                    onclick: |_| async {
                        crate::marisa::hallo().await;
                    },
                    "click for a cool easter egg"
                }
            }
        }
    }
}

#[component]
pub fn ProjectsPage() -> Element {
    rsx! {
        div {
            id: "about-projects-page",
            h2 {
                "projects"
            }
                
            ul {
                for (name, desc, link) in crate::info::PROJECTS {
                    li {
                        a {
                            href: link.to_owned(),
                            target: "_blank",
                            {name.to_owned()}
                        },
                        span {
                            {format!(" - {desc}")}
                        }
                    },
                }
            }
        }
    }
}

#[component]
pub fn WindowAbout() -> Element {
    let mut page = use_signal(|| AboutPage::Home);
    rsx! {
        div {
            id: "about-container",
            class: "bg-[#131313] p-8 flex flex-row w-full h-full",

            div {
                id: "about-content",

                div {
                    class: "about-header",
                    h1 {
                        "inpr.sn"
                    }
                    
                    div {
                        class: "about-links",
                        a {
                            class: if matches!(*page.read(), AboutPage::Home) { "active" } else { "" },
                            onclick: move |_| {
                                *page.write() = AboutPage::Home;
                            },
                            "about"
                        },
                        a {
                            class: if matches!(*page.read(), AboutPage::Projects) { "active" } else { "" },
                            onclick: move |_| {
                                *page.write() = AboutPage::Projects;
                            },
                            "projects"
                        }
                    }
                }
                
                div {
                    if matches!(*page.read(), AboutPage::Home) {
                        HomePage {}
                    } else if matches!(*page.read(), AboutPage::Projects) {
                        ProjectsPage {}
                    } else {
                        span {
                            "Unknown page"
                        }
                    }
                }
            }
        }
    }
}