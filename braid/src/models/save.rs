use crate::{Braid, DateTime, Hash, Oid};

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
