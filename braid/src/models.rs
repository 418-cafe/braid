use sqlx::types::chrono;

use crate::{hash::OrDefaultHashable, Ancestry, Braid, Hash, Oid};

pub type DateTime = chrono::DateTime<chrono::FixedOffset>;

trait Str<'a, S> {
    fn as_str(&self) -> Option<&'a str>;
}

impl<'a, S: AsRef<str>> Str<'a, S> for &'a Option<S> {
    fn as_str(&self) -> Option<&'a str> {
        self.as_ref().map(AsRef::as_ref)
    }
}

pub(crate) struct Branch<S> {
    pub(crate) name: S,
    pub(crate) tip: Oid,
    pub(crate) is_default: bool,
}

pub(crate) struct CombinedCommitData<S> {
    pub(crate) subject: Option<S>,
    pub(crate) body: Option<S>,
    pub(crate) author: S,
    pub(crate) when: DateTime,
    pub(crate) committer: S,
    pub(crate) ancestry: Ancestry<Oid>,
    pub(crate) when_impl: DateTime,
}

impl<S> CombinedCommitData<S> {
    pub(crate) fn split_with(
        self,
        commit_id: Oid,
        impl_id: Oid,
    ) -> (CommitData<S>, CommitImplData<S>) {
        let CombinedCommitData {
            subject,
            body,
            author,
            when,
            committer,
            ancestry,
            when_impl,
        } = self;
        let data = CommitData {
            subject,
            body,
            author,
            when,
            implementation: impl_id,
        };
        let impl_data = CommitImplData {
            commit: commit_id,
            committer,
            ancestry,
            when: when_impl,
        };
        (data, impl_data)
    }
}

impl<S: AsRef<str>> Hash for CombinedCommitData<S> {
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        let Self {
            subject,
            body,
            author,
            committer,
            when,
            ancestry,
            when_impl,
        } = self;

        ancestry.hash(hasher);
        when.timestamp_millis().hash(hasher);
        when_impl.timestamp_millis().hash(hasher);

        subject.as_str().or_default_hashable().hash(hasher);
        hasher.push_null();

        body.as_str().or_default_hashable().hash(hasher);
        hasher.push_null();

        author.as_ref().hash(hasher);
        hasher.push_null();

        committer.as_ref().hash(hasher);
        hasher.push_null();
    }
}

pub struct CommitImpl<S> {
    pub(crate) id: Oid,
    pub(crate) data: CommitImplData<S>,
}

pub(crate) struct CommitImplData<S> {
    pub(crate) commit: Oid,
    pub(crate) committer: S,
    pub(crate) ancestry: Ancestry<Oid>,
    pub(crate) when: DateTime,
}

pub struct Commit<S> {
    pub(crate) id: Oid,
    pub(crate) data: CommitData<S>,
}

pub(crate) struct CommitData<S> {
    pub(crate) subject: Option<S>,
    pub(crate) body: Option<S>,
    pub(crate) author: S,
    pub(crate) when: DateTime,
    pub(crate) implementation: Oid,
}

#[derive(Debug, Clone)]
pub struct Save<S> {
    pub(crate) id: Oid,
    pub(crate) data: SaveData<S>,
}

#[derive(Debug, Clone)]
pub(crate) struct SaveData<S> {
    pub(crate) parent: Option<Oid>,
    pub(crate) branch: S,
    pub(crate) key: S,
    pub(crate) when: DateTime,
    pub(crate) content: Oid,
}

impl<S: AsRef<str>> SaveData<S> {
    pub(crate) fn hash(self) -> Save<S> {
        let id = Braid::hash(&self);
        Save { id, data: self }
    }
}

impl<S> Hash for SaveData<S>
where
    S: AsRef<str>,
{
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        let Self {
            parent,
            when,
            key,
            content,
            branch,
        } = self;

        parent.as_ref().unwrap_or(&Oid::ZERO).hash(hasher);
        when.timestamp_millis().hash(hasher);
        content.hash(hasher);
        branch.as_ref().hash(hasher);
        hasher.push_null();
        key.as_ref().hash(hasher);
    }
}
