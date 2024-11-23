mod buf;
mod commit;

pub use commit::{CommitField, CommitRow};
use buf::Buffer;

type Result<T> = std::result::Result<T, Error>;

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

#[const_trait]
pub(crate) trait FieldData : Copy {
    const TABLE: &'static str;
    const KEY: Self;

    fn quoted_name(&self) -> &'static str;
    fn flag(&self) -> u32;
}

#[const_trait]
#[allow(private_bounds)]
pub trait Field : ~const FieldData {}

impl<T> const Field for T where T: ~const FieldData {}

#[const_trait]
pub trait Fields<const N: usize, F: ~const Field> {
    const FIELDS: [F; N];
}

#[const_trait]
pub(crate) trait Select<const N: usize, const S: usize, F: ~const Field> : Fields<N, F> {
    const SELECT_BYTES: [u8; S];
    const SELECT: &str = match std::str::from_utf8(&Self::SELECT_BYTES) {
        Ok(s) => s,
        Err(_) => panic!("Invalid utf8"),
    };
}

impl <const N: usize, F: const Field, T: Fields<N, F>> const Select<N, { select_len::<N, F, T>() }, F> for T {
    const SELECT_BYTES: [u8; select_len::<N, F, T>()] = {
        let mut buf = Buffer::<{ select_len::<N, F, T>() }>::new();
        fill_select_bytes(&mut buf, &T::FIELDS);
        buf.finalize()
    };
}

const fn fill_select_bytes<const N: usize, F: ~const Field, const B: usize>(buf: &mut Buffer<B>, fields: &[F; N]) {
    buf.copy_str("SELECT ");
    
    let mut i = 0;
    loop {
        buf.copy_str(fields[i].quoted_name());
        i += 1;

        if i == N {
            break;
        }

        buf.copy_str(", ");
    }

    buf.copy_str(" FROM ");
    buf.copy_str(F::TABLE);
    buf.copy_str(" where ");
    buf.copy_str(F::KEY.quoted_name());
    buf.copy_str(" = $1;");
}

const fn select_len<const N: usize, F: ~const Field, T: Fields<N, F>>() -> usize {
    let mut s = 0usize;
    let mut i = 0;

    macro_rules! checked {
        ($expr:expr) => {
            match s.checked_add($expr) {
                Some(s) => s,
                None => panic!("Overflow trying to calculate select_len"),
            }
        };
    }

    let mut fields = 0;

    while i < N {
        if fields & T::FIELDS[i].flag() != 0 {
            panic!("Duplicate field");
        }

        fields |= T::FIELDS[i].flag();

        s = checked!(T::FIELDS[i].quoted_name().len());
        i += 1;
    }

    let comma_space = (N - 1) * ", ".len();

    s = checked!(comma_space);
    s = checked!("SELECT  FROM ".len());
    s = checked!(F::TABLE.len());
    s = checked!(" where ".len());
    s = checked!(F::KEY.quoted_name().len());
    s = checked!(" = $1;".len());

    s
}
