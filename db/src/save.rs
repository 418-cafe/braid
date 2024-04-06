use hash::Oid;

pub struct SaveData<S> {
    pub(crate) author: S,
    pub(crate) date: time::OffsetDateTime,
    pub(crate) content: Oid,
    pub(crate) parent: SaveParent,
}

impl<S> crate::sealed::Sealed for SaveData<S> {}

impl<S> SaveData<S> {
    pub fn new(author: S, date: time::OffsetDateTime, content: Oid, parent: SaveParent) -> Self {
        Self {
            author,
            date,
            content,
            parent,
        }
    }
}

pub enum SaveParentKind {
    Save,
    Parent,
}

pub struct SaveParent {
    pub(crate) kind: SaveParentKind,
    pub(crate) oid: Oid,
}

impl SaveParent {
    pub fn new(kind: SaveParentKind, oid: Oid) -> Self {
        Self { kind, oid }
    }
}
