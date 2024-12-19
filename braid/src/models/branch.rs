use crate::Oid;

pub(crate) struct BranchExists<'a>(pub(crate) &'a str);

pub struct Branch<S> {
    pub(crate) name: S,
    pub(crate) tip: Oid,
    pub(crate) is_default: bool,
}
