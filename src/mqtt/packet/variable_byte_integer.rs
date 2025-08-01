/**
 * MIT License
 *
 * Copyright (c) 2025 Takatoshi Kondo
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use arrayvec::ArrayVec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::{From, TryFrom};
use std::fmt;
use std::io::IoSlice;

/// MQTT Variable Byte Integer representation with pre-encoded byte buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableByteInteger {
    /// Encoded bytes, at most 4 bytes.
    encoded: ArrayVec<u8, 4>,
}

impl VariableByteInteger {
    pub const MAX: u32 = 0x0FFF_FFFF;

    /// Encode a `u32` into 1-4 bytes, return `None` if too large.
    pub fn from_u32(mut value: u32) -> Option<Self> {
        if value > Self::MAX {
            return None;
        }
        let mut buf = ArrayVec::<u8, 4>::new();
        loop {
            let mut byte = (value % 128) as u8;
            value /= 128;
            if value > 0 {
                byte |= 0x80;
            }
            buf.push(byte);
            if value == 0 {
                break;
            }
        }
        Some(Self { encoded: buf })
    }

    /// Decode back to `u32`.
    pub fn to_u32(&self) -> u32 {
        let mut multiplier = 1u32;
        let mut result = 0u32;
        for &b in &*self.encoded {
            result += u32::from(b & 0x7F) * multiplier;
            multiplier = multiplier.saturating_mul(128);
        }
        result
    }

    /// Number of bytes in the encoding.
    pub fn size(&self) -> usize {
        self.encoded.len()
    }

    /// Borrow as byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.encoded
    }

    /// For scatter-gather I/O.
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        vec![IoSlice::new(&self.encoded)]
    }

    /// Streaming decode: if enough bytes, returns `(vbi, consumed)`,
    /// if too few bytes then `Incomplete`, else error.
    pub fn decode_stream(buf: &[u8]) -> DecodeResult<Self> {
        let mut multiplier = 1u32;
        let mut value = 0u32;
        let mut read = ArrayVec::<u8, 4>::new();

        for (i, &b) in buf.iter().take(4).enumerate() {
            value = value
                .checked_add(u32::from(b & 0x7F) * multiplier)
                .unwrap_or(u32::MAX);
            if value > Self::MAX {
                return DecodeResult::Err("VariableByteInteger too large");
            }
            read.push(b);

            if (b & 0x80) == 0 {
                // complete
                return match Self::from_u32(value) {
                    Some(vbi) => DecodeResult::Ok(vbi, i + 1),
                    None => DecodeResult::Err("Encoding failure"),
                };
            }
            multiplier = multiplier.checked_mul(128).unwrap_or(0);
        }

        if buf.len() < 4 {
            DecodeResult::Incomplete
        } else {
            DecodeResult::Err("Malformed VariableByteInteger: too many bytes")
        }
    }
}

/// Result for streaming decode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeResult<T> {
    Ok(T, usize),
    Incomplete,
    Err(&'static str),
}

impl Serialize for VariableByteInteger {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.to_u32())
    }
}

impl<'de> Deserialize<'de> for VariableByteInteger {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = u32::deserialize(deserializer)?;
        VariableByteInteger::from_u32(val)
            .ok_or_else(|| serde::de::Error::custom("Value too large"))
    }
}

impl fmt::Display for VariableByteInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_u32())
    }
}

impl From<VariableByteInteger> for u32 {
    fn from(vbi: VariableByteInteger) -> Self {
        vbi.to_u32()
    }
}

impl TryFrom<u32> for VariableByteInteger {
    type Error = &'static str;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        VariableByteInteger::from_u32(value).ok_or("Value too large")
    }
}
