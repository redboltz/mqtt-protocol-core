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
use serde::{Serialize, Serializer};
use std::sync::Arc;

/// A reference-counted byte payload with slice semantics
///
/// `ArcPayload` provides an efficient way to handle byte data by using `Arc<[u8]>`
/// for reference counting, combined with offset and length information to represent
/// a slice view of the underlying data. This allows for zero-copy sharing of payload
/// data across multiple consumers while maintaining slice-like semantics.
#[derive(Clone, Debug)]
pub struct ArcPayload {
    data: Arc<[u8]>,
    start: usize,
    length: usize,
}

impl PartialEq for ArcPayload {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for ArcPayload {}

impl Serialize for ArcPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

impl ArcPayload {
    /// Create a new `ArcPayload` from reference-counted data with specified range
    ///
    /// Creates a new payload that represents a slice view of the provided `Arc<[u8]>` data
    /// starting at the specified offset with the given length.
    ///
    /// # Parameters
    ///
    /// * `data` - The reference-counted byte data
    /// * `start` - The starting offset within the data
    /// * `length` - The length of the payload slice
    ///
    /// # Panics
    ///
    /// Panics in debug mode if `start + length > data.len()` (payload out of bounds)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::sync::Arc;
    /// use mqtt_protocol_core::mqtt::ArcPayload;
    ///
    /// let data = Arc::from(&b"hello world"[..]);
    /// let payload = ArcPayload::new(data, 0, 5); // "hello"
    /// ```
    pub fn new(data: Arc<[u8]>, start: usize, length: usize) -> Self {
        debug_assert!(start + length <= data.len(), "payload out of bounds",);
        Self {
            data,
            start,
            length,
        }
    }

    /// Get a slice view of the payload data
    ///
    /// Returns a byte slice representing the payload data within the specified range.
    ///
    /// # Returns
    ///
    /// A `&[u8]` slice of the payload data
    pub fn as_slice(&self) -> &[u8] {
        &self.data[self.start..self.start + self.length]
    }

    /// Get the length of the payload
    ///
    /// Returns the number of bytes in the payload slice.
    ///
    /// # Returns
    ///
    /// The length of the payload in bytes
    pub fn len(&self) -> usize {
        self.length
    }

    /// Check if the payload is empty
    ///
    /// Returns `true` if the payload contains no bytes.
    ///
    /// # Returns
    ///
    /// `true` if the payload length is zero, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Get a reference to the underlying `Arc<[u8]>` data
    ///
    /// Returns a reference to the reference-counted byte array that contains
    /// the actual data. This provides access to the full underlying data,
    /// not just the slice view represented by this payload.
    ///
    /// # Returns
    ///
    /// A reference to the underlying `Arc<[u8]>` data
    pub fn arc_data(&self) -> &Arc<[u8]> {
        &self.data
    }
}

impl Default for ArcPayload {
    fn default() -> Self {
        ArcPayload {
            data: Arc::from(&[] as &[u8]),
            start: 0,
            length: 0,
        }
    }
}

/// Trait for converting various types into `ArcPayload`
///
/// This trait provides a uniform interface for converting different data types
/// into `ArcPayload` instances. It allows for convenient creation of payloads
/// from common types like strings, byte slices, vectors, and arrays.
pub trait IntoPayload {
    /// Convert the value into an `ArcPayload`
    ///
    /// # Returns
    ///
    /// An `ArcPayload` containing the converted data
    fn into_payload(self) -> ArcPayload;
}

/// Convert a string slice (`&str`) into an `ArcPayload`
impl IntoPayload for &str {
    fn into_payload(self) -> ArcPayload {
        let bytes = self.as_bytes();
        ArcPayload::new(Arc::from(bytes), 0, bytes.len())
    }
}

/// Convert an owned string (`String`) into an `ArcPayload`
impl IntoPayload for String {
    fn into_payload(self) -> ArcPayload {
        let bytes = self.as_bytes();
        ArcPayload::new(Arc::from(bytes), 0, bytes.len())
    }
}

/// Convert a byte slice (`&[u8]`) into an `ArcPayload`
impl IntoPayload for &[u8] {
    fn into_payload(self) -> ArcPayload {
        ArcPayload::new(Arc::from(self), 0, self.len())
    }
}

/// Convert an owned byte vector (`Vec<u8>`) into an `ArcPayload`
impl IntoPayload for Vec<u8> {
    fn into_payload(self) -> ArcPayload {
        let len = self.len();
        ArcPayload::new(Arc::from(self), 0, len)
    }
}

/// Convert a reference to a byte vector (`&Vec<u8>`) into an `ArcPayload`
impl IntoPayload for &Vec<u8> {
    fn into_payload(self) -> ArcPayload {
        let slice: &[u8] = self.as_slice();
        ArcPayload::new(Arc::from(slice), 0, slice.len())
    }
}

/// Convert a reference to a byte array (`&[u8; N]`) into an `ArcPayload`
impl<const N: usize> IntoPayload for &[u8; N] {
    fn into_payload(self) -> ArcPayload {
        let slice: &[u8] = self.as_slice();
        ArcPayload::new(Arc::from(slice), 0, slice.len())
    }
}

/// Convert an `Arc<[u8]>` directly into an `ArcPayload`
impl IntoPayload for Arc<[u8]> {
    fn into_payload(self) -> ArcPayload {
        let len = self.len();
        ArcPayload::new(self, 0, len)
    }
}

/// Convert unit type (`()`) into an empty `ArcPayload`
impl IntoPayload for () {
    fn into_payload(self) -> ArcPayload {
        ArcPayload::default() // Empty payload
    }
}

/// Identity conversion for `ArcPayload` (no-op)
impl IntoPayload for ArcPayload {
    fn into_payload(self) -> ArcPayload {
        self
    }
}
