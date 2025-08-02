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
use std::fmt;
use std::io::IoSlice;
use std::mem;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::mqtt::packet::mqtt_string::MqttString;
use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::packet::{
    Properties, PropertiesParse, PropertiesSize, PropertiesToBuffers, Property,
};
use crate::mqtt::result_code::MqttError;

/// MQTT 5.0 UNSUBSCRIBE packet representation
///
/// The UNSUBSCRIBE packet is sent by a client to unsubscribe from one or more topic filters
/// on the server. This removes existing subscriptions and stops the flow of messages
/// from the server to the client for the specified topic filters.
///
/// According to MQTT 5.0 specification, the UNSUBSCRIBE packet contains:
/// - Fixed header with packet type (bit 7-4 = 1010), reserved flags (bit 3-0 = 0010), and remaining length
/// - Variable header with packet identifier and properties
/// - Payload containing one or more topic filter strings to unsubscribe from
///
/// # Fixed Header
///
/// The UNSUBSCRIBE packet uses a fixed header with:
/// - **Packet Type**: 10 (1010 binary) in bits 7-4 of the first byte
/// - **Reserved Flags**: 2 (0010 binary) in bits 3-0 - these flags are reserved and must be set as shown
/// - **Remaining Length**: Variable length encoding of the remaining packet data
///
/// # Variable Header
///
/// The variable header contains:
/// - **Packet Identifier**: A 16-bit identifier used to match UNSUBSCRIBE with UNSUBACK packets
/// - **Properties**: MQTT 5.0 properties that can modify the behavior of the unsubscription
///
/// # Payload
///
/// The payload contains one or more topic filter strings to unsubscribe from.
/// Each topic filter is a UTF-8 encoded string that matches the exact topic filter
/// used in the original SUBSCRIBE packet. Wildcards are allowed and work the same
/// as in SUBSCRIBE packets.
///
/// # Topic Filters and Wildcards
///
/// Topic filters in UNSUBSCRIBE packets can include the same wildcards as SUBSCRIBE:
/// - **Single-level wildcard (+)**: Matches exactly one topic level (e.g., "sport/+/player1")
/// - **Multi-level wildcard (#)**: Matches any number of topic levels (e.g., "sport/#")
///
/// The topic filter must exactly match the topic filter used in the original subscription.
///
/// # Properties
///
/// MQTT 5.0 UNSUBSCRIBE packets can include:
/// - **User Properties**: Custom key-value pairs for application-specific data
///
/// Other properties are not allowed in UNSUBSCRIBE packets and will result in a protocol error.
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
///
/// // Create a simple UNSUBSCRIBE packet for a single topic
/// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
///     .packet_id(42)
///     .entries(vec!["sensors/temperature"])
///     .unwrap()
///     .build()
///     .unwrap();
///
/// assert_eq!(unsubscribe.packet_id(), 42);
/// assert_eq!(unsubscribe.entries().len(), 1);
/// assert_eq!(unsubscribe.entries()[0].as_str(), "sensors/temperature");
///
/// // Create an UNSUBSCRIBE packet with multiple topics
/// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
///     .packet_id(123)
///     .entries(vec![
///         "home/+/temperature",
///         "alerts/#",
///         "sensors/humidity"
///     ])
///     .unwrap()
///     .build()
///     .unwrap();
///
/// assert_eq!(unsubscribe.packet_id(), 123);
/// assert_eq!(unsubscribe.entries().len(), 3);
///
/// // Serialize to bytes for network transmission
/// let buffers = unsubscribe.to_buffers();
/// let total_size = unsubscribe.size();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericUnsubscribe<PacketIdType>
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

    /// MQTT 5.0 properties for the UNSUBSCRIBE packet
    ///
    /// Contains the properties that modify the behavior of the unsubscription.
    /// For UNSUBSCRIBE packets, only User Properties are allowed.
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Properties,

    /// Topic filter entries to unsubscribe from
    ///
    /// Contains the list of topic filter strings that the client wants to
    /// unsubscribe from. Each entry must exactly match a topic filter from
    /// a previous SUBSCRIBE packet.
    #[builder(default = "Vec::new()", setter(custom))]
    entry_bufs: Vec<MqttString>,
}

/// Type alias for UNSUBSCRIBE packet with standard u16 packet identifiers
///
/// This is the most commonly used UNSUBSCRIBE packet type for standard MQTT 5.0
/// implementations that use 16-bit packet identifiers as specified in the protocol.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
///     .packet_id(1)
///     .entries(vec!["my/topic", "another/topic"])
///     .unwrap()
///     .build()
///     .unwrap();
/// ```
pub type Unsubscribe = GenericUnsubscribe<u16>;

