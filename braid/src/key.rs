use std::ops::Not;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("key is zero-length")]
    KeyIsZeroLength,

    #[error("key contains null byte")]
    KeyContainsNullByte,
}

pub struct Key<'a>(&'a str);

impl<'a> Key<'a> {
    pub fn new(key: &'a str) -> Result<Self, Error> {
        if key.is_empty() {
            return Err(Error::KeyIsZeroLength);
        }

        key.contains('\0')
            .not()
            .then_some(Self(key))
            .ok_or(Error::KeyContainsNullByte)
    }

    pub fn as_str(&self) -> &'a str {
        self.0
    }

    pub(crate) const fn new_unchecked(key: &'a str) -> Self {
        Self(key)
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
