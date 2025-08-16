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
use core::fmt;
use std::io::IoSlice;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::result_code::ConnectReturnCode;
use crate::mqtt::result_code::MqttError;

/// MQTT v3.1.1 CONNACK packet representation
///
/// The CONNACK packet is sent by the MQTT server (broker) in response to a CONNECT packet
/// from a client. It indicates whether the connection attempt was successful and provides
/// session state information. This is the first packet sent by the server after receiving
/// a CONNECT packet from the client.
///
/// According to MQTT v3.1.1 specification, the CONNACK packet contains:
/// - Fixed header with packet type and remaining length
/// - Variable header with acknowledgment flags and return code
/// - No payload
///
/// # Acknowledgment Flags
///
/// The acknowledgment flags byte contains:
/// - **Session Present flag** (bit 0): Indicates whether the server has stored session state
///   from a previous connection. Set to 1 if the server has session state, 0 if starting
///   a clean session or if the connection was rejected.
/// - Bits 1-7: Reserved and must be set to 0
///
/// # Return Codes
///
/// The return code indicates the result of the connection attempt:
/// - `0` Accepted - Connection accepted
/// - `1` Unacceptable protocol version - The server does not support the MQTT protocol version requested
/// - `2` Identifier rejected - The client identifier is correct UTF-8 but not allowed by the server
/// - `3` Server unavailable - The network connection has been made but the MQTT service is unavailable
/// - `4` Bad user name or password - The data in the user name or password is malformed
/// - `5` Not authorized - The client is not authorized to connect
///
/// # Session State Behavior
///
/// - If the client connected with CleanSession=1, Session Present must be 0
/// - If the client connected with CleanSession=0, Session Present depends on whether
///   the server has session state for the client identifier
/// - If the return code is non-zero (connection rejected), Session Present must be 0
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
///
/// // Create a successful CONNACK with clean session
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// assert_eq!(connack.return_code(), ConnectReturnCode::Accepted);
/// assert!(!connack.session_present());
///
/// // Create CONNACK with session present
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(true)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = connack.to_buffers();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct Connack {
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    ack_flags: [u8; 1],
    #[builder(private)]
    return_code_buf: [u8; 1],
}

