use std::mem::MaybeUninit;

use crate::Hash;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Oid([u8; Oid::LEN]);

impl Oid {
    pub const LEN: usize = blake3::OUT_LEN;
    pub const ZERO: Oid = Oid([0; Self::LEN]);

    pub fn new(data: [u8; Self::LEN]) -> Self {
        Oid(data)
    }

    pub const fn as_bytes(&self) -> &[u8; Self::LEN] {
        &self.0
    }

    const fn hex_bytes(&self) -> [u8; Self::LEN * 2] {
        const HEX_TABLE: [u8; 16] = *b"0123456789abcdef";
        let mut result: MaybeUninit<[u8; Self::LEN * 2]> = MaybeUninit::uninit();
        let ptr = result.as_mut_ptr() as *mut u8;

        let mut i = 0;
        while i < Self::LEN {
            let byte = self.0[i];
            unsafe {
                *ptr.add(2 * i) = HEX_TABLE[(byte >> 4) as usize];
                *ptr.add(2 * i + 1) = HEX_TABLE[(byte & 0x0F) as usize];
            }
            i += 1;
        }

        // SAFETY: `result` has been fully initialized.
        unsafe { result.assume_init() }
    }
}

impl std::fmt::Debug for Oid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_bytes = self.hex_bytes();

        // SAFETY: `hex_bytes` guarantees that the bytes are valid UTF-8.
        let hex = unsafe { std::str::from_utf8_unchecked(&hex_bytes) };

        <&str as std::fmt::Display>::fmt(&hex, f)
    }
}

impl Hash for Oid {
    fn hash<H: crate::Hasher>(&self, hasher: &mut H) {
        hasher.update(&self.0);
    }
}
