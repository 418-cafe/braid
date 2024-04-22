use braid_hash::Oid;
use sqlx::{Postgres, QueryBuilder};

use crate::{
    bytes::{register::Write, Hash},
    register::{EntryData, Register, RegisterEntryCollection, RegisterEntryKind},
    RegisterEntryKey, Result,
};

pub(super) async fn get(
    id: Oid,
    exec: impl super::Executor<'_>,
) -> Result<Option<Register<String, EntryData>>> {
    let entries = sqlx::query_as::<_, Entry>(
        "
        SELECT re.key, re.content, re.is_executable, cr.is_content
        FROM braid.register as r
        INNER JOIN braid.register_entry as re
        ON r.id = re.register
        INNER JOIN braid.content_register as cr
        ON cr.id = re.content
        WHERE r.id = $1
        ",
    )
    .bind(id.as_bytes())
    .fetch_all(exec)
    .await?;

    let mut data = RegisterEntryCollection::new();

    for Entry {
        key,
        content,
        is_executable,
        is_content,
    } in entries
    {
        let key = RegisterEntryKey::new_unchecked(key);
        let kind = match (is_content, is_executable) {
            (false, _) => RegisterEntryKind::Register,
            (true, false) => RegisterEntryKind::Content,
            (true, true) => RegisterEntryKind::ExecutableContent,
        };
        data.insert(key, EntryData { content, kind });
    }

    Ok(Some(Register { id, data }))
}

impl<S: AsRef<str>, D: AsRef<EntryData> + Write> super::write_to_tran::Write
    for RegisterEntryCollection<S, D>
{
    async fn write(&self, e: &mut super::Transaction<'_>) -> Result<Oid> {
        // we already inserted the empty collection on init
        if self.is_empty() {
            return Ok(Register::EMPTY_ID);
        }

        let (id, _) = Hash::hash(self)?;

        let query = sqlx::query(
            "
            INSERT INTO braid.content_register (id, is_content)
            VALUES ($1, false)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(id.as_bytes());

        query.execute(&mut **e).await?;

        let mut builder = QueryBuilder::<Postgres>::new(
            "
            INSERT INTO braid.register_entry (register, key, content, is_executable)
            ",
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

#[derive(sqlx::FromRow)]
struct Entry {
    key: String,
    content: Oid,
    is_executable: bool,
    is_content: bool,
}
