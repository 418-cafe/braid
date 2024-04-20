#[macro_use]
mod kind;
mod bytes;
mod err;

pub mod commit;
mod key;
pub mod oid;
pub mod register;
pub mod save;

#[cfg(feature = "postgres")]
pub mod postgres;

pub use key::{Key, RegisterEntryKey, SaveEntryKey};

use braid_hash::Oid;

pub type Result<T> = std::result::Result<T, err::Error>;

pub use err::Error;

pub(crate) use kind::Kind;

kind! {
    pub enum ObjectKind {
        Register = 0,
        Commit = 1,
        Save = 2,
        SaveRegister = 3,
    }

    ObjectKindError => "Invalid object kind: {0:?}"
}

#[derive(Debug, Clone)]
pub struct Object {
    pub(crate) oid: Oid,
    pub(crate) kind: ObjectKind,
    pub(crate) size: u32,
}

impl Object {
    pub fn oid(&self) -> Oid {
        self.oid
    }

    pub fn kind(&self) -> ObjectKind {
        self.kind
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

pub(crate) mod sealed {
    pub trait Sealed {}

    impl<T: Sealed> Sealed for &T {}
}
