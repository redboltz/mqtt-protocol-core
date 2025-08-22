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

// SSO buffer size configuration - priority-based selection for maximum size
#[cfg(feature = "sso-lv20")]
const SSO_BUFFER_SIZE: usize = 255; // Highest priority: 255 bytes
#[cfg(all(not(feature = "sso-lv20"), feature = "sso-lv10"))]
const SSO_BUFFER_SIZE: usize = 127; // Second priority: 127 bytes
#[cfg(all(
    not(any(feature = "sso-lv20", feature = "sso-lv10")),
    feature = "sso-min-64bit"
))]
const SSO_BUFFER_SIZE: usize = 31; // Third priority: 31 bytes
#[cfg(all(
    not(any(feature = "sso-lv20", feature = "sso-lv10", feature = "sso-min-64bit")),
    feature = "sso-min-32bit"
))]
const SSO_BUFFER_SIZE: usize = 15; // Fourth priority: 15 bytes
#[cfg(not(any(
    feature = "sso-min-32bit",
    feature = "sso-min-64bit",
    feature = "sso-lv10",
    feature = "sso-lv20"
)))]
#[allow(dead_code)]
const SSO_BUFFER_SIZE: usize = 0; // No SSO features enabled

// Length type is always u8 since all buffer sizes fit in u8 range (max 255)
#[cfg(any(
    feature = "sso-min-32bit",
    feature = "sso-min-64bit",
    feature = "sso-lv10",
    feature = "sso-lv20"
))]
type LengthType = u8;

#[cfg(not(any(
    feature = "sso-min-32bit",
    feature = "sso-min-64bit",
    feature = "sso-lv10",
    feature = "sso-lv20"
)))]
#[allow(dead_code)]
type LengthType = u8;

/// A reference-counted byte payload with slice semantics
///
/// `ArcPayload` provides an efficient way to handle byte data by using `Arc<[u8]>`
/// for reference counting, combined with offset and length information to represent
/// a slice view of the underlying data. This allows for zero-copy sharing of payload
/// data across multiple consumers while maintaining slice-like semantics.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ArcPayload {
    #[cfg(any(
        feature = "sso-min-32bit",
        feature = "sso-min-64bit",
        feature = "sso-lv10",
        feature = "sso-lv20"
    ))]
    Small([u8; SSO_BUFFER_SIZE], LengthType), // buffer, actual_length
    Large {
        data: Arc<[u8]>,
        start: usize,
        length: usize,
    },
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
    /// use alloc::sync::Arc;
    /// use mqtt_protocol_core::mqtt::ArcPayload;
    ///
    /// let data = Arc::from(&b"hello world"[..]);
    /// let payload = ArcPayload::new(data, 0, 5); // "hello"
    /// ```
    pub fn new(data: Arc<[u8]>, start: usize, length: usize) -> Self {
        debug_assert!(start + length <= data.len(), "payload out of bounds",);

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if length <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..length].copy_from_slice(&data[start..start + length]);
            return Self::Small(buffer, length as LengthType);
        }

        Self::Large {
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
        match self {
            ArcPayload::Large {
                data,
                start,
                length,
            } => &data[*start..*start + *length],
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            ArcPayload::Small(buffer, length) => &buffer[..*length as usize],
        }
    }

    /// Get the length of the payload
    ///
    /// Returns the number of bytes in the payload slice.
    ///
    /// # Returns
    ///
    /// The length of the payload in bytes
    pub fn len(&self) -> usize {
        match self {
            ArcPayload::Large { length, .. } => *length,
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            ArcPayload::Small(_, length) => *length as usize,
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
    /// Returns a reference to the reference-counted byte array that contains
    /// the actual data. This provides access to the full underlying data,
    /// not just the slice view represented by this payload.
    ///
    /// # Returns
    ///
    /// A reference to the underlying `Arc<[u8]>` data
    pub fn arc_data(&self) -> Option<&Arc<[u8]>> {
        match self {
            ArcPayload::Large { data, .. } => Some(data),
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            ArcPayload::Small(_, _) => None, // Small variant doesn't use Arc data
        }
    }
}

impl Default for ArcPayload {
    fn default() -> Self {
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        return ArcPayload::Small([0u8; SSO_BUFFER_SIZE], 0 as LengthType);

        #[cfg(not(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        )))]
        return ArcPayload::Large {
            data: Arc::from(&[] as &[u8]),
            start: 0,
            length: 0,
        };
    }
}

impl core::fmt::Debug for ArcPayload {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ArcPayload")
            .field("data", &self.as_slice())
            .field("len", &self.len())
            .finish()
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

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if bytes.len() <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..bytes.len()].copy_from_slice(bytes);
            return ArcPayload::Small(buffer, bytes.len() as LengthType);
        }

        ArcPayload::Large {
            data: Arc::from(bytes),
            start: 0,
            length: bytes.len(),
        }
    }
}

/// Convert an owned string (`String`) into an `ArcPayload`
impl IntoPayload for String {
    fn into_payload(self) -> ArcPayload {
        let bytes = self.as_bytes();

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if bytes.len() <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..bytes.len()].copy_from_slice(bytes);
            return ArcPayload::Small(buffer, bytes.len() as LengthType);
        }

        ArcPayload::Large {
            data: Arc::from(bytes),
            start: 0,
            length: bytes.len(),
        }
    }
}

/// Convert a byte slice (`&[u8]`) into an `ArcPayload`
impl IntoPayload for &[u8] {
    fn into_payload(self) -> ArcPayload {
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if self.len() <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..self.len()].copy_from_slice(self);
            return ArcPayload::Small(buffer, self.len() as LengthType);
        }

        ArcPayload::Large {
            data: Arc::from(self),
            start: 0,
            length: self.len(),
        }
    }
}

