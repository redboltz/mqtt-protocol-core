use alloc::vec::Vec;
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
use crate::mqtt::packet::SubEntry;
use crate::mqtt::result_code::MqttError;

/// MQTT 3.1.1 SUBSCRIBE packet representation
///
/// The SUBSCRIBE packet is sent by a client to subscribe to one or more topic filters
/// on the server. Each subscription establishes a flow of messages from the server to
/// the client based on the matching topic filters and their associated Quality of Service (QoS) levels.
///
/// According to MQTT 3.1.1 specification section 3.8, the SUBSCRIBE packet contains:
/// - Fixed header with packet type (1000), reserved flags (0010), and remaining length
/// - Variable header with packet identifier
/// - Payload containing one or more topic filter entries with QoS levels
///
/// # Fixed Header
///
/// The SUBSCRIBE packet uses a fixed header with:
/// - **Packet Type**: 8 (1000 binary) in bits 7-4 of the first byte
/// - **Reserved Flags**: 2 (0010 binary) in bits 3-0 - these flags are reserved and must be set as shown
/// - **Remaining Length**: Variable length encoding of the remaining packet data
///
/// # Variable Header
///
/// The variable header contains:
/// - **Packet Identifier**: A 16-bit identifier used to match SUBSCRIBE with SUBACK packets
///
/// # Payload
///
/// The payload contains one or more subscription entries, each consisting of:
/// - **Topic Filter**: UTF-8 encoded string that may contain wildcards (+ and #)
/// - **QoS Level**: Single byte indicating the maximum QoS level for messages (0, 1, or 2)
///
/// # Quality of Service (QoS)
///
/// Each subscription specifies a maximum QoS level:
/// - **QoS 0**: At most once delivery
/// - **QoS 1**: At least once delivery
/// - **QoS 2**: Exactly once delivery
///
/// # Topic Filters and Wildcards
///
/// Topic filters can include wildcards:
/// - **Single-level wildcard (+)**: Matches exactly one topic level (e.g., "sport/+/player1")
/// - **Multi-level wildcard (#)**: Matches any number of topic levels (e.g., "sport/#")
///
/// # Protocol Differences from MQTT 5.0
///
/// MQTT 3.1.1 SUBSCRIBE packets differ from MQTT 5.0 in several ways:
/// - No properties section in the variable header
/// - Subscription options are limited to QoS level only
/// - No subscription identifiers, user properties, or advanced subscription options
/// - Simpler packet structure with fewer configuration options
///
/// # Generic Type Parameter
///
/// The `PacketIdType` generic parameter allows using packet identifiers larger than
/// the standard u16, which can be useful for broker clusters to avoid packet ID
/// exhaustion when extending the MQTT protocol.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// // Create a simple SUBSCRIBE packet for a single topic
/// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
///     .packet_id(42)
///     .entries(vec![
///         SubEntry::builder()
///             .topic_filter("sensors/temperature")
///             .unwrap()
///             .qos(Qos::AtLeastOnce)
///             .build()
///             .unwrap()
///     ])
///     .build()
///     .unwrap();
///
/// assert_eq!(subscribe.packet_id(), 42);
/// assert_eq!(subscribe.entries().len(), 1);
/// assert_eq!(subscribe.entries()[0].topic_filter(), "sensors/temperature");
///
/// // Create a SUBSCRIBE packet with multiple topics and different QoS levels
/// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
///     .packet_id(123)
///     .entries(vec![
///         SubEntry::builder()
///             .topic_filter("home/+/temperature")
///             .unwrap()
///             .qos(Qos::AtMostOnce)
///             .build()
///             .unwrap(),
///         SubEntry::builder()
///             .topic_filter("alerts/#")
///             .unwrap()
///             .qos(Qos::ExactlyOnce)
///             .build()
///             .unwrap()
///     ])
///     .build()
///     .unwrap();
///
/// assert_eq!(subscribe.packet_id(), 123);
/// assert_eq!(subscribe.entries().len(), 2);
///
/// // Serialize to bytes for network transmission
/// let buffers = subscribe.to_buffers();
/// let total_size = subscribe.size();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericSubscribe<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    packet_id_buf: PacketIdType::Buffer,

    #[getset(get = "pub")]
    entries: Vec<SubEntry>,
}

