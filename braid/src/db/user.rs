use sqlx::PgExecutor;

use super::{Result, User};

pub(crate) async fn persist(ex: impl PgExecutor<'_>, user: &User<'_>) -> Result {
    let User(id) = user;

    sqlx::query(r#"INSERT INTO "user"(id) VALUES ($1)"#)
        .bind(id)
        .execute(ex)
        .await?;

    Ok(())
}
