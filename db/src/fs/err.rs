use std::string::FromUtf8Error;

use hash::Oid;
use thiserror::Error;

use crate::{register::RegisterEntryKind, Kind, ObjectKind};

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
    Mapped(RegisterEntryKind),
    Unmapped(u8),
    Missing,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid entry kind for operation: was {was:?}")]
    InvalidEntryKind { was: WasEntryKind },

    #[error("Invalid entry name: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    #[error("Invalid oid: {0}")]
    InvalidOid(Oid),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(time::error::ComponentRange),

    #[error("Invalid offset: {0}")]
    InvalidOffset(time::error::ComponentRange),

    #[error(transparent)]
    ObjectKind(#[from] crate::ObjectKindError),

    #[error(transparent)]
    RegisterEntryKind(#[from] crate::register::RegisterEntryKindError),
}
