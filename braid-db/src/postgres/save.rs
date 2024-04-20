use braid_hash::Oid;

use crate::{bytes::Hash, save::SaveData, Result};

impl<S: AsRef<str>> super::write_to_tran::Write for SaveData<S> {
    async fn write(&self, tran: &mut super::Transaction<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        sqlx::query("INSERT INTO braid_obj.save_parent (id, is_commit) VALUES ($1, false)")
            .bind(id)
            .execute(&mut **tran)
            .await?;

        let query = sqlx::query(
            "
            INSERT INTO braid_obj.save (id, author, date, kind, content, parent)
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

        query.execute(&mut **tran).await?;

        Ok(id)
    }
}
