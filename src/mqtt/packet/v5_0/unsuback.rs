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
use core::mem;
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
use crate::mqtt::packet::IsPacketId;
#[cfg(feature = "std")]
use crate::mqtt::packet::PropertiesToBuffers;
use crate::mqtt::packet::{Properties, PropertiesParse, PropertiesSize, Property};
use crate::mqtt::result_code::MqttError;
use crate::mqtt::result_code::UnsubackReasonCode;

/// MQTT 5.0 UNSUBACK packet representation with generic packet ID support
///
/// The UNSUBACK packet is sent by the MQTT server (broker) in response to an UNSUBSCRIBE packet
/// from a client. It indicates the result of each unsubscription request and confirms that
/// the client has been unsubscribed from the specified topic filters.
///
/// According to MQTT 5.0 specification, the UNSUBACK packet contains:
/// - Fixed header with packet type and remaining length
/// - Variable header with packet identifier, properties, and reason codes
/// - No payload
///
/// # Packet Structure
///
/// ```text
/// UNSUBACK Packet Structure:
/// +----------------+
/// | Fixed Header   |  - Packet type (0xB0) and remaining length
/// +----------------+
/// | Packet ID      |  - 2 bytes (or PacketIdType::Buffer size)
/// +----------------+
/// | Properties     |  - Property length + properties
/// +----------------+
/// | Reason Codes   |  - One or more reason codes (1 byte each)
/// +----------------+
/// ```
///
/// # Reason Codes
///
/// Each reason code in the UNSUBACK packet corresponds to a topic filter in the original
/// UNSUBSCRIBE packet and indicates the result of the unsubscription request:
///
/// **Success codes:**
/// - `0x00` Success - The unsubscription was successful
/// - `0x11` No subscription existed - No matching subscription found
///
/// **Error codes:**
/// - `0x80` Unspecified error - An unspecified error occurred
/// - `0x83` Implementation specific error - Server implementation specific error
/// - `0x87` Not authorized - The client is not authorized to unsubscribe
/// - `0x8F` Topic filter invalid - The topic filter is malformed
/// - `0x91` Packet identifier in use - The packet identifier is already in use
///
/// # Properties
///
/// MQTT 5.0 UNSUBACK packets can include the following properties:
/// - **Reason String**: Human readable string for diagnostic purposes
/// - **User Properties**: Application-specific name-value pairs
///
/// Only one Reason String property is allowed per packet.
///
/// # Generic Packet ID Support
///
/// This implementation supports generic packet ID types through the `PacketIdType` parameter.
/// While MQTT specification uses 16-bit packet IDs, this allows for extended packet IDs
/// (e.g., 32-bit) for broker cluster implementations to prevent packet ID exhaustion.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
///
/// // Create an UNSUBACK with successful unsubscriptions
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(42u16)
///     .reason_codes(vec![
///         UnsubackReasonCode::Success,
///         UnsubackReasonCode::NoSubscriptionExisted,
///     ])
///     .build()
///     .unwrap();
///
/// assert_eq!(unsuback.packet_id(), 42);
/// assert_eq!(unsuback.reason_codes().len(), 2);
///
/// // Create UNSUBACK with mixed success and error codes
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(100u16)
///     .reason_codes(vec![
///         UnsubackReasonCode::Success,
///         UnsubackReasonCode::NotAuthorized,
///         UnsubackReasonCode::TopicFilterInvalid,
///     ])
///     .build()
///     .unwrap();
///
/// // Add properties for diagnostics
/// let mut props = mqtt::packet::Properties::new();
/// // props.add(mqtt::packet::Property::ReasonString("Partial success".to_string()));
///
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(200u16)
///     .reason_codes(vec![UnsubackReasonCode::Success])
///     .props(props)
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = unsuback.to_buffers();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    packet_id_buf: PacketIdType::Buffer,
    #[builder(private)]
    property_length: VariableByteInteger,

    /// MQTT 5.0 properties for the UNSUBACK packet
    ///
    /// Contains optional properties like reason string and user properties.
    /// The properties are validated to ensure only allowed properties are included.
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Properties,

    #[builder(private)]
    reason_codes_buf: Vec<u8>,
}