impl<PacketIdType> GenericUnsubscribe<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing an UNSUBSCRIBE packet
    ///
    /// The builder pattern allows for flexible construction of UNSUBSCRIBE packets
    /// with various combinations of properties and topic filters.
    /// All UNSUBSCRIBE packets must have a packet identifier and at least one topic filter.
    ///
    /// # Returns
    ///
    /// A `GenericUnsubscribeBuilder` instance with default values
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .packet_id(42)
    ///     .entries(vec!["sensors/+", "alerts/#"])
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericUnsubscribeBuilder<PacketIdType> {
        GenericUnsubscribeBuilder::<PacketIdType>::default()
    }

    /// Returns the packet type for UNSUBSCRIBE packets
    ///
    /// This is always `PacketType::Unsubscribe` for UNSUBSCRIBE packet instances.
    /// The numeric value is 10, represented as 1010 in the upper 4 bits of the
    /// fixed header's first byte.
    ///
    /// # Returns
    ///
    /// `PacketType::Unsubscribe`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::packet_type::PacketType;
    ///
    /// assert_eq!(mqtt::packet::v5_0::Unsubscribe::packet_type(), PacketType::Unsubscribe);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Unsubscribe
    }

    /// Returns the packet identifier for this UNSUBSCRIBE packet
    ///
    /// The packet identifier is used to match UNSUBSCRIBE packets with their corresponding
    /// UNSUBACK responses. It must be non-zero as specified in the MQTT protocol.
    /// The same packet identifier should not be reused until the UNSUBACK is received.
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
    /// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .packet_id(123)
    ///     .entries(vec!["test/topic"])
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(unsubscribe.packet_id(), 123);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Returns the topic filter entries to unsubscribe from
    ///
    /// Returns a reference to the vector of topic filter strings that this
    /// UNSUBSCRIBE packet requests to unsubscribe from. Each topic filter
    /// must exactly match a topic filter from a previous SUBSCRIBE packet.
    ///
    /// # Returns
    ///
    /// A reference to the vector of `MqttString` topic filters
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .packet_id(1)
    ///     .entries(vec!["home/temperature", "sensors/+"])
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    ///
    /// let entries = unsubscribe.entries();
    /// assert_eq!(entries.len(), 2);
    /// assert_eq!(entries[0].as_str(), "home/temperature");
    /// assert_eq!(entries[1].as_str(), "sensors/+");
    /// ```
    pub fn entries(&self) -> &Vec<MqttString> {
        &self.entry_bufs
    }

    /// Parses an UNSUBSCRIBE packet from raw bytes
    ///
    /// Deserializes an UNSUBSCRIBE packet from its binary representation according
    /// to the MQTT 5.0 specification. The input should contain the variable header
    /// and payload data (excluding the fixed header).
    ///
    /// # Arguments
    ///
    /// * `data` - Byte slice containing the packet data (variable header + payload)
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The parsed `GenericUnsubscribe` packet
    /// - The number of bytes consumed during parsing
    ///
    /// # Errors
    ///
    /// Returns `MqttError` if:
    /// - The packet is malformed or incomplete
    /// - The packet identifier is zero (invalid)
    /// - No topic filter entries are present (protocol error)
    /// - Invalid properties are present
    /// - UTF-8 decoding fails for topic filters
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Assuming you have raw packet data
    /// let packet_data = &[/* packet bytes */];
    ///
    /// match mqtt::packet::v5_0::Unsubscribe::parse(packet_data) {
    ///     Ok((unsubscribe, bytes_consumed)) => {
    ///         println!("Parsed UNSUBSCRIBE with packet ID: {}", unsubscribe.packet_id());
    ///         println!("Consumed {} bytes", bytes_consumed);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to parse UNSUBSCRIBE packet: {:?}", e);
    ///     }
    /// }
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
        validate_unsubscribe_properties(&props)?;
        let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

        let mut entries = Vec::new();
        while cursor < data.len() {
            let (mqtt_string, consumed) = MqttString::decode(&data[cursor..])?;
            entries.push(mqtt_string);
            cursor += consumed;
        }

        if entries.is_empty() {
            return Err(MqttError::ProtocolError);
        }

        let remaining_size =
            buffer_size + property_length + entries.iter().map(|e| e.size()).sum::<usize>();
        let remaining_length = VariableByteInteger::from_u32(remaining_size as u32).unwrap();

        let unsubscribe = GenericUnsubscribe {
            fixed_header: [FixedHeader::Unsubscribe as u8],
            remaining_length,
            packet_id_buf,
            property_length: prop_len,
            props,
            entry_bufs: entries,
        };

        Ok((unsubscribe, cursor))
    }

    /// Returns the total size of the UNSUBSCRIBE packet in bytes
    ///
    /// Calculates the total size including the fixed header, variable header,
    /// and payload. This is useful for buffer allocation and network transmission.
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
    /// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .packet_id(1)
    ///     .entries(vec!["test/topic"])
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = unsubscribe.size();
    /// println!("UNSUBSCRIBE packet size: {} bytes", size);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the UNSUBSCRIBE packet to a vector of I/O slices for efficient network transmission
    ///
    /// Creates a vector of `IoSlice` references that can be used with vectored I/O operations
    /// for efficient transmission without copying data. The slices represent the complete
    /// MQTT packet in wire format.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` containing references to the packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use std::io::IoSlice;
    ///
    /// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .packet_id(1)
    ///     .entries(vec!["test/topic"])
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers: Vec<IoSlice> = unsubscribe.to_buffers();
    /// // Use buffers for vectored I/O operations
    /// ```
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(self.packet_id_buf.as_ref()));
        bufs.push(IoSlice::new(self.property_length.as_bytes()));
        bufs.extend(self.props.to_buffers());

        for entry in &self.entry_bufs {
            bufs.extend(entry.to_buffers());
        }

        bufs
    }
}

