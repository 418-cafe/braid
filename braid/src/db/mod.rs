pub(crate) mod branch;
pub(crate) mod commit;
pub(crate) mod commit_impl;
pub(crate) mod save;
pub(crate) mod state;
pub(crate) mod user;

mod types;

use sqlx::{PgExecutor, Postgres};

use crate::{models::User, Oid};

pub type Transaction<'a> = sqlx::Transaction<'a, Postgres>;

type Result<T = ()> = std::result::Result<T, sqlx::Error>;

pub(crate) async fn write_external_object(tx: impl PgExecutor<'_>, id: Oid) -> Result<bool> {
    sqlx::query("INSERT INTO external_object (id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(id.as_bytes())
        .execute(tx)
        .await
        .map(|r| r.rows_affected() != 0)
}

pub(crate) async fn init(tx: &mut Transaction<'_>) -> Result {
    for statement in crate::sql::init_statements() {
        sqlx::query(statement).execute(&mut **tx).await?;
    }

    Ok(())
}