impl Connack {
    /// Create a new ConnackBuilder for constructing CONNACK packets
    ///
    /// Returns a builder instance that allows setting the various fields of a CONNACK packet
    /// in a fluent interface style. The builder ensures all required fields are set before
    /// creating the final packet.
    ///
    /// # Returns
    ///
    /// A new `ConnackBuilder` instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(false)
    ///     .return_code(ConnectReturnCode::Accepted)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> ConnackBuilder {
        ConnackBuilder::default()
    }

    /// Get the packet type for CONNACK packets
    ///
    /// Returns the constant packet type identifier for CONNACK packets.
    /// This is always `PacketType::Connack` for CONNACK packets.
    ///
    /// # Returns
    ///
    /// The packet type `PacketType::Connack`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::packet_type::PacketType;
    ///
    /// assert_eq!(mqtt::packet::v3_1_1::Connack::packet_type(), PacketType::Connack);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Connack
    }

    /// Get the return code from the CONNACK packet
    ///
    /// Returns the return code that indicates the result of the connection attempt.
    /// The return code provides information about whether the connection was successful
    /// or failed, and if failed, the specific reason for failure.
    ///
    /// # Returns
    ///
    /// The `ConnectReturnCode` indicating the connection result
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(false)
    ///     .return_code(ConnectReturnCode::Accepted)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(connack.return_code(), ConnectReturnCode::Accepted);
    /// ```
    pub fn return_code(&self) -> ConnectReturnCode {
        ConnectReturnCode::try_from(self.return_code_buf[0]).unwrap()
    }

    /// Get the session present flag
    ///
    /// Returns `true` if the server has stored session state for this client from a
    /// previous connection, `false` if this is a clean session or no session state exists.
    /// This corresponds to bit 0 of the acknowledgment flags byte.
    ///
    /// According to MQTT v3.1.1 specification:
    /// - If CleanSession was set to 1 in the CONNECT, this must be 0
    /// - If CleanSession was set to 0, this indicates whether session state exists
    /// - If the return code is non-zero (connection rejected), this must be 0
    ///
    /// # Returns
    ///
    /// `true` if session state is present, `false` for a clean session or rejected connection
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(true)
    ///     .return_code(ConnectReturnCode::Accepted)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(connack.session_present());
    /// ```
    pub fn session_present(&self) -> bool {
        (self.ack_flags[0] & 0b0000_0001) != 0
    }

    /// Get the total size of the CONNACK packet in bytes
    ///
    /// Returns the complete size of the CONNACK packet including the fixed header
    /// and variable header. This is useful for memory allocation and buffer management.
    /// For MQTT v3.1.1 CONNACK packets, this is always 4 bytes:
    /// - Fixed header: 1 byte (packet type and flags)
    /// - Remaining length: 1 byte (always 2 for CONNACK)
    /// - Acknowledgment flags: 1 byte
    /// - Return code: 1 byte
    ///
    /// # Returns
    ///
    /// The total packet size in bytes (always 4 for v3.1.1 CONNACK)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(false)
    ///     .return_code(ConnectReturnCode::Accepted)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(connack.size(), 4); // Always 4 bytes for v3.1.1 CONNACK
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is compatible with no-std environments.
    ///
    /// The returned buffer contains:
    /// 1. Fixed header (packet type and flags) - 1 byte
    /// 2. Remaining length field - 1 byte (always 0x02)
    /// 3. Acknowledgment flags - 1 byte
    /// 4. Return code - 1 byte
    ///
    /// # Returns
    ///
    /// A vector containing the complete packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(false)
    ///     .return_code(ConnectReturnCode::Accepted)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = connack.to_continuous_buffer();
    /// // buffer contains all packet bytes
    /// ```
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.extend_from_slice(&self.ack_flags);
        buf.extend_from_slice(&self.return_code_buf);
        buf
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// represent the complete CONNACK packet in wire format.
    ///
    /// The returned buffers contain:
    /// 1. Fixed header (packet type and flags) - 1 byte
    /// 2. Remaining length field - 1 byte (always 0x02)
    /// 3. Acknowledgment flags - 1 byte
    /// 4. Return code - 1 byte
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` objects for vectored I/O operations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(false)
    ///     .return_code(ConnectReturnCode::Accepted)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = connack.to_buffers();
    /// // Use with vectored write: socket.write_vectored(&buffers)?;
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        vec![
            IoSlice::new(&self.fixed_header),
            IoSlice::new(self.remaining_length.as_bytes()),
            IoSlice::new(&self.ack_flags),
            IoSlice::new(&self.return_code_buf),
        ]
    }

    /// Parse a CONNACK packet from raw bytes
    ///
    /// Decodes a CONNACK packet from a byte buffer according to the MQTT v3.1.1 protocol
    /// specification. The buffer should contain the variable header but not the fixed
    /// header (which should have been processed separately).
    ///
    /// The parsing process:
    /// 1. Reads acknowledgment flags (1 byte)
    /// 2. Reads return code (1 byte)
    /// 3. Validates the return code is within the valid range
    /// 4. Constructs the CONNACK packet
    ///
    /// # Parameters
    ///
    /// * `data` - Byte buffer containing the CONNACK packet data (without fixed header)
    ///            Must be at least 2 bytes: [ack_flags, return_code]
    ///
    /// # Returns
    ///
    /// * `Ok((Connack, bytes_consumed))` - Successfully parsed packet and number of bytes consumed (always 2)
    /// * `Err(MqttError::MalformedPacket)` - If the buffer is too short or contains an invalid return code
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// // Buffer contains: [ack_flags, return_code]
    /// let buffer = &[0x00, 0x00]; // No session present, connection accepted
    /// let (connack, consumed) = mqtt::packet::v3_1_1::Connack::parse(buffer).unwrap();
    ///
    /// assert!(!connack.session_present());
    /// assert_eq!(connack.return_code(), ConnectReturnCode::Accepted);
    /// assert_eq!(consumed, 2);
    ///
    /// // Example with session present
    /// let buffer = &[0x01, 0x00]; // Session present, connection accepted
    /// let (connack, consumed) = mqtt::packet::v3_1_1::Connack::parse(buffer).unwrap();
    ///
    /// assert!(connack.session_present());
    /// assert_eq!(consumed, 2);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        if data.len() < 2 {
            // ack_flags(1) + return_code(1)
            return Err(MqttError::MalformedPacket);
        }

        let flags = data[0];
        let session_present = (flags & 0x01) != 0;

        let code = data[1];
        let return_code =
            ConnectReturnCode::try_from(code).map_err(|_| MqttError::MalformedPacket)?;

        let connack = ConnackBuilder::default()
            .session_present(session_present)
            .return_code(return_code)
            .build()?;

        Ok((connack, 2))
    }
}

/// Builder for constructing MQTT v3.1.1 CONNACK packets
///
/// The `ConnackBuilder` provides a fluent interface for creating CONNACK packets with
/// all necessary validation. It ensures that required fields are set and that the
/// packet conforms to MQTT v3.1.1 specifications.
///
/// # Required Fields
///
/// - `session_present`: Whether the server has session state for this client
/// - `return_code`: The result of the connection attempt
///
/// # Protocol Constraints
///
/// The builder enforces MQTT v3.1.1 protocol rules:
/// - If return_code is not Accepted, session_present should be false
/// - Reserved bits in acknowledgment flags are automatically set to 0
/// - Return codes must be within the valid range (0-5)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
///
/// // Build a successful CONNACK with clean session
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// // Build CONNACK with existing session
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(true)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// // Build CONNACK for rejected connection
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)  // Must be false for rejected connections
///     .return_code(ConnectReturnCode::NotAuthorized)
///     .build()
///     .unwrap();
/// ```
impl ConnackBuilder {
    /// Set the session present flag
    ///
    /// This method sets whether the server has stored session state for the client
    /// from a previous connection. When `true`, the server has session state and
    /// will deliver any queued messages. When `false`, this is a clean session.
    ///
    /// According to MQTT v3.1.1 specification:
    /// - If the CONNECT packet had CleanSession=1, this must be `false`
    /// - If the CONNECT packet had CleanSession=0, this indicates whether session state exists
    /// - If the connection is rejected (non-zero return code), this must be `false`
    ///
    /// # Parameters
    ///
    /// * `v` - `true` if session state is present, `false` for clean session or rejected connection
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Clean session (or rejected connection)
    /// let builder = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(false);
    ///
    /// // Existing session present
    /// let builder = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(true);
    /// ```
    pub fn session_present(mut self, v: bool) -> Self {
        self.ack_flags = Some([v as u8]);
        self
    }

