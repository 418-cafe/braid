use crate::{sql::SetupGet, Oid};

type DateTime = sqlx::types::chrono::DateTime<sqlx::types::chrono::FixedOffset>;

macro_rules! object_id {
    ($name:ident) => {
        impl crate::object::ObjectId for $name {
            fn oid(&self) -> Oid {
                self.oid
            }
        }
    };
}

#[derive(sqlx::FromRow)]
pub struct Commit {
    oid: Oid,
    parent: Option<Oid>,
    merge_parent: Option<Oid>,
    subject: String,
    message: String,

    #[sqlx(flatten)]
    author: Author,

    #[sqlx(flatten)]
    committer: Author,
}

#[derive(sqlx::FromRow)]
pub struct Author {
    user_id: String,
    when: DateTime,
}

object_id!(Commit);

impl SetupGet for Commit {
    const QUERY: &'static str = "SELECT oid FROM commit WHERE oid = $1";
}

#[derive(sqlx::FromRow)]
pub struct Register {
    oid: Oid,
}

object_id!(Register);

impl SetupGet for Register {
    const QUERY: &'static str = "SELECT oid FROM register WHERE oid = $1";
}

#[derive(sqlx::FromRow)]
pub(crate) struct CommitParents {
    pub(crate) oid: Oid,
    pub(crate) parent: Option<Oid>,
    pub(crate) merge_parent: Option<Oid>,
}

object_id!(CommitParents);

impl SetupGet for CommitParents {
    const QUERY: &'static str =
        "SELECT oid, parent, merge_parent FROM commit_parents WHERE oid = $1";
}