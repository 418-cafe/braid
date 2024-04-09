use thiserror::Error;

use crate::ObjectKind;

macro_rules! impl_key {
    ($name:ident ($kind:ident) => $fn:ident) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name<S>(S);

        impl<S> $name<S> {
            pub fn into_inner(self) -> S {
                self.0
            }

            pub(crate) fn new_unchecked(key: S) -> Self {
                Self(key)
            }
        }

        impl<S: AsRef<str>> $name<S> {
            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }

            pub fn as_ref(&self) -> $name<&S> {
                $name(&self.0)
            }
        }

        impl<S: AsRef<str>> Key<S> for $name<S> {
            const OBJECT_KIND: crate::ObjectKind = crate::ObjectKind::$kind;

            fn try_from(source: S) -> Result<Self, InvalidCharacterInKeyError> {
                for c in source.as_ref().chars() {
                    if let Some(invalid_char) = $fn(c) {
                        return Err(InvalidCharacterInKeyError {
                            key: source.as_ref().to_string(),
                            invalid_char,
                        });
                    }
                }

                Ok(Self(source))
            }
        }

        impl<S: AsRef<str>> AsRef<str> for $name<S> {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

pub(crate) trait Key<S>: AsRef<str>
where
    Self: Sized,
{
    const OBJECT_KIND: ObjectKind;

    fn try_from(source: S) -> Result<Self, InvalidCharacterInKeyError>;
}

impl_key!(RegisterEntryKey (Register) => is_invalid_unix_path_char);
impl_key!(SaveEntryKey  (SaveRegister) => is_non_slash_invalid_unix_path_char);

#[derive(Error)]
pub struct InvalidCharacterInKeyError {
    pub key: String,
    pub invalid_char: char,
}

impl std::fmt::Debug for InvalidCharacterInKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl std::fmt::Display for InvalidCharacterInKeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Invalid character in key `{:?}`: {:?}",
            self.key, self.invalid_char
        ))
    }
}

#[inline]
pub(crate) const fn is_invalid_unix_path_char(c: char) -> Option<char> {
    if c == '/' {
        return Some(c);
    }
    is_non_slash_invalid_unix_path_char(c)
}

#[inline]
pub(crate) const fn is_non_slash_invalid_unix_path_char(c: char) -> Option<char> {
    match c {
        '\0' | '\n' | '\r' => Some(c),
        _ => None,
    }
}
