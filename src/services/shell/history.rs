use std::fmt::Write as _;

use crate::services::fs::FILESYSTEM;

pub(super) fn add_to_history(entry: &str) {
    let mut writer = FILESYSTEM.write().unwrap();
    let history_path = "/home/inparsian/.sheesh_history";

    // if this file does not exist, it should be made
    if !writer.resolve_path(history_path, None).is_some() {
        writer.create_file(history_path, None, &[]).unwrap();
    }
    
    let mut content = String::from_utf8_lossy(writer.read_file(history_path, None).unwrap()).to_string();
    let _ = write!(content, "\r\n{}", entry);
    writer.write_file(history_path, None, content.as_bytes()).unwrap();
}

pub(super) fn read_history() -> Vec<String> {
    let reader = FILESYSTEM.read().unwrap();
    let content = String::from_utf8_lossy(reader.read_file("/home/inparsian/.sheesh_history", None).unwrap()).to_string();
    content.lines().map(|l| l.to_owned()).collect()
}