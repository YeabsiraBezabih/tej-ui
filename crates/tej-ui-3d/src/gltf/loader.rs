//! Zero-copy glTF 2.0 binary (`.glb`) chunk parser and geometry extractor.
//!
//! Parses standard 12-byte `.glb` container headers, extracts structured JSON
//! metadata and zero-copy binary buffer views directly mapped into CPU memory.

use crate::math::Vec3;

/// Magic byte sequence for glTF 2.0 binary containers (`0x46546C67` = "glTF").
pub const GLB_MAGIC: u32 = 0x4654_6C67;
/// glTF container specification version.
pub const GLB_VERSION_2: u32 = 2;
/// Chunk type for JSON metadata (`0x4E4F534A` = "JSON").
pub const CHUNK_TYPE_JSON: u32 = 0x4E4F_534A;
/// Chunk type for binary buffer data (`0x004E4942` = "BIN\0").
pub const CHUNK_TYPE_BIN: u32 = 0x004E_4942;

/// Typed errors encountered during glTF binary parsing.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum GlbError {
    #[error("File too small for GLB header (minimum 12 bytes required)")]
    BufferTooShort,
    #[error("Invalid GLB magic: expected 0x46546C67 ('glTF'), found {0:#010X}")]
    InvalidMagic(u32),
    #[error("Unsupported glTF container version: expected 2, found {0}")]
    UnsupportedVersion(u32),
    #[error("Missing or malformed JSON chunk")]
    MissingJsonChunk,
    #[error("Invalid UTF-8 in JSON chunk")]
    InvalidJsonUtf8,
    #[error("Buffer overrun: needed {needed} bytes, but only {available} available")]
    BufferOverrun { needed: usize, available: usize },
}

/// 12-Byte glTF 2.0 binary container header.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlbHeader {
    pub magic: u32,
    pub version: u32,
    pub length: u32,
}

/// Zero-copy parsed view over a `.glb` binary container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlbPackage<'a> {
    pub header: GlbHeader,
    pub json_bytes: &'a [u8],
    pub bin_bytes: Option<&'a [u8]>,
}

impl<'a> GlbPackage<'a> {
    /// Parses a raw `.glb` byte slice into a zero-copy package.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, GlbError> {
        if bytes.len() < 12 {
            return Err(GlbError::BufferTooShort);
        }

        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != GLB_MAGIC {
            return Err(GlbError::InvalidMagic(magic));
        }

        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != GLB_VERSION_2 {
            return Err(GlbError::UnsupportedVersion(version));
        }

        let length = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        if (length as usize) > bytes.len() {
            return Err(GlbError::BufferOverrun {
                needed: length as usize,
                available: bytes.len(),
            });
        }

        let header = GlbHeader {
            magic,
            version,
            length,
        };

        // Parse Chunk 0 (Must be JSON)
        let mut offset = 12;
        if offset + 8 > bytes.len() {
            return Err(GlbError::MissingJsonChunk);
        }

        let chunk0_len = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;
        let chunk0_type = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        if chunk0_type != CHUNK_TYPE_JSON || offset + chunk0_len > bytes.len() {
            return Err(GlbError::MissingJsonChunk);
        }
        let json_bytes = &bytes[offset..offset + chunk0_len];
        offset += chunk0_len;

        // Optional Chunk 1 (BIN buffer)
        let mut bin_bytes = None;
        if offset + 8 <= bytes.len() {
            let chunk1_len = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            let chunk1_type = u32::from_le_bytes([
                bytes[offset + 4],
                bytes[offset + 5],
                bytes[offset + 6],
                bytes[offset + 7],
            ]);
            offset += 8;

            if chunk1_type == CHUNK_TYPE_BIN && offset + chunk1_len <= bytes.len() {
                bin_bytes = Some(&bytes[offset..offset + chunk1_len]);
            }
        }

