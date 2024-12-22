use sqlx::types::chrono::{self};

use crate::const_unwrap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixedOffset(pub(crate) chrono::FixedOffset);

impl From<chrono::FixedOffset> for FixedOffset {
    fn from(value: chrono::FixedOffset) -> Self {
        Self(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct DateTime(chrono::DateTime<chrono::FixedOffset>);

impl sqlx::Type<sqlx::Postgres> for DateTime {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        <chrono::DateTime<chrono::FixedOffset> as sqlx::Type<_>>::type_info()
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

impl From<chrono::DateTime<chrono::FixedOffset>> for DateTime {
    fn from(value: chrono::DateTime<chrono::FixedOffset>) -> Self {
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
    now_with_offset(const { Option::<chrono::FixedOffset>::None })
}

pub(crate) fn now_with_offset(tz: Option<impl IntoChrono>) -> DateTime {
    DateTime(
        chrono::Utc::now().with_timezone(
            &tz.map(IntoChrono::into_chrono)
                .unwrap_or(const_unwrap!(Some of chrono::FixedOffset::east_opt(0))),
        ),
    )
}

pub(crate) trait IntoChrono {
    fn into_chrono(self) -> chrono::FixedOffset;
}

impl IntoChrono for chrono::FixedOffset {
    #[inline]
    fn into_chrono(self) -> chrono::FixedOffset {
        self
    }
}

impl IntoChrono for super::FixedOffset {
    #[inline]
    fn into_chrono(self) -> chrono::FixedOffset {
        self.0
    }
}
