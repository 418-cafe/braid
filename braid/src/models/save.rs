use crate::{BraidTransaction, DateTime, Hash, Oid};

#[derive(Debug, Clone)]
pub struct Save<S> {
    pub(crate) id: Oid,
    pub(crate) data: SaveData<S>,
}

impl<S> Save<S> {
    pub fn id(&self) -> Oid {
        self.id
    }

    pub fn parent(&self) -> Option<Oid> {
        self.data.parent
    }

    pub fn branch(&self) -> &S {
        &self.data.branch
    }

    pub fn key(&self) -> &S {
        &self.data.key
    }

    pub fn is_current(&self) -> bool {
        self.data.is_current
    }

    pub fn saved(&self) -> DateTime {
        self.data.when
    }

    pub fn content(&self) -> Oid {
        self.data.content
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub(crate) struct SaveData<S> {
    pub(crate) parent: Option<Oid>,
    pub(crate) branch: S,
    pub(crate) key: S,
    pub(crate) is_current: bool,
    pub(crate) when: DateTime,
    pub(crate) content: Oid,
}

impl<S: AsRef<str>> SaveData<S> {
    pub(crate) fn hash(self) -> Save<S> {
        let id = BraidTransaction::hash(&self);
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

            // whether it's the latest save does not affect the hash
            is_current: _,
        } = self;

        parent.as_ref().unwrap_or(&Oid::ZERO).hash(hasher);
        when.timestamp_millis().hash(hasher);
        content.hash(hasher);
        branch.as_ref().hash(hasher);
        hasher.push_null();
        key.as_ref().hash(hasher);
    }
}