    /// Set the return code for the connection result
    ///
    /// This method sets the return code that indicates whether the connection
    /// attempt was successful or failed, and if failed, the specific reason.
    /// The return code is mandatory for all CONNACK packets.
    ///
    /// # Parameters
    ///
    /// * `rc` - The connection result return code
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// // Successful connection
    /// let builder = mqtt::packet::v3_1_1::Connack::builder()
    ///     .return_code(ConnectReturnCode::Accepted);
    ///
    /// // Failed connection - bad credentials
    /// let builder = mqtt::packet::v3_1_1::Connack::builder()
    ///     .return_code(ConnectReturnCode::BadUserNameOrPassword);
    ///
    /// // Failed connection - protocol version not supported
    /// let builder = mqtt::packet::v3_1_1::Connack::builder()
    ///     .return_code(ConnectReturnCode::UnacceptableProtocolVersion);
    /// ```
    pub fn return_code(mut self, rc: ConnectReturnCode) -> Self {
        self.return_code_buf = Some([rc as u8]);
        self
    }

    /// Validate the builder state before constructing the CONNACK packet
    ///
    /// This method ensures that all required fields are set and that the values
    /// conform to MQTT v3.1.1 protocol requirements.
    ///
    /// # Validation Rules
    ///
    /// 1. Both session_present and return_code must be set
    /// 2. All values must be within valid ranges
    /// 3. Protocol constraints are checked during build, not validate
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All required fields are present
    /// * `Err(MqttError::MalformedPacket)` - Required fields are missing
    fn validate(&self) -> Result<(), MqttError> {
        if self.ack_flags.is_none() {
            return Err(MqttError::MalformedPacket);
        }
        if self.return_code_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }
        Ok(())
    }

    /// Build the final CONNACK packet
    ///
    /// This method validates all fields and constructs the final CONNACK packet.
    /// It calculates the remaining length and other header fields according to
    /// the MQTT v3.1.1 specification.
    ///
    /// The build process:
    /// 1. Validates all required fields are present
    /// 2. Sets default values for unspecified optional fields
    /// 3. Calculates packet header fields (remaining length is always 2)
    /// 4. Constructs the final packet structure
    ///
    /// # Default Values
    ///
    /// - `session_present`: `false` if not specified
    /// - `return_code`: `Accepted` if not specified
    ///
    /// # Returns
    ///
    /// * `Ok(Connack)` - Successfully built packet
    /// * `Err(MqttError::MalformedPacket)` - Missing required fields
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
    ///
    /// // Successful connection with clean session
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .session_present(false)
    ///     .return_code(ConnectReturnCode::Accepted)
    ///     .build()
    ///     .unwrap();
    ///
    /// // Using defaults (equivalent to above)
    /// let connack = mqtt::packet::v3_1_1::Connack::builder()
    ///     .build()
    ///     .unwrap();  // session_present=false, return_code=Accepted
    /// ```
    pub fn build(self) -> Result<Connack, MqttError> {
        self.validate()?;

        let ack_flags = self.ack_flags.unwrap_or([0]);
        let return_code_buf = self
            .return_code_buf
            .unwrap_or([ConnectReturnCode::Accepted as u8]);

        let remaining_length = VariableByteInteger::from_u32(2).unwrap(); // ack_flags(1) + return_code(1)

        Ok(Connack {
            fixed_header: [FixedHeader::Connack.as_u8()],
            remaining_length,
            ack_flags,
            return_code_buf,
        })
    }
}

