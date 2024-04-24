use std::{borrow::Borrow, collections::BTreeMap};

use braid_hash::Oid;

use crate::key::{RegisterEntryKey, SaveEntryKey};

#[derive(Debug)]
pub struct Register<S = String, D = EntryData> {
    pub(crate) id: Oid,
    pub(crate) data: RegisterEntryCollection<S, D>,
}

impl Register<&'static str, EntryData> {
    pub const EMPTY_ID: Oid = Oid::from_bytes([
        249, 230, 187, 110, 142, 190, 207, 255, 22, 36, 155, 39, 181, 94, 64, 99, 83, 212, 240,
        161, 116, 63, 80, 246, 150, 238, 139, 55, 60, 211, 211, 141,
    ]);
}

#[derive(Debug)]
pub struct SaveRegister<S = String> {
    pub(crate) id: Oid,
    pub(crate) data: SaveEntryCollection<S>,
}

impl SaveRegister<&'static str> {
    pub const EMPTY_ID: Oid = Oid::from_bytes([
        170, 108, 88, 58, 52, 10, 245, 194, 182, 224, 232, 252, 161, 20, 33, 183, 207, 7, 140, 128,
        144, 172, 178, 229, 60, 64, 135, 65, 223, 103, 176, 193,
    ]);
}

#[derive(Clone, Debug)]
pub struct RegisterEntryCollection<S = String, D = EntryData>(BTreeMap<S, D>);

impl<S, D> RegisterEntryCollection<S, D> {
    pub(crate) fn new_inner() -> Self {
        Self(BTreeMap::new())
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (RegisterEntryKey<&S>, &D)> {
        self.0
            .iter()
            .map(|(key, data)| (RegisterEntryKey::new_unchecked(key), data))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<S: Ord, D> RegisterEntryCollection<S, D> {
    pub fn get<K: Ord + ?Sized>(&self, key: &K) -> Option<&D>
    where
        S: Borrow<K>,
    {
        self.0.get(key.borrow())
    }

    pub fn insert(&mut self, key: RegisterEntryKey<S>, data: D) {
        self.0.insert(key.into_inner(), data);
    }
}

impl<S: Ord + AsRef<str>, D: AsRef<EntryData>> Default for RegisterEntryCollection<S, D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Ord + AsRef<str>, D: AsRef<EntryData>> RegisterEntryCollection<S, D> {
    pub fn new() -> Self {
        Self::new_inner()
    }
}

impl<S, D> crate::sealed::Sealed for RegisterEntryCollection<S, D> {}

impl<S: AsRef<str> + Eq + Ord, D> FromIterator<(RegisterEntryKey<S>, D)>
    for RegisterEntryCollection<S, D>
{
    fn from_iter<T: IntoIterator<Item = (RegisterEntryKey<S>, D)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(key, data)| (key.into_inner(), data))
                .collect(),
        )
    }
}

impl<'a, S: AsRef<str> + Eq + Ord, D> FromIterator<&'a (RegisterEntryKey<S>, D)>
    for RegisterEntryCollection<&'a S, &'a D>
{
    fn from_iter<T: IntoIterator<Item = &'a (RegisterEntryKey<S>, D)>>(iter: T) -> Self {
        let mut map = BTreeMap::new();
        for (key, data) in iter.into_iter() {
            let key = key.as_ref().into_inner();
            map.insert(key, data);
        }
        Self(map)
    }
}

#[derive(Clone, Debug)]
pub struct SaveEntryCollection<S>(BTreeMap<S, Oid>);

impl<S> SaveEntryCollection<S> {
    pub(crate) fn new_inner() -> Self {
        Self(BTreeMap::new())
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = (SaveEntryKey<&S>, &Oid)> {
        self.0
            .iter()
            .map(|(key, data)| (SaveEntryKey::new_unchecked(key), data))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<S: Ord> SaveEntryCollection<S> {
    pub fn get<K: Ord + ?Sized>(&self, key: &K) -> Option<&Oid>
    where
        S: Borrow<K>,
    {
        self.0.get(key.borrow())
    }

    pub fn insert(&mut self, key: SaveEntryKey<S>, oid: Oid) {
        self.0.insert(key.into_inner(), oid);
    }
}

impl<S: Ord + AsRef<str>> Default for SaveEntryCollection<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Ord + AsRef<str>> SaveEntryCollection<S> {
    pub fn new() -> Self {
        Self::new_inner()
    }
}

impl<K> crate::sealed::Sealed for SaveEntryCollection<K> {}

impl<S: AsRef<str> + Eq + Ord> FromIterator<(SaveEntryKey<S>, Oid)> for SaveEntryCollection<S> {
    fn from_iter<T: IntoIterator<Item = (SaveEntryKey<S>, Oid)>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(key, data)| (key.into_inner(), data))
                .collect(),
        )
    }
}

impl<'a, S: AsRef<str> + Eq + Ord> FromIterator<&'a (SaveEntryKey<S>, Oid)>
    for SaveEntryCollection<&'a S>
{
    fn from_iter<T: IntoIterator<Item = &'a (SaveEntryKey<S>, Oid)>>(iter: T) -> Self {
        let mut map = BTreeMap::new();
        for (key, oid) in iter.into_iter() {
            let key = key.as_ref().into_inner();
            map.insert(key, *oid);
        }
        Self(map)
    }
}

kind! {
    pub enum RegisterEntryKind {
        Executable = 0,
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

    pub fn kind(&self) -> RegisterEntryKind {
        self.kind
    }

    pub fn content(&self) -> Oid {
        self.content
    }
}

impl AsRef<EntryData> for EntryData {
    fn as_ref(&self) -> &EntryData {
        self
    }
}
