use std::borrow::Borrow;

use hash::Oid;

use crate::{Object, ObjectKind};

pub struct Register<E> {
    pub(crate) id: Oid,
    pub(crate) data: RegisterData<E>,
}

pub struct RegisterData<E> {
    pub(crate) entries: E,
}

impl<E> Object for Register<E> {
    const KIND: ObjectKind = ObjectKind::Register;
}

impl<E> Object for RegisterData<E> {
    const KIND: ObjectKind = Register::<E>::KIND;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum EntryKind {
    ExecutableContent = 0,
    Content = 1,
    Register = 2,
}

impl EntryKind {
    pub fn from_u8(value: u8) -> Option<Self> {
        const EXECUTABLE_CONTENT: u8 = EntryKind::ExecutableContent as u8;
        const CONTENT: u8 = EntryKind::Content as u8;
        const REGISTER: u8 = EntryKind::Register as u8;

        match value {
            EXECUTABLE_CONTENT => Some(EntryKind::ExecutableContent),
            CONTENT => Some(EntryKind::Content),
            REGISTER => Some(EntryKind::Register),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Entry<S> {
    pub(crate) id: Oid,
    pub(crate) name: S,
    pub(crate) kind: EntryKind,
}

impl Borrow<str> for Entry<&str> {
    fn borrow(&self) -> &str {
        self.name
    }
}
