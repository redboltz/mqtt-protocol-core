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
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::result_code::MqttError;

/// MQTT v3.1.1 UNSUBACK packet representation with generic packet ID support
///
/// The UNSUBACK packet is sent by the MQTT server (broker) in response to an UNSUBSCRIBE packet
/// from a client. It confirms that the client has been unsubscribed from the specified topic
/// filters. In MQTT v3.1.1, the UNSUBACK packet contains only a packet identifier and no
/// reason codes or properties.
///
/// According to the MQTT v3.1.1 specification (Section 3.11), the UNSUBACK packet contains:
/// - Fixed header with packet type (0xB0) and remaining length
/// - Variable header with packet identifier (2 bytes)
/// - No payload
///
/// # Packet Structure
///
/// ```text
/// UNSUBACK Packet Structure (MQTT v3.1.1):
/// +----------------+
/// | Fixed Header   |  - Packet type (0xB0) and remaining length
/// +----------------+
/// | Packet ID      |  - 2 bytes (or PacketIdType::Buffer size)
/// +----------------+
/// ```
///
/// # Protocol Differences from MQTT 5.0
///
/// Unlike MQTT 5.0, the v3.1.1 UNSUBACK packet:
/// - Contains no reason codes (unsubscription is always considered successful)
/// - Contains no properties
/// - Has a simpler structure with only packet identifier
/// - Always indicates successful unsubscription from all requested topic filters
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
///
/// // Create an UNSUBACK packet for a specific UNSUBSCRIBE request
/// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
///     .packet_id(42u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(unsuback.packet_id(), 42);
///
/// // Parse UNSUBACK from network data
/// let data = &[0x00, 0x2A]; // packet_id = 42
/// let (parsed_unsuback, consumed) = mqtt::packet::v3_1_1::Unsuback::parse(data).unwrap();
/// assert_eq!(parsed_unsuback.packet_id(), 42);
/// assert_eq!(consumed, 2);
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
}

/// Standard MQTT v3.1.1 UNSUBACK packet with 16-bit packet IDs
///
/// This is a type alias for `GenericUnsuback<u16>` that provides the standard MQTT v3.1.1
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
///
/// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
///     .packet_id(42u16)
///     .build()
///     .unwrap();
/// ```

impl<PacketIdType> GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Create a new GenericUnsubackBuilder for constructing UNSUBACK packets
    ///
    /// Returns a builder instance that allows setting the packet identifier for an UNSUBACK packet
    /// in a fluent interface style. The builder ensures the required packet ID field is set before
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
    ///
    /// let unsuback = mqtt::packet::v3_1_1::GenericUnsuback::<u16>::builder()
    ///     .packet_id(42u16)
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
    /// assert_eq!(mqtt::packet::v3_1_1::Unsuback::packet_type(), PacketType::Unsuback);
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
    ///
    /// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
    ///     .packet_id(1234u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(unsuback.packet_id(), 1234);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Calculate the total size of the UNSUBACK packet in bytes
    ///
    /// Returns the total number of bytes required to represent this UNSUBACK packet
    /// when serialized for network transmission. This includes the fixed header
    /// and the packet identifier (2 bytes for standard u16 packet IDs).
    ///
    /// # Returns
    ///
    /// The total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = unsuback.size();
    /// assert!(size > 0);
    /// // For standard u16 packet IDs: 1 (fixed header) + 1 (remaining length) + 2 (packet ID) = 4 bytes
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
    ///
    /// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
    ///     .packet_id(42u16)
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

        bufs
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is an alternative to [`to_buffers()`] and is compatible with
    /// no-std environments where vectored I/O may not be available.
    ///
    /// The returned buffer contains the complete UNSUBACK packet serialized according
    /// to the MQTT v3.1.1 protocol specification, including fixed header, remaining
    /// length, and packet identifier.
    ///
    /// # Returns
    ///
    /// A vector containing the complete packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
    ///     .packet_id(42u16)
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
        buf
    }

    /// Parse an UNSUBACK packet from raw bytes
    ///
    /// Parses the variable header of an UNSUBACK packet from the provided byte buffer.
    /// The fixed header should already be parsed before calling this method.
    /// In MQTT v3.1.1, the UNSUBACK packet only contains a packet identifier in the variable header.
    ///
    /// # Parameters
    ///
    /// * `data` - The raw bytes containing the UNSUBACK packet variable header
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The parsed `GenericUnsuback` instance
    /// - The number of bytes consumed during parsing
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - The packet is malformed (insufficient bytes for packet identifier)
    /// - The packet identifier is zero (invalid according to MQTT specification)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse UNSUBACK packet from network data
    /// let data = &[0x00, 0x10]; // packet_id = 16
    /// let (unsuback, consumed) = mqtt::packet::v3_1_1::Unsuback::parse(data).unwrap();
    ///
    /// assert_eq!(unsuback.packet_id(), 16);
    /// assert_eq!(consumed, 2);
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

        let remaining_size = buffer_size;
        let remaining_length = VariableByteInteger::from_u32(remaining_size as u32).unwrap();

        let unsuback = GenericUnsuback {
            fixed_header: [FixedHeader::Unsuback as u8],
            remaining_length,
            packet_id_buf,
        };

        Ok((unsuback, cursor))
    }
}

