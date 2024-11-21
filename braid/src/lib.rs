mod braid;
mod models;
mod postgres;

pub extern crate braid_derive;

pub use braid_derive::FromCommitData;
pub use braid_fields::commit::CommitField;

#[derive(FromCommitData)]
struct Foo {
    oid: usize,
    parent: usize,
}

impl Default for Foo {
    fn default() -> Self {
        Self {
            oid: 0,
            parent: 0,
        }
    }
}

impl Foo {
    fn oid(&self) -> usize {
        self.oid
    }
}

pub trait FromCommitData<const N: usize, const S: usize> : Default {
    const FIELDS: [CommitField; N];
}

trait FromCommitDataInternal<const N: usize, const S: usize> : FromCommitData<N, S> {
    const SELECT: &'static str = match std::str::from_utf8(&make_select_bytes::<S>(&fields_to_strs(&Self::FIELDS), b" FROM commit where \"oid\" = $1")) {
        Ok(s) => s,
        Err(_) => panic!("Invalid UTF-8 sequence"),
    };
}

impl<const N: usize, const S: usize, T: FromCommitData<N, S>> FromCommitDataInternal<N, S> for T {}

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

#[test]
fn test() {
    fn s<const N: usize, const S: usize, T: FromCommitData<N, S>>() -> T {
        println!("{}", T::SELECT);
        Default::default()
    }

    let foo: Foo = s();

    println!("{}", foo.oid());
}