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
use crate::mqtt::packet::property::PropertiesToContinuousBuffer;
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::IsPacketId;
#[cfg(feature = "std")]
use crate::mqtt::packet::PropertiesToBuffers;
use crate::mqtt::packet::{Properties, PropertiesParse, PropertiesSize, Property};
use crate::mqtt::result_code::MqttError;
use crate::mqtt::result_code::PubrecReasonCode;

/// A PUBREC packet for MQTT v5.0 protocol.
///
/// The PUBREC packet is the response to a PUBLISH packet with QoS level 2 and is the second
/// step in the four-part handshake for QoS 2 message delivery. The complete QoS 2 flow is:
/// 1. PUBLISH (sender -> receiver)
/// 2. **PUBREC** (receiver -> sender) - this packet
/// 3. PUBREL (sender -> receiver)
/// 4. PUBCOMP (receiver -> sender)
///
/// # MQTT v5.0 Specification
///
/// According to the MQTT v5.0 specification, the PUBREC packet:
/// - Is sent by the receiver of a PUBLISH packet with QoS 2
/// - Contains the same Packet Identifier as the PUBLISH packet being acknowledged
/// - Indicates that the message has been received but not yet processed
/// - May optionally include a reason code indicating the result of the message reception
/// - May optionally include properties for additional metadata
/// - Must be followed by a PUBREL packet from the original sender
///
/// # QoS 2 Message Flow
///
/// The PUBREC packet ensures exactly-once delivery semantics:
/// - Upon receiving a QoS 2 PUBLISH, the receiver stores the message and sends PUBREC
/// - The PUBREC acknowledges message receipt and requests the sender to proceed
/// - The sender must respond with PUBREL to confirm message processing should continue
/// - Only after receiving PUBREL does the receiver process the message and send PUBCOMP
///
/// # Reason Codes
///
/// PUBREC packets can include reason codes to indicate the result of message processing:
/// - `Success` (0x00): Message received successfully
/// - `NoMatchingSubscribers` (0x10): No subscribers matched the topic
/// - `UnspecifiedError` (0x80): An unspecified error occurred
/// - `ImplementationSpecificError` (0x83): Implementation-specific error
/// - `NotAuthorized` (0x87): Client not authorized to publish to this topic
/// - `TopicNameInvalid` (0x90): Topic name format is invalid
/// - `PacketIdentifierInUse` (0x91): Packet identifier already in use
/// - `QuotaExceeded` (0x97): Quota exceeded
/// - `PayloadFormatInvalid` (0x99): Payload format does not match the format indicator
///
/// # Generic Support
///
/// This struct supports generic packet identifiers through the `PacketIdType` parameter,
/// allowing for extended packet ID ranges (e.g., u32) for broker clustering scenarios.
/// The standard type alias `Pubrec` uses `u16` packet identifiers as per MQTT specification.
///
/// # Examples
///
/// Creating a basic PUBREC packet:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubrec.packet_id(), 123u16);
/// assert_eq!(pubrec.reason_code(), None);
/// ```
///
/// Creating a PUBREC with reason code:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(456u16)
///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubrec.packet_id(), 456u16);
/// assert_eq!(pubrec.reason_code(), Some(mqtt::result_code::PubrecReasonCode::Success));
/// ```
///
/// Creating a PUBREC with properties:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let properties = vec![
///     mqtt::packet::Property::ReasonString("Message received".to_string()),
///     mqtt::packet::Property::UserProperty(("key".to_string(), "value".to_string())),
/// ];
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(789u16)
///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
///     .props(properties)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubrec.packet_id(), 789u16);
/// assert!(pubrec.props().is_some());
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericPubrec<PacketIdType>
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
    reason_code_buf: Option<[u8; 1]>,
    #[builder(private)]
    property_length: Option<VariableByteInteger>,

    /// Optional MQTT v5.0 properties associated with this PUBREC packet.
    ///
    /// Properties can include:
    /// - `ReasonString`: Human readable string designed for diagnostics
    /// - `UserProperty`: Name-value pairs for application-specific metadata
    ///
    /// Only one `ReasonString` property is allowed per packet.
    /// Properties can only be included if a reason code is also present.
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Option<Properties>,
}

