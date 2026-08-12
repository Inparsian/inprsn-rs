mod img;

pub mod encoding;

use std::sync::{LazyLock, RwLock};
use dioxus::logger::tracing::{error, info};

pub static FILESYSTEM: LazyLock<RwLock<Filesystem>> = LazyLock::new(|| {
    // see if we have our fs in local storage, if so, we can get our fs from that
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let fs = if let Some(encoded) = storage.get("fs").unwrap()
        && let Ok(cfs) = Filesystem::decode(encoded.as_str()) 
    {
        let entries: usize = cfs.root.iter().map(|e| e.size_entries()).sum();
        let bytes: usize = cfs.root.iter().map(|e| e.size_bytes()).sum();
        info!("Filesystem found in local storage ({} bytes, {} entries), restoring...", bytes, entries);
        cfs
    } else {
        info!("No filesystem found in local storage, initializing...");
        Filesystem::init()
    };
    RwLock::new(fs)
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilesystemData {
    File {
        content: Vec<u8>,
    },

    Directory {
        children: Vec<FilesystemEntry>,
    },

    SymbolicLink {
        target: String,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilesystemEntry {
    pub name: String,
    pub data: FilesystemData,
}

impl FilesystemEntry {
    pub fn new_file(name: &str, content: &str) -> Self {
        Self {
            name: name.to_owned(),
            data: FilesystemData::File { content: content.as_bytes().to_vec() },
        }
    }

    pub fn new_dir(name: &str, children: Vec<FilesystemEntry>) -> Self {
        Self {
            name: name.to_owned(),
            data: FilesystemData::Directory { children },
        }
    }

    pub fn size_entries(&self) -> usize {
        match &self.data {
            FilesystemData::Directory { children } => children.iter().map(|c| c.size_entries()).sum(),
            FilesystemData::File { .. } | FilesystemData::SymbolicLink { .. } => 1,
        }
    }

    pub fn size_bytes(&self) -> usize {
        match &self.data {
            FilesystemData::File { content } => content.len(),
            FilesystemData::Directory { children } => children.iter().map(|c| c.size_bytes()).sum(),
            FilesystemData::SymbolicLink { .. } => 0,
        }
    }
}

pub struct Filesystem {
    pub root: Vec<FilesystemEntry>,
}

impl Filesystem {
    pub fn init() -> Self {
        Self {
            root: img::image(),
        }
    }
    
    pub fn encode(&self) -> String {
        encoding::encode(self)
    }

    pub fn decode(value: &str) -> Result<Self, encoding::EncodingError> {
        encoding::decode(value)
    }

    pub fn save_locally(&self) {
        let encoded = self.encode();

        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        if let Err(e) = storage.set("fs", encoded.as_str()) {
            error!("Failed to save fs: {:?}", e);
        }
    }

    fn resolve_parts(path: &str, pwd: Option<&str>) -> Option<Vec<String>> {
        let target_path = if path.starts_with('/') {
            path.chars().skip(1).collect::<String>()
        } else {
            pwd.map_or_else(|| path.to_owned(), |pwd| format!("{}/{}", pwd, path))
        };

        let mut resolved_path: Vec<String> = Vec::new();
        for part in target_path
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
        {
            if part == ".." {
                resolved_path.pop();
            } else {
                resolved_path.push(part.to_owned());
            }
        }

        Some(resolved_path)
    }

    fn resolve_in_entries<'a>(
        entries: &'a [FilesystemEntry],
        parts: &[&str],
    ) -> Option<&'a FilesystemEntry> {
        let Some((first, rest)) = parts.split_first() else {
            return Some(Box::leak(Box::new(FilesystemEntry {
                name: String::new(),
                data: FilesystemData::Directory {
                    children: entries.to_vec(),
                },
            })));
        };
        let idx = entries.iter().position(|entry| entry.name == *first)?;

        if rest.is_empty() {
            return entries.get(idx);
        }

        match &entries.get(idx)?.data {
            FilesystemData::Directory { children } => Self::resolve_in_entries(children, rest),
            _ => None,
        }
    }

    fn resolve_mut_in_entries<'a>(
        entries: &'a mut [FilesystemEntry],
        parts: &[&str],
    ) -> Option<&'a mut FilesystemEntry> {
        let Some((first, rest)) = parts.split_first() else {
            return Some(Box::leak(Box::new(FilesystemEntry {
                name: String::new(),
                data: FilesystemData::Directory {
                    children: entries.to_vec(),
                },
            })));
        };
        let idx = entries.iter().position(|entry| entry.name == *first)?;

        if rest.is_empty() {
            return entries.get_mut(idx);
        }

        match &mut entries.get_mut(idx)?.data {
            FilesystemData::Directory { children } => Self::resolve_mut_in_entries(children, rest),
            _ => None,
        }
    }

    pub fn resolve_path(&self, path: &str, pwd: Option<&str>) -> Option<String> {
        let resolved_path = Self::resolve_parts(path, pwd)?;
        let resolved_path_parts = resolved_path.iter().map(String::as_str).collect::<Vec<_>>();
        
        Self::resolve_in_entries(&self.root, &resolved_path_parts)
            .map(|_| format!("/{}", resolved_path.join("/")))
    }

    pub fn resolve_read(&self, path: &str, pwd: Option<&str>) -> Option<&FilesystemEntry> {
        let resolved_path = Self::resolve_parts(path, pwd)?;
        let resolved_path_parts = resolved_path.iter().map(String::as_str).collect::<Vec<_>>();
        
        Self::resolve_in_entries(&self.root, &resolved_path_parts)
    }

    pub fn resolve_read_dir(&self, path: &str, pwd: Option<&str>) -> Option<Vec<FilesystemEntry>> {
        let resolved_path = Self::resolve_parts(path, pwd)?;
        let resolved_path_parts = resolved_path.iter().map(String::as_str).collect::<Vec<_>>();
        let entry = Self::resolve_in_entries(&self.root, &resolved_path_parts)?;
        
        // directory?
        match &entry.data {
            FilesystemData::Directory { children } => Some(children.clone()),
            _ => None,
        }
    }

    pub fn resolve_write(&mut self, path: &str, pwd: Option<&str>) -> Option<&mut FilesystemEntry> {
        let resolved_path = Self::resolve_parts(path, pwd)?;
        let resolved_path_parts = resolved_path.iter().map(String::as_str).collect::<Vec<_>>();
        
        Self::resolve_mut_in_entries(&mut self.root, &resolved_path_parts)
    }

    pub fn create_file(&mut self, path: &str, pwd: Option<&str>, data: &[u8]) -> Result<(), String> {
        if self.resolve_read(path, pwd).is_some() {
            return Err("File already exists".to_owned());
        }

        let parent_path = path.rsplit_once('/').map_or("", |(p, _)| p);
        let file_name = path.rsplit_once('/').map_or(path, |(_, n)| n);

        if let Some(parent_entry) = self.resolve_write(parent_path, pwd) {
            if let FilesystemData::Directory { children } = &mut parent_entry.data {
                children.push(FilesystemEntry {
                    name: file_name.to_owned(),
                    data: FilesystemData::File { content: data.to_vec() },
                });
                self.save_locally();
                Ok(())
            } else {
                Err(format!("Path is not a directory: {}", path))
            }
        } else {
            Err("Parent directory does not exist".to_owned())
        }
    }

    pub fn create_directory(&mut self, path: &str, pwd: Option<&str>) -> Result<(), String> {
        if self.resolve_read(path, pwd).is_some() {
            return Err("Directory already exists".to_owned());
        }

        let parent_path = path.rsplit_once('/').map_or("", |(p, _)| p);
        let dir_name = path.rsplit_once('/').map_or(path, |(_, n)| n);

        if let Some(parent_entry) = self.resolve_write(parent_path, pwd) {
            if let FilesystemData::Directory { children } = &mut parent_entry.data {
                children.push(FilesystemEntry {
                    name: dir_name.to_owned(),
                    data: FilesystemData::Directory { children: Vec::new() },
                });
                self.save_locally();
                Ok(())
            } else {
                Err(format!("Path is not a directory: {}", path))
            }
        } else {
            Err("Parent directory does not exist".to_owned())
        }
    }

    pub fn read_file(&self, path: &str, pwd: Option<&str>) -> Result<&[u8], String> {
        match self.resolve_read(path, pwd) {
            Some(FilesystemEntry {
                data: FilesystemData::File { content },
                ..
            }) => Ok(content.as_slice()),
            Some(_) => Err(format!("Path is not a file: {}", path)),
            None => Err(format!("File does not exist: {}", path)),
        }
    }

    pub fn write_file(&mut self, path: &str, pwd: Option<&str>, data: &[u8]) -> Result<(), String> {
        match self.resolve_write(path, pwd) {
            Some(FilesystemEntry {
                data: FilesystemData::File { content },
                ..
            }) => {
                content.clear();
                content.extend_from_slice(data);
                self.save_locally();
                Ok(())
            }
            Some(_) => Err(format!("Path is not a file: {}", path)),
            None => Err(format!("File does not exist: {}", path)),
        }
    }

    pub fn remove(&mut self, path: &str, pwd: Option<&str>) -> Result<(), String> {
        let target_path = if path.starts_with('/') {
            path.chars().skip(1).collect::<String>()
        } else {
            pwd.map_or_else(|| path.to_owned(), |pwd| format!("{}/{}", pwd, path))
        };

        let path_parts = target_path
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect::<Vec<_>>();

        let mut resolved_path = Vec::new();
        for part in path_parts {
            if part == ".." {
                resolved_path.pop();
            } else {
                resolved_path.push(part);
            }
        }

        if resolved_path.is_empty() {
            return Err("Cannot remove root".to_owned());
        }

        let (name, parent_parts) = resolved_path
            .split_last()
            .ok_or_else(|| "Invalid path".to_owned())?;

        let children = if parent_parts.is_empty() {
            &mut self.root
        } else {
            match Self::resolve_mut_in_entries(&mut self.root, parent_parts) {
                Some(FilesystemEntry {
                    data: FilesystemData::Directory { children },
                    ..
                }) => children,
                Some(_) => return Err("Parent path is not a directory".to_owned()),
                None => return Err("Parent directory does not exist".to_owned()),
            }
        };

        let res = children.iter()
            .position(|entry| entry.name == *name)
            .map_or_else(|| Err(format!("Path does not exist: {}", path)), |idx| {
                children.remove(idx);
                Ok(())
            });
        self.save_locally();
        res
    }
}