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

use alloc::vec::Vec;
use core::fmt;
use derive_builder::Builder;
#[cfg(feature = "std")]
use std::io::IoSlice;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use getset::{CopyGetters, Getters};

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::property::PropertiesToContinuousBuffer;
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
#[cfg(feature = "std")]
use crate::mqtt::packet::PropertiesToBuffers;
use crate::mqtt::packet::{GenericProperties, GenericProperty, PropertiesParse, PropertiesSize};
use crate::mqtt::result_code::ConnectReasonCode;
use crate::mqtt::result_code::MqttError;

/// MQTT 5.0 CONNACK packet representation
///
/// The CONNACK packet is sent by the MQTT server (broker) in response to a CONNECT packet
/// from a client. It indicates whether the connection attempt was successful and provides
/// various connection-related parameters and capabilities.
///
/// According to MQTT 5.0 specification, the CONNACK packet contains:
/// - Fixed header with packet type and remaining length
/// - Variable header with acknowledgment flags, reason code, and properties
/// - No payload
///
/// # Acknowledgment Flags
///
/// The acknowledgment flags byte contains:
/// - **Session Present flag** (bit 0): Indicates whether the server has stored session state
///   from a previous connection. Set to 1 if the server has session state, 0 if starting
///   a clean session.
/// - Bits 1-7: Reserved and must be set to 0
///
/// # Reason Codes
///
/// The reason code indicates the result of the connection attempt:
/// - `0x00` Success - Connection accepted
/// - `0x80` Unspecified error
/// - `0x81` Malformed packet
/// - `0x82` Protocol error
/// - `0x83` Implementation specific error
/// - `0x84` Unsupported protocol version
/// - `0x85` Client identifier not valid
/// - `0x86` Bad username or password
/// - `0x87` Not authorized
/// - `0x88` Server unavailable
/// - `0x89` Server busy
/// - `0x8A` Banned
/// - `0x8C` Bad authentication method
/// - `0x90` Topic name invalid
/// - `0x95` Packet too large
/// - `0x97` Quota exceeded
/// - `0x99` Payload format invalid
/// - `0x9A` Retain not supported
/// - `0x9B` QoS not supported
/// - `0x9C` Use another server
/// - `0x9D` Server moved
/// - `0x9F` Connection rate exceeded
///
/// # Properties
///
/// MQTT 5.0 CONNACK packets can include various properties:
/// - Session Expiry Interval
/// - Receive Maximum
/// - Maximum QoS
/// - Retain Available
/// - Maximum Packet Size
/// - Assigned Client Identifier
/// - Topic Alias Maximum
/// - Reason String
/// - User Properties
/// - Wildcard Subscription Available
/// - Subscription Identifiers Available
/// - Shared Subscription Available
/// - Server Keep Alive
/// - Response Information
/// - Server Reference
/// - Authentication Method
/// - Authentication Data
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
///
/// // Create a successful CONNACK with clean session
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(false)
///     .reason_code(ConnectReasonCode::Success)
///     .build()
///     .unwrap();
///
/// assert_eq!(connack.reason_code(), ConnectReasonCode::Success);
/// assert!(!connack.session_present());
///
/// // Create CONNACK with session present and properties
/// let props = mqtt::packet::Properties::new();
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(true)
///     .reason_code(ConnectReasonCode::Success)
///     .props(props)
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = connack.to_buffers();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericConnack<
    const STRING_BUFFER_SIZE: usize = 32,
    const BINARY_BUFFER_SIZE: usize = 32,
