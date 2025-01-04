use sqlx::{PgExecutor, Row};

use crate::{FullKey, Oid};

use super::{types::BindMany, Result};

pub(crate) enum State {
    None,
    Saved {
        id: Oid,
        content: Option<Oid>,
    },
    Committed {
        content: Oid,
    },
    SavedCommitted {
        id: Oid,
        content: Option<Oid>,
        committed: Oid,
    },
}

pub(crate) async fn current(
    ex: impl PgExecutor<'_>,
    branch: FullKey<&str>,
    key: FullKey<&str>,
) -> Result<State> {
    let Some(row) = sqlx::query(
        r#"
        SELECT st."save", st."committed", sv."content"
        FROM "state" AS st
        LEFT JOIN "save" AS sv ON st."save" = sv.id
        WHERE "branch" = $1 AND "key" = $2
        "#,
    )
    .bind_many((branch, key))
    .fetch_optional(ex)
    .await?
    else {
        return Ok(State::None);
    };

    const SAVE: usize = 0;
    const COMMITTED: usize = 1;
    const CONTENT: usize = 2;

    Ok(match row.get_unchecked(SAVE) {
        Some(id) => match row.get_unchecked(COMMITTED) {
            Some(committed) => State::SavedCommitted {
                id,
                content: row.get_unchecked(CONTENT),
                committed,
            },
            None => State::Saved {
                id,
                content: row.get_unchecked(CONTENT),
            },
        },
        None => State::Committed {
            content: row.get_unchecked(COMMITTED),
        },
    })
}

pub(crate) async fn save_or_update(
    ex: impl PgExecutor<'_>,
    branch: FullKey<&str>,
    key: FullKey<&str>,
    save: Oid,
    current: State,
) -> Result<bool> {
    let (saved, committed) = match current {
        State::Saved { id, .. } => (Some(id), None),
        State::Committed { content } => (None, Some(content)),
        State::SavedCommitted { id, committed, .. } => (Some(id), Some(committed)),
        State::None => {
            return Ok(sqlx::query(
                r#"
                INSERT INTO "state"(branch, "key", "committed", "save")
                VALUES ($1, $2, $3, $4)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind_many((branch, key, Option::<Oid>::None, save))
            .execute(ex)
            .await?
            .rows_affected()
            .gt(&0))
        }
    };

    Ok(sqlx::query(
        r#"
        UPDATE "state"
        SET "save" = $1
        WHERE branch = $2 AND "key" = $3
        AND (($4 IS NULL AND "committed" IS NULL) OR "committed" = $4)
        AND (($5 IS NULL AND "save" IS NULL) OR "save" = $5)
        "#,
    )
    .bind_many((save, branch, key, committed, saved))
    .execute(ex)
    .await?
    .rows_affected()
    .gt(&0))
}
