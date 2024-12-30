use sqlx::{FromRow, PgConnection, Row};

use crate::{Key, Oid, Save, SaveData, SaveLineageCriteria};

use super::types::BindMany;

type Result<T> = std::result::Result<T, sqlx::Error>;

pub(crate) trait GetMany<C>: Sized {
    async fn get_many(conn: &mut PgConnection, criteria: C) -> Result<Self>;
}

// todo: grouping
impl<'a, I> GetMany<SaveLineageCriteria<'a, I>> for Vec<Save<String, Option<Oid>>>
where
    I: IntoIterator<Item = Key<'a>>,
{
    async fn get_many(
        conn: &mut PgConnection,
        criteria: SaveLineageCriteria<'a, I>,
    ) -> Result<Self> {

        let SaveLineageCriteria { branch, keys } = criteria;

        sqlx::query(
            r#"
            SELECT id, branch, "key", parent, "when", "content"
            FROM save_lineage
            WHERE branch = $1
            AND key = ANY($2::text[])
            ORDER BY depth
            "#,
        )
        .bind_many((branch, keys.into_iter().collect::<Vec<_>>()))
        .fetch_all(conn)
        .await
        .map(|r| {
            r.into_iter()
                .map(|r| Save {
                    id: r.get("id"),
                    parent: r.get("parent"),
                    data: SaveData::from_row(&r).expect("unable to decode save data"),
                })
                .collect()
        })
    }
}
