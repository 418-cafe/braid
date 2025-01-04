use crate::{DateTime, FullKey, Oid};

pub struct Save<S> {
    pub(crate) id: Oid,
    pub(crate) key: FullKey<S>,
    pub(crate) parent: Option<Oid>,
    pub(crate) when: DateTime,
    pub(crate) content: Option<Oid>,
}

impl<S> Save<S> {
    pub fn id(&self) -> Oid {
        self.id
    }

    pub fn key(&self) -> &FullKey<S> {
        &self.key
    }

    pub fn parent(&self) -> Option<Oid> {
        self.parent
    }

    pub fn when(&self) -> DateTime {
        self.when
    }

    pub fn content(&self) -> Option<Oid> {
        self.content
    }
}

#[derive(Debug, Clone, Copy, sqlx::FromRow)]
pub(crate) struct SaveData {
    pub(crate) parent: Option<Oid>,
    pub(crate) when: DateTime,
    pub(crate) content: Option<Oid>,
}

impl crate::Hash for SaveData {
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        let Self {
            parent,
            when,
            content,
        } = self;

        parent.unwrap_or(Oid::ZERO).hash(hasher);
        when.hash(hasher);
        content.unwrap_or(Oid::ZERO).hash(hasher);
    }
}

pub(crate) struct SaveLineageCriteria<'a, I> {
    pub(crate) branch: FullKey<&'a str>,
    pub(crate) keys: I,
}
