pub(crate) enum Error<K> {
    Kind(K),
    Unmapped(u8),
}

pub(crate) trait Kind: Copy + std::cmp::Eq + Sized {
    type Error: From<crate::kind::Error<Self>>;

    fn from_u8(value: u8) -> Option<Self>
    where
        Self: Sized;

    fn from_u8_or_err(value: u8) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_u8(value).ok_or(Error::Unmapped(value).into())
    }

    fn expect(value: u8, to_be: Self) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match Self::from_u8(value) {
            Some(kind) if kind == to_be => Ok(kind),
            Some(kind) => Err(Error::Kind(kind).into()),
            None => Err(Error::Unmapped(value).into()),
        }
    }

    fn test(self, against: Self) -> Result<(), Self::Error>
    where
        Self: Sized,
    {
        if self == against {
            Ok(())
        } else {
            Err(Error::Kind(self).into())
        }
    }

    fn as_u8(&self) -> u8;
}

macro_rules! kind {
    (
        $vis:vis enum $name:ident {
            $($variant:ident = $value:expr,)*
        }

        $err:ident => $display:expr
    ) => {
        #[repr(u8)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        $vis enum $name {
            $($variant = $value,)*
        }

        #[derive(Debug, thiserror::Error)]
        $vis enum $err {
            #[error($display)]
            Kind($name),
            #[error($display)]
            Unmapped(u8),
        }

        impl From<crate::kind::Error<$name>> for $err {
            #[inline]
            fn from(err: crate::kind::Error<$name>) -> Self {
                match err {
                    crate::kind::Error::Kind(kind) => $err::Kind(kind),
                    crate::kind::Error::Unmapped(byte) => $err::Unmapped(byte),
                }
            }
        }

        impl crate::Kind for $name {
            type Error = $err;

            fn from_u8(value: u8) -> Option<Self> {
                match value {
                    $(
                        $value => Some($name::$variant),
                    )*
                    _ => None,
                }
            }

            fn as_u8(&self) -> u8 {
                *self as u8
            }
        }
    };
}
