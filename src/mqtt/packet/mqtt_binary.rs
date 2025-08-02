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
use crate::mqtt::result_code::MqttError;
use serde::{Serialize, Serializer};
use std::convert::TryFrom;
use std::io::IoSlice;

/// MQTT Binary Data representation with pre-encoded byte buffer
///
/// This struct represents binary data as specified in the MQTT protocol specification.
/// It efficiently stores binary data with a 2-byte length prefix, following the MQTT
/// wire format for binary data fields.
///
/// The binary data is stored in a pre-encoded format internally, which includes:
/// - 2 bytes for the length prefix (big-endian u16)
/// - The actual binary data bytes
///
/// This approach provides several benefits:
/// - Zero-copy serialization to network buffers
/// - Efficient memory usage with single allocation
/// - Guaranteed MQTT protocol compliance
/// - Fast size calculations
///
/// # Size Limits
///
/// The maximum size of binary data is 65,535 bytes (2^16 - 1), as specified
/// by the MQTT protocol. Attempting to create an `MqttBinary` with larger
/// data will result in an error.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Create binary data from a byte slice
/// let data = b"hello world";
/// let mqtt_binary = mqtt::packet::MqttBinary::new(data).unwrap();
///
/// // Access the binary data
/// assert_eq!(mqtt_binary.as_slice(), b"hello world");
/// assert_eq!(mqtt_binary.len(), 11);
///
/// // Get the complete encoded buffer (length prefix + data)
/// let encoded = mqtt_binary.as_bytes();
/// assert_eq!(encoded.len(), 13); // 2 bytes length + 11 bytes data
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MqttBinary {
    /// Complete buffer containing length prefix (2 bytes) + binary data
    encoded: Vec<u8>,
}

impl MqttBinary {
    /// Create a new MqttBinary from binary data
    ///
    /// Creates an `MqttBinary` instance from the provided binary data.
    /// The data is copied into an internal buffer with a 2-byte length prefix.
    ///
    /// # Parameters
    ///
    /// * `data` - Binary data to store. Can be any type that implements `AsRef<[u8]>`
    ///   such as `&[u8]`, `Vec<u8>`, or `&str`
    ///
    /// # Returns
    ///
    /// * `Ok(MqttBinary)` - Successfully created binary data
    /// * `Err(MqttError::MalformedPacket)` - If data length exceeds 65,535 bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // From byte slice
    /// let binary = mqtt::packet::MqttBinary::new(b"hello").unwrap();
    ///
    /// // From Vec<u8>
    /// let vec_data = vec![1, 2, 3, 4, 5];
    /// let binary = mqtt::packet::MqttBinary::new(vec_data).unwrap();
    ///
    /// // From string (converted to bytes)
    /// let binary = mqtt::packet::MqttBinary::new("text data").unwrap();
    /// ```
    pub fn new(data: impl AsRef<[u8]>) -> Result<Self, MqttError> {
        let data_ref = data.as_ref();
        if data_ref.len() > 65535 {
            return Err(MqttError::MalformedPacket);
        }
        let len = data_ref.len() as u16;

        let mut encoded = Vec::with_capacity(2 + data_ref.len());
        encoded.push((len >> 8) as u8);
        encoded.push(len as u8);
        encoded.extend_from_slice(data_ref);

        Ok(Self { encoded })
    }

