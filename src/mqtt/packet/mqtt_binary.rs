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
use crate::mqtt::result_code::MqttError;
use alloc::vec::Vec;
use core::convert::TryFrom;
use serde::{Serialize, Serializer};
#[cfg(feature = "std")]
use std::io::IoSlice;

// SSO buffer size configuration - priority-based selection for maximum size
#[cfg(feature = "sso-lv20")]
const SSO_BUFFER_SIZE: usize = 48; // Highest priority: 48 bytes
#[cfg(all(
    not(feature = "sso-lv20"),
    any(feature = "sso-lv10", feature = "sso-min-64bit")
))]
const SSO_BUFFER_SIZE: usize = 24; // Second priority: 24 bytes
#[cfg(all(
    not(any(feature = "sso-lv20", feature = "sso-lv10", feature = "sso-min-64bit")),
    feature = "sso-min-32bit"
))]
const SSO_BUFFER_SIZE: usize = 12; // Third priority: 12 bytes
#[cfg(not(any(
    feature = "sso-min-32bit",
    feature = "sso-min-64bit",
    feature = "sso-lv10",
    feature = "sso-lv20"
)))]
#[allow(dead_code)]
const SSO_BUFFER_SIZE: usize = 0; // No SSO features enabled

// Determine data threshold
#[cfg(any(
    feature = "sso-min-32bit",
    feature = "sso-min-64bit",
    feature = "sso-lv10",
    feature = "sso-lv20"
))]
const SSO_DATA_THRESHOLD: usize = SSO_BUFFER_SIZE - 2;

/// MQTT Binary Data representation with Small String Optimization (SSO)
///
/// This enum represents binary data as specified in the MQTT protocol specification.
/// It uses SSO to store small binary data on the stack and larger data on the heap,
/// following the MQTT wire format for binary data fields.
///
/// The binary data is stored with a 2-byte length prefix, which includes:
/// - 2 bytes for the length prefix (big-endian u16)
/// - The actual binary data bytes
///
/// This approach provides several benefits:
/// - Zero-copy serialization to network buffers
/// - Efficient memory usage (stack for small data, heap for large)
/// - Guaranteed MQTT protocol compliance
/// - Fast size calculations
///
/// # Size Limits
///
/// The maximum size of binary data is 65,535 bytes (2^16 - 1), as specified
/// by the MQTT protocol. The SSO threshold is determined by feature flags.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::large_enum_variant)]
pub enum MqttBinary {
    #[cfg(any(
        feature = "sso-min-32bit",
        feature = "sso-min-64bit",
        feature = "sso-lv10",
        feature = "sso-lv20"
    ))]
    Small([u8; SSO_BUFFER_SIZE]), // buffer with length prefix
    Large(Vec<u8>),
}

impl MqttBinary {
    /// Create a new MqttBinary from binary data
    pub fn new(data: impl AsRef<[u8]>) -> Result<Self, MqttError> {
        let data_ref = data.as_ref();
        if data_ref.len() > 65535 {
            return Err(MqttError::MalformedPacket);
        }

        // Try to fit in Small variant if SSO is enabled
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if data_ref.len() <= SSO_DATA_THRESHOLD {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            let len = data_ref.len() as u16;
            buffer[0] = (len >> 8) as u8;
            buffer[1] = len as u8;
            buffer[2..2 + data_ref.len()].copy_from_slice(data_ref);
            return Ok(MqttBinary::Small(buffer));
        }

        // Fallback to Large variant
        let len = data_ref.len() as u16;
        let mut encoded = Vec::with_capacity(2 + data_ref.len());
        encoded.push((len >> 8) as u8);
        encoded.push(len as u8);
        encoded.extend_from_slice(data_ref);
        Ok(MqttBinary::Large(encoded))
    }

    /// Get the complete encoded byte sequence including length prefix
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttBinary::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                &buffer[..2 + len]
            }
            MqttBinary::Large(encoded) => encoded,
        }
    }

    /// Get only the binary data without the length prefix
    pub fn as_slice(&self) -> &[u8] {
        match self {
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttBinary::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                &buffer[2..2 + len]
            }
            MqttBinary::Large(encoded) => &encoded[2..],
        }
    }

    /// Get the length of the binary data in bytes
    pub fn len(&self) -> usize {
        match self {
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttBinary::Small(buffer) => ((buffer[0] as usize) << 8) | (buffer[1] as usize),
            MqttBinary::Large(encoded) => encoded.len() - 2,
        }
    }

    /// Check if the binary data is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total encoded size including the length field
    pub fn size(&self) -> usize {
        match self {
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttBinary::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                2 + len
            }
            MqttBinary::Large(encoded) => encoded.len(),
        }
    }

    /// Create IoSlice buffers for efficient network I/O
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        match self {
            MqttBinary::Large(encoded) => vec![IoSlice::new(encoded)],
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttBinary::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                vec![IoSlice::new(&buffer[..2 + len])]
            }
        }
    }

    /// Create a continuous buffer containing the complete packet data
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    /// Parse binary data from a byte sequence
    pub fn decode(data: &[u8]) -> Result<(Self, usize), MqttError> {
        if data.len() < 2 {
            return Err(MqttError::MalformedPacket);
        }

        let data_len = ((data[0] as usize) << 8) | (data[1] as usize);
        if data.len() < 2 + data_len {
            return Err(MqttError::MalformedPacket);
        }

        // Try to fit in Small variant if SSO is enabled
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if data_len <= SSO_DATA_THRESHOLD {
            let payload = &data[2..2 + data_len];
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[0] = data[0];
            buffer[1] = data[1];
            buffer[2..2 + data_len].copy_from_slice(payload);
            return Ok((MqttBinary::Small(buffer), 2 + data_len));
        }

        // Fallback to Large variant
        let mut encoded = Vec::with_capacity(2 + data_len);
        encoded.extend_from_slice(&data[0..2 + data_len]);
        Ok((MqttBinary::Large(encoded), 2 + data_len))
    }
}