/// Type alias for SUBSCRIBE packet with standard u16 packet identifiers
///
/// This is the most commonly used SUBSCRIBE packet type for standard MQTT 3.1.1
/// implementations that use 16-bit packet identifiers as specified in the protocol.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
///     .packet_id(1)
///     .entries(vec![
///         SubEntry::builder()
///             .topic_filter("my/topic")
///             .unwrap()
///             .qos(Qos::AtLeastOnce)
///             .build()
///             .unwrap()
///     ])
///     .build()
///     .unwrap();
/// ```
pub type Subscribe = GenericSubscribe<u16>;

impl<PacketIdType> GenericSubscribe<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a SUBSCRIBE packet
    ///
    /// The builder pattern allows for flexible construction of SUBSCRIBE packets
    /// with various combinations of topic filters and QoS levels. All SUBSCRIBE
    /// packets must have a packet identifier and at least one subscription entry.
    ///
    /// # Returns
    ///
    /// A `GenericSubscribeBuilder` instance with default values
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    /// use mqtt_protocol_core::mqtt::packet::SubEntry;
    ///
    /// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
    ///     .packet_id(42)
    ///     .entries(vec![
    ///         SubEntry::builder()
    ///             .topic_filter("sensors/+")
    ///             .unwrap()
    ///             .qos(Qos::AtLeastOnce)
    ///             .build()
    ///             .unwrap()
    ///     ])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericSubscribeBuilder<PacketIdType> {
        GenericSubscribeBuilder::<PacketIdType>::default()
    }

    /// Returns the packet type for SUBSCRIBE packets
    ///
    /// This is always `PacketType::Subscribe` for SUBSCRIBE packet instances.
    /// The numeric value is 8, represented as 1000 in the upper 4 bits of the
    /// fixed header's first byte.
    ///
    /// # Returns
    ///
    /// `PacketType::Subscribe`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::packet_type::PacketType;
    ///
    /// assert_eq!(mqtt::packet::v3_1_1::Subscribe::packet_type(), PacketType::Subscribe);
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Subscribe
    }

    /// Returns the packet identifier for this SUBSCRIBE packet
    ///
    /// The packet identifier is used to match SUBSCRIBE packets with their corresponding
    /// SUBACK responses. It must be non-zero as specified in the MQTT protocol.
    /// The same packet identifier should not be reused until the SUBACK is received.
    ///
    /// # Returns
    ///
    /// The packet identifier as `PacketIdType`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    /// use mqtt_protocol_core::mqtt::packet::SubEntry;
    ///
    /// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
    ///     .packet_id(123)
    ///     .entries(vec![
    ///         SubEntry::builder()
    ///             .topic_filter("test/topic")
    ///             .unwrap()
    ///             .qos(Qos::AtMostOnce)
    ///             .build()
    ///             .unwrap()
    ///     ])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(subscribe.packet_id(), 123);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Parses a SUBSCRIBE packet from a byte buffer
    ///
    /// This method parses the variable header and payload of a SUBSCRIBE packet,
    /// extracting the packet identifier and subscription entries. The fixed header
    /// should be parsed separately before calling this method.
    ///
    /// # Parameters
    ///
    /// * `data` - Byte buffer containing the SUBSCRIBE packet data (without fixed header)
    ///
    /// # Returns
    ///
    /// * `Ok((GenericSubscribe, usize))` - The parsed SUBSCRIBE packet and number of bytes consumed
    /// * `Err(MqttError)` - If the packet is malformed or contains invalid data
    ///
    /// # Errors
    ///
    /// * `MqttError::MalformedPacket` - If the packet structure is invalid
    /// * `MqttError::ProtocolError` - If the packet violates MQTT protocol rules (e.g., no entries)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // This would typically be called by a packet parser after reading the fixed header
    /// let data = &[0x00, 0x01, 0x00, 0x09, b't', b'e', b's', b't', b'/', b't', b'o', b'p', b'i', b'c', 0x01];
    /// let (subscribe, consumed) = mqtt::packet::v3_1_1::Subscribe::parse(data).unwrap();
    /// assert_eq!(subscribe.packet_id(), 1);
    /// assert_eq!(consumed, data.len());
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

        let mut entries = Vec::new();
        while cursor < data.len() {
            let (entry, consumed) = SubEntry::parse(&data[cursor..])?;
            entries.push(entry);
            cursor += consumed;
        }

        if entries.is_empty() {
            return Err(MqttError::ProtocolError);
        }

        let remaining_size = buffer_size + entries.iter().map(|e| e.size()).sum::<usize>();
        let remaining_length = VariableByteInteger::from_u32(remaining_size as u32).unwrap();

        let subscribe = GenericSubscribe {
            fixed_header: [FixedHeader::Subscribe as u8],
            remaining_length,
            packet_id_buf,
            entries,
        };

        Ok((subscribe, cursor))
    }

    /// Returns the total size of the SUBSCRIBE packet in bytes
    ///
    /// This includes the fixed header (1 byte for packet type + remaining length encoding)
    /// plus all variable header and payload data. This size can be used for buffer
    /// allocation before serialization.
    ///
    /// # Returns
    ///
    /// Total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    /// use mqtt_protocol_core::mqtt::packet::SubEntry;
    ///
    /// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
    ///     .packet_id(1)
    ///     .entries(vec![
    ///         SubEntry::builder()
    ///             .topic_filter("test")
    ///             .unwrap()
    ///             .qos(Qos::AtMostOnce)
    ///             .build()
    ///             .unwrap()
    ///     ])
    ///     .build()
    ///     .unwrap();
    ///
    /// let total_size = subscribe.size();
    /// // Size includes: fixed header + packet_id + entries
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the SUBSCRIBE packet to a vector of I/O slices for efficient network transmission
    ///
    /// This method serializes the packet into multiple buffer slices without copying data,
    /// allowing for efficient vectored I/O operations. The slices are ordered according
    /// to the MQTT packet structure.
    ///
    /// # Returns
    ///
    /// Vector of `IoSlice` containing the serialized packet data:
    /// 1. Fixed header (packet type and flags)
    /// 2. Remaining length encoding
    /// 3. Packet identifier
    /// 4. Subscription entries (topic filters and QoS levels)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    /// use mqtt_protocol_core::mqtt::packet::SubEntry;
    ///
    /// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
    ///     .packet_id(1)
    ///     .entries(vec![
    ///         SubEntry::builder()
    ///             .topic_filter("test/topic")
    ///             .unwrap()
    ///             .qos(Qos::AtLeastOnce)
    ///             .build()
    ///             .unwrap()
    ///     ])
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = subscribe.to_buffers();
    /// // Use buffers for vectored write operations
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(self.packet_id_buf.as_ref()));

        for entry in &self.entries {
            bufs.extend(entry.to_buffers());
        }

        bufs
    }

    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.extend_from_slice(self.packet_id_buf.as_ref());

        for entry in &self.entries {
            buf.append(&mut entry.to_continuous_buffer());
        }

        buf
    }
}

