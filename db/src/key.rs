macro_rules! impl_key {
    ($name:ident => $fn:ident) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name<S>(S);

        impl<S: AsRef<str>> $name<S> {
            pub fn try_from(source: S) -> Result<Self, InvalidCharacterInKeyError<S>> {
                for c in source.as_ref().chars() {
                    if let Some(invalid_char) = $fn(c) {
                        return Err(InvalidCharacterInKeyError {
                            source,
                            invalid_char,
                        });
                    }
                }

                Ok(Self(source))
            }
        }

        impl<S: AsRef<str>> AsRef<str> for $name<S> {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }
    };
}

impl_key!(Key => is_invalid_unix_path_char);
impl_key!(KeyWithPathing => is_invalid_unix_path_char);

pub struct InvalidCharacterInKeyError<S> {
    pub source: S,
    pub invalid_char: char,
}

impl<S> std::fmt::Debug for InvalidCharacterInKeyError<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Invalid character in key: {:?}", self.invalid_char))
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