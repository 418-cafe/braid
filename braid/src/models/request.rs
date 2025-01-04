use crate::{FixedOffset, FullKey, Oid};

pub type SaveParentContent = Option<Oid>;

#[derive(Clone, Copy)]
pub struct SaveRequest<'a, T> {
    pub key: FullKey<&'a str>,
    pub branch: FullKey<&'a str>,
    pub object: Option<&'a T>,
    pub tz: Option<FixedOffset>,
    pub parent_content: SaveParentContent,
}
