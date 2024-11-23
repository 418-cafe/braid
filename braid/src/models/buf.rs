pub(super) struct Buffer<const N: usize> {
    data: [u8; N],
    offset: usize,
}

impl<const N: usize> Buffer<N> {
    pub(super) const fn new() -> Self {
        Self {
            data: [0; N],
            offset: 0,
        }
    }

    pub(super) const fn copy_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let len = bytes.len();

        let mut i = 0;

        while i < len {
            self.data[self.offset] = bytes[i];
            self.offset += 1;
            i += 1;
        }
    }

    pub(super) const fn finalize(self) -> [u8; N] {
        if self.offset != N {
            panic!("Buffer not filled!");
        }
        
        self.data
    }
}