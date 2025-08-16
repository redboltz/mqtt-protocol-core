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
use std::io::IoSlice;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::packet::{
    Properties, PropertiesParse, PropertiesSize, PropertiesToBuffers, Property,
};
use crate::mqtt::result_code::MqttError;
use crate::mqtt::result_code::PubcompReasonCode;

/// A PUBCOMP packet for MQTT v5.0 protocol.
///
/// The PUBCOMP packet is the fourth and final packet in the QoS 2 PUBLISH message flow.
/// It is sent by the sender of the original PUBLISH packet in response to a PUBREL packet
/// from the receiver, completing the QoS 2 message delivery guarantee.
///
/// # MQTT v5.0 Specification
///
/// According to the MQTT v5.0 specification, the PUBCOMP packet:
/// - Is the final packet in the QoS 2 PUBLISH message handshake
/// - Is sent by the original sender in response to a PUBREL packet
/// - Contains the same Packet Identifier as the original PUBLISH packet
/// - May optionally include a reason code indicating the result of the message completion
/// - May optionally include properties for additional metadata
/// - Completes the QoS 2 message delivery flow: PUBLISH -> PUBREC -> PUBREL -> PUBCOMP
///
/// # Generic Support
///
/// This struct supports generic packet identifiers through the `PacketIdType` parameter,
/// allowing for extended packet ID ranges (e.g., u32) for broker clustering scenarios.
/// The standard type alias `Pubcomp` uses `u16` packet identifiers as per MQTT specification.
///
/// # QoS 2 Message Flow
///
/// The PUBCOMP packet is part of the QoS 2 message delivery flow:
/// 1. Sender sends PUBLISH packet with QoS 2
/// 2. Receiver responds with PUBREC packet
/// 3. Sender responds with PUBREL packet
/// 4. Receiver responds with PUBCOMP packet (this packet)
///
/// # Examples
///
/// Creating a basic PUBCOMP packet:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubcomp.packet_id(), 123u16);
/// ```
///
/// Creating a PUBCOMP with reason code:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(456u16)
///     .reason_code(mqtt::result_code::PubcompReasonCode::Success)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubcomp.packet_id(), 456u16);
/// assert_eq!(pubcomp.reason_code(), Some(mqtt::result_code::PubcompReasonCode::Success));
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericPubcomp<PacketIdType>
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

    /// Optional MQTT v5.0 properties associated with this PUBCOMP packet.
    ///
    /// Properties can include:
    /// - `ReasonString`: Human readable string designed for diagnostics
    /// - `UserProperty`: Name-value pairs for application-specific metadata
    ///
    /// Only one `ReasonString` property is allowed per packet.
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Option<Properties>,
}

/// Type alias for PUBCOMP packet with standard u16 packet identifiers.
///
/// This is the standard PUBCOMP packet type that most applications should use,
/// conforming to the MQTT v5.0 specification's u16 packet identifier requirement.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
/// ```
pub type Pubcomp = GenericPubcomp<u16>;

