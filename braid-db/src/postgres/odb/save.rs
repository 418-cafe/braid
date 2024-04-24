use braid_hash::Oid;

use crate::{
    bytes::Hash,
    postgres::Executor,
    save::{Save, SaveData, SaveEntryKind, SaveParent, SaveParentKind},
    Result,
};

use super::OidData;

pub(super) async fn get(oid: Oid, exec: impl Executor<'_>) -> Result<Option<Save<String>>> {
    let save = sqlx::query_as("SELECT * FROM braid.get_save($1)")
        .bind(oid)
        .fetch_optional(exec)
        .await?;
    Ok(save)
}

impl<S: AsRef<str>> super::write::Write for SaveData<S> {
    async fn write(&self, exec: impl super::Executor<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        sqlx::query("CALL braid.create_save($1, $2::varchar, $3, $4, $5, $6)")
            .bind(id)
            .bind(self.author.as_ref())
            .bind(self.date)
            .bind(self.kind)
            .bind(self.content)
            .bind(self.parent.id)
            .execute(exec)
            .await?;

        Ok(id)
    }
}

impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for SaveData<String> {
    fn from_row(row: &'_ sqlx::postgres::PgRow) -> std::prelude::v1::Result<Self, sqlx::Error> {
        use sqlx::Row;
        let author = row.try_get("author")?;
        let date = row.try_get("date")?;
        let kind = row.try_get("kind")?;
        let content = row.try_get("content")?;
        let parent = row.try_get("parent")?;
        let parent_kind = if row.try_get("is_commit")? {
            SaveParentKind::Commit
        } else {
            SaveParentKind::Save
        };
        Ok(Self {
            author,
            date,
            kind,
            content,
            parent: SaveParent {
                id: parent,
                kind: parent_kind,
            },
        })
    }
}

impl OidData<'_> for Save<String> {
    type Data = SaveData<String>;

    fn create(id: Oid, data: Self::Data) -> Self {
        Self { id, data }
    }
}

// https://github.com/launchbadge/sqlx/issues/2831
impl sqlx::Type<sqlx::Postgres> for SaveEntryKind {
    fn type_info() -> <sqlx::Postgres as sqlx::Database>::TypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("braid.save_entry_kind")
    }

    fn compatible(ty: &<sqlx::Postgres as sqlx::Database>::TypeInfo) -> bool {
        *ty == Self::type_info() || *ty == sqlx::postgres::PgTypeInfo::with_name("save_entry_kind")
    }
}