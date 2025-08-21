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

use crate::mqtt_internal::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt_internal::packet::property::PropertiesToContinuousBuffer;
use crate::mqtt_internal::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt_internal::packet::GenericPacketDisplay;
use crate::mqtt_internal::packet::GenericPacketTrait;
use crate::mqtt_internal::packet::IsPacketId;
#[cfg(feature = "std")]
use crate::mqtt_internal::packet::PropertiesToBuffers;
use crate::mqtt_internal::packet::SubEntry;
use crate::mqtt_internal::packet::{
    GenericProperties, GenericProperty, PropertiesParse, PropertiesSize,
};
use crate::mqtt_internal::result_code::MqttError;

/// MQTT 5.0 SUBSCRIBE packet representation
///
/// The SUBSCRIBE packet is sent by a client to subscribe to one or more topic filters
/// on the server. Each subscription establishes a flow of messages from the server to
/// the client based on the matching topic filters and their associated subscription options.
///
/// According to MQTT 5.0 specification, the SUBSCRIBE packet contains:
/// - Fixed header with packet type (bit 7-4 = 1000), reserved flags (bit 3-0 = 0010), and remaining length
/// - Variable header with packet identifier and properties
/// - Payload containing one or more topic filter entries with subscription options
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
/// - **Properties**: MQTT 5.0 properties that modify the behavior of the subscription
///
/// # Payload
///
/// The payload contains one or more subscription entries, each consisting of:
/// - **Topic Filter**: UTF-8 encoded string that may contain wildcards (+ and #)
/// - **Subscription Options**: Byte containing QoS, No Local, Retain As Published, and Retain Handling flags
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
/// # MQTT 5.0 Subscription Options
///
/// - **No Local**: If set, messages published by this client are not sent back to it
/// - **Retain As Published**: If set, retain flag is preserved when forwarding messages
/// - **Retain Handling**: Controls how retained messages are sent when subscription is established
///
/// # Properties
///
/// MQTT 5.0 SUBSCRIBE packets can include:
/// - **Subscription Identifier**: Numeric identifier to associate with the subscription
/// - **User Properties**: Custom key-value pairs for application-specific data
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
/// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
/// // Create a SUBSCRIBE packet with multiple topics and properties
/// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
///             .no_local(true)
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
pub struct GenericSubscribe<
    PacketIdType,
    const STRING_BUFFER_SIZE: usize,
    const BINARY_BUFFER_SIZE: usize,
> where
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

    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: GenericProperties<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,

    #[getset(get = "pub")]
    entries: Vec<SubEntry>,
}

