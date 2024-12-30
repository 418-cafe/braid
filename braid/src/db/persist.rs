use sqlx::{PgConnection, Row};

use crate::{db::types::BindMany, Ancestry, Branch, Commit, CommitImpl, Oid, Save, SaveData, SaveParentContent};

use super::User;

type Result<T = ()> = std::result::Result<T, sqlx::Error>;

pub(crate) trait Persist {
    type Output;
    type Error;

    async fn execute(
        &self,
        conn: &mut PgConnection,
    ) -> std::result::Result<Self::Output, Self::Error>;
}

impl<'a> Persist for (SaveParentContent, Save<&'a str>) {
    type Output = Option<Save<&'a str, Option<Oid>>>;
    type Error = crate::Error;

    async fn execute(
        &self,
        conn: &mut PgConnection,
    ) -> std::result::Result<Self::Output, Self::Error> {
        use crate::Error;

        let (
            expected_parent_content,
            Save {
                id,
                parent: _,
                data:
                    SaveData {
                        branch,
                        key,
                        when,
                        content,
                    },
            },
        ) = *self;

        let insert = |conn, id, parent, when, content| {
            sqlx::query(
                r#"
                    INSERT INTO "save" (id, parent, "when", "content")
                    VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind_many((id, parent, when, content))
            .execute(conn)
        };

        let Some((committed, parent, current_content)) = sqlx::query(
            r#"
            SELECT st."committed", st."save", s."content"
            FROM "state" AS st
            LEFT JOIN "save" AS s ON st."save" = s.id
            WHERE branch = $1 and "key" = $2
            "#,
        )
        .bind_many((branch, key))
        .fetch_optional(&mut *conn)
        .await?
        .map::<(Option<Oid>, _, _), _>(|r| (r.get("committed"), r.get("save"), r.get("content"))) else {
            if content.is_none() {
                return Ok(None);
            }

            if expected_parent_content.is_some() {
                return Err(Error::ExpectedParentContentDoesNotMatch);
            }

            insert(&mut *conn, id, None, when, content).await?;

            return match sqlx::query(
                r#"
                INSERT INTO "state" (branch, "key", "save")
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING;
                "#,
            )
            .bind_many((branch, key, id))
            .execute(&mut *conn)
            .await?
            .rows_affected()
            {
                0 => Err(Error::ExpectedParentContentDoesNotMatch),
                _ => Ok(Some(Save {
                    id,
                    parent: None,
                    data: self.1.data,
                })),
            };
        };

        if current_content == content {
            return Ok(None);
        }

        insert(conn, id, parent, when, content).await?;

        if expected_parent_content != current_content {
            return Err(Error::ExpectedParentContentDoesNotMatch);
        }

        match sqlx::query(
            r#"
            UPDATE "state" AS s
            SET "save" = $1
            WHERE s.branch = $2
            AND s."key" = $3
            AND (
                ($4 IS NULL AND s."save" IS NULL)
                OR
                ($4= s."save")
            )
            AND (
                ($5 IS NULL AND "committed" IS NULL)
                OR
                ($5 = "committed")
            );
            "#,
        )
        .bind_many((id, branch, key, parent, committed))
        .execute(conn)
        .await?
        .rows_affected()
        {
            0 => Err(Error::ExpectedParentContentDoesNotMatch),
            _ => Ok(Some(Save {
                id,
                parent,
                data: self.1.data,
            })),
        }
    }
}

impl Persist for Commit<&str> {
    type Output = ();
    type Error = sqlx::Error;

    async fn execute(
        &self,
        conn: &mut PgConnection,
    ) -> std::result::Result<Self::Output, Self::Error> {
        let Self {
            id,
            subject,
            body,
            author,
            authored,
        } = self;

        sqlx::query(r#"INSERT INTO "commit" (id, subject, body, author, authored) VALUES ($1, $2, $3, $4, $5)"#)
            .bind_many((id, subject, body, author, authored))
            .execute(conn)
            .await?;

        Ok(())
    }
}

impl Persist for CommitImpl<&str> {
    type Output = ();
    type Error = sqlx::Error;

    async fn execute(
        &self,
        conn: &mut PgConnection,
    ) -> std::result::Result<Self::Output, Self::Error> {
        let Self {
            id,
            commit,
            ancestry,
            committer,
            committed,
        } = self;

        let (parent, merge_parent) = match ancestry.as_ref() {
            Ancestry::Root => (None, None),
            Ancestry::Parent {
                parent,
                merge_parent,
            } => (Some(parent), merge_parent),
        };

        sqlx::query(r#"INSERT INTO "commit_impl" (id, commit, parent, merge_parent, committer, "committed") VALUES ($1, $2, $3, $4, $5, $6)"#)
            .bind_many((id, commit, parent, merge_parent, committer, committed))
            .execute(conn)
            .await?;

        Ok(())
    }
}

impl Persist for User<'_> {
    type Output = ();
    type Error = sqlx::Error;

    async fn execute(&self, conn: &mut PgConnection) -> Result {
        let Self(id) = self;

        sqlx::query(r#"INSERT INTO "user"(id) VALUES ($1)"#)
            .bind(id)
            .execute(conn)
            .await?;

        Ok(())
    }
}

impl Persist for Branch<&str> {
    type Output = ();
    type Error = sqlx::Error;

    async fn execute(&self, conn: &mut PgConnection) -> Result {
        let Self {
            name,
            tip,
            is_default,
        } = self;

        sqlx::query("INSERT INTO \"branch\" (name, tip, is_default) VALUES ($1, $2, $3)")
            .bind_many((name, tip, is_default))
            .execute(conn)
            .await?;

        Ok(())
    }
}