/// Standard MQTT 5.0 UNSUBACK packet with 16-bit packet IDs
///
/// This is a type alias for `GenericUnsuback<u16>` that provides the standard MQTT 5.0
/// UNSUBACK packet implementation using 16-bit packet identifiers as defined in the
/// MQTT specification.
///
/// Most applications should use this type unless they specifically need extended
/// packet ID support for broker cluster implementations.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
///
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(42u16)
///     .reason_codes(vec![UnsubackReasonCode::Success])
///     .build()
///     .unwrap();
/// ```
pub type Unsuback = GenericUnsuback<u16>;

impl<PacketIdType> GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Create a new GenericUnsubackBuilder for constructing UNSUBACK packets
    ///
    /// Returns a builder instance that allows setting the various fields of an UNSUBACK packet
    /// in a fluent interface style. The builder ensures all required fields are set before
    /// creating the final packet.
    ///
    /// # Returns
    ///
    /// A new `GenericUnsubackBuilder<PacketIdType>` instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::GenericUnsuback::<u16>::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![UnsubackReasonCode::Success])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericUnsubackBuilder<PacketIdType> {
        GenericUnsubackBuilder::<PacketIdType>::default()
    }

    /// Get the packet type for UNSUBACK packets
    ///
    /// Returns the constant packet type identifier for UNSUBACK packets.
    /// This is always `PacketType::Unsuback` for UNSUBACK packets.
    ///
    /// # Returns
    ///
    /// The packet type `PacketType::Unsuback`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::packet_type::PacketType;
    ///
    /// assert_eq!(mqtt::packet::v5_0::Unsuback::packet_type(), PacketType::Unsuback);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Unsuback
    }

    /// Get the packet identifier from the UNSUBACK packet
    ///
    /// Returns the packet identifier that matches the UNSUBSCRIBE packet this UNSUBACK
    /// is responding to. The packet identifier is used to correlate the UNSUBACK
    /// with the original UNSUBSCRIBE request.
    ///
    /// # Returns
    ///
    /// The packet identifier as `PacketIdType`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(1234u16)
    ///     .reason_codes(vec![UnsubackReasonCode::Success])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(unsuback.packet_id(), 1234);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Get the reason codes from the UNSUBACK packet
    ///
    /// Returns a vector of reason codes indicating the result of each unsubscription
    /// request in the original UNSUBSCRIBE packet. Each reason code corresponds to
    /// a topic filter in the UNSUBSCRIBE packet, in the same order.
    ///
    /// Invalid reason code bytes are converted to `UnsubackReasonCode::UnspecifiedError`
    /// to maintain packet integrity.
    ///
    /// # Returns
    ///
    /// A `Vec<UnsubackReasonCode>` containing the unsubscription results
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![
    ///         UnsubackReasonCode::Success,
    ///         UnsubackReasonCode::NotAuthorized,
    ///         UnsubackReasonCode::NoSubscriptionExisted,
    ///     ])
    ///     .build()
    ///     .unwrap();
    ///
    /// let codes = unsuback.reason_codes();
    /// assert_eq!(codes.len(), 3);
    /// assert_eq!(codes[0], UnsubackReasonCode::Success);
    /// assert_eq!(codes[1], UnsubackReasonCode::NotAuthorized);
    /// assert_eq!(codes[2], UnsubackReasonCode::NoSubscriptionExisted);
    /// ```
    pub fn reason_codes(&self) -> Vec<UnsubackReasonCode> {
        self.reason_codes_buf
            .iter()
            .map(|&byte| {
                UnsubackReasonCode::try_from(byte).unwrap_or(UnsubackReasonCode::UnspecifiedError)
            })
            .collect()
    }

    /// Parse an UNSUBACK packet from raw bytes
    ///
    /// Parses the variable header and payload of an UNSUBACK packet from the provided
    /// byte buffer. The fixed header should already be parsed before calling this method.
    ///
    /// # Parameters
    ///
    /// * `data` - The raw bytes containing the UNSUBACK packet variable header and payload
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The parsed `GenericUnsuback` instance
    /// - The number of bytes consumed during parsing
    ///
    /// # Errors
    ///
    /// Returns `MqttError` if:
    /// - The packet is malformed (insufficient bytes, invalid packet ID, invalid reason codes)
    /// - The packet violates protocol rules (no reason codes provided)
    /// - Properties contain invalid or disallowed property types
    /// - Multiple Reason String properties are present
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse UNSUBACK packet from network data
    /// let data = &[0x00, 0x10, 0x00, 0x00]; // packet_id=16, no properties, success code
    /// let (unsuback, consumed) = mqtt::packet::v5_0::Unsuback::parse(data).unwrap();
    ///
    /// assert_eq!(unsuback.packet_id(), 16);
    /// assert_eq!(consumed, 4);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;
        let buffer_size = mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();

        if data.len() < buffer_size {
            return Err(MqttError::MalformedPacket);
        }

        let packet_id = PacketIdType::from_buffer(&data[0..buffer_size]);
        let packet_id_buf = packet_id.to_buffer();
        cursor += buffer_size;

        let (props, property_length) = Properties::parse(&data[cursor..])?;
        cursor += property_length;
        validate_unsuback_properties(&props)?;
        let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

        let mut reason_codes_buf = Vec::new();
        while cursor < data.len() {
            let _reason_code = UnsubackReasonCode::try_from(data[cursor])
                .map_err(|_| MqttError::MalformedPacket)?;
            reason_codes_buf.push(data[cursor]);
            cursor += 1;
        }

        if reason_codes_buf.is_empty() {
            return Err(MqttError::ProtocolError);
        }

        let remaining_size = buffer_size + property_length + reason_codes_buf.len();
        let remaining_length = VariableByteInteger::from_u32(remaining_size as u32).unwrap();

        let unsuback = GenericUnsuback {
            fixed_header: [FixedHeader::Unsuback as u8],
            remaining_length,
            packet_id_buf,
            property_length: prop_len,
            props,
            reason_codes_buf,
        };

        Ok((unsuback, cursor))
    }

    /// Calculate the total size of the UNSUBACK packet in bytes
    ///
    /// Returns the total number of bytes required to represent this UNSUBACK packet
    /// when serialized for network transmission. This includes the fixed header,
    /// variable header, and all reason codes.
    ///
    /// # Returns
    ///
    /// The total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![UnsubackReasonCode::Success])
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = unsuback.size();
    /// assert!(size > 0);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// represent the complete UNSUBACK packet in wire format.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` objects for vectored I/O operations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![UnsubackReasonCode::Success])
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = unsuback.to_buffers();
    /// // Use with vectored write: socket.write_vectored(&buffers)?;
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(self.packet_id_buf.as_ref()));
        bufs.push(IoSlice::new(self.property_length.as_bytes()));
        bufs.extend(self.props.to_buffers());

        if !self.reason_codes_buf.is_empty() {
            bufs.push(IoSlice::new(&self.reason_codes_buf));
        }

        bufs
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method provides an alternative to `to_buffers()` for no-std environments
    /// where vectored I/O is not available.
    ///
    /// The returned buffer contains the complete UNSUBACK packet serialized according
    /// to the MQTT v5.0 protocol specification, including fixed header, remaining
    /// length, packet identifier, properties, and reason codes.
    ///
    /// # Returns
    ///
    /// A vector containing the complete packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![UnsubackReasonCode::Success])
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = unsuback.to_continuous_buffer();
    /// // buffer contains all packet bytes
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.extend_from_slice(self.packet_id_buf.as_ref());
        buf.extend_from_slice(self.property_length.as_bytes());
        buf.append(&mut self.props.to_continuous_buffer());

        if !self.reason_codes_buf.is_empty() {
            buf.extend_from_slice(&self.reason_codes_buf);
        }

        buf
    }
}

