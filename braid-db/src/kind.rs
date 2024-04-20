pub(crate) struct UnmappedKindError(pub(crate) u8);

pub(crate) trait Kind: 'static + Copy + std::cmp::Eq + Sized {
    const MIN: Self;
    const MAX: Self;
    const VARIANTS: &'static [Self];

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

        impl $name {
            const MIN_MAX_VALUE: (Self, Self) = Self::min_max_value();

            const fn try_from_u8(value: u8) -> Option<Self> {
                match value {
                    $(
                        $value => Some($name::$variant),
                    )*
                    _ => None,
                }
            }

            #[allow(unused_comparisons)]
            #[allow(unused_assignments)]
            const fn min_max_value() -> (Self, Self) {
                const I8_MAX: u8 = i8::MAX as u8;

                let mut max = 0;
                let mut min = 0;

                // assert we start at 0 and increment by 1
                let mut running = 0;

                $(
                    assert!(running == $value);
                    running += 1;

                    min = if $value < min { $value } else { min };
                    max = if $value > max { $value } else { max };
                )*

                assert!(max <= I8_MAX);

                match (Self::try_from_u8(min), Self::try_from_u8(max)) {
                    (Some(min), Some(max)) => (min, max),
                    _ => unreachable!(),
                }
            }

            #[allow(unused_assignments)]
            const fn variants() -> [Self; Self::MIN_MAX_VALUE.1 as usize + 1] {
                let mut all = [Self::MIN_MAX_VALUE.0; Self::MIN_MAX_VALUE.1 as usize + 1];
                let mut i = 0;
                $(
                    all[i] = $name::$variant;
                    i += 1;
                )*
                all
            }
        }

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
            const MIN: Self = Self::MIN_MAX_VALUE.0;
            const MAX: Self = Self::MIN_MAX_VALUE.1;
            const VARIANTS: &'static [Self] = &Self::variants();

            fn from_u8(value: u8) -> Option<Self> {
                Self::try_from_u8(value)
            }

            fn as_u8(self) -> u8 {
                self as u8
            }
        }

        #[cfg(feature = "postgres")]
        impl sqlx::Encode<'_, sqlx::Postgres> for $name {
            fn encode_by_ref(&self, buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer) -> sqlx::encode::IsNull {
                <i16 as sqlx::Encode<'_, sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
            }
        }

        #[cfg(feature = "postgres")]
        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
                <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
            }

            fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
                <i16 as sqlx::Type<sqlx::Postgres>>::compatible(ty)
            }
        }
    };
}
