use super::FilesystemEntry;

pub fn image() -> Vec<FilesystemEntry> {
    vec![
        FilesystemEntry::new_dir("etc", vec![
            FilesystemEntry::new_file("hostname", "my-device"),
            FilesystemEntry::new_file("config.toml", "[network]\ndhcp = true\n"),
        ]),
        FilesystemEntry::new_dir("home", vec![
            FilesystemEntry::new_dir("inparsian", vec![
                FilesystemEntry::new_file("readme.txt", "Welcome to the static FS image."),
                FilesystemEntry::new_file("notes.txt", "- item 1\n- item 2\n"),
            ]),
        ]),
        FilesystemEntry::new_dir("var", vec![
            FilesystemEntry::new_dir("log", vec![
                FilesystemEntry::new_file("boot.log", "[0.0] system boot"),
            ]),
        ]),
        FilesystemEntry::new_dir("bin", vec![
            FilesystemEntry::new_file("init.sh", "#!/bin/sh\necho init\n"),
        ]),
    ]
}