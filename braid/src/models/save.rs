use crate::{Braid, DateTime, Hash, Key, Oid};

#[derive(Debug, Clone, Copy)]
pub struct Save<S, P = ()> {
    pub(crate) id: Oid,
    pub(crate) parent: P,
    pub(crate) data: SaveData<S>,
}

impl<S> Save<S, Option<Oid>> {
    pub fn id(&self) -> Oid {
        self.id
    }

    pub fn parent(&self) -> Option<Oid> {
        self.parent
    }

    pub fn branch(&self) -> &S {
        &self.data.branch
    }

    pub fn key(&self) -> &S {
        &self.data.key
    }

    pub fn saved(&self) -> DateTime {
        self.data.when
    }

    pub fn content(&self) -> Option<Oid> {
        self.data.content
    }
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub(crate) struct SaveData<S> {
    pub(crate) branch: S,
    pub(crate) key: S,
    pub(crate) when: DateTime,
    pub(crate) content: Option<Oid>,
}

impl<S: AsRef<str>> SaveData<S> {
    pub(crate) fn hash(self) -> Save<S> {
        let id = Braid::hash(&self);
        Save {
            id,
            parent: (),
            data: self,
        }
    }
}

impl<S> Hash for SaveData<S>
where
    S: AsRef<str>,
{
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        let Self {
            when,
            key,
            content,
            branch,
        } = self;

        when.timestamp_millis().hash(hasher);
        content.unwrap_or(Oid::ZERO).hash(hasher);
        branch.as_ref().hash(hasher);
        hasher.push_null();
        key.as_ref().hash(hasher);
    }
}

pub(crate) struct SaveLineageCriteria<'a, I> {
    pub(crate) branch: Key<'a>,
    pub(crate) keys: I,
}
