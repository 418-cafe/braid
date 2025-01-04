use sqlx::PgExecutor;

use crate::{Ancestry, CommitImpl};

use super::{types::BindMany, Result};

pub(crate) async fn persist(ex: impl PgExecutor<'_>, commit: &CommitImpl<&str>) -> Result {
    let CommitImpl {
        id,
        commit,
        ancestry,
        committer,
        committed,
    } = commit;

    let (parent, merge_parent) = match ancestry.as_ref() {
        Ancestry::Root => (None, None),
        Ancestry::Parent {
            parent,
            merge_parent,
        } => (Some(parent), merge_parent),
    };

    sqlx::query(r#"INSERT INTO "commit_impl" (id, commit, parent, merge_parent, committer, "committed") VALUES ($1, $2, $3, $4, $5, $6)"#)
        .bind_many((id, commit, parent, merge_parent, committer, committed))
        .execute(ex)
        .await?;

    Ok(())
}
