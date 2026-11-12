use std::str::Utf8Error;

/// Zero-copy row reference reading directly from database memory-mapped page slices.
#[derive(Clone, Copy, Debug)]
pub struct ZeroCopyRow<'a> {
    pub key: &'a [u8],
    pub value: &'a [u8],
}

impl<'a> ZeroCopyRow<'a> {
    /// Construct from raw key and value slices.
    #[inline]
    #[must_use]
    pub const fn new(key: &'a [u8], value: &'a [u8]) -> Self {
        Self { key, value }
    }

    /// Read value bytes as a zero-copy UTF-8 string slice.
    #[inline]
    pub fn value_str(&self) -> Result<&'a str, Utf8Error> {
        std::str::from_utf8(self.value)
    }

    /// Read key bytes as a zero-copy UTF-8 string slice.
    #[inline]
    pub fn key_str(&self) -> Result<&'a str, Utf8Error> {
        std::str::from_utf8(self.key)
    }

    /// Extract 64-bit unsigned integer ID from the key if big-endian encoded.
    #[must_use]
    pub fn id_u64(&self) -> Option<u64> {
        if self.key.len() >= 8 {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&self.key[..8]);
            Some(u64::from_be_bytes(bytes))
        } else {
            None
        }
    }
}
