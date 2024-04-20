use std::string::FromUtf8Error;

use braid_hash::Oid;
use thiserror::Error;

use crate::ObjectKind;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid entry name: {0}")]
    FromUtf8(#[from] FromUtf8Error),

    #[error("Unable to validate oid against kind `{0:?}`: {1}")]
    ObjectNotFound(ObjectKind, Oid),

    #[error("Invalid timestamp: {0}")]
    InvalidTimestamp(time::error::ComponentRange),

    #[error("Invalid offset: {0}")]
    InvalidOffset(time::error::ComponentRange),

    #[error(transparent)]
    ObjectKind(#[from] crate::ObjectKindError),

    #[error("Expected object kind {expected:?}, but got {actual:?}")]
    UnexpectedKind {
        expected: ObjectKind,
        actual: ObjectKind,
    },

    #[error(transparent)]
    RegisterEntryKind(#[from] crate::register::RegisterEntryKindError),

    #[error(transparent)]
    SaveParentKind(#[from] crate::save::SaveParentKindError),

    #[error(transparent)]
    InvalidCharacterInKey(#[from] crate::key::InvalidCharacterInKeyError),

    #[cfg(feature = "rocks")]
    #[error("Unhandled RocksDB error: {0}")]
    RocksDbError(#[from] rocksdb::Error),

    #[cfg(feature = "postgres")]
    #[error("Unhandled Postgres error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[cfg(feature = "postgres")]
    #[error("Postgres backend already initialized")]
    PostgresBackendAlreadyInitialized,
}