    /// Get the complete encoded byte sequence including length prefix
    ///
    /// Returns the complete internal buffer, which includes the 2-byte length prefix
    /// followed by the binary data. This is the exact format used in MQTT wire protocol.
    ///
    /// # Returns
    ///
    /// A byte slice containing [length_high, length_low, data...]
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let binary = mqtt::packet::MqttBinary::new(b"hi").unwrap();
    /// let bytes = binary.as_bytes();
    /// assert_eq!(bytes, &[0x00, 0x02, b'h', b'i']);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        &self.encoded
    }

    /// Get only the binary data without the length prefix
    ///
    /// Returns a byte slice containing only the binary data portion,
    /// excluding the 2-byte length prefix.
    ///
    /// # Returns
    ///
    /// A byte slice containing the raw binary data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let binary = mqtt::packet::MqttBinary::new(b"hello").unwrap();
    /// assert_eq!(binary.as_slice(), b"hello");
    /// ```
    pub fn as_slice(&self) -> &[u8] {
        &self.encoded[2..]
    }

    /// Get the length of the binary data in bytes
    ///
    /// Returns the number of bytes in the binary data portion only,
    /// excluding the 2-byte length prefix.
    ///
    /// # Returns
    ///
    /// The length of the binary data in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let binary = mqtt::packet::MqttBinary::new(b"hello").unwrap();
    /// assert_eq!(binary.len(), 5);
    ///
    /// let empty = mqtt::packet::MqttBinary::new(b"").unwrap();
    /// assert_eq!(empty.len(), 0);
    /// ```
    pub fn len(&self) -> usize {
        self.encoded.len() - 2
    }

    /// Check if the binary data is empty
    ///
    /// Returns `true` if the binary data contains no bytes,
    /// `false` otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the binary data is empty, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let empty = mqtt::packet::MqttBinary::new(b"").unwrap();
    /// assert!(empty.is_empty());
    ///
    /// let data = mqtt::packet::MqttBinary::new(b"x").unwrap();
    /// assert!(!data.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.encoded.len() <= 2
    }

    /// Get the total encoded size including the length field
    ///
    /// Returns the total number of bytes in the encoded representation,
    /// including the 2-byte length prefix and the binary data.
    ///
    /// # Returns
    ///
    /// The total size in bytes (length prefix + binary data)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let binary = mqtt::packet::MqttBinary::new(b"hello").unwrap();
    /// assert_eq!(binary.size(), 7); // 2 bytes prefix + 5 bytes data
    /// assert_eq!(binary.len(), 5);  // Only data length
    /// ```
    pub fn size(&self) -> usize {
        self.encoded.len()
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets.
    ///
    /// # Returns
    ///
    /// A vector containing a single `IoSlice` referencing the complete encoded buffer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use std::io::IoSlice;
    ///
    /// let binary = mqtt::packet::MqttBinary::new(b"data").unwrap();
    /// let buffers = binary.to_buffers();
    /// // Can be used with vectored write operations
    /// // socket.write_vectored(&buffers)?;
    /// ```
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        vec![IoSlice::new(&self.encoded)]
    }

    /// Parse binary data from a byte sequence
    ///
    /// Decodes MQTT binary data from a byte buffer according to the MQTT protocol.
    /// The buffer must start with a 2-byte length prefix followed by the binary data.
    ///
    /// # Parameters
    ///
    /// * `data` - Byte buffer containing the encoded binary data
    ///
    /// # Returns
    ///
    /// * `Ok((MqttBinary, bytes_consumed))` - Successfully parsed binary data and number of bytes consumed
    /// * `Err(MqttError::MalformedPacket)` - If the buffer is too short or malformed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Buffer: [length_high, length_low, data...]
    /// let buffer = &[0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];
    /// let (binary, consumed) = mqtt::packet::MqttBinary::decode(buffer).unwrap();
    ///
    /// assert_eq!(binary.as_slice(), b"hello");
    /// assert_eq!(consumed, 7);
    /// ```
    pub fn decode(data: &[u8]) -> Result<(Self, usize), MqttError> {
        if data.len() < 2 {
            return Err(MqttError::MalformedPacket);
        }

        let data_len = ((data[0] as usize) << 8) | (data[1] as usize);
        if data.len() < 2 + data_len {
            return Err(MqttError::MalformedPacket);
        }

        // Create encoded buffer
        let mut encoded = Vec::with_capacity(2 + data_len);
        encoded.extend_from_slice(&data[0..2 + data_len]);

        Ok((Self { encoded }, 2 + data_len))
    }
}

/// Implementation of `AsRef<[u8]>` for `MqttBinary`
///
/// Returns the binary data portion (without length prefix) when the
/// `MqttBinary` is used in contexts expecting a byte slice reference.
impl AsRef<[u8]> for MqttBinary {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Implementation of `Deref` for `MqttBinary`
///
/// Allows `MqttBinary` to be used directly as a byte slice in many contexts
/// through automatic dereferencing. The dereferenced value is the binary data
/// without the length prefix.
impl std::ops::Deref for MqttBinary {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// Implementation of `Serialize` for `MqttBinary`
///
/// Serializes the binary data portion (without length prefix) as bytes.
/// This is useful for JSON serialization and other serialization formats.
impl Serialize for MqttBinary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_slice())
    }
}

/// Implementation of `TryFrom<&str>` for `MqttBinary`
///
/// Converts a string slice to `MqttBinary` by converting the string to UTF-8 bytes.
/// This is useful when you need to store text data as binary in MQTT packets.
impl TryFrom<&str> for MqttBinary {
    type Error = MqttError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s.as_bytes())
    }
}

/// Implementation of `Default` for `MqttBinary`
///
/// Creates an empty `MqttBinary` with zero-length binary data.
/// The internal buffer contains only the 2-byte length prefix (0x00, 0x00).
impl Default for MqttBinary {
    fn default() -> Self {
        MqttBinary {
            encoded: vec![0x00, 0x00],
        }
    }
}
