use sqlx::types::chrono::{self, FixedOffset};

pub(crate) fn now_utc() -> chrono::DateTime<FixedOffset> {
    now_with_offset(None)
}

pub(crate) fn now_with_offset(tz: Option<FixedOffset>) -> chrono::DateTime<FixedOffset> {
    chrono::Utc::now().with_timezone(&tz.unwrap_or(
        const {
            match FixedOffset::east_opt(0) {
                Some(offset) => offset,
                None => panic!("UTC offset of 0 should always be valid"),
            }
        },
    ))
}