/// Builder implementation for `GenericUnsuback`
///
/// Provides a fluent interface for constructing UNSUBACK packets with proper validation.
/// The builder ensures the required packet identifier field is set and validates the
/// packet structure before creating the final packet instance.
impl<PacketIdType> GenericUnsubackBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Set the packet identifier for the UNSUBACK packet
    ///
    /// The packet identifier must match the packet identifier from the original
    /// UNSUBSCRIBE packet that this UNSUBACK is responding to. The packet identifier
    /// cannot be zero according to MQTT v3.1.1 specification.
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
    ///
    /// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Validate the builder state before constructing the packet
    ///
    /// Performs validation of the builder fields to ensure the resulting UNSUBACK packet
    /// will be valid according to MQTT v3.1.1 specification.
    ///
    /// # Validation Rules
    ///
    /// - Packet identifier must be set and non-zero
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - Missing or invalid packet identifier
    fn validate(&self) -> Result<(), MqttError> {
        if self.packet_id_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }

        let packet_id_bytes = self.packet_id_buf.as_ref().unwrap().as_ref();
        let all_zeros = packet_id_bytes.iter().all(|&b| b == 0);
        if all_zeros {
            return Err(MqttError::MalformedPacket);
        }

        Ok(())
    }

    /// Build the final UNSUBACK packet
    ///
    /// Validates the builder fields and constructs the final `GenericUnsuback` instance.
    /// This method consumes the builder and returns either a valid UNSUBACK packet
    /// or an error if validation fails.
    ///
    /// The method automatically calculates the remaining length based on the packet identifier size.
    /// For standard u16 packet IDs, the remaining length is always 2 bytes.
    ///
    /// # Returns
    ///
    /// `Ok(GenericUnsuback<PacketIdType>)` containing the constructed packet,
    /// or `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - Missing or invalid packet identifier
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(unsuback.packet_id(), 42);
    /// ```
    pub fn build(self) -> Result<GenericUnsuback<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let remaining_length = VariableByteInteger::from_u32(2).unwrap(); // packet_id(2)

        Ok(GenericUnsuback {
            fixed_header: [FixedHeader::Unsuback.as_u8()],
            remaining_length,
            packet_id_buf,
        })
    }
}

/// Display trait implementation for GenericUnsuback
///
/// Provides a human-readable JSON representation of the UNSUBACK packet.
/// The display format includes the packet type and packet ID.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
///     .packet_id(42u16)
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
///
/// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
///     .packet_id(42u16)
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
/// includes the packet type and packet ID.
///
/// The serialized structure contains:
/// - `type`: Always "unsuback"
/// - `packet_id`: The packet identifier
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
///     .packet_id(42u16)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&unsuback).unwrap();
/// // json contains: {"type":"unsuback","packet_id":42}
/// ```
impl<PacketIdType> Serialize for GenericUnsuback<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = 2; // type and packet_id are always present

        let mut state = serializer.serialize_struct("Unsuback", field_count)?;

        state.serialize_field("type", "unsuback")?;
        state.serialize_field("packet_id", &self.packet_id())?;

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
///
/// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
///     .packet_id(42u16)
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
///
/// let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
///     .packet_id(42u16)
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
