#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
pub struct Oid([u8; Oid::LEN]);

impl Oid {
    pub const LEN: usize = blake3::OUT_LEN;

    pub fn new(data: [u8; Self::LEN]) -> Self {
        Oid(data)
    }

    pub fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }
}

macro_rules! hashable_by_oid {
    ($struct:ident<$($generics:tt),*>) => {
        impl<$($generics),*> crate::object::ObjectId for $struct<$($generics),*> {
            fn oid(&self) -> Oid {
                self.oid
            }
        }

        impl<$($generics),*> std::hash::Hash for $struct<$($generics),*> {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.oid.hash(state);
            }
        }

        impl<$($generics),*> std::cmp::PartialEq for $struct<$($generics),*> {
            fn eq(&self, other: &Self) -> bool {
                self.oid == other.oid
            }
        }

        impl<$($generics),*> std::cmp::Eq for $struct<$($generics),*> {}

        impl<$($generics),*> std::borrow::Borrow<Oid> for $struct<$($generics),*> {
            fn borrow(&self) -> &Oid {
                &self.oid
            }
        }
    };
}

pub(crate) use hashable_by_oid;