> {
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    ack_flags: [u8; 1],
    #[builder(private)]
    reason_code_buf: [u8; 1],
    #[builder(private)]
    property_length: VariableByteInteger,

    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: GenericProperties<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
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
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// let connack = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(false)
    ///     .reason_code(ConnectReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericConnackBuilder<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE> {
        GenericConnackBuilder::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::default()
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
    /// assert_eq!(mqtt::packet::v5_0::Connack::packet_type(), PacketType::Connack);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Connack
    }

    /// Get the session present flag
    ///
    /// Returns `true` if the server has stored session state for this client from a
    /// previous connection, `false` if this is a clean session or no session state exists.
    /// This corresponds to bit 0 of the acknowledgment flags byte.
    ///
    /// # Returns
    ///
    /// `true` if session state is present, `false` for a clean session
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// let connack = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(true)
    ///     .reason_code(ConnectReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(connack.session_present());
    /// ```
    pub fn session_present(&self) -> bool {
        (self.ack_flags[0] & 0b0000_0001) != 0
    }

    /// Get the reason code from the CONNACK packet
    ///
    /// Returns the reason code that indicates the result of the connection attempt.
    /// The reason code provides detailed information about whether the connection
    /// was successful or failed, and if failed, the specific reason for failure.
    ///
    /// # Returns
    ///
    /// The `ConnectReasonCode` indicating the connection result
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// let connack = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(false)
    ///     .reason_code(ConnectReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(connack.reason_code(), ConnectReasonCode::Success);
    /// ```
    pub fn reason_code(&self) -> ConnectReasonCode {
        ConnectReasonCode::try_from(self.reason_code_buf[0]).unwrap()
    }

    /// Get the total size of the CONNACK packet in bytes
    ///
    /// Returns the complete size of the CONNACK packet including the fixed header,
    /// variable header, and all properties. This is useful for memory allocation
    /// and buffer management.
    ///
    /// # Returns
    ///
    /// The total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// let connack = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(false)
    ///     .reason_code(ConnectReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = connack.size();
    /// // Size includes fixed header + variable header + properties
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// represent the complete CONNACK packet in wire format.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` objects for vectored I/O operations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// let connack = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(false)
    ///     .reason_code(ConnectReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = connack.to_buffers();
    /// // Use with vectored write: socket.write_vectored(&buffers)?;
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(&self.ack_flags));
        bufs.push(IoSlice::new(&self.reason_code_buf));
        bufs.push(IoSlice::new(self.property_length.as_bytes()));
        bufs.extend(self.props.to_buffers());

        bufs
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method provides an alternative to `to_buffers()` for no-std environments
    /// where vectored I/O is not available.
    ///
    /// The returned buffer contains the complete CONNACK packet serialized according
    /// to the MQTT v5.0 protocol specification, including fixed header, remaining
    /// length, acknowledgment flags, reason code, property length, and properties.
    ///
    /// # Returns
    ///
    /// A vector containing the complete packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// let connack = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(false)
    ///     .reason_code(ConnectReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = connack.to_continuous_buffer();
    /// // buffer contains all packet bytes
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.extend_from_slice(&self.ack_flags);
        buf.extend_from_slice(&self.reason_code_buf);
        buf.extend_from_slice(self.property_length.as_bytes());
        buf.append(&mut self.props.to_continuous_buffer());
        buf
    }

    /// Parse a CONNACK packet from raw bytes
    ///
    /// Decodes a CONNACK packet from a byte buffer according to the MQTT 5.0 protocol
    /// specification. The buffer should contain the variable header and any properties,
    /// but not the fixed header (which should have been processed separately).
    ///
    /// The parsing process:
    /// 1. Reads acknowledgment flags (1 byte)
    /// 2. Reads reason code (1 byte)
    /// 3. Parses properties section
    /// 4. Validates the packet structure and properties
    ///
    /// # Parameters
    ///
    /// * `data` - Byte buffer containing the CONNACK packet data (without fixed header)
    ///
    /// # Returns
    ///
    /// * `Ok((Connack, bytes_consumed))` - Successfully parsed packet and number of bytes consumed
    /// * `Err(MqttError::MalformedPacket)` - If the buffer is malformed or too short
    /// * `Err(MqttError::ProtocolError)` - If the packet contains invalid properties
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Buffer contains: [flags, reason_code, prop_length, ...properties]
    /// let buffer = &[0x00, 0x00, 0x00]; // No session, success, no properties
    /// let (connack, consumed) = mqtt::packet::v5_0::Connack::parse(buffer).unwrap();
    ///
    /// assert!(!connack.session_present());
    /// assert_eq!(consumed, 3);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;
        if data.len() < 3 {
            // ack_flags(1) + reason_code(1) + prop_length_bytes(at least 1)
            return Err(MqttError::MalformedPacket);
        }
        let flags = data[cursor];
        cursor += 1;
        let _session = (flags & 0x01) != 0;
        let code = data[cursor];
        cursor += 1;
        let _reason = ConnectReasonCode::try_from(code).map_err(|_| MqttError::MalformedPacket)?;

        // properties
        let (props, consumed) =
            GenericProperties::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(&data[cursor..])?;
        cursor += consumed;
        validate_connack_properties(&props)?;
        let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

        let connack = GenericConnack {
            fixed_header: [FixedHeader::Connack.as_u8()],
            remaining_length: VariableByteInteger::from_u32(cursor as u32).unwrap(),
            ack_flags: [flags],
            reason_code_buf: [code],
            property_length: prop_len,
            props,
        };

        Ok((connack, cursor))
    }
}

