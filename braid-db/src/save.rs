use braid_hash::Oid;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "postgres", derive(sqlx::FromRow))]
pub struct SaveData<S = String> {
    pub(crate) author: S,
    pub(crate) date: time::OffsetDateTime,
    pub(crate) content: Oid,
    pub(crate) parent: Oid,
}

impl<S> crate::sealed::Sealed for SaveData<S> {}

impl<S> SaveData<S> {
    pub fn new(
        author: S,
        date: time::OffsetDateTime,
        content: Oid,
        parent: Oid,
    ) -> Self {
        Self {
            author,
            date,
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

    pub fn content(&self) -> Oid {
        self.content
    }

    pub fn parent(&self) -> Oid {
        self.parent
    }
}

#[derive(Debug)]
pub struct Save<S = String> {
    pub(crate) id: Oid,
    pub(crate) data: SaveData<S>,
}