impl<PacketIdType> GenericPubcomp<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a PUBCOMP packet.
    ///
    /// # Returns
    ///
    /// A new `GenericPubcompBuilder` instance for building PUBCOMP packets.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v5_0::Pubcomp::builder();
    /// let pubcomp = builder
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericPubcompBuilder<PacketIdType> {
        GenericPubcompBuilder::<PacketIdType>::default()
    }

    /// Returns the packet type for PUBCOMP packets.
    ///
    /// # Returns
    ///
    /// Always returns `PacketType::Pubcomp`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let packet_type = mqtt::packet::v5_0::Pubcomp::packet_type();
    /// assert_eq!(packet_type, mqtt::packet::packet_type::PacketType::Pubcomp);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Pubcomp
    }

    /// Returns the packet identifier of this PUBCOMP packet.
    ///
    /// The packet identifier must match the packet identifier of the
    /// original PUBLISH packet in the QoS 2 message flow.
    ///
    /// # Returns
    ///
    /// The packet identifier as the specified `PacketIdType`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(1337u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubcomp.packet_id(), 1337u16);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Returns the reason code of this PUBCOMP packet, if present.
    ///
    /// The reason code indicates the result of the QoS 2 message completion.
    /// If no reason code is present, it implies successful completion
    /// (equivalent to `PubcompReasonCode::Success`).
    ///
    /// # Returns
    ///
    /// An `Option<PubcompReasonCode>` containing the reason code if present,
    /// or `None` if no reason code was included in the packet.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// // PUBCOMP without reason code (implies success)
    /// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubcomp.reason_code(), None);
    ///
    /// // PUBCOMP with explicit reason code
    /// let pubcomp_with_reason = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(2u16)
    ///     .reason_code(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubcomp_with_reason.reason_code(),
    ///            Some(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound));
    /// ```
    pub fn reason_code(&self) -> Option<PubcompReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| PubcompReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of the PUBCOMP packet in bytes.
    ///
    /// This includes the fixed header, remaining length field, packet identifier,
    /// optional reason code, optional property length, and optional properties.
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
    /// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = pubcomp.size();
    /// // Minimum size: 1 (fixed header) + 1 (remaining length) + 2 (packet id) = 4 bytes
    /// assert!(size >= 4);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the PUBCOMP packet into a vector of I/O slices for efficient transmission.
    ///
    /// This method prepares the packet data for network transmission by organizing
    /// it into a series of byte slices that can be sent without additional copying.
    ///
    /// # Returns
    ///
    /// A `Vec<IoSlice<'_>>` containing the packet data organized as I/O slices.
    /// The slices are ordered as: fixed header, remaining length, packet identifier,
    /// optional reason code, optional property length, and optional properties.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = pubcomp.to_buffers();
    /// assert!(!buffers.is_empty());
    /// ```
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

    /// Parses a PUBCOMP packet from raw bytes.
    ///
    /// This method deserializes a byte array into a PUBCOMP packet structure,
    /// validating the packet format and extracting all components including
    /// packet identifier, optional reason code, and optional properties.
    ///
    /// # Parameters
    ///
    /// * `data` - A byte slice containing the PUBCOMP packet data (excluding fixed header and remaining length)
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok((Self, usize))` - The parsed PUBCOMP packet and the number of bytes consumed
    /// - `Err(MqttError)` - An error if the packet is malformed or invalid
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - The packet identifier is zero (invalid)
    /// - The data is too short to contain a valid packet identifier
    /// - The reason code is invalid
    /// - Properties are malformed
    ///
    /// Returns `MqttError::ProtocolError` if:
    /// - Invalid properties are present for PUBCOMP packets
    /// - More than one ReasonString property is present
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse a minimal PUBCOMP packet (packet ID only)
    /// let data = [0x00, 0x01]; // packet ID = 1
    /// let (pubcomp, consumed) = mqtt::packet::v5_0::Pubcomp::parse(&data).unwrap();
    /// assert_eq!(pubcomp.packet_id(), 1u16);
    /// assert_eq!(consumed, 2);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;

        // packet_id
        let buffer_size = std::mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
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
            let _ = PubcompReasonCode::try_from(rc).map_err(|_| MqttError::MalformedPacket)?;
            cursor += 1;
            Some([rc])
        } else {
            None
        };

        // properties
        let (property_length, props) = if reason_code_buf.is_some() && cursor < data.len() {
            let (props, consumed) = Properties::parse(&data[cursor..])?;
            cursor += consumed;
            validate_pubcomp_properties(&props)?;
            let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

            (Some(prop_len), Some(props))
        } else {
            (None, None)
        };

        let remaining_size = buffer_size
            + reason_code_buf.as_ref().map_or(0, |_| 1)
            + property_length.as_ref().map_or(0, |pl| pl.size())
            + props.as_ref().map_or(0, |ps| ps.size());

        let pubcomp = GenericPubcomp {
            fixed_header: [FixedHeader::Pubcomp.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            packet_id_buf,
            reason_code_buf,
            property_length,
            props,
        };

        Ok((pubcomp, cursor))
    }
}

/// Builder implementation for constructing PUBCOMP packets.
///
/// The builder provides a fluent interface for creating PUBCOMP packets with
/// optional components like reason codes and properties. All PUBCOMP packets
/// require a packet identifier.
impl<PacketIdType> GenericPubcompBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the PUBCOMP packet.
    ///
    /// The packet identifier must match the packet identifier of the original
    /// PUBLISH packet in the QoS 2 message flow. The identifier must be non-zero.
    ///
    /// # Parameters
    ///
    /// * `id` - The packet identifier to set
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
    /// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(1234u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubcomp.packet_id(), 1234u16);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Sets the reason code for the PUBCOMP packet.
    ///
    /// The reason code indicates the result of the QoS 2 message completion.
    /// If not set, successful completion is implied.
    ///
    /// # Parameters
    ///
    /// * `rc` - The reason code to set
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
    /// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .reason_code(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubcomp.reason_code(),
    ///            Some(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound));
    /// ```
    pub fn reason_code(mut self, rc: PubcompReasonCode) -> Self {
        self.reason_code_buf = Some(Some([rc as u8]));
        self
    }

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
            validate_pubcomp_properties(props)?;
        }
        Ok(())
    }

    /// Builds the PUBCOMP packet from the configured builder.
    ///
    /// This method performs validation of the packet configuration and constructs
    /// the final PUBCOMP packet. All required fields must be set before calling build.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok(GenericPubcomp<PacketIdType>)` - The constructed PUBCOMP packet
    /// - `Err(MqttError)` - An error if the packet configuration is invalid
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - No packet identifier is set
    /// - The packet identifier is zero
    /// - Properties are set without a reason code (protocol violation)
    /// - Invalid properties are present for PUBCOMP packets
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// // Build a basic PUBCOMP packet
    /// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// // Build a PUBCOMP packet with reason code
    /// let pubcomp_with_reason = mqtt::packet::v5_0::Pubcomp::builder()
    ///     .packet_id(43u16)
    ///     .reason_code(mqtt::result_code::PubcompReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<GenericPubcomp<PacketIdType>, MqttError> {
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

        Ok(GenericPubcomp {
            fixed_header: [FixedHeader::Pubcomp.as_u8()],
            remaining_length,
            packet_id_buf,
            reason_code_buf,
            property_length,
            props,
        })
    }
}

