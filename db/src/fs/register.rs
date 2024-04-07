use hash::Oid;

use super::Result;
use crate::key::Key;
use crate::register::{RegisterEntryCollection, SaveEntryCollection};
use crate::{register::EntryData, ObjectKind};

const OBJECT_KIND_SIZE: usize = std::mem::size_of::<crate::ObjectKind>();
const DATA_SIZE: usize = std::mem::size_of::<u32>();
const LEN_SIZE: usize = std::mem::size_of::<u32>();

const HEADER_SIZE: usize = OBJECT_KIND_SIZE + DATA_SIZE + LEN_SIZE;

const NEWLINE_SIZE: usize = '\n'.len_utf8();
const AVG_STR_SIZE: usize = 20;

const ENTRY_SIZE: usize = Oid::LEN + AVG_STR_SIZE + NEWLINE_SIZE;

pub(super) type ReturnRegisterEntryCollection = RegisterEntryCollection<String, EntryData>;
pub(super) type ReturnSaveEntryCollection = SaveEntryCollection<String>;

type CountOfEntries = u32;

impl<S: AsRef<str>, D: Write> super::Hash for RegisterEntryCollection<S, D> {
    fn hash(&self) -> Result<(Oid, Vec<u8>)> {
        hash(ObjectKind::Register, self.iter())
    }
}

impl<S, D> super::Validate for RegisterEntryCollection<S, D> {}

impl<S: Ord + AsRef<str>> super::Hash for SaveEntryCollection<S> {
    fn hash(&self) -> Result<(hash::Oid, Vec<u8>)> {
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
    let buf = HEADER_SIZE + data.len() * ENTRY_SIZE;

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
    let size = size - HEADER_SIZE as u32;

    buf[1..=DATA_SIZE].copy_from_slice(&size.to_le_bytes());

    let oid = hash::hash(&buf[HEADER_SIZE..]);
    Ok((oid, buf))
}

pub(super) fn read_register<R: std::io::Read>(reader: &mut R) -> Result<ReturnRegisterEntryCollection> {
    let mut reader = super::rw::Reader(reader);

    reader.expect_kind(ObjectKind::Register)?;
    reader.eat::<DATA_SIZE>()?;

    let len = {
        let len: CountOfEntries = reader.read_le_bytes()?;
        len as usize
    };

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

pub(super) fn read_save<R: std::io::Read>(reader: &mut R) -> Result<ReturnSaveEntryCollection> {
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