/// Builder for constructing MQTT 5.0 CONNACK packets
///
/// The `ConnackBuilder` provides a fluent interface for creating CONNACK packets with
/// all necessary validation. It ensures that required fields are set and that the
/// properties conform to MQTT 5.0 specifications.
///
/// # Required Fields
///
/// - `session_present`: Whether the server has session state for this client
/// - `reason_code`: The result of the connection attempt
///
/// # Optional Fields
///
/// - `props`: MQTT 5.0 properties (defaults to empty if not set)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
///
/// // Build a successful CONNACK
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(false)
///     .reason_code(ConnectReasonCode::Success)
///     .build()
///     .unwrap();
///
/// // Build CONNACK with properties
/// let mut props = mqtt::packet::Properties::new();
/// props.push(mqtt::packet::Property::SessionExpiryInterval(3600));
///
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(true)
///     .reason_code(ConnectReasonCode::Success)
///     .props(props)
///     .build()
///     .unwrap();
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericConnackBuilder<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    /// Set the session present flag
    ///
    /// This method sets whether the server has stored session state for the client
    /// from a previous connection. When `true`, the server has session state and
    /// will deliver any queued messages. When `false`, this is a clean session.
    ///
    /// # Parameters
    ///
    /// * `v` - `true` if session state is present, `false` for clean session
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
    /// let builder = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(true);  // Server has session state
    /// ```
    pub fn session_present(mut self, v: bool) -> Self {
        self.ack_flags = Some([v as u8]);
        self
    }

    /// Set the reason code for the connection result
    ///
    /// This method sets the reason code that indicates whether the connection
    /// attempt was successful or failed, and if failed, the specific reason.
    /// The reason code is mandatory for all CONNACK packets.
    ///
    /// # Parameters
    ///
    /// * `rc` - The connection result reason code
    ///
    /// # Returns
    ///
    /// Self for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// // Successful connection
    /// let builder = mqtt::packet::v5_0::Connack::builder()
    ///     .reason_code(ConnectReasonCode::Success);
    ///
    /// // Failed connection - bad credentials
    /// let builder = mqtt::packet::v5_0::Connack::builder()
    ///     .reason_code(ConnectReasonCode::BadUserNameOrPassword);
    /// ```
    pub fn reason_code(mut self, rc: ConnectReasonCode) -> Self {
        self.reason_code_buf = Some([rc as u8]);
        self
    }

    /// Validate the builder state before constructing the CONNACK packet
    ///
    /// This method ensures that all required fields are set and that any provided
    /// properties are valid for CONNACK packets according to MQTT 5.0 specification.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All fields are valid and present
    /// * `Err(MqttError::MalformedPacket)` - Required fields are missing
    /// * `Err(MqttError::ProtocolError)` - Invalid properties or property duplicates
    fn validate(&self) -> Result<(), MqttError> {
        if self.ack_flags.is_none() {
            return Err(MqttError::MalformedPacket);
        }
        if self.reason_code_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }
        if let Some(ref props) = self.props {
            validate_connack_properties(props)?;
        }
        Ok(())
    }

    /// Build the final CONNACK packet
    ///
    /// This method validates all fields and constructs the final CONNACK packet.
    /// It calculates the remaining length, property length, and other header fields
    /// according to the MQTT 5.0 specification.
    ///
    /// The build process:
    /// 1. Validates all required fields are present
    /// 2. Validates properties (if any) are legal for CONNACK
    /// 3. Calculates packet sizes and lengths
    /// 4. Constructs the final packet structure
    ///
    /// # Returns
    ///
    /// * `Ok(Connack)` - Successfully built packet
    /// * `Err(MqttError::MalformedPacket)` - Missing required fields
    /// * `Err(MqttError::ProtocolError)` - Invalid properties
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
    ///
    /// let connack = mqtt::packet::v5_0::Connack::builder()
    ///     .session_present(false)
    ///     .reason_code(ConnectReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(
        self,
    ) -> Result<GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>, MqttError> {
        self.validate()?;

        let ack_flags = self.ack_flags.unwrap_or([0]);
        let reason_code_buf = self.reason_code_buf.unwrap_or([0]);
        let props = self
            .props
            .unwrap_or_else(GenericProperties::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::new);
        let props_size: usize = props.size();
        let property_length = VariableByteInteger::from_u32(props_size as u32).unwrap();

        // remaining length: ack_flags(1) + reason(1) + prop_length_size + props_size
        let remaining = 1 + 1 + property_length.size() + props_size;
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericConnack {
            fixed_header: [FixedHeader::Connack.as_u8()],
            remaining_length,
            ack_flags,
            reason_code_buf,
            property_length,
            props,
        })
    }
}

