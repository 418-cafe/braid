mod exists;
mod get_many;
mod persist;
mod types;

use exists::Exists;
use get_many::GetMany;
use persist::Persist;
use sqlx::Postgres;

use crate::{
    models::User, Oid,
};

pub type Transaction<'a> = sqlx::Transaction<'a, Postgres>;

type Result<T = ()> = std::result::Result<T, sqlx::Error>;

pub(crate) struct DatabaseTransaction<'t> {
    tx: Transaction<'t>,
}

impl<'t> DatabaseTransaction<'t> {
    pub(crate) fn open(tx: Transaction<'t>) -> Self {
        Self { tx }
    }

    pub(crate) fn into_inner(self) -> Transaction<'t> {
        self.tx
    }
}

impl DatabaseTransaction<'_> {
    pub(crate) async fn init(&mut self) -> Result {
        for statement in crate::sql::init_statements() {
            sqlx::query(statement).execute(&mut *self.tx).await?;
        }

        Ok(())
    }

    pub(crate) async fn write_external_object(&mut self, id: Oid) -> Result<bool> {
        sqlx::query("INSERT INTO external_object (id) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(id.as_bytes())
            .execute(&mut *self.tx)
            .await
            .map(|r| r.rows_affected() != 0)
    }

    pub(crate) async fn get_root(&mut self) -> Result<Option<crate::models::CommitWithImpl>> {
        const SELECT: &str = "
            SELECT
                c.id,
                c.subject,
                c.body,
                c.author,
                c.authored,
                ci.id AS impl_id,
                ci.parent,
                ci.merge_parent,
                ci.committer,
                ci.committed
            FROM \"commit\" c
            JOIN \"commit_impl\" ci ON c.id = ci.id
            WHERE ci.parent IS NULL
        ";

        sqlx::query_as(SELECT).fetch_optional(&mut *self.tx).await
    }

    pub(crate) async fn exists<'a, E: Exists<'a>>(&mut self, data: &'a E) -> Result<bool> {
        let query = sqlx::query_scalar(E::EXISTS);
        data.bind(query).fetch_one(&mut *self.tx).await
    }

    pub(crate) async fn persist<P: Persist>(
        &mut self,
        data: &P,
    ) -> std::result::Result<<P as Persist>::Output, <P as Persist>::Error> {
        data.execute(&mut self.tx).await
    }

    pub(crate) async fn get_many<T, C>(&mut self, criteria: C) -> Result<T>
    where
        T: GetMany<C>,
    {
        T::get_many(&mut self.tx, criteria).await
    }
}
