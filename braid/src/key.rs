#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("key is zero-length")]
    KeyIsZeroLength,

    #[error("key contains null byte")]
    KeyContainsNullByte,
}

#[derive(Clone, Copy)]
pub struct Key<'a>(&'a str);

impl<'a> Key<'a> {
    pub const fn new(key: &'a str) -> Result<Self, Error> {
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

        Ok(Self(key))
    }

    pub(crate) const fn new_unchecked(key: &'a str) -> Self {
        Self(key)
    }

    pub const fn as_str(&self) -> &'a str {
        self.0
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
