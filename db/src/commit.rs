use hash::Oid;

use crate::register::SaveEntryCollection;

pub struct CommitData<S1 = String, S2 = String> {
    pub(crate) register: Oid,
    pub(crate) parent: Option<Oid>,
    pub(crate) merge_parent: Option<Oid>,
    pub(crate) rebase_of: Option<Oid>,
    pub(crate) saves: SaveEntryCollection<S2>,
    pub(crate) date: time::OffsetDateTime,
    pub(crate) committer: S1,
    pub(crate) summary: S1,
    pub(crate) body: S1,
}

impl<S1, S2> CommitData<S1, S2> {
    pub fn new(
        register: Oid,
        parent: Option<Oid>,
        merge_parent: Option<Oid>,
        rebase_of: Option<Oid>,
        saves: SaveEntryCollection<S2>,
        date: time::OffsetDateTime,
        committer: S1,
        summary: S1,
        body: S1,
    ) -> Self {
        Self {
            register,
            parent,
            merge_parent,
            rebase_of,
            saves,
            date,
            committer,
            summary,
            body,
        }
    }
}

impl<S, I> crate::sealed::Sealed for CommitData<S, I> {}