use sqlx::{ColumnIndex, Row};

use crate::{ancestry::Ancestry, FieldData, Oid};

use super::{Error, Result};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CommitField {
    Oid,
    Parent,
}

impl CommitField {
    const fn quoted_name(&self) -> &'static str {
        match self {
            CommitField::Oid => "\"oid\"",
            CommitField::Parent => "\"parent\"",
        }
    }

    const fn flag(&self) -> u32 {
        match self {
            CommitField::Oid => 1,
            CommitField::Parent => 1 << 1,
        }
    }
}

impl const FieldData for CommitField {
    const TABLE: &'static str = "\"commit\"";
    const KEY: Self = Self::Oid;

    fn quoted_name(&self) -> &'static str {
        Self::quoted_name(self)
    }

    fn flag(&self) -> u32 {
        Self::flag(self)
    }
}

pub trait CommitRow {
    fn oid(&self) -> Result<Oid>;
    fn ancestry(&self) -> Result<Ancestry<Oid>>;
}

pub(crate) struct CommitRowImpl<R>(R);

impl<R> CommitRowImpl<R> {
    pub fn new(row: R) -> Self {
        Self(row)
    }
}

impl<'r, R> CommitRow for CommitRowImpl<&'r R>
where
    R: Row,
    str: ColumnIndex<R>,
    Oid: sqlx::Decode<'r, R::Database>,
    Oid: sqlx::Type<R::Database>,
{
    fn oid(&self) -> Result<Oid> {
        Ok(self.0.try_get("oid")?)
    }

    fn ancestry(&self) -> Result<Ancestry<Oid>> {
        const FIELD_NAME: &str = "ancestry";

        let map_err = |e| {
            match e {
                sqlx::Error::ColumnNotFound(_) => Error::ColumnNotRequested(FIELD_NAME.to_string()),
                e => Error::Sql(e),
            }
        };

        let parent = match self.0.try_get("parent").map_err(map_err)? {
            Some(oid) => oid,
            None => return Ok(Ancestry::Root),
        };

        Ok(Ancestry::Parent {
            parent,
            merge_parent: self.0.try_get("merge_parent").map_err(map_err)?,
        })
    }
}