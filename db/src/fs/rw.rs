use std::io::{Read, Write};
use hash::Oid;
use time::UtcOffset;

use crate::{Kind, ObjectKind};
use super::{err::Error, DataSize, Result};


type UnixTimestamp = i128;
type OffsetSeconds = i32;

pub(crate) const DATETIME_SIZE: usize = OffsetDateTime::SIZE;

struct OffsetDateTime(time::OffsetDateTime);

impl OffsetDateTime {
    const TIMESTAMP_SIZE: usize = std::mem::size_of::<UnixTimestamp>();
    const OFFSET_SIZE: usize = std::mem::size_of::<OffsetSeconds>();
    const SIZE: usize = Self::TIMESTAMP_SIZE + Self::OFFSET_SIZE;

    fn try_from_le_bytes(bytes: [u8; Self::SIZE]) -> Result<Self> {
        let timestamp = UnixTimestamp::from_le_bytes(bytes[..Self::TIMESTAMP_SIZE].try_into().unwrap());
        let offset = OffsetSeconds::from_le_bytes(bytes[Self::TIMESTAMP_SIZE..].try_into().unwrap());
        let date = time::OffsetDateTime::from_unix_timestamp_nanos(timestamp)
            .map_err(|e| Error::InvalidTimestamp(e))?;
        let offset = UtcOffset::from_whole_seconds(offset).map_err(|e| Error::InvalidOffset(e))?;
        Ok(Self(date.to_offset(offset)))
    }

    fn to_le_bytes(&self) -> [u8; Self::SIZE] {
        let timestamp = self.0.unix_timestamp_nanos();
        let offset = self.0.offset().whole_seconds();
        let mut bytes = [0; Self::SIZE];
        bytes[..Self::TIMESTAMP_SIZE].copy_from_slice(&timestamp.to_le_bytes());
        bytes[Self::TIMESTAMP_SIZE..].copy_from_slice(&offset.to_le_bytes());
        bytes
    }
}

pub(crate) struct Reader<R>(pub(crate) R);

impl<R: Read> Reader<R> {
    #[inline]
    pub(crate) fn eat<const N: usize>(&mut self) -> Result<()> {
        self.0.read_exact(&mut [0; N])?;
        Ok(())
    }

    #[inline]
    pub(crate) fn read_header(&mut self) -> Result<(ObjectKind, DataSize)> {
        let kind = self.read_kind()?;
        let size = self.read_le_bytes()?;
        Ok((kind, size))
    }

    pub(crate) fn expect_kind<E: From<crate::kind::Error<K>>, K: Kind<Error = E>>(&mut self, expected: K) -> Result<()>
    where Error: From<E>
    {
        match self.read_kind() {
            Ok(kind) if kind == expected => Ok(()),
            Ok(kind) => {
                let error = crate::kind::Error::Kind(kind);
                let error = E::from(error);
                Err(error.into())
            },
            Err(e) => Err(e),
        }
    }

    #[inline]
    pub(crate) fn read_oid(&mut self) -> Result<Oid> {
        let mut bytes = [0; Oid::LEN];
        self.0.read_exact(&mut bytes)?;
        Ok(Oid::from_bytes(bytes))
    }

    #[inline]
    pub(crate) fn read_optional_oid(&mut self) -> Result<Option<Oid>> {
        let mut bytes = [0; Oid::LEN];
        self.0.read_exact(&mut bytes)?;
        if &bytes == Oid::ZERO.as_bytes() {
            Ok(None)
        } else {
            Ok(Some(Oid::from_bytes(bytes)))
        }
    }

    #[inline]
    pub(crate) fn read_timestamp(&mut self) -> Result<time::OffsetDateTime> {
        let mut bytes = [0; OffsetDateTime::SIZE];
        self.0.read_exact(&mut bytes)?;
        OffsetDateTime::try_from_le_bytes(bytes).map(|odt| odt.0)
    }

