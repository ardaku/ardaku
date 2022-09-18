//! Utilities to help parsing data between WASM and host

/// WASM memory writer
pub struct Writer<'a>(&'a mut [u8]);

impl<'a> Writer<'a> {
    /// Make new writer
    pub fn new(bytes: &'a mut [u8]) -> Self {
        Self(bytes)
    }

    /// Send a u32 to the WASM module
    pub fn u32(&'a mut self, word: u32) { // not sure why this lifetime req'd?
        self.0[..4].copy_from_slice(&word.to_le_bytes());
        self.0 = &mut self.0[4..];
    }
}

/// WASM memory reader
pub struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    /// Make new reader
    pub fn new(bytes: &'a [u8]) -> Self {
        Self(bytes)
    }

    /// Receive a u32 from the WASM module
    pub fn u32(&mut self) -> u32 {
        let bytes = self.0[0..4].try_into().expect("Out of bounds read");
        self.0 = &self.0[4..];
        u32::from_le_bytes(bytes)
    }

    /// Receive a u8 from the WASM module
    pub fn u8(&mut self) -> u8 {
        let bytes = self.0[0..1].try_into().expect("Out of bounds read");
        self.0 = &self.0[1..];
        u8::from_le_bytes(bytes)
    }

    /// Receive a UTF-8 string from the WASM module
    pub fn str(&mut self, len: usize) -> Result<&str, core::str::Utf8Error> {
        let bytes = self.0.get(0..len).expect("Out of bounds read");
        self.0 = &self.0[len..];
        core::str::from_utf8(bytes)
    }
}
