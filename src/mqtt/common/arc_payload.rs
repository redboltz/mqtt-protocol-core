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

use alloc::{string::String, sync::Arc, vec::Vec};
use serde::{Serialize, Serializer};

// Default stack buffer size for small payload optimization
const DEFAULT_STACK_BUFFER_SIZE: usize = 32;

/// A reference-counted byte payload with slice semantics and Small Buffer Optimization (SBO)
///
/// `GenericArcPayload` provides an efficient way to handle byte data with automatic optimization
/// for small payloads. Small payloads (up to `STACK_BUFFER_SIZE` bytes) are stored on the
/// stack to avoid heap allocation, while larger payloads use `Arc<[u8]>` for reference counting.
/// This provides optimal performance for both small and large payloads.
///
/// # Small Buffer Optimization
///
/// - Payloads with size ≤ STACK_BUFFER_SIZE bytes are stored on the stack
/// - Larger payloads are stored on the heap using Arc<[u8]> for reference counting
/// - This provides zero heap allocation for typical small MQTT payloads
///
/// # Type Parameters
///
/// * `STACK_BUFFER_SIZE` - Size of the stack buffer in bytes (default: 32)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Default stack buffer size (32 bytes)
/// let small_payload = mqtt::ArcPayload::from(&b"hello"[..]);
///
/// // Custom stack buffer size (64 bytes)
/// let custom_payload = mqtt::GenericArcPayload::<64>::from(&b"hello"[..]);
/// ```
#[derive(Clone, Debug)]
pub enum GenericArcPayload<const STACK_BUFFER_SIZE: usize = DEFAULT_STACK_BUFFER_SIZE> {
    /// Small payload stored on the stack (size ≤ STACK_BUFFER_SIZE)
    Small([u8; STACK_BUFFER_SIZE], usize), // buffer, actual_length
    /// Large payload stored with Arc for reference counting
    Large(Arc<[u8]>, usize, usize), // data, start, length
}

impl<const STACK_BUFFER_SIZE: usize> PartialEq for GenericArcPayload<STACK_BUFFER_SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<const STACK_BUFFER_SIZE: usize> Eq for GenericArcPayload<STACK_BUFFER_SIZE> {}

impl<const STACK_BUFFER_SIZE: usize> Serialize for GenericArcPayload<STACK_BUFFER_SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

impl<const STACK_BUFFER_SIZE: usize> GenericArcPayload<STACK_BUFFER_SIZE> {
    /// Create a new payload from byte data
    ///
    /// Creates a payload from the provided byte data. Small data (≤ STACK_BUFFER_SIZE bytes)
    /// is stored on the stack, while larger data uses heap allocation.
    ///
    /// # Parameters
    ///
    /// * `data` - The byte data to store
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt::ArcPayload;
    ///
    /// let payload = ArcPayload::from(&b"hello world"[..]);
    /// ```
    pub fn from(data: &[u8]) -> Self {
        if data.len() <= STACK_BUFFER_SIZE {
            let mut buffer = [0u8; STACK_BUFFER_SIZE];
            buffer[..data.len()].copy_from_slice(data);
            Self::Small(buffer, data.len())
        } else {
            Self::Large(Arc::from(data), 0, data.len())
        }
    }

    /// Create a new payload from reference-counted data with specified range
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
    pub fn new(data: Arc<[u8]>, start: usize, length: usize) -> Self {
        debug_assert!(start + length <= data.len(), "payload out of bounds");

        let slice_len = length;
        if slice_len <= STACK_BUFFER_SIZE {
            let mut buffer = [0u8; STACK_BUFFER_SIZE];
            buffer[..slice_len].copy_from_slice(&data[start..start + length]);
            Self::Small(buffer, slice_len)
        } else {
            Self::Large(data, start, length)
        }
    }

    /// Get a slice view of the payload data
    ///
    /// Returns a byte slice representing the payload data.
    ///
    /// # Returns
    ///
    /// A `&[u8]` slice of the payload data
    pub fn as_slice(&self) -> &[u8] {
        match self {
            Self::Small(buffer, len) => &buffer[..*len],
            Self::Large(data, start, length) => &data[*start..*start + *length],
        }
    }

    /// Get the length of the payload
    ///
    /// Returns the number of bytes in the payload.
    ///
    /// # Returns
    ///
    /// The length of the payload in bytes
    pub fn len(&self) -> usize {
        match self {
            Self::Small(_, len) => *len,
            Self::Large(_, _, length) => *length,
        }
    }

    /// Check if the payload is empty
    ///
    /// Returns `true` if the payload contains no bytes.
    ///
    /// # Returns
    ///
    /// `true` if the payload length is zero, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get a reference to the underlying `Arc<[u8]>` data
    ///
    /// Returns a reference to the reference-counted byte array for large payloads,
    /// or None for small payloads stored on the stack.
    ///
    /// # Returns
    ///
    /// An option containing a reference to the underlying `Arc<[u8]>` data
    pub fn arc_data(&self) -> Option<&Arc<[u8]>> {
        match self {
            Self::Small(_, _) => None,
            Self::Large(data, _, _) => Some(data),
        }
    }
}

impl<const STACK_BUFFER_SIZE: usize> Default for GenericArcPayload<STACK_BUFFER_SIZE> {
    fn default() -> Self {
        Self::Small([0u8; STACK_BUFFER_SIZE], 0)
    }
}

/// Trait for converting various types into payload
pub trait IntoPayload<const STACK_BUFFER_SIZE: usize = DEFAULT_STACK_BUFFER_SIZE> {
    /// Convert the value into a payload
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE>;
}

/// Convert a string slice (`&str`) into a payload
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for &str {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        GenericArcPayload::from(self.as_bytes())
    }
}

/// Convert an owned string (`String`) into a payload
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for String {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        GenericArcPayload::from(self.as_bytes())
    }
}

/// Convert a byte slice (`&[u8]`) into a payload
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for &[u8] {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        GenericArcPayload::from(self)
    }
}

/// Convert an owned byte vector (`Vec<u8>`) into a payload
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for Vec<u8> {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        GenericArcPayload::from(&self)
    }
}

/// Convert a reference to a byte vector (`&Vec<u8>`) into a payload
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for &Vec<u8> {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        GenericArcPayload::from(self.as_slice())
    }
}

/// Convert a reference to a byte array (`&[u8; N]`) into a payload
impl<const N: usize, const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for &[u8; N] {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        GenericArcPayload::from(self.as_slice())
    }
}

/// Convert an `Arc<[u8]>` directly into a payload
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for Arc<[u8]> {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        let len = self.len();
        GenericArcPayload::new(self, 0, len)
    }
}

/// Convert unit type (`()`) into an empty payload
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE> for () {
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        GenericArcPayload::default()
    }
}

/// Identity conversion for payload (no-op)
impl<const STACK_BUFFER_SIZE: usize> IntoPayload<STACK_BUFFER_SIZE>
    for GenericArcPayload<STACK_BUFFER_SIZE>
{
    fn into_payload(self) -> GenericArcPayload<STACK_BUFFER_SIZE> {
        self
    }
}