    #[inline]
    pub(crate) fn read_null_terminated_string(&mut self) -> Result<String> {
        let mut buf = Vec::new();
        loop {
            let mut byte = [0];
            self.0.read_exact(&mut byte)?;
            if byte[0] == 0 {
                break String::from_utf8(buf).map_err(|e| e.into())
            }
            buf.push(byte[0]);
        }
    }

    #[inline]
    pub(crate) fn read_string_until_end(&mut self) -> Result<String> {
        let mut buf = String::new();
        self.0.read_to_string(&mut buf)?;
        Ok(buf)
    }

    #[inline]
    pub(crate) fn read_le_bytes<const N: usize, T: LeBytes<N>>(&mut self) -> Result<T> {
        let mut bytes = [0; N];
        self.0.read_exact(&mut bytes)?;
        Ok(T::from_le_bytes(bytes))
    }

    #[inline]
    pub(crate) fn read_kind<E, K: Kind<Error = E>>(&mut self) -> Result<K> where Error: From<E>{
        let mut bytes = [0; 1];
        self.0.read_exact(&mut bytes)?;
        let byte = bytes[0];
        let kind = K::from_u8_or_err(byte)?;
        Ok(kind)
    }
}

pub(crate) struct Writer<W>(pub(crate) W);

impl<W: Write> Writer<W> {
    #[inline]
    pub(crate) fn write_zeros<const N: usize>(&mut self) -> Result<()> {
        self.0.write_all(&[0; N])?;
        Ok(())
    }

    #[inline]
    pub(crate) fn write_kind(&mut self, kind: impl Kind) -> Result<()> {
        self.0.write_all(&[kind.as_u8()])?;
        Ok(())
    }

    #[inline]
    pub(crate) fn write_oid(&mut self, oid: Oid) -> Result<()> {
        self.0.write_all(oid.as_bytes())?;
        Ok(())
    }

    #[inline]
    pub(crate) fn write_optional_oid(&mut self, oid: Option<Oid>) -> Result<()> {
        self.write_oid(oid.unwrap_or(Oid::ZERO))
    }

    #[inline]
    pub(crate) fn write_timestamp(&mut self, timestamp: time::OffsetDateTime) -> Result<()> {
        let time = OffsetDateTime(timestamp);
        self.0.write_all(&time.to_le_bytes())?;
        Ok(())
    }

    #[inline]
    pub(crate) fn write_string(&mut self, string: &str) -> Result<()> {
        self.0.write_all(string.as_bytes())?;
        Ok(())
    }

    #[inline]
    pub(crate) fn write_null_terminated_string(&mut self, string: &str) -> Result<()> {
        self.write_string(string)?;
        self.0.write_all(&[0])?;
        Ok(())
    }

    #[inline]
    pub(crate) fn write_le_bytes<const N: usize, T: LeBytes<N>>(&mut self, value: T) -> Result<()> {
        let bytes = value.to_le_bytes();
        self.0.write_all(&bytes)?;
        Ok(())
    }

    #[inline]
    pub(crate) fn into_inner(self) -> W {
        self.0
    }
}

pub(crate) trait LeBytes<const N: usize> {
    fn from_le_bytes(bytes: [u8; N]) -> Self;

    fn to_le_bytes(self) -> [u8; N];
}

macro_rules! impl_le_bytes {
    ($($ty:ident),*) => {
        $(
            #[allow(non_upper_case_globals)]
            const $ty: usize = std::mem::size_of::<$ty>();
            impl LeBytes<{$ty}> for $ty {
                #[inline]
                fn from_le_bytes(bytes: [u8; {$ty}]) -> Self {
                    Self::from_le_bytes(bytes)
                }

                #[inline]
                fn to_le_bytes(self) -> [u8; {$ty}] {
                    Self::to_le_bytes(self)
                }
            }
        )*
    };
}

impl_le_bytes!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);