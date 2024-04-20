use braid_hash::Oid;
use sqlx::{Postgres, QueryBuilder};

use crate::{bytes::{register::Write, Hash}, register::{EntryData, Register, RegisterEntryCollection}, Result};

impl<S: AsRef<str>, D: AsRef<EntryData> + Write> super::write_to_tran::Write for RegisterEntryCollection<S, D> {
    async fn write(&self, e: &mut super::Transaction<'_>) -> Result<Oid> {
        // we already inserted the empty collection on init
        if self.is_empty() {
            return Ok(Register::EMPTY_ID);
        }

        let (id, _) = Hash::hash(self)?;

        let query = sqlx::query(
            "
            INSERT INTO obj.content_register (id, is_content)
            VALUES ($1, false)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(id.as_bytes());

        query.execute(&mut **e).await?;

        let mut builder = QueryBuilder::<Postgres>::new(
            "
            INSERT INTO obj.register_entry (register, key, content, is_executable)
            "
        );

        use crate::register::RegisterEntryKind::ExecutableContent;

        builder.push_values(self.iter(), |mut b, (key, entry)| {
                b.push_bind(id.as_bytes())
                .push_bind(key.as_str().to_owned())
                .push_bind(entry.as_ref().content.as_bytes())
                .push_bind(entry.as_ref().kind == ExecutableContent);
        });

        builder.push("ON CONFLICT DO NOTHING");

        builder.build().execute(&mut **e).await?;

        Ok(id)
    }
}