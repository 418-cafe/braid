use braid_hash::Oid;

use crate::{commit::CommitData, ObjectKind};

use super::Result;

const DATA_SIZE: usize = super::DATA_SIZE;

pub(crate) type ReadCommitData = crate::commit::CommitData;

impl<S: AsRef<str>> super::Hash for CommitData<S> {
    const KIND: ObjectKind = ObjectKind::Commit;

    fn hash(&self) -> Result<(Oid, Vec<u8>)> {
        hash(self)
    }
}

fn hash(commit: &CommitData<impl AsRef<str>>) -> Result<(Oid, Vec<u8>)> {
    const OIDS_SIZE: usize =
        1 /* register */ +
        1 /* parent */ +
        1 /* merge_parent */ +
        1 /* rebase_of */ +
        1 /* saves */;

    const BUF_SIZE: usize = super::HEADER_SIZE + super::rw::DATETIME_SIZE + OIDS_SIZE * Oid::LEN;

    let buf = Vec::with_capacity(BUF_SIZE);
    let mut buf = super::rw::Writer(buf);

    buf.write_kind(ObjectKind::Commit)?;
    buf.write_zeros::<DATA_SIZE>()?;

    buf.write_oid(commit.register)?;
    buf.write_oid(commit.saves)?;
    buf.write_optional_oid(commit.parent)?;
    buf.write_optional_oid(commit.merge_parent)?;
    buf.write_optional_oid(commit.rebase_of)?;

    buf.write_timestamp(commit.date)?;

    buf.write_null_terminated_string(commit.committer.as_ref())?;
    buf.write_null_terminated_string(commit.summary.as_ref())?;
    buf.write_null_terminated_string(commit.body.as_ref())?;

    let mut buf = buf.into_inner();
    let size: u32 = buf.len().try_into().expect("More than u32::MAX bytes");
    let size = size - DATA_SIZE as u32;

    buf[1..=DATA_SIZE].copy_from_slice(&size.to_le_bytes());

    let oid = braid_hash::hash(&buf);
    Ok((oid, buf))
}

pub(crate) fn read(reader: &mut impl std::io::Read) -> Result<ReadCommitData> {
    let mut reader = super::rw::Reader(reader);

    reader.expect_kind(ObjectKind::Commit)?;
    reader.eat::<DATA_SIZE>()?;

    let register = reader.read_oid()?;
    let saves = reader.read_oid()?;
    let parent = reader.read_optional_oid()?;
    let merge_parent = reader.read_optional_oid()?;
    let rebase_of = reader.read_optional_oid()?;

    let date = reader.read_timestamp()?;

    let committer = reader.read_null_terminated_string()?;
    let summary = reader.read_null_terminated_string()?;

    let body = reader.read_null_terminated_string()?;

    Ok(ReadCommitData {
        register,
        parent,
        merge_parent,
        rebase_of,
        saves,
        date,
        committer,
        summary,
        body,
    })
}
