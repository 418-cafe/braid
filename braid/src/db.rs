use sqlx::{
    postgres::{PgArguments, PgHasArrayType, PgTypeInfo},
    FromRow, Postgres,
};

use crate::{
    models::{BranchExists, User},
    Ancestry, Branch, Commit, CommitImpl, Key, Oid, Save, SaveData, SaveLineageCriteria,
    SaveParentContent,
};

pub type Transaction<'a> = sqlx::Transaction<'a, Postgres>;
type Query<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;
type QueryExists<'q> = sqlx::query::QueryScalar<'q, Postgres, bool, PgArguments>;

type Result<T = ()> = std::result::Result<T, sqlx::Error>;

pub(crate) struct DatabaseTransaction<'t> {
    tx: Transaction<'t>,
}

impl<'t> DatabaseTransaction<'t> {
    pub(crate) fn open(tx: Transaction<'t>) -> Self {
        Self { tx }
    }

    pub(crate) fn into_inner(self) -> Transaction<'t> {
        self.tx
    }
}

impl DatabaseTransaction<'_> {
    pub(crate) async fn init(&mut self) -> Result {
        for statement in crate::sql::init_statements() {
            sqlx::query(statement).execute(&mut *self.tx).await?;
        }

        Ok(())
    }

    pub(crate) async fn write_external_object(&mut self, id: Oid) -> Result<bool> {
        sqlx::query("INSERT INTO external_object (id) VALUES ($1) ON CONFLICT DO NOTHING")
            .bind(id.as_bytes())
            .execute(&mut *self.tx)
            .await
            .map(|r| r.rows_affected() != 0)
    }

    pub(crate) async fn get_root(&mut self) -> Result<Option<crate::models::CommitWithImpl>> {
        const SELECT: &str = "
            SELECT
                c.id,
                c.subject,
                c.body,
                c.author,
                c.authored,
                ci.id AS impl_id,
                ci.parent,
                ci.merge_parent,
                ci.committer,
                ci.committed
            FROM \"commit\" c
            JOIN \"commit_impl\" ci ON c.id = ci.id
            WHERE ci.parent IS NULL
        ";

        sqlx::query_as(SELECT).fetch_optional(&mut *self.tx).await
    }

    pub(crate) async fn exists<'a, E: Exists<'a>>(&mut self, data: &'a E) -> Result<bool> {
        let query = sqlx::query_scalar(E::EXISTS);
        data.bind(query).fetch_one(&mut *self.tx).await
    }

    pub(crate) async fn persist<P: Persist>(
        &mut self,
        data: &P,
    ) -> std::result::Result<<P as Persist>::Output, <P as Persist>::Error> {
        data.execute(&mut self.tx).await
    }

    pub(crate) async fn get_many<T, C>(&mut self, criteria: C) -> Result<T>
    where
        T: GetMany<C>,
    {
        T::get_many(&mut self.tx, criteria).await
    }
}

pub(crate) trait GetMany<C>: Sized {
    async fn get_many(tx: &mut Transaction<'_>, criteria: C) -> Result<Self>;
}

// todo: grouping
impl<'a, I> GetMany<SaveLineageCriteria<'a, I>> for Vec<Save<String, Option<Oid>>>
where
    I: IntoIterator<Item = Key<'a>>,
{
    async fn get_many(
        tx: &mut Transaction<'_>,
        criteria: SaveLineageCriteria<'a, I>,
    ) -> Result<Self> {
        use sqlx::Row;

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
        .fetch_all(&mut **tx)
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

pub(crate) trait Exists<'a> {
    const EXISTS: &'static str;

    fn bind(&'a self, query: QueryExists<'a>) -> QueryExists<'a>;
}

impl<'a> Exists<'a> for BranchExists<'a> {
    const EXISTS: &'static str = "SELECT EXISTS(SELECT 1 FROM \"branch\" WHERE \"name\" = $1)";

    fn bind(&'a self, query: QueryExists<'a>) -> QueryExists<'a> {
        query.bind(self.0)
    }
}

pub(crate) trait Persist {
    type Output;
    type Error;

    async fn execute(
        &self,
        tx: &mut Transaction<'_>,
    ) -> std::result::Result<Self::Output, Self::Error>;
}

impl<'a> Persist for (SaveParentContent, Save<&'a str>) {
    type Output = Option<Save<&'a str, Option<Oid>>>;
    type Error = crate::Error;

