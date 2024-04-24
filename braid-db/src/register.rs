use std::{borrow::Borrow, collections::BTreeMap};

use braid_hash::Oid;

use crate::{Key, ObjectKind, RegisterEntryKey, SaveEntryKey};

pub enum RegisterKind {
    Register,
    SaveRegister,
}

impl RegisterKind {
    pub const fn as_object_kind(self) -> ObjectKind {
        match self {
            Self::Register => ObjectKind::Register,
            Self::SaveRegister => ObjectKind::SaveRegister,
        }
    }
}

pub trait EntryData<S>: crate::sealed::Sealed {
    const EMPTY_ID: Oid;
    const REGISTER_KIND: RegisterKind;
    type Key: Key<S>;

    fn new() -> Self;

    fn get<Q: ?Sized + Ord>(&self, key: &Q) -> Option<&Oid>
    where
        S: Borrow<Q>;

    fn insert(&mut self, key: Self::Key, oid: Oid);

    fn iter<'a>(&'a self) -> impl ExactSizeIterator<Item = (&S, &Oid)>
    where
        S: 'a;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;
}

pub struct RegisterData<S>(BTreeMap<S, Oid>);

impl<S> crate::sealed::Sealed for RegisterData<S> {}

pub struct Register<S = String> {
    pub(crate) id: Oid,
    pub(crate) data: RegisterData<S>,
}

impl Register<()> {
    pub const EMPTY_ID: Oid = Oid::from_bytes([
        249, 230, 187, 110, 142, 190, 207, 255, 22, 36, 155, 39, 181, 94, 64, 99, 83, 212, 240,
        161, 116, 63, 80, 246, 150, 238, 139, 55, 60, 211, 211, 141,
    ]);
}

impl<S> Register<S> {
    pub fn id(&self) -> Oid {
        self.id
    }

    pub fn data(&self) -> &RegisterData<S> {
        &self.data
    }
}

pub struct SaveRegisterData<S>(BTreeMap<S, Oid>);

impl<S> crate::sealed::Sealed for SaveRegisterData<S> {}

pub struct SaveRegister<S = String> {
    pub(crate) id: Oid,
    pub(crate) data: SaveRegisterData<S>,
}

impl SaveRegister<()> {
    pub const EMPTY_ID: Oid = Oid::from_bytes([
        170, 108, 88, 58, 52, 10, 245, 194, 182, 224, 232, 252, 161, 20, 33, 183, 207, 7, 140, 128,
        144, 172, 178, 229, 60, 64, 135, 65, 223, 103, 176, 193,
    ]);
}

impl<S> SaveRegister<S> {
    pub fn id(&self) -> Oid {
        self.id
    }

    pub fn data(&self) -> &SaveRegisterData<S> {
        &self.data
    }
}

macro_rules! impl_entry_data {
    ($id:ident::$type:ident<$key:ident> => $kind:ident) => {
        impl<S: Ord + AsRef<str>> $type<S> {
            pub fn new() -> Self {
                Self(BTreeMap::new())
            }

            pub fn get<Q: ?Sized + Ord>(&self, key: &Q) -> Option<&Oid>
            where
                S: Borrow<Q>,
            {
                self.0.get(key)
            }

            pub fn insert(&mut self, key: $key<S>, oid: Oid) {
                self.0.insert(key.into_inner(), oid);
            }

            pub fn iter<'a>(&'a self) -> impl ExactSizeIterator<Item = (&S, &Oid)>
            where
                S: 'a,
            {
                self.0.iter()
            }

            pub fn len(&self) -> usize {
                self.0.len()
            }

            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl<S: AsRef<str>> EntryData<S> for $type<S>
        where
            S: Ord,
        {
            const EMPTY_ID: Oid = $id::EMPTY_ID;
            const REGISTER_KIND: RegisterKind = RegisterKind::$kind;
            type Key = $key<S>;

            fn new() -> Self {
                Self::new()
            }

            fn get<Q: ?Sized + Ord>(&self, key: &Q) -> Option<&Oid>
            where
                S: Borrow<Q>,
            {
                self.get(key)
            }

            fn insert(&mut self, key: Self::Key, oid: Oid) {
                self.insert(key, oid);
            }

            fn iter<'a>(&'a self) -> impl ExactSizeIterator<Item = (&S, &Oid)>
            where
                S: 'a,
            {
                self.iter()
            }

            fn len(&self) -> usize {
                self.len()
            }

            fn is_empty(&self) -> bool {
                self.is_empty()
            }
        }
    };
}

impl_entry_data!(Register::RegisterData<RegisterEntryKey> => Register);
impl_entry_data!(SaveRegister::SaveRegisterData<SaveEntryKey> => SaveRegister);