        Ok(Self {
            header,
            json_bytes,
            bin_bytes,
        })
    }

    /// Returns the JSON chunk as a UTF-8 string slice.
    pub fn json_text(&self) -> Result<&'a str, GlbError> {
        std::str::from_utf8(self.json_bytes).map_err(|_| GlbError::InvalidJsonUtf8)
    }

    /// Extracts zero-copy IEEE-754 `Vec3` positions from the binary chunk.
    pub fn extract_positions(
        &self,
        byte_offset: usize,
        vertex_count: usize,
    ) -> Result<Vec<Vec3>, GlbError> {
        let bin = self.bin_bytes.ok_or(GlbError::BufferTooShort)?;
        let needed = byte_offset + vertex_count * 12;
        if needed > bin.len() {
            return Err(GlbError::BufferOverrun {
                needed,
                available: bin.len(),
            });
        }

        let mut positions = Vec::with_capacity(vertex_count);
        for i in 0..vertex_count {
            let base = byte_offset + i * 12;
            let x = f32::from_le_bytes([bin[base], bin[base + 1], bin[base + 2], bin[base + 3]]);
            let y =
                f32::from_le_bytes([bin[base + 4], bin[base + 5], bin[base + 6], bin[base + 7]]);
            let z =
                f32::from_le_bytes([bin[base + 8], bin[base + 9], bin[base + 10], bin[base + 11]]);
            positions.push(Vec3::new(x, y, z));
        }

        Ok(positions)
    }

    /// Extracts `u16` triangle indices from the binary chunk widened to `u32`.
    pub fn extract_indices_u16(
        &self,
        byte_offset: usize,
        index_count: usize,
    ) -> Result<Vec<u32>, GlbError> {
        let bin = self.bin_bytes.ok_or(GlbError::BufferTooShort)?;
        let needed = byte_offset + index_count * 2;
        if needed > bin.len() {
            return Err(GlbError::BufferOverrun {
                needed,
                available: bin.len(),
            });
        }

        let mut indices = Vec::with_capacity(index_count);
        for i in 0..index_count {
            let base = byte_offset + i * 2;
            let idx = u16::from_le_bytes([bin[base], bin[base + 1]]);
            indices.push(u32::from(idx));
        }

        Ok(indices)
    }

    /// Constructs a valid in-memory `.glb` container byte stream from JSON and BIN slices.
    #[must_use]
    pub fn build_glb(json: &str, bin: &[u8]) -> Vec<u8> {
        let json_bytes = json.as_bytes();
        // glTF chunks must be 4-byte aligned, padded with spaces (JSON) or zeros (BIN)
        let json_pad = (4 - (json_bytes.len() % 4)) % 4;
        let bin_pad = (4 - (bin.len() % 4)) % 4;

        let chunk0_len = json_bytes.len() + json_pad;
        let chunk1_len = bin.len() + bin_pad;
        let total_len = 12 + (8 + chunk0_len) + (8 + chunk1_len);

        let mut out = Vec::with_capacity(total_len);

        // Header
        out.extend_from_slice(&GLB_MAGIC.to_le_bytes());
        out.extend_from_slice(&GLB_VERSION_2.to_le_bytes());
        #[allow(clippy::cast_possible_truncation)]
        out.extend_from_slice(&(total_len as u32).to_le_bytes());

        // Chunk 0 (JSON)
        #[allow(clippy::cast_possible_truncation)]
        out.extend_from_slice(&(chunk0_len as u32).to_le_bytes());
        out.extend_from_slice(&CHUNK_TYPE_JSON.to_le_bytes());
        out.extend_from_slice(json_bytes);
        out.resize(out.len() + json_pad, b' ');

        // Chunk 1 (BIN)
        #[allow(clippy::cast_possible_truncation)]
        out.extend_from_slice(&(chunk1_len as u32).to_le_bytes());
        out.extend_from_slice(&CHUNK_TYPE_BIN.to_le_bytes());
        out.extend_from_slice(bin);
        out.resize(out.len() + bin_pad, 0);

        out
    }
}