/// Builder implementation for `GenericUnsuback`
///
/// Provides a fluent interface for constructing UNSUBACK packets with proper validation.
/// The builder ensures all required fields are set and validates the packet structure
/// before creating the final packet instance.
impl<PacketIdType> GenericUnsubackBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Set the packet identifier for the UNSUBACK packet
    ///
    /// The packet identifier must match the packet identifier from the original
    /// UNSUBSCRIBE packet that this UNSUBACK is responding to. The packet identifier
    /// cannot be zero.
    ///
    /// # Parameters
    ///
    /// * `id` - The packet identifier of type `PacketIdType`
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![UnsubackReasonCode::Success])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Set the reason codes for the UNSUBACK packet
    ///
    /// The reason codes indicate the result of each unsubscription request in the original
    /// UNSUBSCRIBE packet. Each reason code corresponds to a topic filter in the UNSUBSCRIBE
    /// packet, in the same order. At least one reason code must be provided.
    ///
    /// # Parameters
    ///
    /// * `codes` - A vector of `UnsubackReasonCode` values indicating unsubscription results
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![
    ///         UnsubackReasonCode::Success,
    ///         UnsubackReasonCode::NotAuthorized,
    ///         UnsubackReasonCode::NoSubscriptionExisted,
    ///     ])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn reason_codes(mut self, codes: Vec<UnsubackReasonCode>) -> Self {
        let reason_codes_buf: Vec<u8> = codes.iter().map(|&rc| rc as u8).collect();
        self.reason_codes_buf = Some(reason_codes_buf);
        self
    }

    /// Validate the builder state before constructing the packet
    ///
    /// Performs comprehensive validation of all builder fields to ensure the
    /// resulting UNSUBACK packet will be valid according to MQTT 5.0 specification.
    ///
    /// # Validation Rules
    ///
    /// - Packet identifier must be set and non-zero
    /// - At least one reason code must be provided
    /// - Properties must only contain allowed property types (ReasonString, UserProperty)
    /// - Only one ReasonString property is allowed
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - Missing or invalid packet identifier
    /// - `MqttError::ProtocolError` - Missing reason codes or invalid properties
    fn validate(&self) -> Result<(), MqttError> {
        if self.packet_id_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }

        let packet_id_bytes = self.packet_id_buf.as_ref().unwrap().as_ref();
        let all_zeros = packet_id_bytes.iter().all(|&b| b == 0);
        if all_zeros {
            return Err(MqttError::MalformedPacket);
        }

        if self
            .reason_codes_buf
            .as_ref()
            .map_or(true, |r| r.is_empty())
        {
            return Err(MqttError::ProtocolError);
        }

        if let Some(ref props) = self.props {
            validate_unsuback_properties(props)?;
        }

        Ok(())
    }

    /// Build the final UNSUBACK packet
    ///
    /// Validates all builder fields and constructs the final `GenericUnsuback` instance.
    /// This method consumes the builder and returns either a valid UNSUBACK packet
    /// or an error if validation fails.
    ///
    /// The method automatically calculates the remaining length and property length
    /// fields based on the provided data.
    ///
    /// # Returns
    ///
    /// `Ok(GenericUnsuback<PacketIdType>)` containing the constructed packet,
    /// or `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - Missing or invalid packet identifier
    /// - `MqttError::ProtocolError` - Missing reason codes or invalid properties
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
    ///
    /// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .reason_codes(vec![UnsubackReasonCode::Success])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(unsuback.packet_id(), 42);
    /// ```
    pub fn build(self) -> Result<GenericUnsuback<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let reason_codes_buf = self.reason_codes_buf.unwrap_or_default();

        let props = self.props.unwrap_or_else(Properties::new);
        let props_size = props.size();
        let property_length = VariableByteInteger::from_u32(props_size as u32).unwrap();

        let packet_id_size = mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
        let prop_len_size = property_length.size();
        let reason_codes_size = reason_codes_buf.len();

        let remaining = packet_id_size + prop_len_size + props_size + reason_codes_size;
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericUnsuback {
            fixed_header: [FixedHeader::Unsuback as u8],
            remaining_length,
            packet_id_buf,
            property_length,
            props,
            reason_codes_buf,
        })
    }
}

