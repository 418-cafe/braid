use braid_hash::Oid;

use crate::register::{Register, SaveRegister};

#[derive(Clone, Debug)]
pub struct CommitData<S = String> {
    pub(crate) register: Oid,
    pub(crate) parent: Option<Oid>,
    pub(crate) merge_parent: Option<Oid>,
    pub(crate) rebase_of: Option<Oid>,
    pub(crate) saves: Oid,
    pub(crate) date: time::OffsetDateTime,
    pub(crate) committer: S,
    pub(crate) summary: S,
    pub(crate) body: S,
}

impl<S> CommitData<S> {
    pub fn new(
        register: Oid,
        parent: Oid,
        merge_parent: Option<Oid>,
        rebase_of: Option<Oid>,
        saves: Oid,
        date: time::OffsetDateTime,
        committer: S,
        summary: S,
        body: S,
    ) -> Self {
        Self {
            register,
            parent: Some(parent),
            merge_parent,
            rebase_of,
            saves,
            date,
            committer,
            summary,
            body,
        }
    }

    pub fn register(&self) -> Oid {
        self.register
    }

    pub fn parent(&self) -> Option<Oid> {
        self.parent
    }

    pub fn merge_parent(&self) -> Option<Oid> {
        self.merge_parent
    }

    pub fn rebase_of(&self) -> Option<Oid> {
        self.rebase_of
    }

    pub fn saves(&self) -> Oid {
        self.saves
    }

    pub fn date(&self) -> time::OffsetDateTime {
        self.date
    }

    pub fn committer(&self) -> &S {
        &self.committer
    }

    pub fn summary(&self) -> &S {
        &self.summary
    }

    pub fn body(&self) -> &S {
        &self.body
    }
}

impl CommitData<&'static str> {
    pub const ROOT_ID: Oid = Oid::from_bytes([12, 46, 2, 176, 91, 76, 242, 95, 187, 36, 182, 14, 106, 55, 234, 69, 19, 82, 131, 152, 198, 253, 24, 229, 177, 158, 229, 37, 115, 159, 138, 217]);

    pub const ROOT: Self = CommitData {
        register: Register::EMPTY_ID,
        parent: None,
        merge_parent: None,
        rebase_of: None,
        saves: SaveRegister::EMPTY_ID,
        date: time::OffsetDateTime::UNIX_EPOCH,
        committer: "",
        summary: "",
        body: "",
    };
}

impl<S> crate::sealed::Sealed for CommitData<S> {}