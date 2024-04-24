use crate::{
    register::{EntryData, RegisterData, SaveRegisterData},
    Key, ObjectKind, Result,
};

use super::DATA_SIZE;

impl<S: AsRef<str> + Ord> super::Hash for RegisterData<S> {
    const KIND: crate::ObjectKind = <Self as EntryData<S>>::REGISTER_KIND.as_object_kind();

    fn hash(&self) -> super::Result<(braid_hash::Oid, Vec<u8>)> {
        hash(self)
    }
}

impl<S: AsRef<str> + Ord> super::Hash for SaveRegisterData<S> {
    const KIND: crate::ObjectKind = <Self as EntryData<S>>::REGISTER_KIND.as_object_kind();

    fn hash(&self) -> super::Result<(braid_hash::Oid, Vec<u8>)> {
        hash(self)
    }
}

fn hash<S: AsRef<str>, R: EntryData<S>>(data: &R) -> Result<(braid_hash::Oid, Vec<u8>)> {
    let mut buf = Vec::new();
    let mut writer = super::rw::Writer(&mut buf);

    let kind = R::REGISTER_KIND.as_object_kind();
    writer.write_kind(kind)?;
    writer.write_zeros::<DATA_SIZE>()?;

    let len: u32 = data.len().try_into().expect("More than u32::MAX entries");
    writer.write_le_bytes(len)?;

    for (name, oid) in data.iter() {
        writer.write_oid(*oid)?;
        writer.write_null_terminated_string(name.as_ref())?;
    }

    let size: u32 = writer.len().try_into().expect("More than u32::MAX bytes");
    let size = size - DATA_SIZE as u32;
    writer.write_data_size(size)?;

    let oid = braid_hash::hash(&buf);
    Ok((oid, buf))
}

fn read_register(reader: &mut impl std::io::Read) -> Result<RegisterData<String>> {
    read(reader)
}

fn read_save_register(reader: &mut impl std::io::Read) -> Result<SaveRegisterData<String>> {
    read(reader)
}

// todo: untested
fn read<R: EntryData<String>>(reader: &mut impl std::io::Read) -> Result<R> {
    let mut reader = super::rw::Reader(reader);

    let kind = R::REGISTER_KIND.as_object_kind();
    reader.expect_kind(kind)?;
    reader.eat::<DATA_SIZE>()?;

    let len: u32 = reader.read_le_bytes()?;

    let mut data = R::new();
    for _ in 0..len {
        let oid = reader.read_oid()?;
        let name = reader.read_null_terminated_string()?;
        let key = Key::try_from(name)?;
        data.insert(key, oid);
    }

    Ok(data)
}
