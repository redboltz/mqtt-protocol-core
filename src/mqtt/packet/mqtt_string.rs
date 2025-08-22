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
use alloc::string::String;
use alloc::vec::Vec;
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
const SSO_BUFFER_SIZE: usize = 0; // No SSO features enabled

// Determine data threshold
#[cfg(any(
    feature = "sso-min-32bit",
    feature = "sso-min-64bit",
    feature = "sso-lv10",
    feature = "sso-lv20"
))]
const SSO_DATA_THRESHOLD: usize = SSO_BUFFER_SIZE - 2;

/// MQTT String representation with pre-encoded byte buffer
///
/// This struct represents UTF-8 strings as specified in the MQTT protocol specification.
/// It efficiently stores string data with a 2-byte length prefix, following the MQTT
/// wire format for string fields.
///
/// The string data is stored in a pre-encoded format internally, which includes:
/// - 2 bytes for the length prefix (big-endian u16)
/// - The UTF-8 encoded string bytes
///
/// This approach provides several benefits:
/// - Zero-copy serialization to network buffers
/// - Efficient memory usage with single allocation
/// - Guaranteed MQTT protocol compliance
/// - Fast size calculations
/// - UTF-8 validation at construction time
///
/// # Size Limits
///
/// The maximum size of string data is 65,535 bytes (2^16 - 1), as specified
/// by the MQTT protocol. Attempting to create an `MqttString` with larger
/// data will result in an error.
///
/// # UTF-8 Validation
///
/// All string data is validated to be valid UTF-8 at construction time.
/// Once created, the string is guaranteed to be valid UTF-8, allowing
/// for safe conversion to `&str` without runtime checks.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Create from string literal
/// let mqtt_str = mqtt::packet::MqttString::new("hello world").unwrap();
///
/// // Access the string content
/// assert_eq!(mqtt_str.as_str(), "hello world");
/// assert_eq!(mqtt_str.len(), 11);
///
/// // Get the complete encoded buffer (length prefix + UTF-8 bytes)
/// let encoded = mqtt_str.as_bytes();
/// assert_eq!(encoded.len(), 13); // 2 bytes length + 11 bytes data
///
/// // String operations
/// assert!(mqtt_str.contains('w'));
/// assert!(mqtt_str.starts_with("hello"));
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::large_enum_variant)]
pub enum MqttString {
    #[cfg(any(
        feature = "sso-min-32bit",
        feature = "sso-min-64bit",
        feature = "sso-lv10",
        feature = "sso-lv20"
    ))]
    Small([u8; SSO_BUFFER_SIZE]),
    Large(Vec<u8>),
}

impl MqttString {
    /// Create a new MqttString from a string
    ///
    /// Creates an `MqttString` instance from the provided string data.
    /// The string is converted to UTF-8 bytes and stored with a 2-byte length prefix.
    ///
    /// # Parameters
    ///
    /// * `s` - String data to store. Can be any type that implements `AsRef<str>`
    ///   such as `&str`, `String`, or `Cow<str>`
    ///
    /// # Returns
    ///
    /// * `Ok(MqttString)` - Successfully created string
    /// * `Err(MqttError::MalformedPacket)` - If string length exceeds 65,535 bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // From string literal
    /// let mqtt_str = mqtt::packet::MqttString::new("hello").unwrap();
    ///
    /// // From String
    /// let owned = String::from("world");
    /// let mqtt_str = mqtt::packet::MqttString::new(owned).unwrap();
    ///
    /// // UTF-8 strings are supported
    /// let mqtt_str = mqtt::packet::MqttString::new("hello").unwrap();
    /// ```
    pub fn new(s: impl AsRef<str>) -> Result<Self, MqttError> {
        let s_ref = s.as_ref();
        let len = s_ref.len();

        if len > 65535 {
            return Err(MqttError::MalformedPacket);
        }

        let total_encoded_len = 2 + len;

        // Try to fit in Small variant if SSO is enabled
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if len <= SSO_DATA_THRESHOLD {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[0] = (len >> 8) as u8;
            buffer[1] = len as u8;
            buffer[2..2 + len].copy_from_slice(s_ref.as_bytes());
            return Ok(Self::Small(buffer));
        }

        // Fallback to Large variant
        let mut encoded = Vec::with_capacity(total_encoded_len);
        encoded.push((len >> 8) as u8);
        encoded.push(len as u8);
        encoded.extend_from_slice(s_ref.as_bytes());

        Ok(Self::Large(encoded))
    }

