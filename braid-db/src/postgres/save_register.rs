use crate::{bytes::Hash, register::{SaveEntryCollection, SaveRegister}};

impl<S: Ord + AsRef<str>> super::write_to_tran::Write for SaveEntryCollection<S> {
    async fn write(&self, tran: &mut super::Transaction<'_>) -> crate::Result<braid_hash::Oid> {
        // we already inserted the empty collection on init
        if self.is_empty() {
            return Ok(SaveRegister::EMPTY_ID);
        }

        let (id, _) = Hash::hash(self)?;

        sqlx::query(
            "
            INSERT INTO obj.save_register (id)
            VALUES ($1)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(id.as_bytes())
        .execute(&mut **tran)
        .await?;

        let mut builder = sqlx::QueryBuilder::new(
            "
            INSERT INTO obj.save_entry (save_register, key, save)
            "
        );

        builder.push_values(self.iter(), |mut b, (key, content)| {
            b.push_bind(id.as_bytes())
            .push_bind(key.as_str().to_owned())
            .push_bind(content.as_bytes());
        });

        builder.push("ON CONFLICT DO NOTHING");

        builder.build().execute(&mut **tran).await?;

        Ok(id)
    }
}