    async fn execute(
        &self,
        tx: &mut Transaction<'_>,
    ) -> std::result::Result<Self::Output, Self::Error> {
        use crate::Error;
        use sqlx::Row;

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

        let insert = |tx, id, parent, when, content| {
            sqlx::query(
                r#"
                    INSERT INTO "save" (id, parent, "when", "content")
                    VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind_many((id, parent, when, content))
            .execute(tx)
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
        .fetch_optional(&mut **tx)
        .await?
        .map::<(Option<Oid>, _, _), _>(|r| (r.get("committed"), r.get("save"), r.get("content"))) else {
            if content.is_none() {
                return Ok(None);
            }

            if expected_parent_content.is_some() {
                return Err(Error::ExpectedParentContentDoesNotMatch);
            }

            insert(&mut **tx, id, None, when, content).await?;

            return match sqlx::query(
                r#"
                INSERT INTO "state" (branch, "key", "save")
                VALUES ($1, $2, $3)
                ON CONFLICT DO NOTHING;
                "#,
            )
            .bind_many((branch, key, id))
            .execute(&mut **tx)
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

        insert(&mut **tx, id, parent, when, content).await?;

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
        .execute(&mut **tx)
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
        tx: &mut Transaction<'_>,
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
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

impl Persist for CommitImpl<&str> {
    type Output = ();
    type Error = sqlx::Error;

    async fn execute(
        &self,
        tx: &mut Transaction<'_>,
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
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

impl Persist for User<'_> {
    type Output = ();
    type Error = sqlx::Error;

    async fn execute(&self, tx: &mut Transaction<'_>) -> Result {
        let Self(id) = self;

        sqlx::query(r#"INSERT INTO "user"(id) VALUES ($1)"#)
            .bind(id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

impl Persist for Branch<&str> {
    type Output = ();
    type Error = sqlx::Error;

    async fn execute(&self, tx: &mut Transaction<'_>) -> Result {
        let Self {
            name,
            tip,
            is_default,
        } = self;

        sqlx::query("INSERT INTO \"branch\" (name, tip, is_default) VALUES ($1, $2, $3)")
            .bind_many((name, tip, is_default))
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

impl sqlx::Type<Postgres> for Oid {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        PgTypeInfo::with_name("bytea")
    }
}

impl sqlx::Encode<'_, Postgres> for Oid {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.as_bytes().encode(buf)
    }
}

impl sqlx::Decode<'_, Postgres> for Oid {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        let bytes = sqlx::decode::Decode::decode(value)?;
        Ok(Self::new(bytes))
    }
}

impl PgHasArrayType for Oid {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        PgTypeInfo::array_of("bytea")
    }
}

impl<D: sqlx::Database> sqlx::Type<D> for crate::Key<'_>
where
    str: sqlx::Type<D>,
{
    fn type_info() -> <D as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<D>>::type_info()
    }
}

impl sqlx::Encode<'_, Postgres> for crate::Key<'_> {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> std::result::Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

impl<'d> sqlx::Decode<'d, Postgres> for crate::Key<'d> {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'d>,
    ) -> std::result::Result<Self, sqlx::error::BoxDynError> {
        Ok(Self::new_unchecked(
            <&str as sqlx::Decode<Postgres>>::decode(value)?,
        ))
    }
}

impl PgHasArrayType for crate::Key<'_> {
    fn array_type_info() -> PgTypeInfo {
        <&str as PgHasArrayType>::array_type_info()
    }
}

trait BindMany<T> {
    fn bind_many(self, value: T) -> Self;
}

macro_rules! impl_bind_many {
    (($($ident:ident),+)) => {
        impl<'a, $($ident),+> BindMany<($($ident),+)> for Query<'a>
        where
            $(
                $ident: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>
            ),+
        {
            fn bind_many(self, value: ($($ident),+)) -> Self {
                #[allow(non_snake_case)]
                let ($($ident),+) = value;
                self
                $(
                    .bind($ident)
                )+
            }
        }

        impl<'a, O, $($ident),+> BindMany<($($ident),+)> for sqlx::query::QueryScalar<'a, Postgres, O, PgArguments>
        where
            $(
                $ident: 'a + sqlx::Encode<'a, Postgres> + sqlx::Type<Postgres>
            ),+
        {
            fn bind_many(self, value: ($($ident),+)) -> Self {
                #[allow(non_snake_case)]
                let ($($ident),+) = value;
                self
                $(
                    .bind($ident)
                )+
            }
        }
    };
}

impl_bind_many!((T1, T2));
impl_bind_many!((T1, T2, T3));
impl_bind_many!((T1, T2, T3, T4));
impl_bind_many!((T1, T2, T3, T4, T5));
impl_bind_many!((T1, T2, T3, T4, T5, T6));
