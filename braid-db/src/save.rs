use braid_hash::Oid;

use crate::register::RegisterEntryKind;

#[derive(Clone, Debug)]
pub struct SaveData<S = String> {
    pub(crate) author: S,
    pub(crate) date: time::OffsetDateTime,
    pub(crate) kind: RegisterEntryKind,
    pub(crate) content: Oid,
    pub(crate) parent: SaveParent,
}

impl<S> crate::sealed::Sealed for SaveData<S> {}

impl<S> SaveData<S> {
    pub fn new(
        author: S,
        date: time::OffsetDateTime,
        kind: RegisterEntryKind,
        content: Oid,
        parent: SaveParent,
    ) -> Self {
        Self {
            author,
            date,
            kind,
            content,
            parent,
        }
    }

    pub fn author(&self) -> &S {
        &self.author
    }

    pub fn date(&self) -> time::OffsetDateTime {
        self.date
    }

    pub fn kind(&self) -> RegisterEntryKind {
        self.kind
    }

    pub fn content(&self) -> Oid {
        self.content
    }

    pub fn parent(&self) -> SaveParent {
        self.parent
    }
}

kind! {
    pub enum SaveParentKind {
        Save = 0,
        Commit = 1,
    }

    SaveParentKindError => "Invalid save parent kind: {0:?}"
}

#[derive(Debug, Copy, Clone)]
pub struct SaveParent {
    pub(crate) kind: SaveParentKind,
    pub(crate) id: Oid,
}

impl SaveParent {
    pub fn new(kind: SaveParentKind, id: Oid) -> Self {
        Self { kind, id }
    }

    pub fn kind(&self) -> SaveParentKind {
        self.kind
    }

    pub fn oid(&self) -> Oid {
        self.id
    }
}

#[derive(Debug)]
pub struct Save<S = String> {
    pub(crate) id: Oid,
    pub(crate) data: SaveData<S>,
}