/// Builder implementation for UNSUBSCRIBE packets
///
/// Provides methods for constructing UNSUBSCRIBE packets with validation.
/// The builder ensures that all required fields are set and validates the
/// packet according to MQTT 5.0 specifications before building.
impl<PacketIdType> GenericUnsubscribeBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the UNSUBSCRIBE packet
    ///
    /// The packet identifier must be non-zero and is used to match the UNSUBSCRIBE
    /// packet with its corresponding UNSUBACK response.
    ///
    /// # Arguments
    ///
    /// * `id` - The packet identifier (must be non-zero)
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
    /// let builder = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .packet_id(42);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Sets the topic filter entries to unsubscribe from
    ///
    /// Accepts any iterable of items that can be converted to MQTT strings.
    /// Each topic filter must be a valid UTF-8 string and can contain wildcards.
    /// The topic filters should exactly match those used in previous SUBSCRIBE packets.
    ///
    /// # Arguments
    ///
    /// * `entries` - An iterable of topic filter strings
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining, or an error if conversion fails
    ///
    /// # Errors
    ///
    /// Returns `MqttError` if:
    /// - Any topic filter contains invalid UTF-8
    /// - Topic filter length exceeds MQTT limits
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Single topic filter
    /// let builder = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .entries(vec!["home/temperature"])
    ///     .unwrap();
    ///
    /// // Multiple topic filters with wildcards
    /// let builder = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .entries(vec!["sensors/+", "alerts/#", "home/kitchen/temperature"])
    ///     .unwrap();
    /// ```
    pub fn entries<I, T>(mut self, entries: I) -> Result<Self, MqttError>
    where
        I: IntoIterator<Item = T>,
        T: TryInto<MqttString>,
        T::Error: Into<MqttError>,
    {
        let mqtt_strings: Result<Vec<_>, _> = entries
            .into_iter()
            .map(|item| item.try_into().map_err(Into::into))
            .collect();

        self.entry_bufs = Some(mqtt_strings?);
        Ok(self)
    }

    /// Validates the builder state before constructing the packet
    ///
    /// Ensures that all required fields are set and validates the packet
    /// according to MQTT 5.0 specifications.
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, otherwise `MqttError`
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - Packet identifier is not set or is zero
    ///
    /// Returns `MqttError::ProtocolError` if:
    /// - No topic filter entries are provided
    /// - Invalid properties are present
    fn validate(&self) -> Result<(), MqttError> {
        if self.packet_id_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }

        let packet_id_bytes = self.packet_id_buf.as_ref().unwrap().as_ref();
        let all_zeros = packet_id_bytes.iter().all(|&b| b == 0);
        if all_zeros {
            return Err(MqttError::MalformedPacket);
        }

        if self.entry_bufs.as_ref().map_or(true, |e| e.is_empty()) {
            return Err(MqttError::ProtocolError);
        }

        if let Some(ref props) = self.props {
            validate_unsubscribe_properties(props)?;
        }

        Ok(())
    }

    /// Builds the UNSUBSCRIBE packet after validation
    ///
    /// Constructs the final UNSUBSCRIBE packet with all specified fields and properties.
    /// Performs validation to ensure the packet conforms to MQTT 5.0 specifications.
    ///
    /// # Returns
    ///
    /// The constructed `GenericUnsubscribe` packet
    ///
    /// # Errors
    ///
    /// Returns `MqttError` if:
    /// - Required fields are missing (packet ID, entries)
    /// - Packet identifier is zero
    /// - No topic filter entries are provided
    /// - Properties contain invalid values for UNSUBSCRIBE packets
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
    ///     .packet_id(42)
    ///     .entries(vec!["home/temperature", "sensors/humidity"])
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<GenericUnsubscribe<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let entries = self.entry_bufs.unwrap_or_default();
        let props = self.props.unwrap_or_else(Properties::new);
        let props_size = props.size();
        let property_length = VariableByteInteger::from_u32(props_size as u32).unwrap();

        let packet_id_size = mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
        let prop_len_size = property_length.size();
        let entries_size = entries.iter().map(|e| e.size()).sum::<usize>();

        let remaining = packet_id_size + prop_len_size + props_size + entries_size;
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericUnsubscribe {
            fixed_header: [FixedHeader::Unsubscribe as u8],
            remaining_length,
            packet_id_buf,
            property_length,
            props,
            entry_bufs: entries,
        })
    }
}

