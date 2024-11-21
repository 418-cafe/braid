use crate::unquote;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommitField {
    Oid,
    Parent,
    MergeParent,
}

impl CommitField {
    pub const fn as_str(&self) -> &'static str {
        unquote(self.as_quoted_str())
    }

    pub const fn as_quoted_str(&self) -> &'static str {
        match self {
            Self::Oid => "\"oid\"",
            Self::Parent => "\"parent\"",
            Self::MergeParent => "\"merge_parent\"",
        }
    }

    pub fn try_from_str(name: &str) -> Option<Self> {
        const OID: &str = CommitField::Oid.as_str();
        const PARENT: &str = CommitField::Parent.as_str();
        const MERGE_PARENT: &str = CommitField::MergeParent.as_str();

        match name {
            OID => Some(Self::Oid),
            PARENT => Some(Self::Parent),
            MERGE_PARENT => Some(Self::MergeParent),
            _ => None,
        }
    }
}