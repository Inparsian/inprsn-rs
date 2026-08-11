use std::rc::Rc;

use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::ld_icons::{
    LdArrowLeft, LdArrowRight, LdArrowUp, LdFile, LdFileText, LdFolder,
    LdFolderPlus, LdHome,
};

use crate::enums::ScreenCoordinates;
use crate::services::fs::{FilesystemData, FilesystemEntry, FILESYSTEM};
use crate::sys::{self, Process, WindowInstance, WindowInstanceProps};

pub fn new_filething_instance() -> Process {
    let mut process = Process::new("filething");
    let pid = process.id;
    process.add_window(WindowInstance::new(WindowInstanceProps {
        title: "file thing".to_owned(),
        size: ScreenCoordinates::Absolute { x: 600, y: 350 },
        on_close: Some(Rc::new(move |_| {
            sys::kill_process(pid);
        })),
        ..Default::default()
    }, move |_| rsx! {
        WindowFileThing {}
    }));

    process
}

#[derive(Clone, Copy, PartialEq)]
enum NewEntry {
    File,
    Directory,
}

impl NewEntry {
    fn label(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
        }
    }
}

fn parent_path(path: &str) -> String {
    match path.rsplit_once('/') {
        Some(("", _)) | None => "/".to_owned(),
        Some((parent, _)) => parent.to_owned(),
    }
}

fn entry_path(directory: &str, name: &str) -> String {
    if directory == "/" {
        format!("/{name}")
    } else {
        format!("{directory}/{name}")
    }
}

