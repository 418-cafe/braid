use hash::Oid;

use crate::register::SaveEntryCollection;

pub struct CommitData<S> {
    pub(crate) register: Oid,
    pub(crate) parent: Option<Oid>,
    pub(crate) merge_parent: Option<Oid>,
    pub(crate) rebase_of: Option<Oid>,
    pub(crate) saves: Vec<Oid>,
    //pub(crate) saves: SaveEntryCollection<S>,
    pub(crate) date: time::OffsetDateTime,
    pub(crate) committer: S,
    pub(crate) summary: S,
    pub(crate) body: S,
}

impl<S> crate::sealed::Sealed for CommitData<S> {}