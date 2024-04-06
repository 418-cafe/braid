use std::{cmp::Ordering, collections::BTreeSet};

use hash::Oid;

use crate::key::Key;

pub struct RegisterEntryCollection<K, D> {
    pub(crate) data: EntryCollection<K, D>,
}

impl<K, D> crate::sealed::Sealed for RegisterEntryCollection<K, D> {}

impl<S: AsRef<str> + Eq + Ord, D> FromIterator<(Key<S>, D)> for RegisterEntryCollection<Key<S>, D> {
    fn from_iter<T: IntoIterator<Item = (Key<S>, D)>>(iter: T) -> Self {
        let data = EntryCollection::from_iter(iter);
        Self { data }
    }
}

impl<'a, S: AsRef<str> + Eq + Ord, D> FromIterator<&'a (Key<S>, D)> for RegisterEntryCollection<&'a Key<S>, &'a D> {
    fn from_iter<T: IntoIterator<Item = &'a (Key<S>, D)>>(iter: T) -> Self {
        let data = EntryCollection::from_iter_borrowed(iter);
        Self { data }
    }
}

kind! {
    pub enum RegisterEntryKind {
        ExecutableContent = 0,
        Content = 1,
        Register = 2,
    }

    RegisterEntryKindError => "Invalid register entry kind: {0:?}"
}


#[derive(Clone, Debug)]
pub(crate) struct EntryCollection<S, D>(BTreeSet<EntryNode<S, D>>);

impl<S, D> EntryCollection<S, D> {
    pub(crate) fn iter(&self) -> impl ExactSizeIterator<Item = (&S, &D)> {
        self.0.iter().map(|node| (&node.name, &node.data))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EntryNode<S, D> {
    pub(crate) name: S,
    pub(crate) data: D,
}

impl<S, D> EntryNode<S, D> {
    pub(crate) fn new(name: S, data: D) -> Self {
        Self { name, data }
    }
}

impl<S: AsRef<str> + Eq, D> Eq for EntryNode<S, D> {}

impl<S: AsRef<str> + Eq, D> PartialEq for EntryNode<S, D> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<S: AsRef<str> + Eq + Ord, D> PartialOrd for EntryNode<S, D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: AsRef<str> + Eq + Ord, D> Ord for EntryNode<S, D> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.as_ref().cmp(other.name.as_ref())
    }
}

impl<S: AsRef<str> + Eq + Ord, D> EntryCollection<S, D> {
    fn from_iter<T: IntoIterator<Item = (S, D)>>(iter: T) -> Self {
        let mut data = BTreeSet::new();

        for (name, entry) in iter {
            let node = EntryNode::new(name, entry);
            data.insert(node);
        }

        Self(data)
    }
}

impl<'a, S: AsRef<str> + Eq + Ord, D> EntryCollection<&'a S, &'a D> {
    fn from_iter_borrowed<T: IntoIterator<Item = &'a (S, D)>>(iter: T) -> Self {
        let mut data = BTreeSet::new();

        for (name, entry) in iter {
            let node = EntryNode::new(name, entry);
            data.insert(node);
        }

        Self(data)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntryData<K> {
    pub(crate) kind: K,
    pub(crate) content: Oid,
}

impl<K> EntryData<K> {
    pub const fn new(kind: K, content: Oid) -> Self {
        Self { kind, content }
    }
}
