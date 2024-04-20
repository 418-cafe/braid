use braid_hash::Oid;

use crate::{bytes::Hash, save::SaveData, Result};

impl<S: AsRef<str>> super::write::Write for SaveData<S> {
    async fn write(&self, e: impl super::Executor<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        let query = sqlx::query(
            "
            INSERT INTO obj.save (id, author, date, kind, content, parent)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(*id.as_bytes())
        .bind(self.author.as_ref())
        .bind(self.date)
        .bind(self.kind)
        .bind(self.content.as_bytes())
        .bind(self.parent.oid.as_bytes());

        query.execute(e).await?;

        Ok(id)
    }
}
