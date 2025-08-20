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
use alloc::{string::String, vec, vec::Vec};
use serde::{Serialize, Serializer};
#[cfg(feature = "std")]
use std::io::IoSlice;

// Default stack buffer size for small string optimization
const DEFAULT_STACK_BUFFER_SIZE: usize = 32;

/// MQTT String representation with Small String Optimization (SSO)
///
/// This struct represents UTF-8 strings as specified in the MQTT protocol specification.
/// It uses Small String Optimization to store small strings on the stack
/// and larger strings on the heap, providing optimal performance for both cases.
///
/// The string data is stored in a pre-encoded format internally, which includes:
/// - 2 bytes for the length prefix (big-endian u16)
/// - The UTF-8 encoded string bytes
///
/// # Small String Optimization
///
/// - Strings with total encoded size ≤ STACK_BUFFER_SIZE bytes are stored on the stack
/// - Larger strings are stored on the heap using Vec<u8>
/// - This provides zero heap allocation for typical MQTT strings like topic names
///
/// # Type Parameters
///
/// * `STACK_BUFFER_SIZE` - Size of the stack buffer in bytes (default: 32)
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
/// // Default stack buffer size (32 bytes)
/// let small_str = mqtt::packet::MqttString::new("hello").unwrap();
///
/// // Custom stack buffer size (64 bytes)
/// let custom_str = mqtt::packet::GenericMqttString::<64>::new("hello").unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum GenericMqttString<const STACK_BUFFER_SIZE: usize = DEFAULT_STACK_BUFFER_SIZE> {
    /// Small string stored on the stack (total size ≤ STACK_BUFFER_SIZE)
    Small([u8; STACK_BUFFER_SIZE], usize), // buffer, actual_length
    /// Large string stored on the heap
    Large(Vec<u8>),
}

impl<const STACK_BUFFER_SIZE: usize> GenericMqttString<STACK_BUFFER_SIZE> {
    /// Create a new MqttString from a string
    ///
    /// Creates an `MqttString` instance from the provided string data.
    /// The string is converted to UTF-8 bytes and stored with a 2-byte length prefix.
    /// Small strings (≤ 32 bytes total) are stored on the stack, larger ones on the heap.
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
    /// // Small string - stored on stack
    /// let small_str = mqtt::packet::MqttString::new("hello").unwrap();
    ///
    /// // Large string - stored on heap
    /// let large_str = mqtt::packet::MqttString::new("a".repeat(100)).unwrap();
    /// ```
    pub fn new(s: impl AsRef<str>) -> Result<Self, MqttError> {
        let s_ref = s.as_ref();
        let len = s_ref.len();

        if len > 65535 {
            return Err(MqttError::MalformedPacket);
        }

        // Calculate total encoded size (2 bytes for length prefix + string bytes)
        let total_size = 2 + len;

        if total_size <= STACK_BUFFER_SIZE {
            // Use stack storage for small strings
            let mut buffer = [0u8; STACK_BUFFER_SIZE];
            buffer[0] = (len >> 8) as u8;
            buffer[1] = len as u8;
            buffer[2..2 + len].copy_from_slice(s_ref.as_bytes());
            Ok(Self::Small(buffer, total_size))
        } else {
            // Use heap storage for large strings
            let mut encoded = Vec::with_capacity(total_size);
            encoded.push((len >> 8) as u8);
            encoded.push(len as u8);
            encoded.extend_from_slice(s_ref.as_bytes());
            Ok(Self::Large(encoded))
        }
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
            Self::Small(buffer, len) => &buffer[..*len],
            Self::Large(encoded) => encoded,
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
        // Also, no direct modification of internal data is provided,
        // ensuring buffer immutability
        match self {
            Self::Small(buffer, len) => unsafe { core::str::from_utf8_unchecked(&buffer[2..*len]) },
            Self::Large(encoded) => unsafe { core::str::from_utf8_unchecked(&encoded[2..]) },
        }
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
            Self::Small(_, len) => len - 2,
            Self::Large(encoded) => encoded.len() - 2,
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
        match self {
            Self::Small(_, len) => *len <= 2,
            Self::Large(encoded) => encoded.len() <= 2,
        }
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
            Self::Small(_, len) => *len,
            Self::Large(encoded) => encoded.len(),
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
        vec![IoSlice::new(self.as_bytes())]
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
        let total_size = 2 + string_len;

        if data.len() < total_size {
            return Err(MqttError::MalformedPacket);
        }

        // Verify UTF-8 validity - return MQTT error on parse failure
        if core::str::from_utf8(&data[2..2 + string_len]).is_err() {
            return Err(MqttError::MalformedPacket);
        }

        if total_size <= STACK_BUFFER_SIZE {
            // Use stack storage for small strings
            let mut buffer = [0u8; STACK_BUFFER_SIZE];
            buffer[..total_size].copy_from_slice(&data[..total_size]);
            Ok((Self::Small(buffer, total_size), total_size))
        } else {
            // Use heap storage for large strings
            let mut encoded = Vec::with_capacity(total_size);
            encoded.extend_from_slice(&data[..total_size]);
            Ok((Self::Large(encoded), total_size))
        }
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

/// Implementation of `AsRef<str>` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> AsRef<str> for GenericMqttString<STACK_BUFFER_SIZE> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Implementation of `Display` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> core::fmt::Display for GenericMqttString<STACK_BUFFER_SIZE> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Implementation of `Deref` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> core::ops::Deref for GenericMqttString<STACK_BUFFER_SIZE> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// Implementation of `Serialize` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> Serialize for GenericMqttString<STACK_BUFFER_SIZE> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

/// Implementation of `PartialEq<str>` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> core::cmp::PartialEq<str>
    for GenericMqttString<STACK_BUFFER_SIZE>
{
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

/// Implementation of `PartialEq<&str>` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> core::cmp::PartialEq<&str>
    for GenericMqttString<STACK_BUFFER_SIZE>
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

/// Implementation of `PartialEq<String>` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> core::cmp::PartialEq<String>
    for GenericMqttString<STACK_BUFFER_SIZE>
{
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

/// Implementation of `Hash` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> core::hash::Hash for GenericMqttString<STACK_BUFFER_SIZE> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

/// Implementation of `Default` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> Default for GenericMqttString<STACK_BUFFER_SIZE> {
    fn default() -> Self {
        let mut buffer = [0u8; STACK_BUFFER_SIZE];
        buffer[0] = 0x00;
        buffer[1] = 0x00;
        Self::Small(buffer, 2)
    }
}

/// Implementation of `TryFrom<&str>` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> TryFrom<&str> for GenericMqttString<STACK_BUFFER_SIZE> {
    type Error = MqttError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

/// Implementation of `TryFrom<String>` for `GenericMqttString`
impl<const STACK_BUFFER_SIZE: usize> TryFrom<String> for GenericMqttString<STACK_BUFFER_SIZE> {
    type Error = MqttError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}
