use crate::{sql::FetchOptional, Oid};

pub trait ObjectId {
    fn oid(&self) -> Oid;
}

pub(crate) trait SealedObject: Sized + ObjectId {
    fn setup_get(oid: Oid) -> impl FetchOptional<Self>;
}

#[allow(private_bounds)]
pub trait Object: SealedObject {}

impl<T: SealedObject> Object for T {}