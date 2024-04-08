#[macro_use]
mod kind;

pub mod key;
pub mod commit;
pub mod oid;
pub mod fs;
pub mod register;
pub mod save;

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

pub struct Object<L> {
    pub(crate) kind: ObjectKind,
    #[allow(dead_code)]
    pub(crate) location: L,
}

impl<L> Object<L> {
    pub fn kind(&self) -> ObjectKind {
        self.kind
    }
}

pub(crate) mod sealed {
    pub trait Sealed {}

    impl<T: Sealed> Sealed for &T {}
}
