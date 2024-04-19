use braid_hash::Oid;
use sqlx::{
    postgres::{PgArguments, PgPool}, PgConnection, Postgres
};

use crate::Result;

mod commit;
mod init;
mod register;
mod save;

type PgTransaction<'a> = sqlx::Transaction<'a, Postgres>;
type Query<'a> = sqlx::query::Query<'a, Postgres, PgArguments>;

trait Executor<'a>: sqlx::Executor<'a, Database = Postgres> {}
impl<'a> Executor<'a> for &'a mut PgConnection {}
impl<'a> Executor<'a> for &'a PgPool {}

pub struct Database {
    inner: PgPool,
}

impl Database {
    pub async fn new(pool: PgPool) -> Result<Self> {
        Ok(Self { inner: pool })
    }

    pub async fn init(&self) -> Result<()> {
        let mut tran = self.inner.begin().await?;
        init::init(&mut tran).await?;
        tran.commit().await?;
        Ok(())
    }

    pub async fn write(&self, obj: impl queryable::QueryAble) -> Result<()> {
        write(&self.inner, obj).await
    }

    pub async fn register_content(&self, oid: Oid) -> Result<()> {
        register_content(&self.inner, oid).await
    }

    pub async fn begin_transaction(&self) -> Result<Transaction> {
        Ok(Transaction {
            inner: self.inner.begin().await?,
        })
    }
}

pub struct Transaction<'a> {
    inner: PgTransaction<'a>,
}

impl<'a> Transaction<'a> {
    pub async fn write(&mut self, obj: impl queryable::QueryAble) -> Result<()> {
        write(&mut *self.inner, obj).await
    }

    pub async fn register_content(&mut self, oid: Oid) -> Result<()> {
        register_content(&mut *self.inner, oid).await
    }

    pub async fn commit(self) -> Result<()> {
        self.inner.commit().await.map_err(Into::into)
    }

    pub fn inner_transaction(&self) -> &PgTransaction {
        &self.inner
    }

    pub fn inner_transaction_mut(&mut self) -> &mut PgTransaction<'a> {
        &mut self.inner
    }
}

async fn write(exec: impl Executor<'_>, obj: impl queryable::QueryAble) -> Result<()> {
    let query = obj.insert_query()?;
    query.execute(exec).await?;
    Ok(())
}

async fn register_content(exec: impl Executor<'_>, oid: Oid) -> Result<()> {
    let query = sqlx::query(
        "
        INSERT INTO obj.content (id)
        VALUES ($1)
        ON CONFLICT DO NOTHING
        ",
    )
    .bind(*oid.as_bytes());

    query.execute(exec).await?;
    Ok(())
}

mod queryable {
    pub trait QueryAble: crate::bytes::Hash {
        fn insert_query(&self) -> crate::Result<super::Query>;
    }
}