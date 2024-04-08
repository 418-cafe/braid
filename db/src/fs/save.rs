use crate::{save::SaveData, ObjectKind};
use super::Result;

pub(super) type ReturnSaveData = crate::save::SaveData;

impl<S: AsRef<str>> super::Hash for SaveData<S> {
    fn hash(&self) -> super::Result<(hash::Oid, Vec<u8>)> {
        hash(self)
    }
}

impl<S> super::Validate for SaveData<S> {
    fn validate(&self, _db: &super::Database) -> Result<()> {
        Ok(())
    }
}

fn hash<S: AsRef<str>>(save: &SaveData<S>) -> Result<(hash::Oid, Vec<u8>)> {
    const BUF_SIZE : usize =
        super::HEADER_SIZE +
        super::rw::DATETIME_SIZE +
        hash::Oid::LEN +
        std::mem::size_of::<crate::save::SaveParentKind>() +
        hash::Oid::LEN;

    let author = save.author.as_ref();
    let data_size = BUF_SIZE + author.len();

    let buf = Vec::with_capacity(data_size);
    let data_size: u32 = data_size.try_into().expect("data_size overflow");

    #[cfg(debug_assertions)]
    let cap = buf.capacity();

    let mut writer = super::rw::Writer(buf);

    writer.write_kind(ObjectKind::Save)?;
    writer.write_le_bytes(data_size)?;

    writer.write_timestamp(save.date)?;
    writer.write_oid(save.content)?;
    writer.write_kind(save.parent.kind)?;
    writer.write_oid(save.parent.oid)?;
    writer.write_string(author)?;

    let buf = writer.into_inner();

    // ensure the size of the buffer is correct and there were no new allocations
    debug_assert_eq!(buf.capacity(), cap);
    debug_assert_eq!(buf.len(), data_size as usize - super::HEADER_SIZE);

    Ok((hash::hash(&buf), buf))
}

pub(super) fn read(mut reader: impl std::io::Read) -> super::Result<ReturnSaveData> {
    let mut reader = super::rw::Reader(&mut reader);

    reader.expect_kind(ObjectKind::Save)?;

    const DATA_SIZE: usize = super::DATA_SIZE;
    reader.eat::<DATA_SIZE>()?;

    let date = reader.read_timestamp()?;
    let content = reader.read_oid()?;
    let kind = reader.read_kind()?;
    let oid = reader.read_oid()?;
    let author = reader.read_string_until_end()?;

    Ok(ReturnSaveData {
        date,
        content,
        parent: crate::save::SaveParent {
            kind,
            oid,
        },
        author,
    })
}