use sqlx::PgExecutor;

use crate::{Commit, CommitWithImpl};

use super::{types::BindMany, Result};

pub(crate) async fn persist(ex: impl PgExecutor<'_>, commit: &Commit<&str>) -> Result {
    let Commit {
        id,
        subject,
        body,
        author,
        authored,
    } = commit;

    sqlx::query(
        r#"INSERT INTO "commit" (id, subject, body, author, authored) VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind_many((id, subject, body, author, authored))
    .execute(ex)
    .await?;

    Ok(())
}

pub(crate) async fn get_root(ex: impl PgExecutor<'_>) -> Result<Option<CommitWithImpl>> {
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

    sqlx::query_as(SELECT).fetch_optional(ex).await
}
