use braid_hash::Oid;

use crate::{bytes::Hash, commit::CommitData, Result};

impl<S: AsRef<str>> super::write::Write for CommitData<S> {
    async fn write(&self, e: impl super::Executor<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        let query = sqlx::query(
            "
            INSERT INTO obj.commit (id, register, parent, merge_parent, rebase_of, saves, date, committer, summary, body)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(id.as_bytes())
        .bind(self.register.as_bytes())
        .bind(self.parent.as_ref().map(|oid| oid.as_bytes()))
        .bind(self.merge_parent.as_ref().map(|oid| oid.as_bytes()))
        .bind(self.rebase_of.as_ref().map(|oid| oid.as_bytes()))
        .bind(self.saves.as_bytes())
        .bind(self.date)
        .bind(self.committer.as_ref())
        .bind(self.summary.as_ref())
        .bind(self.body.as_ref());

        query.execute(e).await?;
        Ok(id)
    }
}
