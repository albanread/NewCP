//! Byte-level read primitives matching `Stores.Reader` from the legacy
//! BlackBox runtime. The cursor wraps a borrowed byte slice and tracks an
//! absolute position; all multi-byte reads are little-endian.

use crate::error::{OdcError, Result};

pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

// The full Stores.Reader primitive surface is included even though some
// methods aren't used by current decoders — they're called by view-body
// decoders we'll add next (rulers, controls, headers, …).
#[allow(dead_code)]
impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn pos(&self) -> u64 {
        self.pos as u64
    }

    pub fn len(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn set_pos(&mut self, pos: u64) -> Result<()> {
        let pos = pos as usize;
        if pos > self.data.len() {
            return Err(OdcError::Truncated {
                at: self.pos as u64,
                want: pos - self.pos,
                have: self.remaining(),
            });
        }
        self.pos = pos;
        Ok(())
    }

    fn need(&self, n: usize) -> Result<()> {
        if self.remaining() < n {
            Err(OdcError::Truncated {
                at: self.pos as u64,
                want: n,
                have: self.remaining(),
            })
        } else {
            Ok(())
        }
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        self.need(1)?;
        let b = self.data[self.pos];
        self.pos += 1;
        Ok(b)
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        Ok(self.read_byte()? != 0)
    }

    /// `ReadSChar` / `ReadXChar`: a single 8-bit Latin-1 character.
    pub fn read_schar(&mut self) -> Result<u8> {
        self.read_byte()
    }

    /// `ReadChar`: a 16-bit little-endian Unicode code point.
    pub fn read_char(&mut self) -> Result<u16> {
        self.need(2)?;
        let b0 = self.data[self.pos] as u16;
        let b1 = self.data[self.pos + 1] as u16;
        self.pos += 2;
        Ok(b0 | (b1 << 8))
    }

    /// `ReadSInt` / `ReadXInt`: 16-bit little-endian signed integer.
    pub fn read_xint(&mut self) -> Result<i16> {
        self.need(2)?;
        let v = i16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    /// `ReadInt`: 32-bit little-endian signed integer.
    pub fn read_int(&mut self) -> Result<i32> {
        self.need(4)?;
        let v = i32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    /// `ReadLong`: 64-bit little-endian signed integer.
    pub fn read_long(&mut self) -> Result<i64> {
        self.need(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(i64::from_le_bytes(buf))
    }

    /// `ReadSReal` / `ReadXReal`: 32-bit little-endian float.
    pub fn read_sreal(&mut self) -> Result<f32> {
        self.need(4)?;
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.data[self.pos..self.pos + 4]);
        self.pos += 4;
        Ok(f32::from_le_bytes(buf))
    }

    /// `ReadReal`: 64-bit little-endian float.
    pub fn read_real(&mut self) -> Result<f64> {
        self.need(8)?;
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&self.data[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(f64::from_le_bytes(buf))
    }

    /// `ReadSet`: 32-bit little-endian unsigned integer holding a `SET`.
    pub fn read_set(&mut self) -> Result<u32> {
        self.need(4)?;
        let v = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    /// `ReadVersion`: a single byte version number (caller validates the range).
    pub fn read_version(&mut self) -> Result<u8> {
        self.read_byte()
    }

    /// `ReadSString` / `ReadXString`: bytes terminated by 0x00.
    /// Returns the bytes excluding the terminator. The legacy code calls
    /// `Kernel.Utf8ToString` afterwards on path components; we keep raw bytes
    /// here and let callers decode (path components are UTF-8, fold labels
    /// for v0 folds are Latin-1).
    pub fn read_sstring(&mut self) -> Result<Vec<u8>> {
        let start = self.pos as u64;
        let mut out = Vec::new();
        loop {
            if self.remaining() == 0 {
                return Err(OdcError::StringNotTerminated { at: start });
            }
            let b = self.data[self.pos];
            self.pos += 1;
            if b == 0 {
                return Ok(out);
            }
            out.push(b);
        }
    }

    /// `ReadString`: u16 LE codepoints terminated by 0x0000.
    pub fn read_string(&mut self) -> Result<Vec<u16>> {
        let start = self.pos as u64;
        let mut out = Vec::new();
        loop {
            if self.remaining() < 2 {
                return Err(OdcError::StringNotTerminated { at: start });
            }
            let lo = self.data[self.pos] as u16;
            let hi = self.data[self.pos + 1] as u16;
            self.pos += 2;
            let cp = lo | (hi << 8);
            if cp == 0 {
                return Ok(out);
            }
            out.push(cp);
        }
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<&'a [u8]> {
        self.need(n)?;
        let slice = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(slice)
    }

    pub fn skip(&mut self, n: usize) -> Result<()> {
        self.need(n)?;
        self.pos += n;
        Ok(())
    }
}

/// Decode an SString interpreted as UTF-8 (matches `Kernel.Utf8ToString`
/// usage in `Stores.ReadPath`).
pub fn sstring_as_utf8(bytes: &[u8], at: u64) -> Result<String> {
    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|_| OdcError::InvalidString { at })
}

/// Decode a SString as Latin-1 (used for fold/link payloads at version 0/1).
pub fn sstring_as_latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

/// Decode a 16-bit string as UTF-16, lossily replacing invalid surrogates.
#[allow(dead_code)]
pub fn string_as_utf16(words: &[u16]) -> String {
    String::from_utf16_lossy(words)
}