/// Serialization implementation for UNSUBSCRIBE packets
///
/// Enables JSON serialization of UNSUBSCRIBE packets for debugging, logging,
/// and API integration purposes. The serialized format includes the packet type,
/// packet identifier, properties (if any), and topic filter entries.
///
/// # Serialized Format
///
/// The packet is serialized as a JSON object with these fields:
/// - `type`: Always "unsubscribe"
/// - `packet_id`: The packet identifier
/// - `props`: Properties object (only included if not empty)
/// - `entries`: Array of topic filter strings (only included if not empty)
impl<PacketIdType> Serialize for GenericUnsubscribe<PacketIdType>
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

        if !self.entry_bufs.is_empty() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("Unsubscribe", field_count)?;

        state.serialize_field("type", "unsubscribe")?;
        state.serialize_field("packet_id", &self.packet_id())?;

        if !self.props.is_empty() {
            state.serialize_field("props", &self.props)?;
        }

        if !self.entry_bufs.is_empty() {
            state.serialize_field("entries", &self.entry_bufs)?;
        }

        state.end()
    }
}

/// Display implementation for UNSUBSCRIBE packets
///
/// Provides a human-readable string representation of the UNSUBSCRIBE packet
/// using JSON formatting. This is useful for debugging and logging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
///     .packet_id(42)
///     .entries(vec!["home/temperature"])
///     .unwrap()
///     .build()
///     .unwrap();
///
/// println!("UNSUBSCRIBE packet: {}", unsubscribe);
/// ```
impl<PacketIdType> fmt::Display for GenericUnsubscribe<PacketIdType>
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

/// Debug implementation for UNSUBSCRIBE packets
///
/// Provides the same output as the Display implementation for consistent
/// formatting across different contexts.
impl<PacketIdType> fmt::Debug for GenericUnsubscribe<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Generic packet trait implementation for UNSUBSCRIBE packets
///
/// Provides common packet operations required by the MQTT protocol framework.
/// This trait allows UNSUBSCRIBE packets to be used polymorphically with other
/// packet types in the protocol implementation.
impl<PacketIdType> GenericPacketTrait for GenericUnsubscribe<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    fn size(&self) -> usize {
        self.size()
    }

    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }
}

/// Generic packet display trait implementation for UNSUBSCRIBE packets
///
/// Provides display formatting methods for consistent packet output across
/// the MQTT protocol framework.
impl<PacketIdType> GenericPacketDisplay for GenericUnsubscribe<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Validates properties for UNSUBSCRIBE packets according to MQTT 5.0 specification
///
/// UNSUBSCRIBE packets can only contain User Properties. Any other property type
/// is considered a protocol error.
///
/// # Arguments
///
/// * `props` - The properties to validate
///
/// # Returns
///
/// `Ok(())` if all properties are valid for UNSUBSCRIBE packets
///
/// # Errors
///
/// Returns `MqttError::ProtocolError` if any invalid property is found
///
/// # Valid Properties
///
/// - `Property::UserProperty`: Custom key-value pairs for application use
///
/// # Invalid Properties
///
/// All other property types are invalid for UNSUBSCRIBE packets, including:
/// - Subscription Identifier
/// - Topic Alias
/// - Response Information
/// - Server Reference
/// - Reason String
/// - And all other properties defined in MQTT 5.0
fn validate_unsubscribe_properties(props: &Properties) -> Result<(), MqttError> {
    for prop in props {
        match prop {
            Property::UserProperty(_) => {}
            _ => return Err(MqttError::ProtocolError),
        }
    }
    Ok(())
}
