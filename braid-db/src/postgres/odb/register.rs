use braid_hash::Oid;
use sqlx::{
    postgres::{PgHasArrayType, PgTypeInfo},
    Postgres, Type,
};

use crate::{
    bytes::Hash, postgres::Executor, register::{EntryData, Register, RegisterData, RegisterKind, SaveRegister, SaveRegisterData}, Key, Result
};

impl<S: Ord + AsRef<str>> super::write::Write for RegisterData<S> {
    async fn write(&self, exec: impl Executor<'_>) -> crate::Result<Oid> {
        write(self, exec).await
    }
}

impl<S: Ord + AsRef<str>> super::write::Write for SaveRegisterData<S> {
    async fn write(&self, exec: impl Executor<'_>) -> crate::Result<Oid> {
        write(self, exec).await
    }
}

async fn write<S: AsRef<str>, R: EntryData<S> + Hash>(
    data: &R,
    exec: impl Executor<'_>,
) -> Result<Oid> {
    let (oid, _) = Hash::hash(&data)?;

    let mut entries = Vec::with_capacity(data.len());

    for (key, content) in data.iter() {
        entries.push(Entry {
            key: Varchar(key.as_ref().to_string()),
            content: *content,
        });
    }

    let call = match R::REGISTER_KIND {
        RegisterKind::Register => "CALL braid.create_register($1, $2);",
        RegisterKind::SaveRegister => "CALL braid.create_save_register($1, $2);",
    };

    sqlx::query(call)
        .bind(oid)
        .bind(&entries)
        .execute(exec)
        .await?;

    Ok(oid)
}

pub(crate) async fn get_register(id: Oid, exec: impl Executor<'_>) -> Result<Option<Register>> {
    get(id, exec).await.map(|o| o.map(|data| Register { id, data }))
}

pub(crate) async fn get_save_register(id: Oid, exec: impl Executor<'_>) -> Result<Option<SaveRegister>> {
    get(id, exec).await.map(|o| o.map(|data| SaveRegister { id, data }))
}

async fn get<R: EntryData<String>>(id: Oid, exec: impl Executor<'_>) -> Result<Option<R>> {
    if id == R::EMPTY_ID {
        return Ok(Some(R::new()));
    }

    let select = match R::REGISTER_KIND {
        RegisterKind::Register => "SELECT * FROM braid.get_register($1);",
        RegisterKind::SaveRegister => "SELECT * FROM braid.get_save_register($1);",
    };

    let entries = sqlx::query_as(select)
        .bind(id.as_bytes())
        .fetch_all(exec)
        .await?;

    if entries.is_empty() {
        return Ok(None);
    }

    let mut data = R::new();

    for Entry {
        key,
        content,
    } in entries
    {
        let key = Key::try_from(key.0)?;
        data.insert(key, content);
    }

    Ok(Some(data))
}

#[derive(sqlx::Encode, sqlx::Decode, sqlx::FromRow)]
struct Entry {
    key: Varchar,
    content: Oid,
}

impl Type<Postgres> for Entry {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("braid.entry_record")
    }
}

impl PgHasArrayType for Entry {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::with_name("braid.entry_records")
    }
}

#[derive(sqlx::Encode, sqlx::Decode)]
struct Varchar(String);

impl Type<Postgres> for Varchar {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("varchar")
    }
}
