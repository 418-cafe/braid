use sqlx::{Decode, Encode, Postgres, Type};

use crate::{Oid, OID_LEN};

impl Encode<'_, Postgres> for Oid {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer,
    ) -> sqlx::encode::IsNull {
        <[u8; OID_LEN] as Encode<Postgres>>::encode_by_ref(self.as_bytes(), buf)
    }
}

impl Decode<'_, Postgres> for Oid {
    fn decode(
        value: <Postgres as sqlx::database::HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = <[u8; OID_LEN] as Decode<Postgres>>::decode(value)?;
        Ok(Oid::from_bytes(bytes))
    }
}

impl Type<Postgres> for Oid {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <[u8; OID_LEN] as Type<Postgres>>::type_info()
    }
}
