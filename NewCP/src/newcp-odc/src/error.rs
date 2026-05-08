use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum OdcError {
    Io(io::Error),
    BadMagic { path: Option<PathBuf>, found: [u8; 4] },
    Truncated { at: u64, want: usize, have: usize },
    BadStoreKind { at: u64, kind: u8 },
    BadPathKind { at: u64, kind: u8 },
    UnknownTypeId { at: u64, id: i32 },
    StringNotTerminated { at: u64 },
    InvalidString { at: u64 },
    Inconsistent(&'static str),
}

pub type Result<T> = std::result::Result<T, OdcError>;

impl fmt::Display for OdcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OdcError::Io(e) => write!(f, "i/o error: {e}"),
            OdcError::BadMagic { path, found } => {
                write!(
                    f,
                    "bad magic in {:?}: expected 'CDOo' (43 44 4F 6F), got {:02x} {:02x} {:02x} {:02x}",
                    path, found[0], found[1], found[2], found[3]
                )
            }
            OdcError::Truncated { at, want, have } => {
                write!(f, "truncated at byte {at}: wanted {want} bytes, have {have}")
            }
            OdcError::BadStoreKind { at, kind } => {
                write!(f, "unknown store kind 0x{kind:02x} at byte {at}")
            }
            OdcError::BadPathKind { at, kind } => {
                write!(f, "unknown path-component kind 0x{kind:02x} at byte {at}")
            }
            OdcError::UnknownTypeId { at, id } => {
                write!(f, "type id {id} at byte {at} not present in dictionary")
            }
            OdcError::StringNotTerminated { at } => {
                write!(f, "null-terminated string starting at byte {at} ran past end of file")
            }
            OdcError::InvalidString { at } => {
                write!(f, "string starting at byte {at} contains invalid bytes")
            }
            OdcError::Inconsistent(msg) => write!(f, "inconsistent file: {msg}"),
        }
    }
}

impl std::error::Error for OdcError {}

impl From<io::Error> for OdcError {
    fn from(e: io::Error) -> Self {
        OdcError::Io(e)
    }
}
