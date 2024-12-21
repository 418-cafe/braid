use sqlx::{
    postgres::{PgArguments, PgHasArrayType, PgTypeInfo},
    Postgres,
};

use crate::{
    models::{BranchExists, User},
    Ancestry, Branch, Commit, CommitImpl, Oid, Save, SaveData,
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
        for statement in crate::sql::INIT.split(';') {
            sqlx::query(statement)
                .execute(&mut *self.tx)
                .await?;
        }

        Ok(())
    }
}

impl DatabaseTransaction<'_> {
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

        sqlx::query_as(SELECT)
            .fetch_optional(&mut *self.tx)
            .await
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

pub(crate) enum SaveError {
    Sql(sqlx::Error),
    MismatchedParent,
}

impl From<sqlx::Error> for SaveError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sql(value)
    }
}

impl Persist for Save<&str> {
    type Output = ();
    type Error = SaveError;

    async fn execute(
        &self,
        tx: &mut Transaction<'_>,
    ) -> std::result::Result<Self::Output, Self::Error> {
        let Self {
            id,
            data:
                SaveData {
                    parent,
                    branch,
                    key,
                    when,
                    content,
                    is_current: _,
                },
        } = self;

        let _affected: u64 = sqlx::query(r#"INSERT INTO "save" (id, parent, branch, "key", "when", "content") VALUES ($1, $2, $3, $4, $5, $6)"#)
            .bind_many((id, parent, branch, key, when, content))
            .execute(&mut **tx)
            .await
            .map(|result| result.rows_affected())?;

        debug_assert_eq!(_affected, 1);

        use sqlx::Row;

        let mut affected = sqlx::query(
            r#"
            UPDATE "save"
            SET is_current = CASE id WHEN $1 THEN true ELSE false END
            WHERE id = $1 OR is_current = true
            RETURNING id
        "#,
        )
        .bind(id)
        .fetch_all(&mut **tx)
        .await?
        .into_iter()
        .map(|r| r.get("id"))
        .filter(|affected| id != affected);

        let next: Option<Oid> = affected.next();

        debug_assert!(affected.next().is_none());

        if &next == parent {
            Ok(())
        } else {
            Err(SaveError::MismatchedParent)
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
    };
}

impl_bind_many!((T1, T2));
impl_bind_many!((T1, T2, T3));
impl_bind_many!((T1, T2, T3, T4));
impl_bind_many!((T1, T2, T3, T4, T5));
impl_bind_many!((T1, T2, T3, T4, T5, T6));