/// Builder implementation for constructing SUBSCRIBE packets
///
/// The builder provides a fluent interface for constructing SUBSCRIBE packets with
/// validation of required fields and protocol compliance. It ensures that all
/// SUBSCRIBE packets have a valid packet identifier and at least one subscription entry.
impl<PacketIdType> GenericSubscribeBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the SUBSCRIBE packet
    ///
    /// The packet identifier must be non-zero and unique within the client session.
    /// It is used to match SUBSCRIBE packets with their corresponding SUBACK responses.
    /// The same packet identifier should not be reused until the SUBACK is received.
    ///
    /// # Parameters
    ///
    /// * `id` - The packet identifier (must be non-zero)
    ///
    /// # Returns
    ///
    /// Builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v3_1_1::Subscribe::builder()
    ///     .packet_id(42);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Validates the builder state before constructing the SUBSCRIBE packet
    ///
    /// This method checks that all required fields are present and valid according
    /// to the MQTT 3.1.1 specification:
    /// - Packet identifier must be present and non-zero
    /// - At least one subscription entry must be present
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation passes
    /// * `Err(MqttError)` - If validation fails
    ///
    /// # Errors
    ///
    /// * `MqttError::MalformedPacket` - If packet identifier is missing or zero
    /// * `MqttError::ProtocolError` - If no subscription entries
    fn validate(&self) -> Result<(), MqttError> {
        if self.packet_id_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }

        let packet_id_bytes = self.packet_id_buf.as_ref().unwrap().as_ref();
        let all_zeros = packet_id_bytes.iter().all(|&b| b == 0);
        if all_zeros {
            return Err(MqttError::MalformedPacket);
        }

        if self.entries.as_ref().map_or(true, |e| e.is_empty()) {
            return Err(MqttError::ProtocolError);
        }

        Ok(())
    }

    /// Builds the SUBSCRIBE packet from the builder state
    ///
    /// This method validates all fields and constructs the final SUBSCRIBE packet.
    /// It calculates the remaining length encoding required for the MQTT packet format.
    ///
    /// # Returns
    ///
    /// * `Ok(GenericSubscribe)` - The constructed SUBSCRIBE packet
    /// * `Err(MqttError)` - If validation fails or packet cannot be constructed
    ///
    /// # Errors
    ///
    /// * `MqttError::MalformedPacket` - If required fields are missing or invalid
    /// * `MqttError::ProtocolError` - If packet violates MQTT protocol rules
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    /// use mqtt_protocol_core::mqtt::packet::SubEntry;
    ///
    /// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
    ///     .packet_id(1)
    ///     .entries(vec![
    ///         SubEntry::builder()
    ///             .topic_filter("sensors/temperature")
    ///             .unwrap()
    ///             .qos(Qos::AtMostOnce)
    ///             .build()
    ///             .unwrap()
    ///     ])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<GenericSubscribe<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let entries = self.entries.unwrap_or_default();

        let packet_id_size = mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
        let entries_size = entries.iter().map(|e| e.size()).sum::<usize>();

        let remaining = packet_id_size + entries_size;
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericSubscribe {
            fixed_header: [FixedHeader::Subscribe as u8],
            remaining_length,
            packet_id_buf,
            entries,
        })
    }
}

