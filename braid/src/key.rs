use std::marker::PhantomData;

pub trait KeyStyle {}

#[derive(Debug, thiserror::Error)]
pub enum FullKeyError {
    #[error("key is zero-length")]
    IsZeroLength,

    #[error("key contains null byte")]
    ContainsNullByte,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Full {}

impl KeyStyle for Full {}

#[derive(Debug, thiserror::Error)]
pub enum EntryKeyError {
    #[error("key is zero-length")]
    IsZeroLength,

    #[error("key contains null byte")]
    ContainsNullByte,

    #[error("key contains forward slash")]
    ContainsForwardSlash,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Entry {}

impl KeyStyle for Entry {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Key<S, K = String>(K, PhantomData<S>);

macro_rules! validate {
    (
        $key:ident : $zero:path {
            $($lit:literal => $err:path),+
            $(,)?
        }
    ) => {
        if $key.is_empty() {
            return const { Err($zero) };
        }

        {
            let mut i = 0;
            while i < $key.len() {
                match $key[i] {
                    $(
                        $lit => return const { Err($err) }
                    ),+,
                    _ => i += 1,
                }
            }
        }
    };
}

impl<S, K> Key<S, K> {
    pub fn as_ref(&self) -> &K {
        &self.0
    }
}

impl<S: KeyStyle, K> Key<S, K> {
    pub(crate) const fn new_unchecked(key: K) -> Self {
        Self(key, PhantomData)
    }
}

impl<K> Key<Full, K> {
    const fn validate(key: &[u8]) -> Result<(), FullKeyError> {
        validate! {
            key: FullKeyError::IsZeroLength {
                0 => FullKeyError::ContainsNullByte,
            }
        }

        Ok(())
    }
}

impl<K> Key<Entry, K> {
    const fn validate(key: &[u8]) -> Result<(), EntryKeyError> {
        validate! {
            key: EntryKeyError::IsZeroLength {
                0 => EntryKeyError::ContainsNullByte,
                b'/' => EntryKeyError::ContainsForwardSlash,
            }
        }

        Ok(())
    }
}

impl<'a> Key<Full, &'a str> {
    pub const fn new(key: &'a str) -> Result<Self, FullKeyError> {
        match Self::validate(key.as_bytes()) {
            Ok(()) => Ok(Self::new_unchecked(key)),
            Err(e) => Err(e),
        }
    }
}

impl<'a> Key<Entry, &'a str> {
    pub const fn new(key: &'a str) -> Result<Self, EntryKeyError> {
        match Self::validate(key.as_bytes()) {
            Ok(()) => Ok(Self::new_unchecked(key)),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{EntryKey, FullKey};

    #[test]
    fn test_full_key_new() {
        assert!(FullKey::new("foo").is_ok());
        assert!(FullKey::new("foo\0bar").is_err());
        assert!(FullKey::new("foo/bar").is_ok());
        assert!(FullKey::new("foo/bar\0baz").is_err());
    }

    #[test]
    fn test_entry_key_new() {
        assert!(EntryKey::new("foo").is_ok());
        assert!(EntryKey::new("foo\0bar").is_err());
        assert!(EntryKey::new("foo/bar").is_err());
        assert!(EntryKey::new("foo/bar\0baz").is_err());
    }
}
