use std::string::FromUtf8Error;

use thiserror::Error;

use crate::{register::EntryKind, ObjectKind};

#[derive(Debug)]
pub enum WasObjectKind {
    Mapped(ObjectKind),
    Unmapped(u8),
    Missing,
}

impl WasObjectKind {
    pub fn from_u8(value: u8) -> Self {
        match ObjectKind::from_u8(value) {
            Some(kind) => Self::Mapped(kind),
            None => Self::Unmapped(value),
        }
    }
}

#[derive(Debug)]
pub enum WasEntryKind {
    Mapped(EntryKind),
    Unmapped(u8),
    Missing,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid object kind for operation: expected {expected:?}, was {was:?}")]
    InvalidObjectKind {
        expected: ObjectKind,
        was: WasObjectKind,
    },

    #[error("Invalid entry kind for operation: was {was:?}")]
    InvalidEntryKind { was: WasEntryKind },

    #[error("Invalid entry name: {0}")]
    FromUtf8(#[from] FromUtf8Error),
}
