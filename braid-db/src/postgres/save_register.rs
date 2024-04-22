use braid_hash::Oid;

use crate::{
    bytes::Hash,
    register::{SaveEntryCollection, SaveRegister},
    Result, SaveEntryKey,
};

pub(super) async fn get(
    id: Oid,
    exec: impl super::Executor<'_>,
) -> Result<Option<SaveRegister<String>>> {
    let entries = sqlx::query_as::<_, Entry>(
        "
        SELECT sre.key, sre.save
        FROM braid.save_register as sr
        INNER JOIN braid.save_register_entry as sre
        ON sr.id = sre.save_register
        WHERE sr.id = $1
        ",
    )
    .bind(id.as_bytes())
    .fetch_all(exec)
    .await?;

    let mut data = SaveEntryCollection::new();

    for Entry { key, save } in entries {
        let key = SaveEntryKey::new_unchecked(key);
        data.insert(key, save);
    }

    Ok(Some(SaveRegister { id, data }))
}

#[derive(sqlx::FromRow)]
struct Entry {
    key: String,
    save: Oid,
}

impl<S: Ord + AsRef<str>> super::write_to_tran::Write for SaveEntryCollection<S> {
    async fn write(&self, tran: &mut super::Transaction<'_>) -> crate::Result<braid_hash::Oid> {
        // we already inserted the empty collection on init
        if self.is_empty() {
            return Ok(SaveRegister::EMPTY_ID);
        }

        let (id, _) = Hash::hash(self)?;

        sqlx::query(
            "
            INSERT INTO braid.save_register (id)
            VALUES ($1)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(id.as_bytes())
        .execute(&mut **tran)
        .await?;

        let mut builder = sqlx::QueryBuilder::new(
            "
            INSERT INTO braid.save_register_entry (save_register, key, save)
            ",
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
