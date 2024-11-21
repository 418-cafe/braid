#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ancestry<P> {
    Root,
    Parent {
        parent: P,
        merge_parent: Option<P>,
    },
}

impl<P> Ancestry<P> {
    pub fn as_ref(&self) -> Ancestry<&P> {
        match self {
            Ancestry::Root => Ancestry::Root,
            Ancestry::Parent { parent, merge_parent } => {
                Ancestry::Parent {
                    parent,
                    merge_parent: merge_parent.as_ref(),
                }
            }
        }
    }
    
    pub fn map<T>(self, f: impl Fn(P) -> T) -> Ancestry<T> {
        match self {
            Ancestry::Root => Ancestry::Root,
            Ancestry::Parent { parent, merge_parent } => {
                Ancestry::Parent {
                    parent: f(parent),
                    merge_parent: merge_parent.map(f),
                }
            }
        }
    }
}