    /// Get the complete encoded byte sequence including length prefix
    ///
    /// Returns the complete internal buffer, which includes the 2-byte length prefix
    /// followed by the UTF-8 encoded string bytes. This is the exact format used
    /// in MQTT wire protocol.
    ///
    /// # Returns
    ///
    /// A byte slice containing [length_high, length_low, utf8_bytes...]
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mqtt_str = mqtt::packet::MqttString::new("hi").unwrap();
    /// let bytes = mqtt_str.as_bytes();
    /// assert_eq!(bytes, &[0x00, 0x02, b'h', b'i']);
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            MqttString::Large(encoded) => encoded,
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttString::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                &buffer[..2 + len]
            }
        }
    }

    /// Get the string content as a string slice
    ///
    /// Returns a string slice containing the UTF-8 string data,
    /// excluding the 2-byte length prefix. This operation is zero-cost
    /// since UTF-8 validity was verified at construction time.
    ///
    /// # Returns
    ///
    /// A string slice containing the UTF-8 string data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mqtt_str = mqtt::packet::MqttString::new("hello world").unwrap();
    /// assert_eq!(mqtt_str.as_str(), "hello world");
    /// ```
    pub fn as_str(&self) -> &str {
        // SAFETY: UTF-8 validity verified during MqttString creation or decode
        // Also, no direct modification of encoded field is provided,
        // ensuring buffer immutability
        let data = match self {
            MqttString::Large(encoded) => &encoded[2..],
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttString::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                &buffer[2..2 + len]
            }
        };
        unsafe { core::str::from_utf8_unchecked(data) }
    }

    /// Get the length of the string data in bytes
    ///
    /// Returns the number of bytes in the UTF-8 string data,
    /// excluding the 2-byte length prefix. Note that this is
    /// the byte length, not the character count.
    ///
    /// # Returns
    ///
    /// The length of the string data in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let ascii = mqtt::packet::MqttString::new("hello").unwrap();
    /// assert_eq!(ascii.len(), 5);
    ///
    /// // UTF-8 strings: byte length != character count
    /// let utf8 = mqtt::packet::MqttString::new("hello").unwrap();
    /// assert_eq!(utf8.len(), 5); // 5 characters
    /// ```
    pub fn len(&self) -> usize {
        match self {
            MqttString::Large(encoded) => encoded.len() - 2,
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttString::Small(buffer) => ((buffer[0] as usize) << 8) | (buffer[1] as usize),
        }
    }

    /// Check if the string is empty
    ///
    /// Returns `true` if the string contains no characters,
    /// `false` otherwise.
    ///
    /// # Returns
    ///
    /// `true` if the string is empty, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let empty = mqtt::packet::MqttString::new("").unwrap();
    /// assert!(empty.is_empty());
    ///
    /// let text = mqtt::packet::MqttString::new("x").unwrap();
    /// assert!(!text.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the total encoded size including the length field
    ///
    /// Returns the total number of bytes in the encoded representation,
    /// including the 2-byte length prefix and the UTF-8 string data.
    ///
    /// # Returns
    ///
    /// The total size in bytes (length prefix + string data)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mqtt_str = mqtt::packet::MqttString::new("hello").unwrap();
    /// assert_eq!(mqtt_str.size(), 7); // 2 bytes prefix + 5 bytes data
    /// assert_eq!(mqtt_str.len(), 5);  // Only string length
    /// ```
    pub fn size(&self) -> usize {
        match self {
            MqttString::Large(encoded) => encoded.len(),
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttString::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                2 + len
            }
        }
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
    /// let mqtt_str = mqtt::packet::MqttString::new("data").unwrap();
    /// let buffers = mqtt_str.to_buffers();
    /// // Can be used with vectored write operations
    /// // socket.write_vectored(&buffers)?;
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        match self {
            MqttString::Large(encoded) => vec![IoSlice::new(encoded)],
            #[cfg(any(
                feature = "sso-min-32bit",
                feature = "sso-min-64bit",
                feature = "sso-lv10",
                feature = "sso-lv20"
            ))]
            MqttString::Small(buffer) => {
                let len = ((buffer[0] as usize) << 8) | (buffer[1] as usize);
                vec![IoSlice::new(&buffer[..2 + len])]
            }
        }
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is compatible with no-std environments and provides an alternative
    /// to [`to_buffers()`] when vectored I/O is not needed.
    ///
    /// # Returns
    ///
    /// A vector containing the complete encoded buffer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mqtt_str = mqtt::packet::MqttString::new("data").unwrap();
    /// let buffer = mqtt_str.to_continuous_buffer();
    /// // buffer contains all packet bytes
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    /// Parse string data from a byte sequence
    ///
    /// Decodes MQTT string data from a byte buffer according to the MQTT protocol.
    /// The buffer must start with a 2-byte length prefix followed by valid UTF-8 bytes.
    ///
    /// # Parameters
    ///
    /// * `data` - Byte buffer containing the encoded string data
    ///
    /// # Returns
    ///
    /// * `Ok((MqttString, bytes_consumed))` - Successfully parsed string and number of bytes consumed
    /// * `Err(MqttError::MalformedPacket)` - If the buffer is too short, malformed, or contains invalid UTF-8
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Buffer: [length_high, length_low, utf8_bytes...]
    /// let buffer = &[0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];
    /// let (mqtt_str, consumed) = mqtt::packet::MqttString::decode(buffer).unwrap();
    ///
    /// assert_eq!(mqtt_str.as_str(), "hello");
    /// assert_eq!(consumed, 7);
    /// ```
    pub fn decode(data: &[u8]) -> Result<(Self, usize), MqttError> {
        if data.len() < 2 {
            return Err(MqttError::MalformedPacket);
        }

        let string_len = ((data[0] as usize) << 8) | (data[1] as usize);
        if data.len() < 2 + string_len {
            return Err(MqttError::MalformedPacket);
        }

        // Verify UTF-8 validity - return MQTT error on parse failure
        if core::str::from_utf8(&data[2..2 + string_len]).is_err() {
            return Err(MqttError::MalformedPacket);
        }

        let total_encoded_len = 2 + string_len;

        // Try to fit in Small variant if SSO is enabled
        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        if string_len <= SSO_DATA_THRESHOLD {
            let mut buffer = [0u8; SSO_BUFFER_SIZE];
            buffer[0] = data[0];
            buffer[1] = data[1];
            buffer[2..2 + string_len].copy_from_slice(&data[2..2 + string_len]);
            return Ok((Self::Small(buffer), total_encoded_len));
        }

        // Fallback to Large variant
        let mut encoded = Vec::with_capacity(total_encoded_len);
        encoded.extend_from_slice(&data[0..total_encoded_len]);

        Ok((Self::Large(encoded), total_encoded_len))
    }

    /// Check if the string contains a specific character
    ///
    /// Returns `true` if the string contains the specified character,
    /// `false` otherwise.
    ///
    /// # Parameters
    ///
    /// * `c` - The character to search for
    ///
    /// # Returns
    ///
    /// `true` if the character is found, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mqtt_str = mqtt::packet::MqttString::new("hello world").unwrap();
    /// assert!(mqtt_str.contains('w'));
    /// assert!(!mqtt_str.contains('x'));
    /// ```
    pub fn contains(&self, c: char) -> bool {
        self.as_str().contains(c)
    }

    /// Check if the string starts with the specified prefix
    ///
    /// Returns `true` if the string starts with the given prefix,
    /// `false` otherwise.
    ///
    /// # Parameters
    ///
    /// * `prefix` - The prefix string to check for
    ///
    /// # Returns
    ///
    /// `true` if the string starts with the prefix, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mqtt_str = mqtt::packet::MqttString::new("hello world").unwrap();
    /// assert!(mqtt_str.starts_with("hello"));
    /// assert!(!mqtt_str.starts_with("world"));
    /// ```
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.as_str().starts_with(prefix)
    }

    /// Check if the string ends with the specified suffix
    ///
    /// Returns `true` if the string ends with the given suffix,
    /// `false` otherwise.
    ///
    /// # Parameters
    ///
    /// * `suffix` - The suffix string to check for
    ///
    /// # Returns
    ///
    /// `true` if the string ends with the suffix, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mqtt_str = mqtt::packet::MqttString::new("hello world").unwrap();
    /// assert!(mqtt_str.ends_with("world"));
    /// assert!(!mqtt_str.ends_with("hello"));
    /// ```
    pub fn ends_with(&self, suffix: &str) -> bool {
        self.as_str().ends_with(suffix)
    }
}

