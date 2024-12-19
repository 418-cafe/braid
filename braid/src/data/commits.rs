use crate::{models::DateTime, Ancestry, Braid, Hash, Oid};

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

pub(crate) struct NewCommit<S> {
    pub(crate) subject: Option<S>,
    pub(crate) body: Option<S>,
    pub(crate) author: S,
    pub(crate) authored: DateTime,
    pub(crate) ancestry: Ancestry<Oid>,
    pub(crate) committer: S,
    pub(crate) committed: DateTime,
}

impl<'a> NewCommit<&'a str> {
    pub(crate) fn hash_and_split(self) -> (Commit<&'a str>, CommitImpl<&'a str>) {
        let id = Braid::hash(&self);
        let Self { subject, body, author, authored, ancestry, committer, committed } = self;

        let commit = Commit {
            id,
            subject,
            body,
            author,
            authored,
        };

        let commit_impl = CommitImpl {
            id,
            commit: id,
            committer,
            ancestry,
            committed,
        };

        (commit, commit_impl)
    }
}

impl Hash for NewCommit<&str> {
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        let Self { subject, body, author, authored, ancestry, committer, committed } = self;

        ancestry.hash(hasher);
        authored.timestamp_millis().hash(hasher);
        committed.timestamp_millis().hash(hasher);

        subject.unwrap_or_default().hash(hasher);
        hasher.push_null();

        body.unwrap_or_default().hash(hasher);
        hasher.push_null();

        author.hash(hasher);
        hasher.push_null();

        committer.hash(hasher);
        hasher.push_null();
    }
}

#[derive(Debug, Clone)]
pub struct CommitImpl<S = String> {
    pub(crate) id: Oid,
    pub(crate) commit: Oid,
    pub(crate) ancestry: Ancestry<Oid>,
    pub(crate) committer: S,
    pub(crate) committed: DateTime,
}

impl<S> CommitImpl<S> {
    pub fn id(&self) -> &Oid {
        &self.id
    }

    pub fn ancestry(&self) -> &Ancestry<Oid> {
        &self.ancestry
    }

    pub fn committer(&self) -> &S {
        &self.committer
    }

    pub fn committed(&self) -> &DateTime {
        &self.committed
    }
}

impl<'r, R> sqlx::FromRow<'r, R> for CommitImpl
where
    R: sqlx::Row,
    &'r str: sqlx::ColumnIndex<R>,
    Oid: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    DateTime: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let commit = row.try_get("commit")?;
        let ancestry = sqlx::FromRow::from_row(row)?;
        let committer = row.try_get("committer")?;
        let committed = row.try_get("committed")?;

        Ok(Self {
            id,
            commit,
            ancestry,
            committer,
            committed,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Commit<S = String> {
    pub(crate) id: Oid,
    pub(crate) subject: Option<S>,
    pub(crate) body: Option<S>,
    pub(crate) author: S,
    pub(crate) authored: DateTime,
}

impl<S: AsRef<str>> Commit<S> {
}

impl<S> Commit<S> {
    pub fn id(&self) -> &Oid {
        &self.id
    }

    pub fn subject(&self) -> Option<&S> {
        self.subject.as_ref()
    }

    pub fn body(&self) -> Option<&S> {
        self.body.as_ref()
    }

    pub fn author(&self) -> &S {
        &self.author
    }

    pub fn when(&self) -> &DateTime {
        &self.authored
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
        let id = row.try_get("id")?;
        let subject = row.try_get("subject")?;
        let body = row.try_get("body")?;
        let author = row.try_get("author")?;
        let authored = row.try_get("authored")?;

        Ok(Self {
            id,
            subject,
            body,
            author,
            authored,
        })
    }
}

pub struct CommitWithImpl {
    pub commit: Commit,
    pub commit_impl: CommitImpl,
}

impl<'r, R> sqlx::FromRow<'r, R> for CommitWithImpl
where
    R: sqlx::Row,
    &'r str: sqlx::ColumnIndex<R>,
    Oid: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    String: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
    DateTime: sqlx::Decode<'r, R::Database> + sqlx::Type<R::Database>,
{
    fn from_row(row: &'r R) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let subject = row.try_get("subject")?;
        let body = row.try_get("body")?;
        let author = row.try_get("author")?;
        let authored = row.try_get("authored")?;

        let commit = Commit {
            id,
            subject,
            body,
            author,
            authored,
        };

        let id = row.try_get("impl_id")?;
        let ancestry = match row.try_get("parent")? {
            None => Ancestry::Root,
            Some(parent) => Ancestry::Parent { parent, merge_parent: row.try_get("merge_parent")? }
        };
        let committer = row.try_get("committer")?;
        let committed = row.try_get("committed")?;

        let commit_impl = CommitImpl {
            id,
            commit: commit.id,
            ancestry,
            committer,
            committed,
        };

        Ok(Self { commit, commit_impl })
    }
}