#[component]
fn WindowFileThing() -> Element {
    let mut current_path = use_signal(|| "/home/inparsian".to_owned());
    let mut address = use_signal(|| "/home/inparsian".to_owned());
    let mut history = use_signal(|| vec!["/home/inparsian".to_owned()]);
    let mut history_index = use_signal(|| 0_usize);
    let mut selected = use_signal(|| None::<FilesystemEntry>);
    let mut show_preview = use_signal(|| false);
    let mut open_menu = use_signal(|| None::<&'static str>);
    let mut new_entry = use_signal(|| None::<NewEntry>);
    let mut new_entry_name = use_signal(String::new);
    let mut status = use_signal(String::new);
    let mut refresh = use_signal(|| 0_u64);

    let entries = {
        let _ = refresh();
        FILESYSTEM
            .read()
            .expect("filesystem lock poisoned")
            .resolve_read_dir(&current_path(), None)
            .unwrap_or_default()
    };

    let mut navigate = move |target: String, add_to_history: bool| {
        let resolved = FILESYSTEM
            .read()
            .expect("filesystem lock poisoned")
            .resolve_path(&target, None);

        let Some(path) = resolved else {
            status.set(format!("Directory does not exist: {target}"));
            return;
        };

        if FILESYSTEM
            .read()
            .expect("filesystem lock poisoned")
            .resolve_read_dir(&path, None)
            .is_none()
        {
            status.set(format!("Not a directory: {path}"));
            return;
        }

        current_path.set(path.clone());
        address.set(path.clone());
        selected.set(None);
        status.set(String::new());

        if add_to_history {
            let next_index = history_index() + 1;
            let mut entries = history();
            entries.truncate(next_index);
            entries.push(path);
            history.set(entries);
            history_index.set(next_index);
        }
    };

    rsx! {
        div {
            class: "filething",

            if open_menu().is_some() {
                div {
                    class: "filething-menu-dismiss",
                    onclick: move |_| open_menu.set(None),
                }
            }

            div {
                class: "filething-menu-bar",
                div { class: "filething-menu",
                    button {
                        class: if open_menu() == Some("file") { "filething-menu-button active" } else { "filething-menu-button" },
                        onclick: move |_| open_menu.set(if open_menu() == Some("file") { None } else { Some("file") }),
                        "File"
                    }
                    if open_menu() == Some("file") {
                        div { class: "filething-menu-popup",
                            button {
                                onclick: move |_| {
                                    new_entry.set(Some(NewEntry::File));
                                    new_entry_name.set(String::new());
                                    open_menu.set(None);
                                },
                                Icon { icon: LdFileText }
                                "New file"
                            }
                            button {
                                onclick: move |_| {
                                    new_entry.set(Some(NewEntry::Directory));
                                    new_entry_name.set(String::new());
                                    open_menu.set(None);
                                },
                                Icon { icon: LdFolderPlus }
                                "New directory"
                            }
                        }
                    }
                }
                div { class: "filething-menu",
                    button {
                        class: if open_menu() == Some("view") { "filething-menu-button active" } else { "filething-menu-button" },
                        onclick: move |_| open_menu.set(if open_menu() == Some("view") { None } else { Some("view") }),
                        "View"
                    }
                    if open_menu() == Some("view") {
                        div { class: "filething-menu-popup",
                            button {
                                onclick: move |_| {
                                    show_preview.toggle();
                                    open_menu.set(None);
                                },
                                span {
                                    class: if show_preview() { "filething-menu-check checked" } else { "filething-menu-check" },
                                    "✓"
                                }
                                "Show preview"
                            }
                        }
                    }
                }
            }

            div {
                class: "filething-header",
                button {
                    class: "filething-header-button",
                    title: "Back",
                    disabled: history_index() == 0,
                    onclick: move |_| {
                        if history_index() > 0 {
                            let index = history_index() - 1;
                            history_index.set(index);
                            navigate(history()[index].clone(), false);
                        }
                    },
                    Icon { icon: LdArrowLeft }
                }
                button {
                    class: "filething-header-button",
                    title: "Forward",
                    disabled: history_index() + 1 >= history().len(),
                    onclick: move |_| {
                        let index = history_index() + 1;
                        if index < history().len() {
                            history_index.set(index);
                            navigate(history()[index].clone(), false);
                        }
                    },
                    Icon { icon: LdArrowRight }
                }
                button {
                    class: "filething-header-button",
                    title: "Up",
                    disabled: current_path() == "/",
                    onclick: move |_| navigate(parent_path(&current_path()), true),
                    Icon { icon: LdArrowUp }
                }
                button {
                    class: "filething-header-button",
                    title: "Home",
                    onclick: move |_| navigate("/home/inparsian".to_owned(), true),
                    Icon { icon: LdHome }
                }
                input {
                    class: "filething-address",
                    value: "{address}",
                    oninput: move |event| address.set(event.value()),
                    onkeydown: move |event| {
                        if event.key() == Key::Enter {
                            navigate(address(), true);
                        }
                    }
                }
                button {
                    class: "filething-header-button",
                    title: "Go to location",
                    onclick: move |_| navigate(address(), true),
                    Icon { icon: LdArrowRight }
                }
            }

            if let Some(kind) = new_entry() {
                div { class: "filething-create-dialog",
                    label { "New {kind.label()}" }
                    input {
                        autofocus: true,
                        placeholder: "Name",
                        value: "{new_entry_name}",
                        oninput: move |event| new_entry_name.set(event.value()),
                        onkeydown: move |event| {
                            if event.key() == Key::Enter {
                                let name = new_entry_name().trim().to_owned();
                                if name.is_empty() || name.contains('/') {
                                    status.set("Enter a name without a slash.".to_owned());
                                } else {
                                    let path = entry_path(&current_path(), &name);
                                    let result = match kind {
                                        NewEntry::File => FILESYSTEM.write().expect("filesystem lock poisoned").create_file(&path, None, &[]),
                                        NewEntry::Directory => FILESYSTEM.write().expect("filesystem lock poisoned").create_directory(&path, None),
                                    };
                                    match result {
                                        Ok(()) => {
                                            status.set(format!("Created {kind} {name}.", kind = kind.label()));
                                            new_entry.set(None);
                                            refresh += 1;
                                        }
                                        Err(error) => status.set(error),
                                    }
                                }
                            }
                        }
                    }
                    button {
                        class: "filething-dialog-button",
                        onclick: move |_| {
                            let name = new_entry_name().trim().to_owned();
                            if name.is_empty() || name.contains('/') {
                                status.set("Enter a name without a slash.".to_owned());
                                return;
                            }
                            let path = entry_path(&current_path(), &name);
                            let result = match kind {
                                NewEntry::File => FILESYSTEM.write().expect("filesystem lock poisoned").create_file(&path, None, &[]),
                                NewEntry::Directory => FILESYSTEM.write().expect("filesystem lock poisoned").create_directory(&path, None),
                            };
                            match result {
                                Ok(()) => {
                                    status.set(format!("Created {} {name}.", kind.label()));
                                    new_entry.set(None);
                                    refresh += 1;
                                }
                                Err(error) => status.set(error),
                            }
                        },
                        "Create"
                    }
                    button {
                        class: "filething-dialog-button",
                        onclick: move |_| new_entry.set(None),
                        "Cancel"
                    }
                }
            }

            div {
                class: "filething-content",
                div { class: "filething-entries",
                    if entries.is_empty() {
                        p { class: "filething-empty", "This directory is empty." }
                    }
                    for entry in entries {
                        {
                            let is_directory = matches!(entry.data, FilesystemData::Directory { .. });
                            let is_selected = selected().as_ref().is_some_and(|selected| selected.name == entry.name);
                            let entry_for_click = entry.clone();
                            rsx! {
                                button {
                                    key: "{entry.name}",
                                    class: if is_selected { "filething-entry selected" } else { "filething-entry" },
                                    onclick: move |_| selected.set(Some(entry_for_click.clone())),
                                    ondoubleclick: move |_| {
                                        if is_directory {
                                            navigate(entry_path(&current_path(), &entry.name), true);
                                        }
                                    },
                                    if is_directory {
                                        Icon { icon: LdFolder }
                                    } else {
                                        Icon { icon: LdFile }
                                    }
                                    span { "{entry.name}" }
                                }
                            }
                        }
                    }
                }
                if show_preview() {
                    div { class: "filething-preview",
                        h2 { "Preview" }
                        if let Some(entry) = selected() {
                            match &entry.data {
                                FilesystemData::Directory { children } => rsx! {
                                    div { class: "filething-preview-title",
                                        Icon { icon: LdFolder }
                                        h3 { "{entry.name}" }
                                    }
                                    p { "Directory · {children.len()} item(s)" }
                                },
                                FilesystemData::File { content } => rsx! {
                                    div { class: "filething-preview-title",
                                        Icon { icon: LdFile }
                                        h3 { "{entry.name}" }
                                    }
                                    p { "File · {content.len()} bytes" }
                                    pre { "{String::from_utf8_lossy(content)}" }
                                },
                                FilesystemData::SymbolicLink { target } => rsx! {
                                    h3 { "{entry.name}" }
                                    p { "Symbolic link" }
                                    p { "→ {target}" }
                                },
                            }
                        } else {
                            p { class: "filething-empty", "Select an entry to preview it." }
                        }
                    }
                }
            }
            if !status().is_empty() {
                div { class: "filething-status", "{status}" }
            }
        }
    }
}
