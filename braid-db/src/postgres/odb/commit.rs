use braid_hash::Oid;

use crate::{
    bytes::Hash,
    commit::{Commit, CommitData},
    Result,
};

use super::OidData;

pub(super) async fn get(
    oid: Oid,
    exec: impl super::Executor<'_>,
) -> Result<Option<Commit<String>>> {
    let commit = sqlx::query_as("SELECT * FROM braid.get_commit($1)")
        .bind(oid)
        .fetch_optional(exec).await?;
    Ok(commit)
}

impl<S: AsRef<str>> super::write::Write for CommitData<S> {
    async fn write(&self, exec: impl super::Executor<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        sqlx::query("CALL braid.create_commit($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)")
            .bind(id)
            .bind(self.register)
            .bind(self.parent)
            .bind(self.merge_parent)
            .bind(self.rebase_of)
            .bind(self.saves)
            .bind(self.date)
            .bind(self.committer.as_ref())
            .bind(self.summary.as_ref())
            .bind(self.body.as_ref())
            .execute(exec)
            .await?;

        Ok(id)
    }
}

impl OidData<'_> for Commit<String> {
    type Data = CommitData<String>;

    fn create(id: Oid, data: Self::Data) -> Self {
        Self { id, data }
    }
}