/// Implementation of `AsRef<[u8]>` for `MqttBinary`
impl AsRef<[u8]> for MqttBinary {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Implementation of `Deref` for `MqttBinary`
impl core::ops::Deref for MqttBinary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// Implementation of `Serialize` for `MqttBinary`
impl Serialize for MqttBinary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_slice())
    }
}

/// Implementation of `TryFrom<&str>` for `MqttBinary`
impl TryFrom<&str> for MqttBinary {
    type Error = MqttError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s.as_bytes())
    }
}

/// Implementation of `Default` for `MqttBinary`
impl Default for MqttBinary {
    fn default() -> Self {
        Self::new(b"").unwrap()
    }
}

impl core::fmt::Debug for MqttBinary {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MqttBinary")
            .field("data", &self.as_slice())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_binary() {
        let binary = MqttBinary::new(b"").unwrap();
        assert_eq!(binary.len(), 0);
        assert!(binary.is_empty());
        assert_eq!(binary.size(), 2);
        assert_eq!(binary.as_slice(), b"");
        assert_eq!(binary.as_bytes(), vec![0x00, 0x00]);
    }

    #[test]
    fn test_small_binary() {
        let data = b"hello";
        let binary = MqttBinary::new(data).unwrap();
        assert_eq!(binary.len(), 5);
        assert!(!binary.is_empty());
        assert_eq!(binary.size(), 7);
        assert_eq!(binary.as_slice(), data);
        assert_eq!(
            binary.as_bytes(),
            vec![0x00, 0x05, b'h', b'e', b'l', b'l', b'o']
        );
    }

    #[test]
    fn test_decode_roundtrip() {
        let original_data = b"test data";
        let binary = MqttBinary::new(original_data).unwrap();
        let encoded = binary.as_bytes();

        let (decoded_binary, consumed) = MqttBinary::decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded_binary.as_slice(), original_data);
        assert_eq!(decoded_binary.len(), original_data.len());
    }

    #[test]
    fn test_max_size() {
        let data = vec![0u8; 65535];
        let binary = MqttBinary::new(&data).unwrap();
        assert_eq!(binary.len(), 65535);
        assert_eq!(binary.size(), 65537);
    }

    #[test]
    fn test_oversized_data() {
        let data = vec![0u8; 65536];
        let result = MqttBinary::new(&data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MqttError::MalformedPacket);
    }

    #[test]
    fn test_decode_malformed() {
        // Too short buffer
        assert!(MqttBinary::decode(&[0x00]).is_err());

        // Length mismatch
        assert!(MqttBinary::decode(&[0x00, 0x05, b'h', b'i']).is_err());
    }

    #[test]
    fn test_continuous_buffer() {
        let data = b"continuous";
        let binary = MqttBinary::new(data).unwrap();
        let buffer = binary.to_continuous_buffer();
        assert_eq!(buffer[0], 0x00);
        assert_eq!(buffer[1], 0x0A); // length = 10
        assert_eq!(&buffer[2..], data);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_to_buffers() {
        let data = b"buffer test";
        let binary = MqttBinary::new(data).unwrap();
        let buffers = binary.to_buffers();

        // Both variants should return 1 buffer containing the encoded data
        assert_eq!(buffers.len(), 1);

        // Verify the buffer contains the complete encoded data
        let buffer_data: &[u8] = &buffers[0];
        assert_eq!(buffer_data, binary.as_bytes());
    }

    #[test]
    fn test_trait_implementations() {
        let binary = MqttBinary::new(b"trait test").unwrap();

        // AsRef<[u8]>
        let slice: &[u8] = binary.as_ref();
        assert_eq!(slice, b"trait test");

        // Deref
        assert_eq!(&*binary, b"trait test");

        // PartialEq
        let binary2 = MqttBinary::new(b"trait test").unwrap();
        assert_eq!(binary, binary2);

        // Clone
        let cloned = binary.clone();
        assert_eq!(binary, cloned);

        // Default
        let default = MqttBinary::default();
        assert!(default.is_empty());
    }

    #[test]
    fn test_from_conversions() {
        // TryFrom<&str>
        let binary = MqttBinary::try_from("string test").unwrap();
        assert_eq!(binary.as_slice(), b"string test");

        // TryFrom with oversized data should fail
        let long_str = "x".repeat(65536);
        assert!(MqttBinary::try_from(long_str.as_str()).is_err());
    }

    // Feature-specific tests
    #[cfg(any(
        feature = "sso-min-32bit",
        feature = "sso-min-64bit",
        feature = "sso-lv10",
        feature = "sso-lv20"
    ))]
    #[test]
    fn test_sso_features() {
        // Test small data (Small variant if SSO is enabled)
        let small_data = b"small";
        let binary = MqttBinary::new(small_data).unwrap();

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        {
            if let MqttBinary::Small(buffer) = binary {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                assert_eq!(len, 5); // 5 bytes data
            } else {
                panic!("Expected Small variant for small data with SSO enabled");
            }
        }

        #[cfg(not(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        )))]
        assert!(matches!(binary, MqttBinary::Large(_)));

        // Test data that should always be Large variant (larger than largest SSO buffer)
        let very_large_data = b"This is a very long binary data that exceeds even the largest SSO buffer size to ensure it's always stored in the Large variant for consistent testing";
        let binary = MqttBinary::new(very_large_data).unwrap();
        assert!(matches!(binary, MqttBinary::Large(_)));
    }
}
