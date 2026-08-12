use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use super::{Filesystem, FilesystemData, FilesystemEntry};

const MAGIC: &[u8; 4] = b"IFS1";
const MAX_FIELD_SIZE: usize = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 100_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodingError {
    InvalidBase64(String),
    InvalidFormat(&'static str),
    UnexpectedEnd,
    InvalidUtf8,
    FieldTooLarge,
    TooManyEntries,
}

impl std::fmt::Display for EncodingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidBase64(error) => write!(f, "invalid filesystem Base64: {error}"),
            Self::InvalidFormat(message) => write!(f, "invalid filesystem format: {message}"),
            Self::UnexpectedEnd => f.write_str("unexpected end of filesystem data"),
            Self::InvalidUtf8 => f.write_str("filesystem contains invalid UTF-8"),
            Self::FieldTooLarge => f.write_str("filesystem field is too large"),
            Self::TooManyEntries => f.write_str("filesystem contains too many entries"),
        }
    }
}

impl std::error::Error for EncodingError {}

pub fn encode(filesystem: &Filesystem) -> String {
    let mut bytes = MAGIC.to_vec();
    encode_entries(&mut bytes, &filesystem.root);
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn decode(value: &str) -> Result<Filesystem, EncodingError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|error| EncodingError::InvalidBase64(error.to_string()))?;
    if bytes.len() < MAGIC.len() || &bytes[..MAGIC.len()] != MAGIC {
        return Err(EncodingError::InvalidFormat("wrong magic header"));
    }

    let mut reader = Reader::new(&bytes[MAGIC.len()..]);
    let mut entries = Vec::new();
    reader.read_entries(&mut entries)?;
    if !reader.is_empty() {
        return Err(EncodingError::InvalidFormat("trailing bytes"));
    }
    Ok(Filesystem { root: entries })
}

fn encode_entries(output: &mut Vec<u8>, entries: &[FilesystemEntry]) {
    write_u32(output, entries.len() as u32);
    for entry in entries {
        write_bytes(output, entry.name.as_bytes());
        match &entry.data {
            FilesystemData::File { content } => {
                output.push(0);
                write_bytes(output, content);
            }
            FilesystemData::Directory { children } => {
                output.push(1);
                encode_entries(output, children);
            }
            FilesystemData::SymbolicLink { target } => {
                output.push(2);
                write_bytes(output, target.as_bytes());
            }
        }
    }
}

fn write_u32(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_le_bytes());
}

fn write_bytes(output: &mut Vec<u8>, value: &[u8]) {
    write_u32(output, value.len() as u32);
    output.extend_from_slice(value);
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    entries: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0, entries: 0 }
    }

    fn is_empty(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn read_u32(&mut self) -> Result<u32, EncodingError> {
        let end = self.position.checked_add(4).ok_or(EncodingError::UnexpectedEnd)?;
        let bytes = self.bytes.get(self.position..end).ok_or(EncodingError::UnexpectedEnd)?;
        self.position = end;
        Ok(u32::from_le_bytes(bytes.try_into().expect("four bytes")))
    }

    fn read_bytes(&mut self) -> Result<&'a [u8], EncodingError> {
        let length = self.read_u32()? as usize;
        if length > MAX_FIELD_SIZE {
            return Err(EncodingError::FieldTooLarge);
        }
        let end = self.position.checked_add(length).ok_or(EncodingError::UnexpectedEnd)?;
        let value = self.bytes.get(self.position..end).ok_or(EncodingError::UnexpectedEnd)?;
        self.position = end;
        Ok(value)
    }

    fn read_entries(&mut self, output: &mut Vec<FilesystemEntry>) -> Result<(), EncodingError> {
        let count = self.read_u32()? as usize;
        if count > MAX_ENTRIES.saturating_sub(self.entries) {
            return Err(EncodingError::TooManyEntries);
        }
        self.entries += count;
        for _ in 0..count {
            let name = String::from_utf8(self.read_bytes()?.to_vec())
                .map_err(|_| EncodingError::InvalidUtf8)?;
            let kind = *self.bytes.get(self.position).ok_or(EncodingError::UnexpectedEnd)?;
            self.position += 1;
            let data = match kind {
                0 => FilesystemData::File { content: self.read_bytes()?.to_vec() },
                1 => {
                    let mut children = Vec::new();
                    self.read_entries(&mut children)?;
                    FilesystemData::Directory { children }
                }
                2 => FilesystemData::SymbolicLink {
                    target: String::from_utf8(self.read_bytes()?.to_vec())
                        .map_err(|_| EncodingError::InvalidUtf8)?,
                },
                _ => return Err(EncodingError::InvalidFormat("unknown entry type")),
            };
            output.push(FilesystemEntry { name, data });
        }
        Ok(())
    }
}