/// Implementation of `AsRef<str>` for `MqttString`
///
/// Returns the string content when the `MqttString` is used in contexts
/// expecting a string slice reference.
impl AsRef<str> for MqttString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Implementation of `Display` for `MqttString`
///
/// Formats the string content for display purposes.
/// This allows `MqttString` to be used with `println!`, `format!`, etc.
impl core::fmt::Display for MqttString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Implementation of `Deref` for `MqttString`
///
/// Allows `MqttString` to be used directly as a string slice in many contexts
/// through automatic dereferencing. This enables method calls like `mqtt_str.len()`
/// to work directly on the string content.
impl core::ops::Deref for MqttString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// Implementation of `Serialize` for `MqttString`
///
/// Serializes the string content as a string value.
/// This is useful for JSON serialization and other serialization formats.
impl Serialize for MqttString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// Implementation of `PartialEq<str>` for `MqttString`
///
/// Allows direct comparison between `MqttString` and `str`.
impl core::cmp::PartialEq<str> for MqttString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

/// Implementation of `PartialEq<&str>` for `MqttString`
///
/// Allows direct comparison between `MqttString` and `&str`.
impl core::cmp::PartialEq<&str> for MqttString {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Implementation of `PartialEq<String>` for `MqttString`
///
/// Allows direct comparison between `MqttString` and `String`.
impl core::cmp::PartialEq<String> for MqttString {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

/// Implementation of `Hash` for `MqttString`
///
/// Hashes the string content, allowing `MqttString` to be used in hash-based
/// collections like `HashMap` and `HashSet`.
impl core::hash::Hash for MqttString {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

/// Implementation of `Default` for `MqttString`
///
/// Creates an empty `MqttString` with zero-length string content.
/// The internal buffer contains only the 2-byte length prefix (0x00, 0x00).
impl Default for MqttString {
    fn default() -> Self {
        MqttString::new("").unwrap()
    }
}

impl core::fmt::Debug for MqttString {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("MqttString")
            .field("value", &self.as_str())
            .finish()
    }
}

/// Implementation of `TryFrom<&str>` for `MqttString`
///
/// Converts a string slice to `MqttString`. This is a convenient way to
/// create `MqttString` instances from string literals.
impl TryFrom<&str> for MqttString {
    type Error = MqttError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        MqttString::new(s)
    }
}

/// Implementation of `TryFrom<String>` for `MqttString`
///
/// Converts an owned `String` to `MqttString`. This is a convenient way to
/// create `MqttString` instances from owned strings.
impl TryFrom<String> for MqttString {
    type Error = MqttError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        MqttString::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_string() {
        let string = MqttString::new("").unwrap();
        assert_eq!(string.len(), 0);
        assert!(string.is_empty());
        assert_eq!(string.as_str(), "");
        assert_eq!(string.as_bytes(), &[0x00, 0x00]);
    }

