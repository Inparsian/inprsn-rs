use std::rc::Rc;
use dioxus::prelude::*;

use crate::enums::ScreenCoordinates;
use crate::sys::{self, Process, WindowInstance, WindowInstanceProps};

#[derive(Clone, Copy, PartialEq)]
enum AboutPage {
    Home,
    Projects,
    Socials,
}

impl std::fmt::Display for AboutPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AboutPage::Home => write!(f, "home"),
            AboutPage::Projects => write!(f, "projects"),
            AboutPage::Socials => write!(f, "socials"),
        }
    }
}

impl AboutPage {
    fn all() -> &'static [AboutPage] {
        &[AboutPage::Home, AboutPage::Projects, AboutPage::Socials]
    }
}

pub fn about_window(pid: u32) -> WindowInstance {
    WindowInstance::new(WindowInstanceProps {
        title: "inparsian".to_owned(),
        size: ScreenCoordinates::Absolute { x: 520, y: 240 },
        on_close: Some(Rc::new(move |_| {
            sys::kill_process(pid);
        })),
        ..Default::default()
    }, move |_| rsx! {
        WindowAbout {}
    })
}

pub fn new_about_instance() -> Process {
    let mut process = Process::new("about");
    let pid = process.id;
    process.add_window(about_window(pid));
    
    process
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
                
                noscript {
                    p {
                        "i see you have javascript disabled; this is the full desktop-as-a-site version of my site and relies heavily on js to function properly. page navigation will also not work. consider enabling js or visiting the nojs version below."
                    }
                
                    br {}
                }
                
                a {
                    href: "/simple",
                    "nojs version"
                },
            }
        }
    }
}

#[component]
fn ProjectsPage() -> Element {
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
fn SocialsPage() -> Element {
    rsx! {
        div {
            id: "about-socials-page",
            h2 {
                "socials"
            }

            ul {
                for (name, _, link) in crate::info::SOCIALS {
                    li {
                        a {
                            href: link.to_owned(),
                            target: "_blank",
                            {name.to_owned()}
                        },
                    },
                }
            }
        }
    }
}

#[component]
fn WindowAbout() -> Element {
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
                        id: "about-title",
                        for (i, c) in "inpr.sn".char_indices() {
                            span {
                                id: "title-char",
                                style: format!("--i: {}", i + 1),
                                {c.to_string()}
                            }
                        }
                    }
                    
                    div {
                        class: "about-links",
                        for about_page in AboutPage::all() {
                            a {
                                class: if *page.read() == *about_page { "active" } else { "" },
                                onclick: move |_| *page.write() = *about_page,
                                {about_page.to_string()}
                            },
                        }
                    }
                }
                
                div {
                    match *page.read() {
                        AboutPage::Home => rsx! { HomePage {} },
                        AboutPage::Projects => rsx! { ProjectsPage {} },
                        AboutPage::Socials => rsx! { SocialsPage {} },
                    }
                }
            }
        }
    }
}