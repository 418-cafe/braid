use sqlx::{encode::IsNull, error::BoxDynError, postgres::{PgArguments, PgHasArrayType, PgTypeInfo}, query::QueryScalar, Database, Decode, Encode, Postgres, Type};

type Query<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;

use crate::Oid;

type Result<T> = std::result::Result<T, BoxDynError>;

impl Type<Postgres> for Oid {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("bytea")
    }
}

impl Encode<'_, Postgres> for Oid {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull> {
        self.as_bytes().encode(buf)
    }
}

impl Decode<'_, Postgres> for Oid {
    fn decode(
        value: <Postgres as Database>::ValueRef<'_>,
    ) -> Result<Self> {
        let bytes = Decode::decode(value)?;
        Ok(Self::new(bytes))
    }
}

impl PgHasArrayType for Oid {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::array_of("bytea")
    }
}

impl<D: Database> Type<D> for crate::Key<'_>
where
    str: Type<D>,
{
    fn type_info() -> <D as Database>::TypeInfo {
        <str as Type<D>>::type_info()
    }
}

impl Encode<'_, Postgres> for crate::Key<'_> {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

impl<'d> Decode<'d, Postgres> for crate::Key<'d> {
    fn decode(
        value: <Postgres as Database>::ValueRef<'d>,
    ) -> Result<Self> {
        Ok(Self::new_unchecked(
            <&str as Decode<Postgres>>::decode(value)?,
        ))
    }
}

impl PgHasArrayType for crate::Key<'_> {
    fn array_type_info() -> PgTypeInfo {
        <&str as PgHasArrayType>::array_type_info()
    }
}

pub(crate) trait BindMany<T> {
    fn bind_many(self, value: T) -> Self;
}

macro_rules! impl_bind_many {
    (($($ident:ident),+)) => {
        impl<'a, $($ident),+> BindMany<($($ident),+)> for Query<'a>
        where
            $(
                $ident: 'a + Encode<'a, Postgres> + Type<Postgres>
            ),+
        {
            fn bind_many(self, value: ($($ident),+)) -> Self {
                #[allow(non_snake_case)]
                let ($($ident),+) = value;
                self
                $(
                    .bind($ident)
                )+
            }
        }

        impl<'a, O, $($ident),+> BindMany<($($ident),+)> for QueryScalar<'a, Postgres, O, PgArguments>
        where
            $(
                $ident: 'a + Encode<'a, Postgres> + Type<Postgres>
            ),+
        {
            fn bind_many(self, value: ($($ident),+)) -> Self {
                #[allow(non_snake_case)]
                let ($($ident),+) = value;
                self
                $(
                    .bind($ident)
                )+
            }
        }
    };
}

impl_bind_many!((T1, T2));
impl_bind_many!((T1, T2, T3));
impl_bind_many!((T1, T2, T3, T4));
impl_bind_many!((T1, T2, T3, T4, T5));
impl_bind_many!((T1, T2, T3, T4, T5, T6));