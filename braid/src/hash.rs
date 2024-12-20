use crate::Oid;

pub(crate) struct HasherImpl {
    hasher: blake3::Hasher,
}

impl HasherImpl {
    pub(crate) fn new() -> Self {
        let hasher = blake3::Hasher::new();
        Self { hasher }
    }

    pub(crate) fn finalize(self) -> Oid {
        let hash = self.hasher.finalize();
        let hash = Into::<[u8; Oid::LEN]>::into(hash);
        Oid::new(hash)
    }
}

impl HasherInternal for HasherImpl {
    fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }
}

impl Hasher for HasherImpl {}

pub(crate) trait HasherInternal {
    fn update(&mut self, data: &[u8]);
}

#[allow(private_bounds)]
pub trait Hasher: HasherInternal {
    fn push_null(&mut self) {
        self.update(&[0]);
    }
}

pub trait Hash {
    fn hash<H: Hasher>(&self, hasher: &mut H);
}

macro_rules! impl_deterministic_bytes {
    ($($t:ty),*) => {
        $(
            impl Hash for $t {
                fn hash<H: Hasher>(&self, hasher: &mut H) {
                    hasher.update(&self.to_le_bytes());
                }
            }
        )*
    };
}

impl_deterministic_bytes!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

impl Hash for str {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.update(self.as_bytes());
    }
}

impl Hash for bool {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.update(&[*self as u8]);
    }
}
