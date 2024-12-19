use sqlx::types::chrono::{self, FixedOffset};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DateTime(chrono::DateTime<FixedOffset>);

impl sqlx::Type<sqlx::Postgres> for DateTime {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <chrono::DateTime<FixedOffset> as sqlx::Type<_>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DateTime {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        sqlx::Encode::encode(self.0, buf)
    }
}

impl<'a> sqlx::Decode<'a, sqlx::Postgres> for DateTime {
    fn decode(
        value: <sqlx::Postgres as sqlx::Database>::ValueRef<'a>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        Ok(Self(sqlx::Decode::decode(value)?))
    }
}

impl From<chrono::DateTime<FixedOffset>> for DateTime {
    fn from(value: chrono::DateTime<FixedOffset>) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Debug for DateTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl crate::Hash for DateTime {
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        crate::Hash::hash(&self.0.timestamp_millis(), hasher);
    }
}

impl DateTime {
    pub(crate) fn timestamp_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }
}

pub(crate) fn now_utc() -> DateTime {
    now_with_offset(None)
}

pub(crate) fn now_with_offset(tz: Option<FixedOffset>) -> DateTime {
    DateTime(chrono::Utc::now().with_timezone(&tz.unwrap_or(
        const {
            match FixedOffset::east_opt(0) {
                Some(offset) => offset,
                None => panic!("UTC offset of 0 should always be valid"),
            }
        },
    )))
}