/// Display trait implementation for SUBSCRIBE packets
///
/// Provides a human-readable JSON representation of the SUBSCRIBE packet,
/// including the packet type, packet identifier, and subscription entries.
/// This is useful for debugging and logging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
///     .packet_id(42)
///     .entries(vec![
///         SubEntry::builder()
///             .topic_filter("test/topic")
///             .unwrap()
///             .qos(Qos::AtLeastOnce)
///             .build()
///             .unwrap()
///     ])
///     .build()
///     .unwrap();
///
/// println!("{}", subscribe);
/// // Output: {"type":"subscribe","packet_id":42,"entries":[...]}
/// ```
impl<PacketIdType> fmt::Display for GenericSubscribe<PacketIdType>
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

/// Debug trait implementation for SUBSCRIBE packets
///
/// Provides the same output as the Display trait, showing a JSON representation
/// of the packet contents. This is consistent with other packet types in the library.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
///     .packet_id(1)
///     .entries(vec![
///         SubEntry::builder()
///             .topic_filter("debug/topic")
///             .unwrap()
///             .qos(Qos::AtMostOnce)
///             .build()
///             .unwrap()
///     ])
///     .build()
///     .unwrap();
///
/// println!("{:?}", subscribe);
/// ```
impl<PacketIdType> fmt::Debug for GenericSubscribe<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Serde Serialize trait implementation for SUBSCRIBE packets
///
/// Serializes the SUBSCRIBE packet to a structured format (typically JSON)
/// containing the packet type, packet identifier, and subscription entries
/// (if any). Empty collections are omitted from the output.
///
/// # Serialized Fields
///
/// - `type`: Always "subscribe"
/// - `packet_id`: The packet identifier
/// - `entries`: Array of subscription entries (only if non-empty)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
///     .packet_id(123)
///     .entries(vec![
///         SubEntry::builder()
///             .topic_filter("home/sensor")
///             .unwrap()
///             .qos(Qos::ExactlyOnce)
///             .build()
///             .unwrap()
///     ])
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&subscribe).unwrap();
/// // json contains: {"type":"subscribe","packet_id":123,"entries":[...]}
/// ```
impl<PacketIdType> Serialize for GenericSubscribe<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 2; // type and packet_id are always present

        if !self.entries.is_empty() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("Subscribe", field_count)?;

        state.serialize_field("type", "subscribe")?;
        state.serialize_field("packet_id", &self.packet_id())?;

        if !self.entries.is_empty() {
            state.serialize_field("entries", &self.entries)?;
        }

        state.end()
    }
}

/// Generic packet trait implementation for SUBSCRIBE packets
///
/// Provides the common packet interface used by the MQTT protocol handler.
/// This allows SUBSCRIBE packets to be treated uniformly with other packet types
/// for size calculation and serialization operations.
///
/// # Methods
///
/// - `size()`: Returns the total packet size in bytes
/// - `to_buffers()`: Returns I/O slices for efficient transmission
impl<PacketIdType> GenericPacketTrait for GenericSubscribe<PacketIdType>
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

/// Generic packet display trait implementation for SUBSCRIBE packets
///
/// Provides unified display formatting for packet types, supporting both
/// debug and display formatting through a common interface. This is used
/// by the packet handling infrastructure for consistent logging and debugging.
impl<PacketIdType> GenericPacketDisplay for GenericSubscribe<PacketIdType>
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
