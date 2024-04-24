use braid_hash::Oid;
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    Postgres, Type,
};

use crate::{
    bytes::{register::Write, Hash},
    postgres::Executor,
    register::{EntryData, Register, RegisterEntryCollection, RegisterEntryKind},
    RegisterEntryKey, Result,
};

pub(super) async fn get(
    id: Oid,
    exec: impl Executor<'_>,
) -> Result<Option<Register<String, EntryData>>> {
    let entries = sqlx::query_as("SELECT * FROM braid.get_register($1)")
        .bind(id.as_bytes())
        .fetch_all(exec)
        .await?;

    let mut data = RegisterEntryCollection::new();

    for EntryWithContent {
        key,
        content,
        is_executable,
        is_content,
    } in entries
    {
        let kind = match (is_content, is_executable) {
            (false, _) => RegisterEntryKind::Register,
            (true, false) => RegisterEntryKind::Content,
            (true, true) => RegisterEntryKind::Executable,
        };
        data.insert(key, EntryData { content, kind });
    }

    Ok(Some(Register { id, data }))
}

impl<S: AsRef<str>, D: AsRef<EntryData> + Write> super::write::Write
    for RegisterEntryCollection<S, D>
{
    async fn write(&self, exec: impl super::Executor<'_>) -> Result<Oid> {
        // we already inserted the empty collection on init
        if self.is_empty() {
            return Ok(Register::EMPTY_ID);
        }

        let (id, _) = Hash::hash(self)?;

        let entries: Vec<_> = self
            .iter()
            .map(|(key, entry)| Entry {
                key: key.map(|s| s.as_ref().to_string()),
                content: entry.as_ref().content,
                is_executable: entry.as_ref().kind == RegisterEntryKind::Executable,
            })
            .collect();

        sqlx::query("CALL braid.create_register($1, $2)")
            .bind(id)
            .bind(&entries)
            .execute(exec)
            .await?;

        Ok(id)
    }
}

#[derive(sqlx::FromRow)]
struct EntryWithContent {
    key: RegisterEntryKey<String>,
    content: Oid,
    is_executable: bool,
    is_content: bool,
}

#[derive(sqlx::Encode, sqlx::Decode, sqlx::FromRow)]
struct Entry {
    key: RegisterEntryKey<String>,
    content: Oid,
    is_executable: bool,
}

impl Type<Postgres> for Entry {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("braid.register_entry_record")
    }
}

impl PgHasArrayType for Entry {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("braid.register_entry_records")
    }
}