/// Implementation of `Serialize` trait for JSON serialization
///
/// Serializes the CONNACK packet to a structured format containing:
/// - `type`: The packet type as a string ("CONNACK")
/// - `session_present`: Boolean indicating session state
/// - `reason_code`: The connection result reason code
/// - `props`: MQTT 5.0 properties (if any)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
///
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(false)
///     .reason_code(ConnectReasonCode::Success)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&connack).unwrap();
/// // Produces: {"type":"CONNACK","session_present":false,"reason_code":"Success","props":{}}
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> Serialize
    for GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 3; // type, session_present, reason_code

        if !self.props.is_empty() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("Connack", field_count)?;
        state.serialize_field("type", PacketType::Connack.as_str())?;
        state.serialize_field("session_present", &self.session_present())?;
        state.serialize_field("reason_code", &self.reason_code())?;
        state.serialize_field("props", &self.props)?;

        state.end()
    }
}

/// Implementation of `Display` trait for human-readable output
///
/// Formats the CONNACK packet as a JSON string for easy reading and debugging.
/// If serialization fails, it returns an error message in JSON format.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
///
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(true)
///     .reason_code(ConnectReasonCode::Success)
///     .build()
///     .unwrap();
///
/// println!("{}", connack);
/// // Output: {"type":"CONNACK","session_present":true,"reason_code":"Success","props":{}}
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Display
    for GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
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
/// This provides detailed, structured information about the CONNACK packet state.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
///
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(false)
///     .reason_code(ConnectReasonCode::BadUserNameOrPassword)
///     .build()
///     .unwrap();
///
/// println!("{:?}", connack);
/// // Output: {"type":"CONNACK","session_present":false,"reason_code":"BadUserNameOrPassword","props":{}}
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Debug
    for GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
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
/// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
///
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(false)
///     .reason_code(ConnectReasonCode::Success)
///     .build()
///     .unwrap();
///
/// // Use generic trait methods
/// let size = connack.size();
/// let buffers = connack.to_buffers();
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> GenericPacketTrait
    for GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn size(&self) -> usize {
        self.size()
    }

    #[cfg(feature = "std")]
    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }

    fn to_continuous_buffer(&self) -> Vec<u8> {
        self.to_continuous_buffer()
    }
}

