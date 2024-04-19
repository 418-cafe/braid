use crate::ObjectKind;

pub(crate) mod commit;
pub(crate) mod register;
pub(crate) mod rw;
pub(crate) mod save;

pub(crate) type Result<T> = std::result::Result<T, crate::err::Error>;
type DataSize = u32;

const DATA_SIZE: usize = std::mem::size_of::<DataSize>();
const OBJECT_KIND_SIZE: usize = std::mem::size_of::<crate::ObjectKind>();
const HEADER_SIZE: usize = OBJECT_KIND_SIZE + DATA_SIZE;

pub trait Hash: crate::sealed::Sealed {
    const KIND: ObjectKind;

    fn hash(&self) -> Result<(braid_hash::Oid, Vec<u8>)>;
}

impl<T: Hash> Hash for &T {
    const KIND: ObjectKind = T::KIND;

    fn hash(&self) -> Result<(braid_hash::Oid, Vec<u8>)> {
        (*self).hash()
    }
}