use crate::services::fs::{FilesystemData, FILESYSTEM};

pub(super) fn prepare_redirect(path: &str, append: bool, pwd: Option<&str>) -> Result<(), String> {
    let mut fs = FILESYSTEM.write().unwrap();

    if let Some(entry) = fs.resolve_read(path, pwd) {
        return match &entry.data {
            FilesystemData::File { .. } => {
                if append {
                    Ok(())
                } else {
                    // `>` truncates before command execution.
                    fs.write_file(path, pwd, b"")
                }
            }
            _ => Err(format!("Path is not a file: {}", path)),
        };
    }

    // Target doesn't exist: create it as an empty file before command execution.
    fs.create_file(path, pwd, b"")
}

pub(super) fn write_redirect(path: &str, append: bool, data: &str, pwd: Option<&str>) -> Result<(), String> {
    let mut fs = FILESYSTEM.write().unwrap();

    if append {
        if let Some(entry) = fs.resolve_read(path, pwd) {
            match &entry.data {
                FilesystemData::File { .. } => {
                    let mut existing = fs.read_file(path, pwd)?.to_vec();
                    existing.extend_from_slice(data.as_bytes());
                    fs.write_file(path, pwd, &existing)
                }
                _ => Err(format!("Path is not a file: {}", path)),
            }
        } else {
            fs.create_file(path, pwd, data.as_bytes())
        }
    } else if fs.resolve_read(path, pwd).is_some() {
        fs.write_file(path, pwd, data.as_bytes())
    } else {
        fs.create_file(path, pwd, data.as_bytes())
    }
}
