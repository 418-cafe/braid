use sqlx::{pool::PoolConnection, PgConnection, PgPool, Postgres};
use thiserror::Error;

use crate::{sql::FetchOptional, object::SealedObject, Oid};

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}

pub(crate) struct Database {
    pool: PgPool,
}

impl Database {
    pub(crate) async fn get<T: SealedObject>(&self, oid: Oid) -> Result<Option<T>, Error> {
        let query = T::setup_get(oid);
        let mut conn = self.acquire().await?;
        query.fetch(&mut *conn).await
    }

    pub(crate) fn with_pool(pool: PgPool) -> Self {
        Database { pool }
    }

    async fn acquire(&self) -> Result<PoolConnection<Postgres>, Error> {
        self.pool.acquire().await.map_err(From::from)
    }
}
