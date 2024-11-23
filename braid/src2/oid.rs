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
