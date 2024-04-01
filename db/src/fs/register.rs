use hash::Oid;
use itertools::Itertools;
use std::io::Write;

use super::err::{Error, WasObjectKind};
use super::Result;
use crate::fs::err::WasEntryKind;
use crate::register::EntryKind;
use crate::{
    register::Entry, ObjectKind,
};

const OBJECT_KIND_SIZE: usize = std::mem::size_of::<crate::ObjectKind>();
const DATA_SIZE: usize = std::mem::size_of::<u32>();
const LEN_SIZE: usize = std::mem::size_of::<u32>();

const HEADER_SIZE: usize = OBJECT_KIND_SIZE + DATA_SIZE + LEN_SIZE;

const NEWLINE_SIZE: usize = '\n'.len_utf8();
const AVG_STR_SIZE: usize = 20;

const ENTRY_SIZE: usize = Oid::LEN + AVG_STR_SIZE + NEWLINE_SIZE;

fn hash<'a>(data: impl Iterator<Item = &'a Entry<impl 'a + AsRef<str>>>) -> Result<(Oid, Vec<u8>)> {
    let data = data.sorted_by(|lhs, rhs| lhs.name.as_ref().cmp(rhs.name.as_ref()));
    
    let buf = HEADER_SIZE + data.len() * ENTRY_SIZE;
    let mut buf = Vec::with_capacity(buf);

    buf.write_all(&[ObjectKind::Register as u8])?;
    buf.write_all(&[0; DATA_SIZE])?;

    let len: u32 = data.len().try_into().expect("More than u32::MAX entries");
    buf.write_all(&len.to_le_bytes())?;

    for entry in data {
        buf.write_all(entry.id.as_bytes())?;
        buf.write_all(&[entry.kind as u8])?;
        buf.write_all(entry.name.as_ref().as_bytes())?;
        buf.write_all(b"\n")?;
    }

    let size: u32 = buf.len().try_into().expect("More than u32::MAX bytes");
    let size = size - HEADER_SIZE as u32;

    buf[1..=DATA_SIZE].copy_from_slice(&size.to_le_bytes());

    let oid = hash::hash(&buf[HEADER_SIZE..]);
    Ok((oid, buf))
}

fn read<R: std::io::Read>(mut reader: R) -> Result<ReadIter<R>> {
    const EXPECTED: ObjectKind = ObjectKind::Register;

    let mut bytes = [0; OBJECT_KIND_SIZE];

    reader.read_exact(&mut bytes).map_err(|e| match e.kind() {
        std::io::ErrorKind::UnexpectedEof => Error::InvalidObjectKind {
            expected: EXPECTED,
            was: WasObjectKind::Missing,
        },
        _ => e.into(),
    })?;

    let kind = ObjectKind::from_u8(bytes[0]).ok_or_else(|| Error::InvalidObjectKind {
        expected: EXPECTED,
        was: WasObjectKind::Unmapped(bytes[0]),
    })?;

    if kind != EXPECTED {
        return Err(Error::InvalidObjectKind {
            expected: EXPECTED,
            was: WasObjectKind::Mapped(kind),
        });
    }

    reader.read_exact(&mut [0; DATA_SIZE])?;

    let mut bytes = [0; LEN_SIZE];
    reader.read_exact(&mut bytes)?;
    let len = u32::from_be_bytes(bytes) as usize;

    Ok(ReadIter {
        reader,
        len,
        remaining: len,
    })
}

struct ReadIter<R> {
    reader: R,
    len: usize,
    remaining: usize,
}

impl<R: std::io::Read> Iterator for ReadIter<R> {
    type Item = Result<Entry<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let next = self.next_impl();

        if next.is_err() {
            self.remaining = 0;
        } else {
            self.remaining -= 1;
        }

        Some(next)
    }
}

impl<R: std::io::Read> ReadIter<R> {
    fn next_impl(&mut self) -> <Self as Iterator>::Item {
        let mut oid = [0; Oid::LEN];
        self.reader.read_exact(&mut oid)?;
        let oid = Oid::from_bytes(oid);

        let mut kind = [0];
        self.reader.read_exact(&mut kind)?;
        let kind = EntryKind::from_u8(kind[0]).ok_or_else(|| Error::InvalidEntryKind {
            was: WasEntryKind::Unmapped(kind[0]),
        })?;

        let mut name = Vec::new();
        let mut byte = [0];
        let name = loop {
            self.reader.read_exact(&mut byte)?;
            if byte[0] == b'\n' {
                break String::from_utf8(name)?;
            }

            name.push(byte[0]);
        };

        Ok(Entry {
            id: oid,
            name,
            kind,
        })
    }
}

impl<R: std::io::Read> ExactSizeIterator for ReadIter<R> {
    fn len(&self) -> usize {
        self.len
    }
}