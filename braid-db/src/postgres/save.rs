use braid_hash::Oid;

use crate::{
    bytes::Hash,
    save::{Save, SaveData, SaveParent, SaveParentKind},
    Result,
};

use super::OidData;

pub(super) async fn get(oid: Oid, exec: impl super::Executor<'_>) -> Result<Option<Save<String>>> {
    let query = sqlx::query_as(
        "
        SELECT s.id, s.author, s.date, s.kind, s.content, s.parent, sp.is_commit
        FROM braid.save as s
        INNER JOIN braid.save_parent as sp
        ON s.parent = sp.id
        WHERE s.id = $1
        ",
    )
    .bind(oid);

    let save = query.fetch_optional(exec).await?;
    Ok(save)
}

impl<S: AsRef<str>> super::write_to_tran::Write for SaveData<S> {
    async fn write(&self, tran: &mut super::Transaction<'_>) -> Result<Oid> {
        let (id, _) = Hash::hash(self)?;

        sqlx::query("INSERT INTO braid.save_parent (id, is_commit) VALUES ($1, false)")
            .bind(id)
            .execute(&mut **tran)
            .await?;

        let query = sqlx::query(
            "
            INSERT INTO braid.save (id, author, date, kind, content, parent)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(*id.as_bytes())
        .bind(self.author.as_ref())
        .bind(self.date)
        .bind(self.kind)
        .bind(self.content.as_bytes())
        .bind(self.parent.id.as_bytes());

        query.execute(&mut **tran).await?;

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
        let is_commit = row.try_get("is_commit")?;
        let parent_kind = if is_commit {
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
