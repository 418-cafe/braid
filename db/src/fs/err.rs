use std::string::FromUtf8Error;

use hash::Oid;
use thiserror::Error;

use crate::ObjectKind;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid entry name: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    #[error("Invalid oid: `{oid:?}` is for {is_for:?}")]
    InvalidOid {
        oid: Oid,
        is_for: ObjectKind,
    },

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(time::error::ComponentRange),

    #[error("Invalid offset: {0}")]
    InvalidOffset(time::error::ComponentRange),

    #[error(transparent)]
    ObjectKind(#[from] crate::ObjectKindError),

    #[error(transparent)]
    RegisterEntryKind(#[from] crate::register::RegisterEntryKindError),

    #[error(transparent)]
    SaveParentKind(#[from] crate::save::SaveParentKindError),

    #[error(transparent)]
    InvalidCharacterInKey(#[from] crate::key::InvalidCharacterInKeyError),
}
