use braid_hash::Oid;

use super::Result;
use crate::key::Key;
use crate::register::{RegisterEntryCollection, SaveEntryCollection};
use crate::{register::EntryData, ObjectKind};

const DATA_SIZE: usize = super::DATA_SIZE;

const NULL_SIZE: usize = '\0'.len_utf8();
const AVG_STR_SIZE: usize = 20;

const ENTRY_SIZE: usize = Oid::LEN + AVG_STR_SIZE + NULL_SIZE;

pub(crate) type ReadRegisterEntryCollection = RegisterEntryCollection<String, EntryData>;
pub(crate) type ReadSaveEntryCollection = SaveEntryCollection<String>;

type CountOfEntries = u32;

impl<S: AsRef<str>, D: Write> super::Hash for RegisterEntryCollection<S, D> {
    const KIND: ObjectKind = ObjectKind::Register;

    fn hash(&self) -> Result<(Oid, Vec<u8>)> {
        hash(ObjectKind::Register, self.iter())
    }
}

impl<S: Ord + AsRef<str>> super::Hash for SaveEntryCollection<S> {
    const KIND: ObjectKind = ObjectKind::SaveRegister;

    fn hash(&self) -> Result<(braid_hash::Oid, Vec<u8>)> {
        hash(ObjectKind::SaveRegister, self.iter())
    }
}

trait Write {
    fn write(&self, buf: &mut super::rw::Writer<impl std::io::Write>) -> Result<()>;
}

impl Write for EntryData {
    fn write(&self, buf: &mut super::rw::Writer<impl std::io::Write>) -> Result<()> {
        buf.write_oid(self.content)?;
        buf.write_kind(self.kind)?;
        Ok(())
    }
}

impl<D: Write> Write for &D {
    fn write(&self, buf: &mut super::rw::Writer<impl std::io::Write>) -> Result<()> {
        (*self).write(buf)
    }
}

impl Write for Oid {
    fn write(&self, buf: &mut super::rw::Writer<impl std::io::Write>) -> Result<()> {
        buf.write_oid(*self)
    }
}

fn hash<S: AsRef<str>, D: Write>(
    kind: ObjectKind,
    data: impl ExactSizeIterator<Item = (S, D)>,
) -> Result<(Oid, Vec<u8>)> {
    let buf = super::HEADER_SIZE + data.len() * ENTRY_SIZE;

    let buf = Vec::with_capacity(buf);
    let mut buf = super::rw::Writer(buf);

    buf.write_kind(kind)?;
    buf.write_zeros::<DATA_SIZE>()?;

    let len: CountOfEntries = data.len().try_into().expect("More than u32::MAX entries");
    buf.write_le_bytes(len)?;

    for (name, entry) in data {
        entry.write(&mut buf)?;
        buf.write_null_terminated_string(name.as_ref())?;
    }

    let mut buf = buf.into_inner();
    let size: u32 = buf.len().try_into().expect("More than u32::MAX bytes");
    let size = size - DATA_SIZE as u32;

    buf[1..=DATA_SIZE].copy_from_slice(&size.to_le_bytes());

    let oid = braid_hash::hash(&buf);
    Ok((oid, buf))
}

pub(crate) fn read_register<R: std::io::Read>(
    reader: &mut R,
) -> Result<ReadRegisterEntryCollection> {
    let mut reader = super::rw::Reader(reader);

    reader.expect_kind(ObjectKind::Register)?;
    reader.eat::<DATA_SIZE>()?;

    let len: CountOfEntries = reader.read_le_bytes()?;

    let mut map = RegisterEntryCollection::new();

    for _ in 0..len {
        let oid = reader.read_oid()?;
        let kind = reader.read_kind()?;
        let name = reader.read_null_terminated_string()?;

        let key = Key::try_from(name)?;

        map.insert(key, EntryData { content: oid, kind });
    }

    Ok(map)
}

pub(crate) fn read_save_register<R: std::io::Read>(
    reader: &mut R,
) -> Result<ReadSaveEntryCollection> {
    let mut reader = super::rw::Reader(reader);

    reader.expect_kind(ObjectKind::SaveRegister)?;
    reader.eat::<DATA_SIZE>()?;

    let len = {
        let len: CountOfEntries = reader.read_le_bytes()?;
        len as usize
    };

    let mut map = SaveEntryCollection::new();

    for _ in 0..len {
        let oid = reader.read_oid()?;
        let name = reader.read_null_terminated_string()?;

        let key = Key::try_from(name)?;

        map.insert(key, oid);
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use crate::{bytes::Hash, register::{EntryData, Register, SaveRegister}};

    type RegisterEntryCollection = crate::register::RegisterEntryCollection<String, EntryData>;
    type SaveEntryCollection = crate::register::SaveEntryCollection<String>;

    #[test]
    fn test_empty_register() {
        let register = RegisterEntryCollection::new();
        let (oid, _) = register.hash().unwrap();
        assert_eq!(oid, Register::EMPTY_ID)
    }

    #[test]
    fn test_empty_save_register() {
        let register = SaveEntryCollection::new();
        let (oid, _) = register.hash().unwrap();
        assert_eq!(oid, SaveRegister::EMPTY_ID)
    }
}