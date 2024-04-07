use std::collections::BTreeMap;

use hash::Oid;

use crate::key::{Key, KeyWithPathing};

pub struct RegisterEntryCollection<S, D>(BTreeMap<Key<S>, D>);

impl<S, D> RegisterEntryCollection<S, D> {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&Key<S>, &D)> {
        self.0.iter()
    }
}

impl<S, D> crate::sealed::Sealed for RegisterEntryCollection<S, D> {}

impl<S: AsRef<str> + Eq + Ord, D> FromIterator<(Key<S>, D)> for RegisterEntryCollection<S, D> {
    fn from_iter<T: IntoIterator<Item = (Key<S>, D)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, S: AsRef<str> + Eq + Ord, D> FromIterator<&'a (Key<S>, D)> for RegisterEntryCollection<&'a S, &'a D> {
    fn from_iter<T: IntoIterator<Item = &'a (Key<S>, D)>>(iter: T) -> Self {
        let mut map = BTreeMap::new();
        for (key, data) in iter.into_iter() {
            let key = key.as_ref();
            map.insert(key, data);
        }
        Self(map)
    }
}

pub struct SaveEntryCollection<S>(BTreeMap<KeyWithPathing<S>, Oid>);

impl<S> SaveEntryCollection<S> {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&KeyWithPathing<S>, &Oid)> {
        self.0.iter()
    }
}

impl<K> crate::sealed::Sealed for SaveEntryCollection<K> {}

impl<S: AsRef<str> + Eq + Ord> FromIterator<(KeyWithPathing<S>, Oid)> for SaveEntryCollection<S> {
    fn from_iter<T: IntoIterator<Item = (KeyWithPathing<S>, Oid)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a, S: AsRef<str> + Eq + Ord> FromIterator<&'a (KeyWithPathing<S>, Oid)> for SaveEntryCollection<&'a S> {
    fn from_iter<T: IntoIterator<Item = &'a (KeyWithPathing<S>, Oid)>>(iter: T) -> Self {
        let mut map = BTreeMap::new();
        for (key, oid) in iter.into_iter() {
            let key = key.as_ref();
            map.insert(key, *oid);
        }
        Self(map)
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EntryData {
    pub(crate) kind: RegisterEntryKind,
    pub(crate) content: Oid,
}

impl EntryData {
    pub const fn new(kind: RegisterEntryKind, content: Oid) -> Self {
        Self { kind, content }
    }
}