/// Type alias for PUBREC packet with standard u16 packet identifiers.
///
/// This is the standard PUBREC packet type that most applications should use,
/// conforming to the MQTT v5.0 specification's u16 packet identifier requirement.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
/// ```
pub type Pubrec = GenericPubrec<u16>;

impl<PacketIdType> GenericPubrec<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a PUBREC packet.
    ///
    /// # Returns
    ///
    /// A new `GenericPubrecBuilder` instance for building a PUBREC packet.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v5_0::Pubrec::builder();
    /// let pubrec = builder
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericPubrecBuilder<PacketIdType> {
        GenericPubrecBuilder::<PacketIdType>::default()
    }

    /// Returns the packet type for PUBREC packets.
    ///
    /// # Returns
    ///
    /// Always returns `PacketType::Pubrec`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// assert_eq!(mqtt::packet::v5_0::Pubrec::packet_type(), mqtt::packet::packet_type::PacketType::Pubrec);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Pubrec
    }

    /// Returns the packet identifier for this PUBREC packet.
    ///
    /// The packet identifier must match the packet identifier of the PUBLISH
    /// packet being acknowledged. It is used to correlate the PUBREC with
    /// the original PUBLISH in the QoS 2 message flow.
    ///
    /// # Returns
    ///
    /// The packet identifier as the generic `PacketIdType`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrec.packet_id(), 123u16);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Returns the reason code for this PUBREC packet, if present.
    ///
    /// The reason code indicates the result of the PUBLISH message processing.
    /// If no reason code is present, it is assumed to be `Success` (0x00).
    ///
    /// # Returns
    ///
    /// - `Some(PubrecReasonCode)` if a reason code was included in the packet
    /// - `None` if no reason code was present (implies success)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// // PUBREC without reason code (implies success)
    /// let pubrec1 = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(pubrec1.reason_code(), None);
    ///
    /// // PUBREC with explicit reason code
    /// let pubrec2 = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(456u16)
    ///     .reason_code(mqtt::result_code::PubrecReasonCode::NoMatchingSubscribers)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(pubrec2.reason_code(), Some(mqtt::result_code::PubrecReasonCode::NoMatchingSubscribers));
    /// ```
    pub fn reason_code(&self) -> Option<PubrecReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| PubrecReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of this PUBREC packet in bytes.
    ///
    /// This includes the fixed header, remaining length field, and all variable
    /// header and payload components.
    ///
    /// # Returns
    ///
    /// The total packet size in bytes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = pubrec.size();
    /// // Size includes: fixed header (1) + remaining length (1) + packet_id (2) = 4 bytes minimum
    /// assert!(size >= 4);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts this PUBREC packet into a vector of I/O slices for efficient writing.
    ///
    /// This method provides zero-copy serialization by returning references to the
    /// internal packet data as `IoSlice` objects, which can be used with vectored
    /// I/O operations.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` objects representing the complete packet data.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = pubrec.to_buffers();
    /// // Use buffers for vectored I/O operations
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(self.packet_id_buf.as_ref()));
        if let Some(buf) = &self.reason_code_buf {
            bufs.push(IoSlice::new(buf));
        }
        if let Some(pl) = &self.property_length {
            bufs.push(IoSlice::new(pl.as_bytes()));
        }
        if let Some(ref props) = self.props {
            bufs.append(&mut props.to_buffers());
        }

        bufs
    }

    /// Converts the PUBREC packet into a continuous buffer for no-std environments.
    ///
    /// This method serializes the entire packet into a single contiguous byte vector,
    /// which is suitable for no-std environments where `IoSlice` is not available.
    /// The resulting buffer contains the complete MQTT v5.0 PUBREC packet ready
    /// for transmission over a network connection.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the complete packet data in MQTT wire format:
    /// - Fixed header (1 byte): Packet type and flags
    /// - Remaining length (1-4 bytes): Variable length encoding
    /// - Packet identifier (2 bytes): Matching the PUBLISH packet
    /// - Reason code (1 byte): Optional, if present
    /// - Property length (1-4 bytes): Optional, if properties present
    /// - Properties: Optional MQTT v5.0 properties
    ///
    /// # Performance
    ///
    /// This method allocates a new vector and copies packet data into it. For
    /// zero-copy operations in std environments, use [`to_buffers()`] instead.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = pubrec.to_continuous_buffer();
    /// // Use buffer for writing to network streams
    /// // stream.write_all(&buffer).await?;
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.extend_from_slice(self.packet_id_buf.as_ref());
        if let Some(rc_buf) = &self.reason_code_buf {
            buf.extend_from_slice(rc_buf);
        }
        if let Some(pl) = &self.property_length {
            buf.extend_from_slice(pl.as_bytes());
        }
        if let Some(ref props) = self.props {
            buf.append(&mut props.to_continuous_buffer());
        }
        buf
    }

    /// Parses a PUBREC packet from raw byte data.
    ///
    /// This method parses the variable header and properties of a PUBREC packet
    /// from the provided byte slice. The fixed header should have already been
    /// parsed and removed from the data.
    ///
    /// # Parameters
    ///
    /// * `data` - The raw packet data excluding the fixed header
    ///
    /// # Returns
    ///
    /// - `Ok((GenericPubrec, usize))` - The parsed PUBREC packet and number of bytes consumed
    /// - `Err(MqttError)` - If the packet data is malformed or invalid
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - The packet identifier is zero (invalid per MQTT specification)
    /// - The reason code is invalid
    /// - The packet structure is malformed
    ///
    /// Returns `MqttError::ProtocolError` if:
    /// - Invalid properties are present
    /// - Multiple ReasonString properties are included
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Assume `packet_data` contains valid PUBREC packet bytes
    /// let packet_data = &[0x00, 0x01]; // packet_id = 1
    /// let (pubrec, consumed) = mqtt::packet::v5_0::Pubrec::parse(packet_data).unwrap();
    /// assert_eq!(pubrec.packet_id(), 1u16);
    /// assert_eq!(consumed, 2);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;

        // packet_id
        let buffer_size = core::mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
        if data.len() < buffer_size {
            return Err(MqttError::MalformedPacket);
        }

        let all_zeros = &data[0..buffer_size].iter().all(|&b| b == 0);
        if *all_zeros {
            return Err(MqttError::MalformedPacket);
        }

        let mut packet_id_buf = PacketIdType::Buffer::default();
        packet_id_buf
            .as_mut()
            .copy_from_slice(&data[0..buffer_size]);
        cursor += buffer_size;

        // reason_code
        let reason_code_buf = if cursor < data.len() {
            let rc = data[cursor];
            let _ = PubrecReasonCode::try_from(rc).map_err(|_| MqttError::MalformedPacket)?;
            cursor += 1;
            Some([rc])
        } else {
            None
        };

        // properties
        let (property_length, props) = if reason_code_buf.is_some() && cursor < data.len() {
            let (props, consumed) = Properties::parse(&data[cursor..])?;
            cursor += consumed;
            validate_pubrec_properties(&props)?;
            let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

            (Some(prop_len), Some(props))
        } else {
            (None, None)
        };

        let remaining_size = buffer_size
            + reason_code_buf.as_ref().map_or(0, |_| 1)
            + property_length.as_ref().map_or(0, |pl| pl.size())
            + props.as_ref().map_or(0, |ps| ps.size());

        let pubrec = GenericPubrec {
            fixed_header: [FixedHeader::Pubrec.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            packet_id_buf,
            reason_code_buf,
            property_length,
            props,
        };

        Ok((pubrec, cursor))
    }
}

