use crate::{Key, Oid};

pub struct Branch<S> {
    pub(crate) name: Key<S>,
    pub(crate) tip: Oid,
    pub(crate) is_default: bool,
}
