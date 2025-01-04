use crate::{FixedOffset, Key, Oid};

pub type SaveParentContent = Option<Oid>;

#[derive(Clone, Copy)]
pub struct SaveRequest<'a, T> {
    pub key: Key<&'a str>,
    pub branch: Key<&'a str>,
    pub object: Option<&'a T>,
    pub tz: Option<FixedOffset>,
    pub parent_content: SaveParentContent,
}