/// Builder for constructing PUBREC packets.
///
/// The `GenericPubrecBuilder` provides a fluent interface for building PUBREC packets
/// with various optional components. All PUBREC packets require a packet identifier,
/// while reason codes and properties are optional.
///
/// # Builder Pattern
///
/// The builder follows the owned pattern, consuming the builder instance on each
/// method call and returning a new instance. The final `build()` method validates
/// the configuration and produces a `GenericPubrec` instance.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// // Minimal PUBREC packet
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// // PUBREC with reason code and properties
/// let properties = vec![
///     mqtt::packet::Property::ReasonString("Message received".to_string()),
/// ];
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(456u16)
///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
///     .props(properties)
///     .build()
///     .unwrap();
/// ```
impl<PacketIdType> GenericPubrecBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the PUBREC packet.
    ///
    /// The packet identifier must match the packet identifier of the PUBLISH
    /// packet being acknowledged. It cannot be zero as per MQTT specification.
    ///
    /// # Parameters
    ///
    /// * `id` - The packet identifier (must be non-zero)
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrec.packet_id(), 42u16);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Sets the reason code for the PUBREC packet.
    ///
    /// The reason code indicates the result of processing the PUBLISH message.
    /// If not set, the absence of a reason code implies success.
    ///
    /// # Parameters
    ///
    /// * `rc` - The reason code indicating the result of message processing
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .reason_code(mqtt::result_code::PubrecReasonCode::NoMatchingSubscribers)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrec.reason_code(), Some(mqtt::result_code::PubrecReasonCode::NoMatchingSubscribers));
    /// ```
    pub fn reason_code(mut self, rc: PubrecReasonCode) -> Self {
        self.reason_code_buf = Some(Some([rc as u8]));
        self
    }

    /// Validates the builder configuration.
    ///
    /// This internal method ensures that:
    /// - A packet identifier has been set and is non-zero
    /// - Properties are only included if a reason code is also present
    /// - Properties contain only valid PUBREC property types
    /// - At most one ReasonString property is included
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the configuration is valid
    /// - `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - No packet identifier is set
    /// - The packet identifier is zero
    /// - Properties are set without a reason code
    ///
    /// Returns `MqttError::ProtocolError` if:
    /// - Invalid properties are included
    /// - Multiple ReasonString properties are present
    fn validate(&self) -> Result<(), MqttError> {
        if self.packet_id_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }

        let packet_id_bytes = self.packet_id_buf.as_ref().unwrap().as_ref();
        let all_zeros = packet_id_bytes.iter().all(|&b| b == 0);
        if all_zeros {
            return Err(MqttError::MalformedPacket);
        }

        if self.reason_code_buf.is_none() && self.props.is_some() {
            return Err(MqttError::MalformedPacket);
        }
        if self.props.is_some() {
            let inner_option = self.props.as_ref().unwrap();
            let props = inner_option
                .as_ref()
                .expect("INTERNAL ERRORS: props was set with None value, this should never happen");
            validate_pubrec_properties(props)?;
        }
        Ok(())
    }

    /// Builds the PUBREC packet from the configured parameters.
    ///
    /// This method validates the builder configuration and constructs a complete
    /// PUBREC packet. It calculates the remaining length, property sizes, and
    /// arranges all components according to the MQTT v5.0 specification.
    ///
    /// # Returns
    ///
    /// - `Ok(GenericPubrec<PacketIdType>)` - The constructed PUBREC packet
    /// - `Err(MqttError)` - If the configuration is invalid
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - No packet identifier is set
    /// - The packet identifier is zero
    /// - Properties are set without a reason code
    ///
    /// Returns `MqttError::ProtocolError` if:
    /// - Invalid properties are included
    /// - Multiple ReasonString properties are present
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrec.packet_id(), 123u16);
    /// assert_eq!(pubrec.reason_code(), Some(mqtt::result_code::PubrecReasonCode::Success));
    /// ```
    pub fn build(self) -> Result<GenericPubrec<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let reason_code_buf = self.reason_code_buf.flatten();
        let props = self.props.flatten();
        let props_size: usize = props.as_ref().map_or(0, |p| p.size());
        // property_length only if properties are present
        let property_length = if props.is_some() {
            Some(VariableByteInteger::from_u32(props_size as u32).unwrap())
        } else {
            None
        };

        let mut remaining = mem::size_of::<PacketIdType>();
        // add reason code if present
        if reason_code_buf.is_some() {
            remaining += 1;
        }
        // add properties if present
        if let Some(ref pl) = property_length {
            remaining += pl.size() + props_size;
        }
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericPubrec {
            fixed_header: [FixedHeader::Pubrec.as_u8()],
            remaining_length,
            packet_id_buf,
            reason_code_buf,
            property_length,
            props,
        })
    }
}

