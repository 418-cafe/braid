use sqlx::{
    postgres::{PgArguments, PgHasArrayType, PgTypeInfo},
    Postgres,
};

use crate::{
    data::{BranchExists, User}, models::{
        Branch, Commit, CommitData, CommitImpl, CommitImplData, Save, SaveData,
    }, Oid
};

pub type Transaction<'a> = sqlx::Transaction<'a, Postgres>;
type Query<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;
type QueryExists<'q> = sqlx::query::QueryScalar<'q, Postgres, bool, PgArguments>;

pub(super) struct Database<'a, 't> {
    tx: &'a mut Transaction<'t>,
}

impl<'a, 't> Database<'a, 't> {
    pub fn open(tx: &'a mut Transaction<'t>) -> Self {
        Self { tx }
    }

    pub(crate) async fn init(&mut self) -> Result<(), sqlx::Error> {
        for statement in crate::sql::INIT.split(';') {
            sqlx::query(statement).execute(&mut **self.tx).await?;
        }

        Ok(())
    }
}

impl Database<'_, '_> {
    pub(crate) async fn write_external_object(&mut self, id: Oid) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO external_object (id) VALUES ($1)")
            .bind(id.as_bytes())
            .execute(&mut **self.tx)
            .await
            .map(|_| ())
    }

    pub(crate) async fn latest_save(
        &mut self,
        key: &str,
        branch: &str,
    ) -> Result<Option<Oid>, sqlx::Error> {
        const SELECT: &str = "
            SELECT id
            FROM \"save\"
            WHERE \"key\" = $1 AND branch = $2 AND \"when\" = (
                SELECT MAX(\"when\")
                FROM \"save\"
                WHERE \"key\" = $1 AND branch = $2
            )
        ";

        #[derive(sqlx::FromRow)]
        struct Row {
            id: Oid,
        }

        sqlx::query_as(SELECT)
            .bind(key)
            .bind(branch)
            .fetch_optional(&mut **self.tx)
            .await
            .map(|r: Option<Row>| r.map(|r| r.id))
    }

    pub(crate) async fn get_root(&mut self) -> Result<Option<crate::data::Commit>, sqlx::Error> {
        const SELECT: &str = "
            SELECT
                c.id,
                c.subject,
                c.body,
                c.author,
                c.\"when\",
                ci.id AS impl_id,
                ci.committer,
                ci.parent,
                ci.merge_parent,
                ci.\"when\" AS impl_when
            FROM \"commit\" c
            JOIN \"commit_impl\" ci ON c.id = ci.id
            WHERE ci.parent IS NULL
        ";

        let commit: Option<crate::data::Commit> = sqlx::query_as(SELECT)
            .fetch_optional(&mut **self.tx)
            .await?;

        Ok(commit)
    }

    pub(crate) async fn exists<'a, E: Exists<'a>>(
        &mut self,
        data: &'a E,
    ) -> Result<bool, sqlx::Error> {
        let query = sqlx::query_scalar(E::EXISTS);
        data.bind(query).fetch_one(&mut **self.tx).await
    }

    pub(crate) async fn persist<'a, P: Persist<'a>>(
        &mut self,
        data: &'a P,
    ) -> Result<(), sqlx::Error> {
        let query = sqlx::query(P::INSERT);
        data.bind(query).execute(&mut **self.tx).await.map(|_| ())
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

pub(crate) trait Persist<'a> {
    const INSERT: &'static str;

    fn bind(&'a self, query: Query<'a>) -> Query<'a>;
}

impl<'a> Persist<'a> for Save<&'a str> {
    const INSERT: &'static str = "INSERT INTO \"save\" (id, parent, branch, \"key\", \"when\", content) VALUES ($1, $2, $3, $4, $5, $6)";

    fn bind(&'a self, query: Query<'a>) -> Query<'a> {
        let Self {
            id,
            data:
                SaveData {
                    parent,
                    branch,
                    key,
                    when,
                    content,
                },
        } = self;

        query
            .bind(id)
            .bind(parent)
            .bind(branch)
            .bind(key)
            .bind(when)
            .bind(content)
    }
}

impl<'a> Persist<'a> for Commit<&'a str> {
    const INSERT: &'static str =
        "INSERT INTO \"commit\" (id, subject, body, author, \"when\") VALUES ($1, $2, $3, $4, $5)";

    fn bind(&'a self, query: Query<'a>) -> Query<'a> {
        let Self {
            id,
            data:
                CommitData {
                    subject,
                    body,
                    author,
                    when,
                    implementation,
                },
        } = self;

        query
            .bind(id)
            .bind(subject)
            .bind(body)
            .bind(author)
            .bind(when)
            .bind(implementation)
    }
}

impl<'a> Persist<'a> for CommitImpl<&'a str> {
    const INSERT: &'static str = "INSERT INTO \"commit_impl\" (id, commit, parent, merge_parent, committer, \"when\") VALUES ($1, $2, $3, $4, $5, $6)";

    fn bind(&'a self, query: Query<'a>) -> Query<'a> {
        let Self {
            id,
            data:
                CommitImplData {
                    commit,
                    ancestry,
                    committer,
                    when,
                },
        } = self;

        let (parent, merge_parent) = ancestry.as_options();

        query
            .bind(id)
            .bind(commit)
            .bind(parent)
            .bind(merge_parent)
            .bind(committer)
            .bind(when)
    }
}

impl<'a> Persist<'a> for User<'a> {
    const INSERT: &'static str = "INSERT INTO \"user\" (id) VALUES ($1)";

    fn bind(&'a self, query: Query<'a>) -> Query<'a> {
        let Self(id) = self;
        query.bind(id)
    }
}

impl<'a> Persist<'a> for Branch<&'a str> {
    const INSERT: &'static str =
        "INSERT INTO \"branch\" (name, tip, is_default) VALUES ($1, $2, $3)";

    fn bind(&'a self, query: Query<'a>) -> Query<'a> {
        let Self {
            name,
            tip,
            is_default,
        } = self;
        query.bind(name).bind(tip).bind(is_default)
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
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        self.as_bytes().encode(buf)
    }
}

impl sqlx::Decode<'_, Postgres> for Oid {
    fn decode(
        value: <Postgres as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
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
