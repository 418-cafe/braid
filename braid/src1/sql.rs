use sqlx::{FromRow, PgExecutor};

use crate::{db::Error, object::SealedObject, ObjectId, Oid};

pub(crate) trait SetupGet: for<'b> FromRow<'b, sqlx::postgres::PgRow> {
    const QUERY: &'static str;

    fn setup_get(oid: Oid) -> impl FetchOptional<Self> {
        let query = sqlx::query(Self::QUERY).bind(*oid.as_bytes());

        sql_impl::FetchOptional { query }
    }
}

pub(crate) trait FetchOptional<T> {
    async fn fetch(self, executor: impl PgExecutor<'_>) -> Result<Option<T>, Error>;
}

impl<T: ObjectId + SetupGet> SealedObject for T {
    fn setup_get(oid: Oid) -> impl FetchOptional<Self> {
        Self::setup_get(oid)
    }
}

mod sql_impl {
    use sqlx::{postgres::PgArguments, query::Query, FromRow, PgExecutor, Postgres};

    use crate::db::Error;

    pub(crate) struct FetchOptional<'a> {
        pub(super) query: Query<'a, Postgres, PgArguments>,
    }

    impl<'a, T> super::FetchOptional<T> for FetchOptional<'a>
    where
        T: for<'b> FromRow<'b, sqlx::postgres::PgRow>,
    {
        async fn fetch(self, executor: impl PgExecutor<'_>) -> Result<Option<T>, Error> {
            let row = self.query.fetch_optional(executor).await?;

            let row = match row {
                None => return Ok(None),
                Some(row) => row,
            };

            let result = FromRow::from_row(&row)?;

            Ok(Some(result))
        }
    }
}
