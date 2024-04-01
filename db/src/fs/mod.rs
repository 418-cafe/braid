use hash::Oid;

mod err;
mod register;

type Result<T> = std::result::Result<T, err::Error>;

pub struct Database {
    pub(crate) mount: std::path::PathBuf,
}

impl Database {
    pub fn hash(&self, object: impl Hash) -> Result<Oid> {
        object.hash().map(|(oid, _)| oid)
    }
}

pub trait Hash {
    fn hash(&self) -> Result<(Oid, Vec<u8>)>;
}