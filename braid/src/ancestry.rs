use crate::Oid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ancestry<P> {
    Root,
    Parent { parent: P, merge_parent: Option<P> },
}

impl crate::Hash for Ancestry<Oid> {
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        let (parent, merge_parent) = self.hashable_bytes();
        hasher.update(parent);
        hasher.update(merge_parent);
    }
}

impl Ancestry<Oid> {
    const fn hashable_bytes(&self) -> (&[u8; Oid::LEN], &[u8; Oid::LEN]) {
        match self {
            Ancestry::Root => const { (Oid::ZERO.as_bytes(), Oid::ZERO.as_bytes()) },
            Ancestry::Parent {
                parent,
                merge_parent: None,
            } => (parent.as_bytes(), const { Oid::ZERO.as_bytes() }),
            Ancestry::Parent {
                parent,
                merge_parent: Some(merge_parent),
            } => (parent.as_bytes(), merge_parent.as_bytes()),
        }
    }
}

impl<P> Ancestry<P> {
    pub const fn root() -> Self {
        Ancestry::Root
    }

    pub const fn as_ref(&self) -> Ancestry<&P> {
        match self {
            Ancestry::Root => Ancestry::Root,
            Ancestry::Parent {
                parent,
                merge_parent,
            } => Ancestry::Parent {
                parent,
                merge_parent: merge_parent.as_ref(),
            },
        }
    }

    pub fn map<T>(self, f: impl Fn(P) -> T) -> Ancestry<T> {
        match self {
            Ancestry::Root => Ancestry::Root,
            Ancestry::Parent {
                parent,
                merge_parent,
            } => Ancestry::Parent {
                parent: f(parent),
                merge_parent: merge_parent.map(f),
            },
        }
    }
}
