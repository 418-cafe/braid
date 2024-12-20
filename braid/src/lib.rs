mod ancestry;
mod braid;
mod db;
mod hash;
mod key;
mod models;
mod oid;
mod sql;
mod time;

pub use ancestry::Ancestry;
pub use braid::{Braid, InitOptions, Timing};
pub use hash::{Hash, Hasher};
pub use key::Key;
pub use models::*;
pub use oid::Oid;
pub use time::DateTime;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("branch does not exist: {0}")]
    BranchDoesNotExist(String),

    #[error("root commit does not exist")]
    RootCommitDoesNotExist,

    #[error("save's expected parent does not match database")]
    MismatchedParent,
}

macro_rules! const_unwrap {
    ($expr:expr) => {
        const {
            match $expr {
                Ok(value) => value,
                Err(_) => panic!("tried to unwrap an Err value at comptime"),
            }
        }
    };
}
pub(crate) use const_unwrap;
