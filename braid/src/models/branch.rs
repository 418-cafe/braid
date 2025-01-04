use crate::{FullKey, Oid};

pub struct Branch<S> {
    pub(crate) name: FullKey<S>,
    pub(crate) tip: Oid,
    pub(crate) is_default: bool,
}