/// Convert an owned byte vector (`Vec<u8>`) into an `ArcPayload`
impl IntoPayload for Vec<u8> {
    fn into_payload(self) -> ArcPayload {
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if self.len() <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..self.len()].copy_from_slice(&self);
            return ArcPayload::Small(buffer, self.len() as LengthType);
        }

        let len = self.len();
        ArcPayload::Large {
            data: Arc::from(self),
            start: 0,
            length: len,
        }
    }
}

/// Convert a reference to a byte vector (`&Vec<u8>`) into an `ArcPayload`
impl IntoPayload for &Vec<u8> {
    fn into_payload(self) -> ArcPayload {
        let slice: &[u8] = self.as_slice();

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if slice.len() <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..slice.len()].copy_from_slice(slice);
            return ArcPayload::Small(buffer, slice.len() as LengthType);
        }

        ArcPayload::Large {
            data: Arc::from(slice),
            start: 0,
            length: slice.len(),
        }
    }
}

/// Convert a reference to a byte array (`&[u8; N]`) into an `ArcPayload`
impl<const N: usize> IntoPayload for &[u8; N] {
    fn into_payload(self) -> ArcPayload {
        let slice: &[u8] = self.as_slice();

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if slice.len() <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..slice.len()].copy_from_slice(slice);
            return ArcPayload::Small(buffer, slice.len() as LengthType);
        }

        ArcPayload::Large {
            data: Arc::from(slice),
            start: 0,
            length: slice.len(),
        }
    }
}

/// Convert an `Arc<[u8]>` directly into an `ArcPayload`
impl IntoPayload for Arc<[u8]> {
    fn into_payload(self) -> ArcPayload {
        let len = self.len();

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if len <= SSO_BUFFER_SIZE {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[..len].copy_from_slice(&self);
            return ArcPayload::Small(buffer, len as LengthType);
        }

        ArcPayload::Large {
            data: self,
            start: 0,
            length: len,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_payload() {
        let payload = ArcPayload::default();
        assert_eq!(payload.len(), 0);
        assert!(payload.is_empty());
        assert_eq!(payload.as_slice(), &[] as &[u8]);
    }

    #[test]
    fn test_small_payload() {
        let data = b"hello";
        let payload = data.into_payload();
        assert_eq!(payload.len(), 5);
        assert!(!payload.is_empty());
        assert_eq!(payload.as_slice(), b"hello");
    }

    #[test]
    fn test_payload_variants() {
        // Test small data (Small variant if any SSO feature is enabled)
        let small_data = b"small";
        let payload = small_data.into_payload();

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        assert!(matches!(payload, ArcPayload::Small(_, _)));

        #[cfg(not(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        )))]
        assert!(matches!(payload, ArcPayload::Large { .. }));

        // Test data that should be Large if using smaller SSO buffers, but Small with sso-lv20
        let medium_data = vec![0u8; 200]; // 200 bytes - larger than most SSO buffers but smaller than sso-lv20
        let _payload = medium_data.into_payload();

        // With sso-lv20 (255 bytes), this should be Small
        #[cfg(feature = "sso-lv20")]
        assert!(matches!(_payload, ArcPayload::Small(_, _)));

        // Without sso-lv20, this should be Large (exceeds other SSO buffer sizes)
        #[cfg(all(
            any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10"
            ),
            not(feature = "sso-lv20")
        ))]
        assert!(matches!(_payload, ArcPayload::Large { .. }));

        // Test data that should always be Large variant (larger than largest SSO buffer)
        let very_large_data = b"This is a very long payload that exceeds even the largest SSO buffer size of 255 bytes. It should definitely be stored in the Large variant regardless of which SSO feature flags are enabled. This ensures consistent behavior across all configurations and provides a reliable test case.";
        let payload = very_large_data.into_payload();
        assert!(matches!(payload, ArcPayload::Large { .. }));
    }

    #[test]
    fn test_arc_data_access() {
        let small_data = b"test";
        let small_payload = small_data.into_payload();

        // Use data larger than largest SSO buffer to ensure Large variant
        let very_large_data = b"This is a very long payload that exceeds even the largest SSO buffer size of 255 bytes. It should definitely be stored in the Large variant regardless of which SSO feature flags are enabled. This ensures consistent behavior across all configurations and provides a reliable test case for arc_data access.";
        let large_payload = very_large_data.into_payload();

        // Small variant should return None for arc_data when SSO is enabled
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if let ArcPayload::Small(_, _) = small_payload {
            assert!(small_payload.arc_data().is_none());
        }

        // Without SSO, small data also uses Large variant
        #[cfg(not(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        )))]
        assert!(small_payload.arc_data().is_some());

        // Large variant should always return Some for arc_data
        assert!(large_payload.arc_data().is_some());
    }

    #[test]
    fn test_into_payload_implementations() {
        // Test various types
        let str_payload = "hello".into_payload();
        assert_eq!(str_payload.as_slice(), b"hello");

        let string_payload = String::from("world").into_payload();
        assert_eq!(string_payload.as_slice(), b"world");

        let vec_payload = vec![1, 2, 3, 4].into_payload();
        assert_eq!(vec_payload.as_slice(), &[1, 2, 3, 4]);

        let arr_payload = (&[5, 6, 7, 8]).into_payload();
        assert_eq!(arr_payload.as_slice(), &[5, 6, 7, 8]);

        let unit_payload = ().into_payload();
        assert!(unit_payload.is_empty());
    }
}
