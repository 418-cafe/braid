use hash::Oid;

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
        parent: Option<Oid>,
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

impl<S> crate::sealed::Sealed for CommitData<S> {}