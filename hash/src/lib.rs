pub const OID_LEN: usize = blake3::OUT_LEN;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Oid([u8; OID_LEN]);

impl Oid {
    pub const ZERO: Self = Self([0; OID_LEN]);
    pub const LEN: usize = OID_LEN;

    pub const fn from_bytes(bytes: [u8; OID_LEN]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; OID_LEN] {
        &self.0
    }

    pub const fn into_inner(self) -> [u8; OID_LEN] {
        self.0
    }

    pub fn to_string(&self) -> String {
        format!("{}", self)
    }
}

impl std::fmt::Display for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.as_bytes() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as std::fmt::Display>::fmt(self, f)
    }
}

type Serializer = rmp_serde::Serializer<blake3::Hasher>;

pub fn hash_obj<T: serde::Serialize>(
    data: &T,
) -> Result<Oid, <&mut Serializer as serde::ser::Serializer>::Error> {
    let hasher = blake3::Hasher::new();
    let mut ser = Serializer::new(hasher);
    data.serialize(&mut ser)?;
    let hash = ser.into_inner().finalize();
    Ok(Oid(hash.into()))
}

pub fn hash(data: &[u8]) -> Oid {
    Oid(blake3::hash(data).into())
}

pub struct Hasher(blake3::Hasher);

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher {
    pub fn new() -> Self {
        Self(blake3::Hasher::new())
    }

    pub fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    pub fn finalize(self) -> Oid {
        Oid(self.0.finalize().into())
    }
}

impl std::io::Write for Hasher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash() {
        let oid = hash_obj(&42).unwrap();
        println!("oid: {}", oid);
    }
}
