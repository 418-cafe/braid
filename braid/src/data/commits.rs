use crate::{models::DateTime, Ancestry, Oid};

impl<'r, R> sqlx::FromRow<'r, R> for Ancestry<Oid>
where
    R: sqlx::Row,
    &'r str: sqlx::ColumnIndex<R>,
    Oid: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
        Ok(match row.try_get("parent")? {
            Some(parent) => Ancestry::Parent {
                parent,
                merge_parent: row.try_get("merge_parent")?,
            },
            None => Ancestry::Root,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CommitImpl {
    id: Oid,
    committer: String,
    ancestry: Ancestry<Oid>,
    when: DateTime,
}

impl CommitImpl {
    pub fn id(&self) -> &Oid {
        &self.id
    }

    pub fn committer(&self) -> &String {
        &self.committer
    }

    pub fn ancestry(&self) -> &Ancestry<Oid> {
        &self.ancestry
    }

    pub fn when(&self) -> &DateTime {
        &self.when
    }
}

#[derive(Debug, Clone)]
pub struct Commit {
    id: Oid,
    subject: Option<String>,
    body: Option<String>,
    author: String,
    when: DateTime,
    implementation: CommitImpl,
}

impl Commit {
    pub fn id(&self) -> &Oid {
        &self.id
    }

    pub fn subject(&self) -> Option<&String> {
        self.subject.as_ref()
    }

    pub fn body(&self) -> Option<&String> {
        self.body.as_ref()
    }

    pub fn author(&self) -> &String {
        &self.author
    }

    pub fn when(&self) -> &DateTime {
        &self.when
    }

    pub fn implementation(&self) -> &CommitImpl {
        &self.implementation
    }
}

impl<'r, R> sqlx::FromRow<'r, R> for Commit
where
    R: sqlx::Row,
    &'r str: sqlx::ColumnIndex<R>,
    Oid: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    DateTime: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
        let id = row.try_get("impl_id")?;
        let committer = row.try_get("committer")?;
        let ancestry = sqlx::FromRow::from_row(row)?;
        let when = row.try_get("impl_when")?;

        let implementation = CommitImpl {
            id,
            committer,
            ancestry,
            when,
        };

        let id = row.try_get("id")?;
        let subject = row.try_get("subject")?;
        let body = row.try_get("body")?;
        let author = row.try_get("author")?;
        let when = row.try_get("when")?;

        Ok(Self {
            id,
            subject,
            body,
            author,
            when,
            implementation,
        })
    }
}