/// Implementation of `GenericPacketDisplay` for generic formatting operations
///
/// Provides a common interface for formatting operations that can be used generically
/// across different MQTT packet types. This enables uniform display handling in
/// collections and generic contexts.
///
/// # Methods
///
/// - `fmt_debug()`: Delegates to the Debug trait implementation
/// - `fmt_display()`: Delegates to the Display trait implementation
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketDisplay;
/// use mqtt_protocol_core::mqtt::result_code::ConnectReasonCode;
///
/// let connack = mqtt::packet::v5_0::Connack::builder()
///     .session_present(false)
///     .reason_code(ConnectReasonCode::Success)
///     .build()
///     .unwrap();
///
/// // Use generic display methods
/// println!("{}", format_args!("{}", connack)); // Uses fmt_display
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> GenericPacketDisplay
    for GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

/// Validate CONNACK packet properties according to MQTT 5.0 specification
///
/// This function ensures that the properties included in a CONNACK packet conform
/// to the MQTT 5.0 protocol requirements. It validates:
///
/// 1. **Property types**: Only properties valid for CONNACK packets are allowed
/// 2. **Property uniqueness**: Most properties can appear at most once (except User Properties)
/// 3. **Protocol compliance**: Ensures the packet follows MQTT 5.0 specification
///
/// # Valid CONNACK Properties
///
/// The following properties are allowed in CONNACK packets:
/// - `SessionExpiryInterval`: Session expiry interval (max 1)
/// - `ReceiveMaximum`: Maximum number of QoS 1 and 2 publications (max 1)
/// - `MaximumQos`: Maximum QoS level supported by the server (max 1)
/// - `RetainAvailable`: Whether the server supports retained messages (max 1)
/// - `MaximumPacketSize`: Maximum packet size accepted by the server (max 1)
/// - `AssignedClientIdentifier`: Client identifier assigned by the server (max 1)
/// - `TopicAliasMaximum`: Maximum topic alias value (max 1)
/// - `ReasonString`: Human-readable reason for the connection result (max 1)
/// - `UserProperty`: Key-value pairs for application-specific data (multiple allowed)
/// - Plus several others defined by the MQTT 5.0 specification
///
/// # Parameters
///
/// * `props` - Slice of properties to validate
///
/// # Returns
///
/// * `Ok(())` - All properties are valid and conform to specification
/// * `Err(MqttError::ProtocolError)` - Invalid property type or duplicate property
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::Property;
///
/// let mut props = mqtt::packet::Properties::new();
/// props.push(Property::SessionExpiryInterval(3600));
/// props.push(Property::ReceiveMaximum(100));
///
/// // This validation is automatically called during packet construction
/// // validate_connack_properties(&props).unwrap();
/// ```
fn validate_connack_properties<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>(
    props: &GenericProperties<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
) -> Result<(), MqttError> {
    let mut count_session_expiry_interval = 0;
    let mut count_receive_maximum = 0;
    let mut count_maximum_qos = 0;
    let mut count_retain_available = 0;
    let mut count_maximum_packet_size = 0;
    let mut count_assigned_client_identifier = 0;
    let mut count_topic_alias_maximum = 0;
    let mut count_reason_string = 0;
    for prop in props {
        match prop {
            GenericProperty::SessionExpiryInterval(_) => count_session_expiry_interval += 1,
            GenericProperty::ReceiveMaximum(_) => count_receive_maximum += 1,
            GenericProperty::MaximumQos(_) => count_maximum_qos += 1,
            GenericProperty::RetainAvailable(_) => count_retain_available += 1,
            GenericProperty::MaximumPacketSize(_) => count_maximum_packet_size += 1,
            GenericProperty::AssignedClientIdentifier(_) => count_assigned_client_identifier += 1,
            GenericProperty::TopicAliasMaximum(_) => count_topic_alias_maximum += 1,
            GenericProperty::ReasonString(_) => count_reason_string += 1,
            GenericProperty::UserProperty(_) => {}
            _ => return Err(MqttError::ProtocolError),
        }
    }
    if count_session_expiry_interval > 1
        || count_receive_maximum > 1
        || count_maximum_qos > 1
        || count_retain_available > 1
        || count_maximum_packet_size > 1
        || count_assigned_client_identifier > 1
        || count_topic_alias_maximum > 1
        || count_reason_string > 1
    {
        return Err(MqttError::ProtocolError);
    }
    Ok(())
}