/// Implementation of `Serialize` trait for JSON serialization
///
/// Serializes the CONNACK packet to a structured format containing:
/// - `type`: The packet type as a string ("CONNACK")
/// - `session_present`: Boolean indicating session state
/// - `return_code`: The connection result return code as a string
///
/// The serialized format is compatible with JSON and other structured data formats.
/// This is useful for logging, debugging, and API responses.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
///
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&connack).unwrap();
/// // Produces: {"type":"CONNACK","session_present":false,"return_code":"Accepted"}
///
/// // Example with session present
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(true)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&connack).unwrap();
/// // Produces: {"type":"CONNACK","session_present":true,"return_code":"Accepted"}
/// ```
impl Serialize for Connack {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("connack", 3)?;
        state.serialize_field("type", PacketType::Connack.as_str())?;
        state.serialize_field("session_present", &self.session_present())?;
        state.serialize_field("return_code", &self.return_code())?;
        state.end()
    }
}

/// Implementation of `Display` trait for human-readable output
///
/// Formats the CONNACK packet as a JSON string for easy reading and debugging.
/// If serialization fails, it returns an error message in JSON format.
///
/// The output includes all relevant packet information in a structured format,
/// making it suitable for logging, debugging, and user interfaces.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
///
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(true)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// println!("{}", connack);
/// // Output: {"type":"CONNACK","session_present":true,"return_code":"Accepted"}
///
/// // Example with rejected connection
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)
///     .return_code(ConnectReturnCode::NotAuthorized)
///     .build()
///     .unwrap();
///
/// println!("{}", connack);
/// // Output: {"type":"CONNACK","session_present":false,"return_code":"NotAuthorized"}
/// ```
impl fmt::Display for Connack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Implementation of `Debug` trait for debugging output
///
/// Uses the same JSON formatting as `Display` for consistent debugging output.
/// This provides detailed, structured information about the CONNACK packet state,
/// making it easy to inspect packet contents during development and troubleshooting.
///
/// The debug output is identical to the display output, as the JSON format
/// already provides comprehensive information about the packet structure.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
///
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)
///     .return_code(ConnectReturnCode::BadUserNameOrPassword)
///     .build()
///     .unwrap();
///
/// println!("{:?}", connack);
/// // Output: {"type":"CONNACK","session_present":false,"return_code":"BadUserNameOrPassword"}
///
/// // Useful in debugging scenarios
/// eprintln!("Received CONNACK: {:?}", connack);
/// ```
impl fmt::Debug for Connack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Implementation of `GenericPacketTrait` for generic packet operations
///
/// Provides a common interface for packet operations that can be used generically
/// across different MQTT packet types. This allows for uniform handling of packets
/// in collections and generic contexts.
///
/// This trait is particularly useful when working with mixed packet types or
/// implementing packet processing pipelines that need to handle various MQTT
/// packet types uniformly.
///
/// # Methods
///
/// - `size()`: Returns the total packet size in bytes
/// - `to_buffers()`: Returns IoSlice buffers for efficient network I/O
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
///
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// // Use generic trait methods
/// let size = connack.size();
/// let buffers = connack.to_buffers();
///
/// // Can be used in generic contexts
/// fn process_packet<T: GenericPacketTrait>(packet: &T) {
///     println!("Packet size: {} bytes", packet.size());
/// }
///
/// process_packet(&connack);
/// ```
impl GenericPacketTrait for Connack {
    fn size(&self) -> usize {
        self.size()
    }

    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }
}

/// Implementation of `GenericPacketDisplay` for generic formatting operations
///
/// Provides a common interface for formatting operations that can be used generically
/// across different MQTT packet types. This enables uniform display handling in
/// collections and generic contexts.
///
/// This trait is useful when implementing logging systems, debugging tools, or
/// user interfaces that need to display various MQTT packet types in a consistent
/// manner without knowing the specific packet type at compile time.
///
/// # Methods
///
/// - `fmt_debug()`: Delegates to the Debug trait implementation
/// - `fmt_display()`: Delegates to the Display trait implementation
///
/// Both methods produce JSON-formatted output for consistency and readability.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketDisplay;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReturnCode;
///
/// let connack = mqtt::packet::v3_1_1::Connack::builder()
///     .session_present(false)
///     .return_code(ConnectReturnCode::Accepted)
///     .build()
///     .unwrap();
///
/// // Use generic display methods
/// let debug_output = format!("{connack:?}");
/// let display_output = format!("{}", connack);
///
/// // Can be used in generic contexts
/// fn log_packet<T: GenericPacketDisplay>(packet: &T) {
///     println!("Packet: {}", packet); // Uses fmt_display
/// }
///
/// log_packet(&connack);
/// ```
impl GenericPacketDisplay for Connack {
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
