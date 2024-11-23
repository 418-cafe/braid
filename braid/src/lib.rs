#![feature(generic_const_exprs)]
#![feature(generic_const_items)]
#![feature(const_trait_impl)]

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

#[const_trait]
trait FieldData : Copy {
    const TABLE: &'static str;
    const KEY: Self;

    fn quoted_name(&self) -> &'static str;
    fn flag(&self) -> u32;
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

#[const_trait]
#[allow(private_bounds)]
pub trait Field : ~const FieldData {}

impl<T> const Field for T where T: ~const FieldData {}

#[const_trait]
pub trait Fields<const N: usize, F: ~const Field> {
    const FIELDS: [F; N];
}

#[const_trait]
trait Select<const N: usize, const S: usize, F: ~const Field> : Fields<N, F> {
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

struct Buffer<const N: usize> {
    data: [u8; N],
    offset: usize,
}

impl<const N: usize> Buffer<N> {
    const fn new() -> Self {
        Self {
            data: [0; N],
            offset: 0,
        }
    }

    const fn copy_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len();

        let mut i = 0;

        while i < len {
            self.data[self.offset] = bytes[i];
            self.offset += 1;
            i += 1;
        }
    }

    const fn finalize(self) -> [u8; N] {
        if self.offset != N {
            panic!("Buffer not filled!");
        }
        
        self.data
    }
}

struct Commit1;

impl Fields<2, CommitField> for Commit1 {
    const FIELDS: [CommitField; 2] = [CommitField::Oid, CommitField::Parent];
}

#[test]
fn test1() {
    println!("{}", Commit1::SELECT);
}