/// Display trait implementation for GenericUnsuback
///
/// Provides a human-readable JSON representation of the UNSUBACK packet.
/// The display format includes the packet type, packet ID, properties (if any),
/// and reason codes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
///
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(42u16)
///     .reason_codes(vec![UnsubackReasonCode::Success])
///     .build()
///     .unwrap();
///
/// println!("{}", unsuback); // Prints JSON representation
/// ```
impl<PacketIdType> fmt::Display for GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Debug trait implementation for GenericUnsuback
///
/// Provides a debug representation of the UNSUBACK packet using the same JSON format
/// as the Display trait. This ensures consistent output for logging and debugging.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
///
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(42u16)
///     .reason_codes(vec![UnsubackReasonCode::Success])
///     .build()
///     .unwrap();
///
/// println!("{:?}", unsuback); // Prints JSON representation
/// ```
impl<PacketIdType> fmt::Debug for GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Serialize trait implementation for GenericUnsuback
///
/// Provides JSON serialization support for UNSUBACK packets. The serialized format
/// includes the packet type, packet ID, properties (if any), and reason codes.
///
/// The serialized structure contains:
/// - `type`: Always "unsuback"
/// - `packet_id`: The packet identifier
/// - `props`: Properties object (only if non-empty)
/// - `reason_codes`: Array of reason codes (only if non-empty)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
///
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(42u16)
///     .reason_codes(vec![UnsubackReasonCode::Success])
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&unsuback).unwrap();
/// // json contains: {"type":"unsuback","packet_id":42,"reason_codes":["Success"]}
/// ```
impl<PacketIdType> Serialize for GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 2; // type and packet_id are always present

        if !self.props.is_empty() {
            field_count += 1;
        }

        if !self.reason_codes_buf.is_empty() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("Unsuback", field_count)?;

        state.serialize_field("type", "unsuback")?;
        state.serialize_field("packet_id", &self.packet_id())?;

        if !self.props.is_empty() {
            state.serialize_field("props", &self.props)?;
        }

        if !self.reason_codes_buf.is_empty() {
            state.serialize_field("reason_codes", &self.reason_codes())?;
        }

        state.end()
    }
}

