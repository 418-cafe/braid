#[macro_use]
mod kind;

pub mod commit;
pub mod fs;
pub mod key;
pub mod oid;
pub mod register;
pub mod save;

use hash::Oid;
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
