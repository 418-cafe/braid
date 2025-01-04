use sqlx::PgExecutor;

use super::{types::BindMany, Result};
use crate::{Branch, FullKey};

#[allow(unused)]
pub(crate) async fn exists<'a>(ex: impl PgExecutor<'a>, branch: FullKey<&str>) -> Result<bool> {
    sqlx::query_scalar(r#"SELECT EXISTS(SELECT 1 FROM "branch" WHERE "name" = $1)"#)
        .bind(branch)
        .fetch_one(ex)
        .await
}

pub(crate) async fn persist(ex: impl PgExecutor<'_>, branch: &Branch<&str>) -> Result {
    let Branch {
        name,
        tip,
        is_default,
    } = branch;

    sqlx::query("INSERT INTO \"branch\" (name, tip, is_default) VALUES ($1, $2, $3)")
        .bind_many((name, tip, is_default))
        .execute(ex)
        .await?;

    Ok(())
}