/// Type alias for SUBSCRIBE packet with standard u16 packet identifiers
///
/// This is the most commonly used SUBSCRIBE packet type for standard MQTT 5.0
/// implementations that use 16-bit packet identifiers as specified in the protocol.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a SUBSCRIBE packet
    ///
    /// The builder pattern allows for flexible construction of SUBSCRIBE packets
    /// with various combinations of properties, topic filters, and subscription options.
    /// All SUBSCRIBE packets must have a packet identifier and at least one subscription entry.
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
    /// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
    ///     .packet_id(42)
    ///     .entries(vec![
    ///         SubEntry::builder()
    ///             .topic_filter("sensors/+")
    ///             .unwrap()
    ///             .qos(Qos::AtLeastOnce)
    ///             .no_local(true)
    ///             .build()
    ///             .unwrap()
    ///     ])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericSubscribeBuilder<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
    {
        GenericSubscribeBuilder::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::default()
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
    /// assert_eq!(mqtt::packet::v5_0::Subscribe::packet_type(), PacketType::Subscribe);
    /// ```
    pub fn packet_type() -> PacketType {
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
    /// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
    /// extracting the packet identifier, properties, and subscription entries.
    /// The fixed header should be parsed separately before calling this method.
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
    /// let data = &[0x00, 0x01, 0x00, 0x00, 0x09, b't', b'e', b's', b't', b'/', b't', b'o', b'p', b'i', b'c', 0x01];
    /// let (subscribe, consumed) = mqtt::packet::v5_0::Subscribe::parse(data).unwrap();
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

        let (props, property_length) =
            GenericProperties::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(&data[cursor..])?;
        cursor += property_length;
        validate_subscribe_properties(&props)?;
        let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

        let mut entries = Vec::new();
        while cursor < data.len() {
            let (entry, consumed) = SubEntry::parse(&data[cursor..])?;
            entries.push(entry);
            cursor += consumed;
        }

        if entries.is_empty() {
            return Err(MqttError::ProtocolError);
        }

        let remaining_size =
            buffer_size + property_length + entries.iter().map(|e| e.size()).sum::<usize>();
        let remaining_length = VariableByteInteger::from_u32(remaining_size as u32).unwrap();

        let subscribe = GenericSubscribe {
            fixed_header: [FixedHeader::Subscribe as u8],
            remaining_length,
            packet_id_buf,
            property_length: prop_len,
            props,
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
    /// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
    /// // Size includes: fixed header + packet_id + properties_length + properties + entries
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// represent the complete SUBSCRIBE packet in wire format.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` objects for vectored I/O operations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::SubEntry;
    /// use mqtt_protocol_core::mqtt::qos::QoS;
    ///
    /// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
    ///     .packet_id(1u16)
    ///     .entries(vec![
    ///         SubEntry::new("test/topic", QoS::AtLeastOnce),
    ///         SubEntry::new("another/topic", QoS::ExactlyOnce),
    ///     ])
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = subscribe.to_buffers();
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

        for entry in &self.entries {
            bufs.extend(entry.to_buffers());
        }

        bufs
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method provides an alternative to `to_buffers()` for no-std environments
    /// where vectored I/O is not available.
    ///
    /// The returned buffer contains the complete SUBSCRIBE packet serialized according
    /// to the MQTT v5.0 protocol specification, including fixed header, remaining
    /// length, packet identifier, properties, and subscription entries.
    ///
    /// # Returns
    ///
    /// A vector containing the complete packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::SubEntry;
    /// use mqtt_protocol_core::mqtt::qos::QoS;
    ///
    /// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
    ///     .packet_id(1u16)
    ///     .entries(vec![
    ///         SubEntry::new("test/topic", QoS::AtLeastOnce),
    ///     ])
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = subscribe.to_continuous_buffer();
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
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericSubscribeBuilder<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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
    /// let builder = mqtt::packet::v5_0::Subscribe::builder()
    ///     .packet_id(42);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Validates the builder state before constructing the SUBSCRIBE packet
    ///
    /// This method checks that all required fields are present and valid according
    /// to the MQTT 5.0 specification:
    /// - Packet identifier must be present and non-zero
    /// - At least one subscription entry must be present
    /// - All properties must be valid for SUBSCRIBE packets
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If validation passes
    /// * `Err(MqttError)` - If validation fails
    ///
    /// # Errors
    ///
    /// * `MqttError::MalformedPacket` - If packet identifier is missing or zero
    /// * `MqttError::ProtocolError` - If no subscription entries or invalid properties
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

        if let Some(ref props) = self.props {
            validate_subscribe_properties(props)?;
        }

        Ok(())
    }

    /// Builds the SUBSCRIBE packet from the builder state
    ///
    /// This method validates all fields and constructs the final SUBSCRIBE packet.
    /// It calculates the remaining length and property length encodings required
    /// for the MQTT packet format.
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
    /// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
    pub fn build(
        self,
    ) -> Result<GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>, MqttError>
    {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let entries = self.entries.unwrap_or_default();

        let props = self
            .props
            .unwrap_or_else(GenericProperties::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::new);
        let props_size = props.size();
        let property_length = VariableByteInteger::from_u32(props_size as u32).unwrap();

        let packet_id_size = mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
        let prop_len_size = property_length.size();
        let entries_size = entries.iter().map(|e| e.size()).sum::<usize>();

        let remaining = packet_id_size + prop_len_size + props_size + entries_size;
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericSubscribe {
            fixed_header: [FixedHeader::Subscribe as u8],
            remaining_length,
            packet_id_buf,
            property_length,
            props,
            entries,
        })
    }
}

/// Display trait implementation for SUBSCRIBE packets
///
/// Provides a human-readable JSON representation of the SUBSCRIBE packet,
/// including the packet type, packet identifier, properties, and subscription entries.
/// This is useful for debugging and logging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Display
    for GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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
/// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Debug
    for GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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
/// containing the packet type, packet identifier, properties (if any), and
/// subscription entries (if any). Empty collections are omitted from the output.
///
/// # Serialized Fields
///
/// - `type`: Always "subscribe"
/// - `packet_id`: The packet identifier
/// - `props`: Properties object (only if non-empty)
/// - `entries`: Array of subscription entries (only if non-empty)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
/// use mqtt_protocol_core::mqtt::packet::SubEntry;
///
/// let subscribe = mqtt::packet::v5_0::Subscribe::builder()
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
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> Serialize
    for GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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

        if !self.entries.is_empty() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("Subscribe", field_count)?;

        state.serialize_field("type", "subscribe")?;
        state.serialize_field("packet_id", &self.packet_id())?;

        if !self.props.is_empty() {
            state.serialize_field("props", &self.props)?;
        }

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
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericPacketTrait for GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericPacketDisplay for GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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

/// Validates properties for SUBSCRIBE packets according to MQTT 5.0 specification
///
/// Only certain properties are allowed in SUBSCRIBE packets:
/// - Subscription Identifier: Associates a numeric identifier with the subscription
/// - User Properties: Application-specific key-value pairs
///
/// Any other properties will result in a protocol error.
///
/// # Parameters
///
/// * `props` - The properties to validate
///
/// # Returns
///
/// * `Ok(())` - If all properties are valid for SUBSCRIBE packets
/// * `Err(MqttError::ProtocolError)` - If invalid properties are found
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::{Properties, Property};
///
/// let mut props = Properties::new();
/// props.push(Property::SubscriptionIdentifier(42));
/// // This would be valid for SUBSCRIBE packets
/// ```
fn validate_subscribe_properties<
    const STRING_BUFFER_SIZE: usize,
    const BINARY_BUFFER_SIZE: usize,
>(
    props: &GenericProperties<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
) -> Result<(), MqttError> {
    for prop in props {
        match prop {
            GenericProperty::SubscriptionIdentifier(_) => {}
            GenericProperty::UserProperty(_) => {}
            _ => return Err(MqttError::ProtocolError),
        }
    }
    Ok(())
}
