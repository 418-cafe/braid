pub(crate) struct UnmappedKindError(pub(crate) u8);

pub(crate) trait Kind: Copy + std::cmp::Eq + Sized {
    type Error: From<crate::kind::UnmappedKindError>;

    fn from_u8(value: u8) -> Option<Self>
    where
        Self: Sized;

    fn try_from_u8(value: u8) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Self::from_u8(value).ok_or(UnmappedKindError(value).into())
    }

    fn as_u8(self) -> u8;
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
        $vis struct $err(pub u8);

        impl From<crate::kind::UnmappedKindError> for $err {
            #[inline]
            fn from(err: crate::kind::UnmappedKindError) -> Self {
                Self(err.0)
            }
        }

        impl std::fmt::Display for $err {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                <Self as std::fmt::Debug>::fmt(self, f)
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

            fn as_u8(self) -> u8 {
                self as u8
            }
        }
    };
}