/// GenericPacketTrait implementation for GenericUnsuback
///
/// Provides the standard packet interface methods for UNSUBACK packets.
/// This trait allows UNSUBACK packets to be used polymorphically with other
/// MQTT packet types.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;
/// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
///
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(42u16)
///     .reason_codes(vec![UnsubackReasonCode::Success])
///     .build()
///     .unwrap();
///
/// // Use trait methods
/// let size = unsuback.size();
/// let buffers = unsuback.to_buffers();
/// ```
impl<PacketIdType> GenericPacketTrait for GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId,
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

/// GenericPacketDisplay implementation for GenericUnsuback
///
/// Provides standardized display formatting for UNSUBACK packets through the
/// GenericPacketDisplay trait. This allows consistent formatting across
/// different packet types in the library.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketDisplay;
/// use mqtt_protocol_core::mqtt::result_code::UnsubackReasonCode;
///
/// let unsuback = mqtt::packet::v5_0::Unsuback::builder()
///     .packet_id(42u16)
///     .reason_codes(vec![UnsubackReasonCode::Success])
///     .build()
///     .unwrap();
///
/// // Use trait methods for consistent formatting
/// println!("{}", format_args!("{}", unsuback));
/// ```
impl<PacketIdType> GenericPacketDisplay for GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

/// Validate UNSUBACK packet properties according to MQTT 5.0 specification
///
/// Ensures that only allowed properties are present in the UNSUBACK packet and that
/// property constraints are met. According to MQTT 5.0, UNSUBACK packets may only
/// contain Reason String and User Property properties.
///
/// # Validation Rules
///
/// - Only `Property::ReasonString` and `Property::UserProperty` are allowed
/// - At most one `Property::ReasonString` property is permitted
/// - Multiple `Property::UserProperty` properties are allowed
/// - All other property types result in a protocol error
///
/// # Parameters
///
/// * `props` - The properties to validate
///
/// # Returns
///
/// `Ok(())` if validation passes, `Err(MqttError::ProtocolError)` if validation fails
///
/// # Errors
///
/// - `MqttError::ProtocolError` - Invalid property type or multiple Reason String properties
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::{Properties, Property};
///
/// let mut props = Properties::new();
/// props.push(Property::ReasonString("Unsubscription successful".to_string()));
/// props.push(Property::UserProperty(("client".to_string(), "info".to_string())));
///
/// // This would be called internally during packet validation
/// // validate_unsuback_properties(&props).unwrap();
/// ```
fn validate_unsuback_properties(props: &Properties) -> Result<(), MqttError> {
    let mut count_reason_string = 0;
    for prop in props {
        match prop {
            Property::ReasonString(_) => count_reason_string += 1,
            Property::UserProperty(_) => {}
            _ => return Err(MqttError::ProtocolError),
        }
    }
    if count_reason_string > 1 {
        return Err(MqttError::ProtocolError);
    }

    Ok(())
}
