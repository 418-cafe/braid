use std::marker::PhantomData;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Key<S, K = String>(K, PhantomData<S>);

pub trait KeyStyle {}

macro_rules! key {
    (
        $style:ident,
        $err:ident,
        $($slash:literal => $slash_err:ident,)?
    ) => {
        #[derive(Debug, thiserror::Error)]
        pub enum $err {
            #[error("key is zero-length")]
            IsZeroLength,

            #[error("key contains null byte")]
            ContainsNullByte,

            $(
                #[error("key contains forward slash")]
                $slash_err,
            )?
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum $style {}

        impl KeyStyle for $style {}

        impl<'a> Key<$style, &'a str> {
            pub const fn new(key: &'a str) -> Result<Self, $err> {
                if key.is_empty() {
                    return Err($err::IsZeroLength);
                }

                let mut i = 0;
                while i < key.len() {
                    match key.as_bytes()[i] {
                        0 => return Err($err::ContainsNullByte),
                        $(
                            $slash => return Err($err::$slash_err),
                        )?
                        _ => i += 1,
                    }
                }

                Ok(Self::new_unchecked(key))
            }
        }
    };
}

key! {
    Full,
    FullKeyError,
}

key! {
    Entry,
    EntryKeyError,
    b'/' => ContainsForwardSlash,
}

impl<S, K> Key<S, K> {
    pub fn as_ref(&self) -> &K {
        &self.0
    }

    pub fn into_inner(self) -> K {
        self.0
    }
}

impl<S: KeyStyle, K> Key<S, K> {
    pub(crate) const fn new_unchecked(key: K) -> Self {
        Self(key, PhantomData)
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
