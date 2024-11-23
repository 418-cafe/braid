use sqlx::{ColumnIndex, Row};

use crate::{ancestry::Ancestry, Oid};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sql error: {0}")]
    Sql(sqlx::Error),

    #[error("column not requested: {0}")]
    ColumnNotRequested(String),
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::ColumnNotFound(column) => {
                Error::ColumnNotRequested(column)
            },

            _ => Error::Sql(e),
        }
    }
}

pub trait CommitRow {
    fn oid(&self) -> Result<Oid>;
    fn ancestry(&self) -> Result<Ancestry<Oid>>;
}

struct CommitRowImpl<R>(R);

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