    #[test]
    fn test_small_string() {
        let string = MqttString::new("hello").unwrap();
        assert_eq!(string.len(), 5);
        assert!(!string.is_empty());
        assert_eq!(string.as_str(), "hello");
        assert_eq!(
            string.as_bytes(),
            &[0x00, 0x05, b'h', b'e', b'l', b'l', b'o']
        );
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_to_buffers() {
        let data = "buffer test";
        let string = MqttString::new(data).unwrap();
        let buffers = string.to_buffers();

        // Both variants should return 1 buffer containing the encoded data
        assert_eq!(buffers.len(), 1);

        // Verify the buffer contains the complete encoded data
        let buffer_data: &[u8] = &buffers[0];
        assert_eq!(buffer_data, string.as_bytes());
    }

    #[test]
    fn test_string_variants() {
        // Test small data (Small variant if SSO is enabled)
        let small_data = "small";
        let string = MqttString::new(small_data).unwrap();

        #[cfg(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        ))]
        assert!(matches!(string, MqttString::Small(_)));

        #[cfg(not(any(
            feature = "sso-min-32bit",
            feature = "sso-min-64bit",
            feature = "sso-lv10",
            feature = "sso-lv20"
        )))]
        assert!(matches!(string, MqttString::Large(_)));

        // Test medium-size data that fits in sso-lv20 but not smaller SSO buffers
        let medium_data = "This is a medium-size string that is longer than small SSO buffers but fits in the largest one"; // ~90 chars
        let string = MqttString::new(medium_data).unwrap();

        // With sso-lv20 (48 bytes), this should be Large (exceeds 48 bytes)
        // With smaller SSO features, this should also be Large
        assert!(matches!(string, MqttString::Large(_)));

        // Test data that should always be Large variant (larger than largest SSO buffer)
        let very_large_data = "This is a very long string that exceeds even the largest SSO buffer size to ensure it's always stored in the Large variant";
        let string = MqttString::new(very_large_data).unwrap();
        assert!(matches!(string, MqttString::Large(_)));
    }
}