/// Implementation of `Serialize` trait for JSON serialization of PUBREC packets.
///
/// This implementation serializes a PUBREC packet into a JSON object containing
/// the packet type, packet identifier, optional reason code, and optional properties.
/// The serialization format is suitable for debugging, logging, and API responses.
///
/// # Serialized Fields
///
/// - `type`: Always "pubrec" (string)
/// - `packet_id`: The packet identifier (number)
/// - `reason_code`: The reason code (optional, only if present)
/// - `props`: The properties (optional, only if present)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(123u16)
///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&pubrec).unwrap();
/// // JSON: {"type":"pubrec","packet_id":123,"reason_code":"Success"}
/// ```
impl<PacketIdType> Serialize for GenericPubrec<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 2; // type, packet_id

        if self.reason_code_buf.is_some() {
            field_count += 1; // reason_code
        }

        if self.props.is_some() {
            field_count += 1; // props
        }

        let mut state = serializer.serialize_struct("pubrec", field_count)?;
        state.serialize_field("type", PacketType::Pubrec.as_str())?;
        state.serialize_field("packet_id", &self.packet_id())?;
        if self.reason_code_buf.is_some() {
            state.serialize_field("reason_code", &self.reason_code())?;
        }
        if let Some(props) = &self.props {
            state.serialize_field("props", props)?;
        }

        state.end()
    }
}

