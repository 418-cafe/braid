use std::vec;

use sqlx::{postgres::PgRow, PgExecutor, Row};

use crate::{Key, Oid, Save, SaveData, SaveLineageCriteria};

use super::{types::BindMany, Result};

pub(crate) async fn persist(ex: impl PgExecutor<'_>, id: Oid, data: SaveData) -> Result {
    let SaveData {
        parent,
        when,
        content,
    } = data;

    sqlx::query(r#"INSERT INTO "save"(id, parent, "when", "content") VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING"#)
        .bind_many((id, parent, when, content))
        .execute(ex)
        .await?;

    Ok(())
}

pub(crate) async fn lineage<'a, I>(
    ex: impl PgExecutor<'_>,
    criteria: SaveLineageCriteria<'a, I>,
) -> Result<LineageIter>
where
    I: IntoIterator<Item = Key<&'a str>>,
{
    let SaveLineageCriteria { branch, keys } = criteria;

    let rows = sqlx::query(
        r#"
        SELECT id, "key", parent, "when", "content"
        FROM save_lineage
        WHERE branch = $1
        AND key = ANY($2::text[])
        ORDER BY "key", depth
        "#,
    )
    .bind_many((branch, keys.into_iter().collect::<Vec<_>>()))
    .fetch_all(ex)
    .await?
    .into_iter();

    Ok(LineageIter { rows })
}

pub struct LineageIter {
    rows: vec::IntoIter<PgRow>,
}

impl LineageIter {
    fn next(&mut self) -> Option<Save<String>> {
        let row = self.rows.next()?;

        let id = row.get_unchecked(0);
        let key = Key::new_unchecked(row.get_unchecked(1));
        let parent = row.get_unchecked(2);
        let when = row.get_unchecked(3);
        let content = row.get_unchecked(4);

        Some(Save {
            id,
            key,
            parent,
            when,
            content,
        })
    }

    pub fn grouped_by_key(self) -> GroupedLineageIter {
        let mut inner = self;
        let chamber = inner.next();
        GroupedLineageIter { inner, chamber }
    }
}

impl Iterator for LineageIter {
    type Item = Save<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

pub struct GroupedLineageIter {
    inner: LineageIter,
    chamber: Option<Save<String>>,
}

impl Iterator for GroupedLineageIter {
    // todo: optimize as this is guaranteed to have at least one element
    type Item = Vec<Save<String>>;

    fn next(&mut self) -> Option<Self::Item> {
        let save = std::mem::replace(&mut self.chamber, self.inner.next())?;
        let mut saves = vec![save];

        while matches!(&self.chamber, Some(save) if save.key == saves[0].key) {
            // SAFETY: we confirm in the matches! above that this option is Some(_)
            saves.push(unsafe {
                std::mem::replace(&mut self.chamber, self.inner.next()).unwrap_unchecked()
            });
        }

        Some(saves)
    }
}
