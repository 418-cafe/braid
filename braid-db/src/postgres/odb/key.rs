use sqlx::{postgres::PgTypeInfo, Decode, Encode, Postgres};

use crate::{RegisterEntryKey, SaveEntryKey};

macro_rules! impl_type {
    ($ty:ty) => {
        impl<S> sqlx::Type<Postgres> for $ty {
            fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
                PgTypeInfo::with_name("varchar")
            }
        }

        impl<'a, S: Encode<'a, Postgres>> Encode<'a, Postgres> for $ty {
            fn encode_by_ref(
                &self,
                buf: &mut <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                <S as Encode<Postgres>>::encode_by_ref(self.as_ref().into_inner(), buf)
            }
        }

        impl<'a, S: Decode<'a, Postgres>> Decode<'a, Postgres> for $ty {
            fn decode(
                value: <Postgres as sqlx::database::HasValueRef<'a>>::ValueRef,
            ) -> std::prelude::v1::Result<Self, sqlx::error::BoxDynError> {
                let inner = <S as Decode<Postgres>>::decode(value)?;
                Ok(Self::new_unchecked(inner))
            }
        }
    };
}

impl_type!(RegisterEntryKey<S>);
impl_type!(SaveEntryKey<S>);
