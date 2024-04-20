use braid_hash::Oid;
use sqlx::FromRow;

use crate::{
    bytes::Hash,
    commit::{Commit, CommitData},
    Result,
};

pub(super) async fn get(
    oid: Oid,
    exec: impl super::Executor<'_>,
) -> Result<Option<Commit<String>>> {
    let query = sqlx::query_as(
        "
        SELECT id, register, parent, merge_parent, rebase_of, saves, date, committer, summary, body
        FROM braid_obj.commit
        WHERE id = $1
        ",
    )
    .bind(oid);

    let commit = query.fetch_optional(exec).await?;
    Ok(commit)
}

impl<S: AsRef<str>> super::write_to_tran::Write for CommitData<S> {
    async fn write(&self, tran: &mut super::Transaction<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        sqlx::query("INSERT INTO braid_obj.save_parent (id, is_commit) VALUES ($1, true)")
            .bind(id)
            .execute(&mut **tran)
            .await?;

        let query = sqlx::query(
            "
            INSERT INTO braid_obj.commit (id, register, parent, merge_parent, rebase_of, saves, date, committer, summary, body)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(id)
        .bind(self.register.as_bytes())
        .bind(self.parent.as_ref().map(|oid| oid.as_bytes()))
        .bind(self.merge_parent.as_ref().map(|oid| oid.as_bytes()))
        .bind(self.rebase_of.as_ref().map(|oid| oid.as_bytes()))
        .bind(self.saves.as_bytes())
        .bind(self.date)
        .bind(self.committer.as_ref())
        .bind(self.summary.as_ref())
        .bind(self.body.as_ref());

        query.execute(&mut **tran).await?;
        Ok(id)
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Commit<String> {
    fn from_row(row: &sqlx::postgres::PgRow) -> std::result::Result<Self, sqlx::Error> {
        use sqlx::Row;

        let id = row.try_get("id")?;
        let data = FromRow::from_row(row)?;

        Ok(Self { id, data })
    }
}
