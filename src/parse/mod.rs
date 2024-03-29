//! Utilities to help parsing data between WASM and host

/// WASM memory writer
pub struct Writer<'a>(&'a mut [u8]);

impl<'a> Writer<'a> {
    /// Make new writer
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self(bytes)
    }

    /// Send a u8 to the WASM module
    pub fn u8(&mut self, byte: u8) {
        self.0[..1].copy_from_slice(&byte.to_le_bytes());

        // Hack around lifetime issues
        let mut swap: &'a mut [u8] = &mut [];
        core::mem::swap(&mut self.0, &mut swap);
        let mut swap = swap.split_at_mut(1).1;
        core::mem::swap(&mut self.0, &mut swap);
    }

    /// Send a u16 to the WASM module
    pub fn u16(&mut self, half: u16) {
        self.0[..2].copy_from_slice(&half.to_le_bytes());

        // Hack around lifetime issues
        let mut swap: &'a mut [u8] = &mut [];
        core::mem::swap(&mut self.0, &mut swap);
        let mut swap = swap.split_at_mut(2).1;
        core::mem::swap(&mut self.0, &mut swap);
    }

    /// Send a u32 to the WASM module
    pub fn u32(&mut self, word: u32) {
        self.0[..4].copy_from_slice(&word.to_le_bytes());

        // Hack around lifetime issues
        let mut swap: &'a mut [u8] = &mut [];
        core::mem::swap(&mut self.0, &mut swap);
        let mut swap = swap.split_at_mut(4).1;
        core::mem::swap(&mut self.0, &mut swap);
    }

    /// Send a u64 to the WASM module
    pub fn u64(&mut self, long: u64) {
        self.0[..8].copy_from_slice(&long.to_le_bytes());

        // Hack around lifetime issues
        let mut swap: &'a mut [u8] = &mut [];
        core::mem::swap(&mut self.0, &mut swap);
        let mut swap = swap.split_at_mut(8).1;
        core::mem::swap(&mut self.0, &mut swap);
    }

    /// Send a UTF-8 string to the WASM module
    pub fn str(&mut self, utf8: &str) {
        let len = utf8.len();
        self.0[..len].copy_from_slice(utf8.as_bytes());

        // Hack around lifetime issues
        let mut swap: &'a mut [u8] = &mut [];
        core::mem::swap(&mut self.0, &mut swap);
        let mut swap = swap.split_at_mut(len).1;
        core::mem::swap(&mut self.0, &mut swap);
    }
}

/// WASM memory reader
pub struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    /// Make new reader
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Receive a u8 from the WASM module
    pub fn u8(&mut self) -> u8 {
        let bytes = self.0[..1].try_into().expect("Out of bounds read");
        self.0 = &self.0[1..];
        u8::from_le_bytes(bytes)
    }

    /// Receive a u16 from the WASM module
    pub fn u16(&mut self) -> u16 {
        let bytes = self.0[..2].try_into().expect("Out of bounds read");
        self.0 = &self.0[2..];
        u16::from_le_bytes(bytes)
    }

    /// Receive a u32 from the WASM module
    pub fn u32(&mut self) -> u32 {
        let bytes = self.0[..4].try_into().expect("Out of bounds read");
        self.0 = &self.0[4..];
        u32::from_le_bytes(bytes)
    }

    /// Receive a u64 from the WASM module
    pub fn u64(&mut self) -> u64 {
        let bytes = self.0[..8].try_into().expect("Out of bounds read");
        self.0 = &self.0[8..];
        u64::from_le_bytes(bytes)
    }

    /// Receive a UTF-8 string from the WASM module
    pub fn str(&mut self) -> Result<&str, core::str::Utf8Error> {
        let bytes = self.0.get(0..).expect("Out of bounds read");
        self.0 = &[];
        core::str::from_utf8(bytes)
    }
}
