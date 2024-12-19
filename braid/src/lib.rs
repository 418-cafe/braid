mod ancestry;
mod braid;
mod data;
mod db;
mod hash;
mod key;
mod oid;
mod sql;
mod time;

pub use ancestry::Ancestry;
pub use braid::{Braid, InitOptions, Timing};
pub use data::*;
pub use hash::{Hash, Hasher};
pub use key::Key;
pub use oid::Oid;

pub type Result<T> = std::result::Result<T, Error>;

pub use time::DateTime;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("branch does not exist: {0}")]
    BranchDoesNotExist(String),

    #[error("root commit does not exist")]
    RootCommitDoesNotExist,
}
