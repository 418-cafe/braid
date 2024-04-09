use hash::Oid;

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

impl<S> crate::sealed::Sealed for CommitData<S> {}
