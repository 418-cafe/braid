use hash::Oid;

use super::Result;
use crate::key::Key;
use crate::register::{RegisterEntryCollection, SaveEntryCollection};
use crate::{register::EntryData, ObjectKind};

const DATA_SIZE: usize = super::DATA_SIZE;

const NULL_SIZE: usize = '\0'.len_utf8();
const AVG_STR_SIZE: usize = 20;

const ENTRY_SIZE: usize = Oid::LEN + AVG_STR_SIZE + NULL_SIZE;

pub(super) type ReturnRegisterEntryCollection = RegisterEntryCollection<String, EntryData>;
pub(super) type ReturnSaveEntryCollection = SaveEntryCollection<String>;

type CountOfEntries = u32;

impl<S: AsRef<str>, D: Write> super::Hash for RegisterEntryCollection<S, D> {
    const KIND: ObjectKind = ObjectKind::Register;

    fn hash(&self) -> Result<(Oid, Vec<u8>)> {
        hash(self.iter())
    }
}

impl<S, D> super::Validate for RegisterEntryCollection<S, D> {}

impl<S: Ord + AsRef<str>> super::Hash for SaveEntryCollection<S> {
    const KIND: ObjectKind = ObjectKind::SaveRegister;

    fn hash(&self) -> Result<(hash::Oid, Vec<u8>)> {
        hash(self.iter())
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
    data: impl ExactSizeIterator<Item = (S, D)>,
) -> Result<(Oid, Vec<u8>)> {
    let buf = DATA_SIZE + data.len() * ENTRY_SIZE;

    let buf = Vec::with_capacity(buf);
    let mut buf = super::rw::Writer(buf);

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

    buf[..DATA_SIZE].copy_from_slice(&size.to_le_bytes());

    let oid = hash::hash(&buf[DATA_SIZE..]);
    Ok((oid, buf))
}

pub(super) fn read_register<R: std::io::Read>(reader: &mut R) -> Result<ReturnRegisterEntryCollection> {
    let mut reader = super::rw::Reader(reader);

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

pub(super) fn read_save_register<R: std::io::Read>(reader: &mut R) -> Result<ReturnSaveEntryCollection> {
    let mut reader = super::rw::Reader(reader);

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