/// Serialize implementation for PUBCOMP packets.
///
/// This implementation allows PUBCOMP packets to be serialized to JSON format
/// for debugging, logging, and testing purposes. The serialization includes
/// the packet type, packet identifier, optional reason code, and optional properties.
///
/// # Serialized Format
///
/// The JSON format includes:
/// - `type`: Always "pubcomp"
/// - `packet_id`: The packet identifier
/// - `reason_code`: The reason code (if present)
/// - `props`: The MQTT properties (if present)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use serde_json;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&pubcomp).unwrap();
/// // JSON: {"type":"pubcomp","packet_id":123}
/// ```
impl<PacketIdType> Serialize for GenericPubcomp<PacketIdType>
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

        let mut state = serializer.serialize_struct("pubcomp", field_count)?;
        state.serialize_field("type", PacketType::Pubcomp.as_str())?;
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

/// Display implementation for PUBCOMP packets.
///
/// Formats the PUBCOMP packet as a JSON string for human-readable output.
/// This is useful for logging, debugging, and diagnostic purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(42u16)
///     .build()
///     .unwrap();
///
/// println!("{}", pubcomp);
/// // Output: {"type":"pubcomp","packet_id":42}
/// ```
impl<PacketIdType> fmt::Display for GenericPubcomp<PacketIdType>
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

/// Debug implementation for PUBCOMP packets.
///
/// Uses the same formatting as Display, providing JSON output for debugging.
/// This ensures consistent representation in debug output, logs, and error messages.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(42u16)
///     .build()
///     .unwrap();
///
/// println!("{:?}", pubcomp);
/// // Output: {"type":"pubcomp","packet_id":42}
/// ```
impl<PacketIdType> fmt::Debug for GenericPubcomp<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// GenericPacketTrait implementation for PUBCOMP packets.
///
/// This trait provides a unified interface for all MQTT packet types,
/// enabling generic packet handling and processing.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
///
/// // Use generic packet interface
/// let packet_size = pubcomp.size();
/// let buffers = pubcomp.to_buffers();
/// ```
impl<PacketIdType> GenericPacketTrait for GenericPubcomp<PacketIdType>
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

/// GenericPacketDisplay implementation for PUBCOMP packets.
///
/// This trait provides unified display formatting for all MQTT packet types,
/// enabling consistent output across different packet implementations.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
///
/// // Use generic display interface
/// println!("{}", pubcomp); // Uses fmt_display
/// println!("{:?}", pubcomp); // Uses fmt_debug
/// ```
impl<PacketIdType> GenericPacketDisplay for GenericPubcomp<PacketIdType>
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

/// Validates MQTT v5.0 properties for PUBCOMP packets.
///
/// According to the MQTT v5.0 specification, PUBCOMP packets may only contain
/// specific properties. This function ensures that only valid properties are
/// present and that property count restrictions are enforced.
///
/// # Valid Properties for PUBCOMP
///
/// - `ReasonString`: Human readable string designed for diagnostics (maximum 1)
/// - `UserProperty`: Name-value pairs for application-specific metadata (unlimited)
///
/// # Parameters
///
/// * `props` - A slice of properties to validate
///
/// # Returns
///
/// A `Result` indicating:
/// - `Ok(())` if all properties are valid
/// - `Err(MqttError::ProtocolError)` if invalid properties are found or restrictions are violated
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
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// // Valid properties
/// let valid_props = vec![
///     mqtt::packet::Property::ReasonString("Success".to_string()),
///     mqtt::packet::Property::UserProperty(("key".to_string(), "value".to_string())),
/// ];
///
/// // This would pass validation
/// // validate_pubcomp_properties(&valid_props).unwrap();
/// ```
fn validate_pubcomp_properties(props: &[Property]) -> Result<(), MqttError> {
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
