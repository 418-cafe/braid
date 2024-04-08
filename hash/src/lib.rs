use std::hint::unreachable_unchecked;

pub const OID_LEN: usize = blake3::OUT_LEN;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Oid([u8; OID_LEN]);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct HexByte(u8);

impl HexByte {
    pub const ZERO: Self = Self(0);

    /// SAFETY: it is the caller's responsibility to ensure
    /// that the byte is in the range 0..=15.
    const unsafe fn from_u8_unchecked(byte: u8) -> Self {
        Self(match byte {
            0..=9 => b'0' + byte,
            10..=15 => b'a' + byte - 10,
            _ => unreachable_unchecked(),
        })
    }
}

impl std::fmt::Display for HexByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Debug for HexByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

pub trait HexByteExtensions: sealed::Sealed {
    fn as_str(&self) -> &str;
}

impl sealed::Sealed for [HexByte] {}

impl HexByteExtensions for [HexByte] {
    fn as_str(&self) -> &str {
        let bytes: &[u8] = unsafe { std::mem::transmute(self) };

        // SAFETY: each byte being a valid ascii character is an invariant
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

pub trait HexBytePairExtensions: sealed::Sealed {
    fn flat(&self) -> &[HexByte];
}

impl sealed::Sealed for [[HexByte; 2]] {}

impl HexBytePairExtensions for [[HexByte; 2]] {
    fn flat(&self) -> &[HexByte] {
        unsafe { std::mem::transmute(self) }
    }
}

#[derive(Debug)]
pub struct InvalidOidStringError<S>(S);

impl Oid {
    pub const ZERO: Self = Self([0; OID_LEN]);
    pub const LEN: usize = OID_LEN;

    pub const fn repeat(byte: u8) -> Self {
        Self([byte; OID_LEN])
    }

    pub const fn from_bytes(bytes: [u8; OID_LEN]) -> Self {
        Self(bytes)
    }

    pub fn try_from_str<S: AsRef<str>>(hex: S) -> Result<Self, InvalidOidStringError<S>> {
        let hex_bytes = hex.as_ref().as_bytes();

        if hex_bytes.len() != OID_LEN * 2 {
            return Err(InvalidOidStringError(hex));
        }

        let mut bytes = [0; OID_LEN];

        for i in 0..OID_LEN {
            let hi = hex_bytes[i * 2];
            let lo = hex_bytes[i * 2 + 1];

            fn to_byte(c: u8) -> u8 {
                match c {
                    b'0'..=b'9' => c - b'0',
                    b'a'..=b'f' => c - b'a' + 10,
                    _ => 0,
                }
            }

            let hi = to_byte(hi);
            let lo = to_byte(lo);

            bytes[i] = (hi << 4) | lo;
        }

        Ok(Self(bytes))
    }

    pub const fn as_bytes(&self) -> &[u8; OID_LEN] {
        &self.0
    }

    pub const fn into_inner(self) -> [u8; OID_LEN] {
        self.0
    }

    pub const fn hex_ascii_bytes(&self) -> [HexByte; OID_LEN * 2] {
        let mut bytes = [HexByte::ZERO; OID_LEN * 2];
        let mut i = 0;
        while i < OID_LEN {
            let byte = self.0[i];
            let hi = byte >> 4;
            let lo = byte & 0x0f;

            // SAFETY: we know hi and lo are in the range 0..=15 from bit shifting.
            let hi = unsafe { HexByte::from_u8_unchecked(hi) };
            let lo = unsafe { HexByte::from_u8_unchecked(lo) };

            bytes[i * 2] = hi;
            bytes[i * 2 + 1] = lo;

            i += 1;
        }
        bytes
    }

    pub const fn hex_ascii_byte_pairs(&self) -> [[HexByte; 2]; OID_LEN] {
        // SAFETY: [u8; OID_LEN * 2] is a contiguous array of bytes OID_LEN * 2 long.
        // [[u8; 2]; OID_LEN] is also a contiguous array of bytes OID_LEN * 2 long
        // represented as pairs of bytes.
        unsafe { std::mem::transmute(self.hex_ascii_bytes()) }
    }

    pub fn to_string(&self) -> String {
        let bytes = self.hex_ascii_bytes();
        debug_assert!(bytes.iter().all(|h| h.0.is_ascii()));

        // SAFETY: we know self only consists of bytes and
        // that each byte is represented by two hex ascii
        // characters from `hex_ascii_bytes`.
        unsafe { String::from_utf8_unchecked(bytes.iter().map(|h| h.0).collect()) }
    }
}

impl std::fmt::Display for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
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

mod sealed {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_string() {
        for mul in [0, 1, 2, 3] {
            let bytes = std::array::from_fn(|i| mul * i as u8);
            let oid = crate::Oid::from_bytes(bytes);

            let string = oid.to_string();
            let oid2 = crate::Oid::try_from_str(&string).unwrap();

            assert_eq!(oid, oid2);
        }
    }
}
