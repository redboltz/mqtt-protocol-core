// MIT License
//
// Copyright (c) 2025 Takatoshi Kondo
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use core::fmt::{Debug, Display};
use core::hash::Hash;
use num_traits::{Bounded, One, PrimInt};
use serde::Serialize;

/// Packet ID types with associated buffer operations
pub trait IsPacketId:
    PrimInt + One + Bounded + Debug + Display + Hash + Eq + Serialize + 'static
{
    /// Fixed-size buffer type
    type Buffer: AsRef<[u8]> + AsMut<[u8]> + Clone + Default + Eq;

    /// Convert packet ID to fixed-size buffer
    fn to_buffer(&self) -> Self::Buffer;

    /// Parse packet ID from buffer
    fn from_buffer(buf: &[u8]) -> Self;
}

impl IsPacketId for u16 {
    type Buffer = [u8; 2];

    fn to_buffer(&self) -> Self::Buffer {
        self.to_be_bytes()
    }

    fn from_buffer(buf: &[u8]) -> Self {
        if buf.len() >= 2 {
            u16::from_be_bytes([buf[0], buf[1]])
        } else {
            0
        }
    }
}

impl IsPacketId for u32 {
    type Buffer = [u8; 4];

    fn to_buffer(&self) -> Self::Buffer {
        self.to_be_bytes()
    }

    fn from_buffer(buf: &[u8]) -> Self {
        if buf.len() >= 4 {
            u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]])
        } else {
            0
        }
    }
}

/// Trait for types that can be converted into an optional packet ID
///
/// This trait enables the packet_id() builder method to accept both direct values
/// (e.g., `packet_id(42)`) and optional values (e.g., `packet_id(Some(42))` or `packet_id(None)`).
///
/// # Examples
///
/// ```
/// use mqtt_protocol_core::mqtt::packet::IntoPacketId;
///
/// // Direct value
/// let id1: Option<u16> = 42u16.into_packet_id();
/// assert_eq!(id1, Some(42));
///
/// // Optional value
/// let id2: Option<u16> = Some(42u16).into_packet_id();
/// assert_eq!(id2, Some(42));
///
/// // None value
/// let id3: Option<u16> = None::<u16>.into_packet_id();
/// assert_eq!(id3, None);
/// ```
pub trait IntoPacketId<T> {
    /// Convert self into an optional packet ID
    fn into_packet_id(self) -> Option<T>;
}

// Implementations for u16

/// Implementation for direct u16 packet ID values
///
/// Allows direct u16 values like `42u16` to be converted to `Some(42)`
impl IntoPacketId<u16> for u16 {
    fn into_packet_id(self) -> Option<u16> {
        Some(self)
    }
}

/// Implementation for optional u16 packet ID values
///
/// Allows optional values like `Some(42u16)` or `None::<u16>` to be passed through
impl IntoPacketId<u16> for Option<u16> {
    fn into_packet_id(self) -> Option<u16> {
        self
    }
}

// Implementations for u32

/// Implementation for direct u32 packet ID values
///
/// Allows direct u32 values like `42u32` to be converted to `Some(42)`
impl IntoPacketId<u32> for u32 {
    fn into_packet_id(self) -> Option<u32> {
        Some(self)
    }
}

/// Implementation for optional u32 packet ID values
///
/// Allows optional values like `Some(42u32)` or `None::<u32>` to be passed through
impl IntoPacketId<u32> for Option<u32> {
    fn into_packet_id(self) -> Option<u32> {
        self
    }
}
