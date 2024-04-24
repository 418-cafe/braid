use braid_hash::Oid;
use sqlx::{postgres::{PgHasArrayType, PgTypeInfo}, Postgres, Type};

use crate::{
    bytes::Hash,
    postgres::Executor,
    register::{SaveEntryCollection, SaveRegister},
    Result, SaveEntryKey,
};

pub(super) async fn get(id: Oid, exec: impl Executor<'_>) -> Result<Option<SaveRegister<String>>> {
    let entries = sqlx::query_as("SELECT * FROM braid.get_save_register($1)")
        .bind(id.as_bytes())
        .fetch_all(exec)
        .await?;

    let mut data = SaveEntryCollection::new();

    for Entry { key, save } in entries {
        data.insert(key, save);
    }

    Ok(Some(SaveRegister { id, data }))
}

impl<S: Ord + AsRef<str>> super::write::Write for SaveEntryCollection<S> {
    async fn write(&self, exec: impl super::Executor<'_>) -> crate::Result<braid_hash::Oid> {
        // we already inserted the empty collection on init
        if self.is_empty() {
            return Ok(SaveRegister::EMPTY_ID);
        }

        let (id, _) = Hash::hash(self)?;

        let entries = self
            .iter()
            .map(|(key, save)| Entry {
                key: key.map(|s| s.as_ref().to_string()),
                save: save.clone(),
            })
            .collect::<Vec<_>>();

        sqlx::query("CALL braid.create_save_register($1, $2)")
            .bind(id)
            .bind(&entries)
            .execute(exec)
            .await?;

        Ok(id)
    }
}

#[derive(sqlx::Encode, sqlx::FromRow)]
struct Entry {
    key: SaveEntryKey<String>,
    save: Oid,
}

impl Type<Postgres> for Entry {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("braid.save_register_entry_record")
    }
}

impl PgHasArrayType for Entry {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("braid.save_register_entry_records")
    }
}
