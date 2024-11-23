mod braid;
mod models;
mod oid;
mod postgres;

pub extern crate braid_derive;

pub use braid_derive::FromCommitData;
pub use braid_fields::commit::CommitField;

use oid::Oid;
use thiserror::Error;


pub trait CommitData {
    fn get_oid(&self) -> Result<Oid, FromDataError>;

    fn get_parent(&self) -> Result<Option<Oid>, FromDataError>;

    fn get_merge_parent(&self) -> Result<Option<Oid>, FromDataError>;
}

pub struct RowCommitData<'a, R>(&'a R);

impl<'a, R: sqlx::Row> CommitData for RowCommitData<'a, R>
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    Oid: sqlx::Type<<R as sqlx::Row>::Database>,
    Oid: sqlx::Decode<'a, <R as sqlx::Row>::Database>,

{
    fn get_oid(&self) -> Result<Oid, FromDataError> {
        Ok(self.0.try_get("oid")?)
    }

    fn get_parent(&self) -> Result<Option<Oid>, FromDataError> {
        Ok(self.0.try_get("parent")?)
    }

    fn get_merge_parent(&self) -> Result<Option<Oid>, FromDataError> {
        Ok(self.0.try_get("merge_parent")?)
    }
}

trait CommitFields<const N: usize, const S: usize> : Fields<N, S, CommitField> {
    const SELECT: &'static str = match std::str::from_utf8(&make_select_bytes::<S>(&fields_to_strs(&Self::FIELDS), b" FROM commit where \"oid\" = $1")) {
        Ok(s) => s,
        Err(_) => panic!("Invalid UTF-8 sequence"),
    };
}

impl<const N: usize, const S: usize, T: Fields<N, S, CommitField>> CommitFields<N, S> for T {}

trait FromCommitData<const N: usize, const S: usize, D: C> : FromData<N, S, CommitField, D> {}

impl<const N: usize, const S: usize, D, T> FromCommitData<N, S, D> for T
where
    D: CommitData,
    T: FromData<N, S, CommitField, D>,
{
}

#[derive(Error, Debug)]
pub enum FromDataError {
    #[error("Field not found: {0}")]
    FieldNotRequested(String),

    #[error("Sql error: {0}")]
    SqlError(sqlx::Error),
}

impl From<sqlx::Error> for FromDataError {
    fn from(value: sqlx::Error) -> Self {
        if let sqlx::Error::ColumnNotFound(column) = value {
            Self::FieldNotRequested(column)
        } else {
            Self::SqlError(value)
        }
    }
}

//#[derive(FromCommitData)]
struct Foo {
    oid: Oid,
    parent: Option<Oid>,
}

impl Default for Foo {
    fn default() -> Self {
        Self {
            oid: Oid::new([0; Oid::LEN]),
            parent: None,
        }
    }
}

#[test]
fn t() {
    struct F;
    impl CommitData for F {
        fn get_oid(&self) -> Result<Oid, FromDataError> {
            Ok(Oid::new([1; Oid::LEN]))
        }
    
        fn get_parent(&self) -> Result<Option<Oid>, FromDataError> {
            Ok(None)
        }
    
        fn get_merge_parent(&self) -> Result<Option<Oid>, FromDataError> {
            Ok(None)
        }
    }

    fn s<const N: usize, const S: usize, T: FromCommitData<N, S, F>>() -> T {
        T::from_data(&F).unwrap()
    }

    //let foo: Foo = s();
    //assert_eq!(foo.oid, Oid::new([1; Oid::LEN]));
}

pub trait Fields<const N: usize, const S: usize, F> {
    const FIELDS: [F; N];
}

pub trait FromData<D> : Sized {
    fn from_data(data: &D) -> Result<Self, FromDataError>;
}

pub trait BraidObject<const N: usize, const S: usize, F, D> : Fields<N, S, F> + FromData<D> {}

impl<const N: usize, const S: usize, F, D, T> BraidObject<N, S, F, D> for T
where
    T: Fields<N, S, F> + FromData<D>,
{
}

const fn fields_to_strs<const N: usize>(fields: &'static [CommitField; N]) -> [&'static str; N] {
    let mut strs = [""; N];

    let mut i = 0;
    while i < N {
        strs[i] = fields[i].as_quoted_str();
        i += 1;
    }

    strs
}

const fn make_select_bytes<const S: usize>(fields: &'static [&str], from: &'static [u8]) -> [u8; S] {
    const SELECT: &[u8] = b"SELECT ";
    const COMMA_SPACE: &[u8] = b", ";

    let mut bytes = [0; S];
    let mut offset = 0;
    
    let mut i = 0;
    while i < SELECT.len() {
        bytes[offset + i] = SELECT[i];
        i += 1;
    }
    
    offset += SELECT.len();
    
    let mut i = 0;
    loop {
        let current = fields[i].as_bytes();
        
        let mut j = 0;
        while j < current.len() {
            bytes[offset + j] = current[j];
            j += 1;
        }

        offset += current.len();
        
        i += 1;
        if i == fields.len() {
            break;
        }

        let mut j = 0;
        while j < COMMA_SPACE.len() {
            bytes[offset + j] = COMMA_SPACE[j];
            j += 1;
        }

        offset += COMMA_SPACE.len();
    }

    let mut i = 0;
    while i < from.len() {
        bytes[offset + i] = from[i];
        i += 1;
    }

    if offset + from.len() != S {
        panic!("Invalid length");
    }

    bytes
}