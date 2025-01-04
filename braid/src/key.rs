#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("key is zero-length")]
    KeyIsZeroLength,

    #[error("key contains null byte")]
    KeyContainsNullByte,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Key<S>(S);

impl Key<&str> {
    pub const fn new(key: &str) -> Result<Key<&str>, Error> {
        if key.is_empty() {
            return const { Err(Error::KeyIsZeroLength) };
        }

        {
            let key = key.as_bytes();
            let mut i = 0;
            while i < key.len() {
                if key[i] == 0 {
                    return const { Err(Error::KeyContainsNullByte) };
                }
                i += 1;
            }
        }

        Ok(Key(key))
    }

    pub const fn as_str(&self) -> &str {
        self.0
    }
}

impl<S> Key<S> {
    pub(crate) const fn new_unchecked(key: S) -> Self {
        Self(key)
    }

    pub fn into_inner(self) -> S {
        self.0
    }

    pub const fn as_ref(&self) -> &S {
        &self.0
    }
}

impl Key<String> {
    pub fn as_str(&self) -> Key<&str> {
        Key(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_new() {
        assert!(Key::new("foo").is_ok());
        assert!(Key::new("foo\0bar").is_err());
    }
}
