use super::Result;
use crate::{save::SaveData, ObjectKind};

const DATA_SIZE: usize = super::DATA_SIZE;

pub(crate) type ReadSaveData = crate::save::SaveData;

impl<S: AsRef<str>> super::Hash for SaveData<S> {
    const KIND: ObjectKind = ObjectKind::Save;

    fn hash(&self) -> super::Result<(braid_hash::Oid, Vec<u8>)> {
        hash(self)
    }
}

fn hash<S: AsRef<str>>(save: &SaveData<S>) -> Result<(braid_hash::Oid, Vec<u8>)> {
    const BUF_SIZE: usize = super::HEADER_SIZE                      // ObjectKind + DataSize
        + super::rw::DATETIME_SIZE                                  // timestamp
        + braid_hash::Oid::LEN                                      // content
        + braid_hash::Oid::LEN;                                     // parent

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
    writer.write_oid(save.parent)?;
    writer.write_string(author)?;

    let buf = writer.into_inner();

    // ensure the size of the buffer is correct and there were no new allocations
    #[cfg(debug_assertions)]
    {
        debug_assert_eq!(buf.capacity(), cap);
        debug_assert_eq!(buf.len(), data_size as usize);
    }

    Ok((braid_hash::hash(&buf), buf))
}

// todo: untested
pub(crate) fn read(reader: &mut impl std::io::Read) -> super::Result<ReadSaveData> {
    let mut reader = super::rw::Reader(reader);

    reader.expect_kind(ObjectKind::Save)?;
    reader.eat::<DATA_SIZE>()?;

    let date = reader.read_timestamp()?;
    let content = reader.read_oid()?;
    let parent = reader.read_oid()?;
    let author = reader.read_string_until_end()?;

    Ok(ReadSaveData {
        date,
        content,
        parent,
        author,
    })
}