/// Implementation of `Display` trait for human-readable PUBREC packet representation.
///
/// This implementation formats the PUBREC packet as a JSON string for easy reading
/// and debugging. It leverages the `Serialize` implementation to generate consistent
/// output across different display contexts.
///
/// # Output Format
///
/// The output is a JSON object containing all relevant packet information:
/// - Packet type and identifier
/// - Reason code (if present)
/// - Properties (if present)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(42u16)
///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
///     .build()
///     .unwrap();
///
/// println!("{}", pubrec);
/// // Output: {"type":"pubrec","packet_id":42,"reason_code":"Success"}
/// ```
impl<PacketIdType> fmt::Display for GenericPubrec<PacketIdType>
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

/// Implementation of `Debug` trait for PUBREC packet debugging.
///
/// This implementation delegates to the `Display` implementation to provide
/// consistent formatting for debug output. The JSON format makes it easy to
/// inspect packet contents during development and debugging.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// println!("{:?}", pubrec);
/// // Output: {"type":"pubrec","packet_id":123}
/// ```
impl<PacketIdType> fmt::Debug for GenericPubrec<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Implementation of `GenericPacketTrait` for PUBREC packets.
///
/// This trait provides common packet functionality such as size calculation
/// and buffer serialization. It allows PUBREC packets to be treated uniformly
/// with other MQTT packet types in generic contexts.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// // Use trait methods
/// let size = pubrec.size();
/// let buffers = pubrec.to_buffers();
/// ```
impl<PacketIdType> GenericPacketTrait for GenericPubrec<PacketIdType>
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

/// Implementation of `GenericPacketDisplay` for PUBREC packets.
///
/// This trait provides consistent display and debug formatting for PUBREC packets
/// when used in generic packet contexts. It delegates to the standard `Display`
/// and `Debug` trait implementations.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v5_0::Pubrec::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// // Use in generic packet display contexts
/// println!("{}", pubrec); // Uses fmt_display
/// println!("{:?}", pubrec); // Uses fmt_debug
/// ```
impl<PacketIdType> GenericPacketDisplay for GenericPubrec<PacketIdType>
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

/// Validates that the provided properties are valid for PUBREC packets.
///
/// According to the MQTT v5.0 specification, PUBREC packets can only contain
/// specific property types. This function ensures that only allowed properties
/// are present and that certain properties do not appear multiple times.
///
/// # Allowed Properties
///
/// - `ReasonString`: Human readable string designed for diagnostics (at most one)
/// - `UserProperty`: Name-value pairs for application-specific metadata (unlimited)
///
/// # Parameters
///
/// * `props` - Slice of properties to validate
///
/// # Returns
///
/// - `Ok(())` if all properties are valid
/// - `Err(MqttError::ProtocolError)` if invalid properties are found
///
/// # Errors
///
/// Returns `MqttError::ProtocolError` if:
/// - Any property other than `ReasonString` or `UserProperty` is present
/// - More than one `ReasonString` property is present
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Valid properties
/// let valid_props = vec![
///     mqtt::packet::Property::ReasonString("Message received".to_string()),
///     mqtt::packet::Property::UserProperty(("key".to_string(), "value".to_string())),
/// ];
/// // This would pass validation
///
/// // Invalid properties
/// let invalid_props = vec![
///     mqtt::packet::Property::ContentType("text/plain".to_string()), // Not allowed in PUBREC
/// ];
/// // This would fail validation
/// ```
fn validate_pubrec_properties(props: &[Property]) -> Result<(), MqttError> {
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
