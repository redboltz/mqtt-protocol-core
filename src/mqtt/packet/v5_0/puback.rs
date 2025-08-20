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
use crate::mqtt::packet::{GenericProperties, GenericProperty, PropertiesParse, PropertiesSize};
use crate::mqtt::result_code::MqttError;
use crate::mqtt::result_code::PubackReasonCode;

/// A PUBACK packet for MQTT v5.0 protocol.
///
/// The PUBACK packet is the response to a PUBLISH packet with QoS level 1.
/// This packet acknowledges the receipt of a PUBLISH packet and may contain
/// a reason code and properties to provide additional information about the
/// acknowledgment status.
///
/// # MQTT v5.0 Specification
///
/// According to the MQTT v5.0 specification, the PUBACK packet:
/// - Is sent by the receiver of a PUBLISH packet with QoS 1
/// - Contains the same Packet Identifier as the PUBLISH packet being acknowledged
/// - May optionally include a reason code indicating the result of the PUBLISH processing
/// - May optionally include properties for additional metadata
///
/// # Generic Support
///
/// This struct supports generic packet identifiers through the `PacketIdType` parameter,
/// allowing for extended packet ID ranges (e.g., u32) for broker clustering scenarios.
/// The standard type alias `Puback` uses `u16` packet identifiers as per MQTT specification.
///
/// # Examples
///
/// Creating a basic PUBACK packet:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let puback = mqtt::packet::v5_0::Puback::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(puback.packet_id(), 123u16);
/// ```
///
/// Creating a PUBACK with reason code:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let puback = mqtt::packet::v5_0::Puback::builder()
///     .packet_id(456u16)
///     .reason_code(mqtt::result_code::PubackReasonCode::Success)
///     .build()
///     .unwrap();
///
/// assert_eq!(puback.packet_id(), 456u16);
/// assert_eq!(puback.reason_code(), Some(mqtt::result_code::PubackReasonCode::Success));
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericPuback<
    PacketIdType,
    const STRING_BUFFER_SIZE: usize = 32,
    const BINARY_BUFFER_SIZE: usize = 32,
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
    reason_code_buf: Option<[u8; 1]>,
    #[builder(private)]
    property_length: Option<VariableByteInteger>,

    /// Optional MQTT v5.0 properties associated with this PUBACK packet.
    ///
    /// Properties can include:
    /// - `ReasonString`: Human readable string designed for diagnostics
    /// - `UserProperty`: Name-value pairs for application-specific metadata
    ///
    /// Only one `ReasonString` property is allowed per packet.
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Option<GenericProperties<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>>,
}

/// Type alias for PUBACK packet with standard u16 packet identifiers.
///
/// This is the standard PUBACK packet type that most applications should use,
/// conforming to the MQTT v5.0 specification's u16 packet identifier requirement.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let puback = mqtt::packet::v5_0::Puback::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
/// ```
pub type Puback = GenericPuback<u16>;

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a PUBACK packet.
    ///
    /// # Returns
    ///
    /// A new `GenericPubackBuilder` instance for building PUBACK packets.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v5_0::Puback::builder();
    /// let puback = builder
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericPubackBuilder<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE> {
        GenericPubackBuilder::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::default()
    }

    /// Returns the packet type for PUBACK packets.
    ///
    /// # Returns
    ///
    /// Always returns `PacketType::Puback`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let packet_type = mqtt::packet::v5_0::Puback::packet_type();
    /// assert_eq!(packet_type, mqtt::packet::packet_type::PacketType::Puback);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Puback
    }

    /// Returns the packet identifier of this PUBACK packet.
    ///
    /// The packet identifier must match the packet identifier of the
    /// PUBLISH packet that this PUBACK is acknowledging.
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
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(1337u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(puback.packet_id(), 1337u16);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Returns the reason code of this PUBACK packet, if present.
    ///
    /// The reason code indicates the result of the PUBLISH packet processing.
    /// If no reason code is present, it implies successful processing
    /// (equivalent to `PubackReasonCode::Success`).
    ///
    /// # Returns
    ///
    /// An `Option<PubackReasonCode>` containing the reason code if present,
    /// or `None` if no reason code was included in the packet.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// // PUBACK without reason code (implies success)
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(puback.reason_code(), None);
    ///
    /// // PUBACK with explicit reason code
    /// let puback_with_reason = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(2u16)
    ///     .reason_code(mqtt::result_code::PubackReasonCode::NoMatchingSubscribers)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(puback_with_reason.reason_code(),
    ///            Some(mqtt::result_code::PubackReasonCode::NoMatchingSubscribers));
    /// ```
    pub fn reason_code(&self) -> Option<PubackReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| PubackReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of the PUBACK packet in bytes.
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
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = puback.size();
    /// // Minimum size: 1 (fixed header) + 1 (remaining length) + 2 (packet id) = 4 bytes
    /// assert!(size >= 4);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the PUBACK packet into a vector of I/O slices for efficient transmission.
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
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = puback.to_buffers();
    /// assert!(!buffers.is_empty());
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

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method provides an alternative to `to_buffers()` for no-std environments
    /// where vectored I/O is not available.
    ///
    /// The returned buffer contains the complete PUBACK packet serialized according
    /// to the MQTT v5.0 protocol specification, including fixed header, remaining
    /// length, packet identifier, optional reason code, and optional properties.
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
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = puback.to_continuous_buffer();
    /// assert!(!buffer.is_empty());
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

    /// Parses a PUBACK packet from raw bytes.
    ///
    /// This method deserializes a byte array into a PUBACK packet structure,
    /// validating the packet format and extracting all components including
    /// packet identifier, optional reason code, and optional properties.
    ///
    /// # Parameters
    ///
    /// * `data` - A byte slice containing the PUBACK packet data (excluding fixed header and remaining length)
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok((Self, usize))` - The parsed PUBACK packet and the number of bytes consumed
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
    /// - Invalid properties are present for PUBACK packets
    /// - More than one ReasonString property is present
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse PUBACK packet data
    /// let data = &[0x00, 0x01]; // packet_id = 1
    /// let (puback, consumed) = mqtt::packet::v5_0::Puback::parse(data).unwrap();
    ///
    /// assert_eq!(puback.packet_id(), 1u16);
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
            let _ = PubackReasonCode::try_from(rc).map_err(|_| MqttError::MalformedPacket)?;
            cursor += 1;
            Some([rc])
        } else {
            None
        };

        // properties
        let (property_length, props) = if reason_code_buf.is_some() && cursor < data.len() {
            let (props, consumed) =
                GenericProperties::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
                    &data[cursor..],
                )?;
            cursor += consumed;
            validate_puback_properties(&props)?;
            let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

            (Some(prop_len), Some(props))
        } else {
            (None, None)
        };

        let remaining_size = buffer_size
            + reason_code_buf.as_ref().map_or(0, |_| 1)
            + property_length.as_ref().map_or(0, |pl| pl.size())
            + props.as_ref().map_or(0, |ps| ps.size());

        let puback = GenericPuback {
            fixed_header: [FixedHeader::Puback.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            packet_id_buf,
            reason_code_buf,
            property_length,
            props,
        };

        Ok((puback, cursor))
    }
}

/// Builder implementation for `GenericPuback`.
///
/// Provides methods for constructing PUBACK packets with validation.
/// The builder ensures that all required fields are set and validates
/// the packet structure according to MQTT v5.0 specification.
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericPubackBuilder<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the PUBACK packet.
    ///
    /// The packet identifier must match the packet identifier of the
    /// PUBLISH packet that this PUBACK is acknowledging. The packet
    /// identifier must be non-zero.
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
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(puback.packet_id(), 42u16);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Sets the reason code for the PUBACK packet.
    ///
    /// The reason code indicates the result of processing the PUBLISH packet.
    /// If no reason code is set, the packet implies successful processing.
    /// Setting a reason code is optional but recommended for proper error reporting.
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
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(1u16)
    ///     .reason_code(mqtt::result_code::PubackReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(puback.reason_code(), Some(mqtt::result_code::PubackReasonCode::Success));
    /// ```
    pub fn reason_code(mut self, rc: PubackReasonCode) -> Self {
        self.reason_code_buf = Some(Some([rc as u8]));
        self
    }

    /// Validates the builder state before constructing the PUBACK packet.
    ///
    /// This method ensures that:
    /// - A packet identifier has been set and is non-zero
    /// - Properties are only present when a reason code is also present
    /// - Properties contain only valid PUBACK-specific properties
    /// - Only one ReasonString property is present (if any)
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, or `MqttError` if validation fails.
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - No packet identifier is set
    /// - Packet identifier is zero
    /// - Properties are set without a reason code
    ///
    /// Returns `MqttError::ProtocolError` if:
    /// - Invalid properties are present
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
            validate_puback_properties(props)?;
        }
        Ok(())
    }

    /// Builds the PUBACK packet from the configured parameters.
    ///
    /// This method validates all parameters and constructs a complete PUBACK packet
    /// with the correct remaining length calculation and buffer organization.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok(GenericPuback<PacketIdType>)` - The constructed PUBACK packet
    /// - `Err(MqttError)` - An error if validation fails or packet construction is invalid
    ///
    /// # Errors
    ///
    /// Returns validation errors from the `validate()` method, including:
    /// - `MqttError::MalformedPacket` - Required fields missing or invalid
    /// - `MqttError::ProtocolError` - Invalid property configuration
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// // Basic PUBACK
    /// let puback = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// // PUBACK with reason code and properties
    /// let props = mqtt::packet::Properties::new();
    /// let puback_with_props = mqtt::packet::v5_0::Puback::builder()
    ///     .packet_id(2u16)
    ///     .reason_code(mqtt::result_code::PubackReasonCode::Success)
    ///     .props(props)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(
        self,
    ) -> Result<GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>, MqttError>
    {
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

        Ok(GenericPuback {
            fixed_header: [FixedHeader::Puback.as_u8()],
            remaining_length,
            packet_id_buf,
            reason_code_buf,
            property_length,
            props,
        })
    }
}

/// `Serialize` implementation for `GenericPuback`.
///
/// Serializes the PUBACK packet into a structured format suitable for JSON output
/// or other serialization formats. The serialization includes the packet type,
/// packet identifier, optional reason code, and optional properties.
///
/// # Serialized Fields
///
/// - `type`: Always "puback"
/// - `packet_id`: The packet identifier
/// - `reason_code`: Present only if a reason code was set
/// - `props`: Present only if properties were set
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let puback = mqtt::packet::v5_0::Puback::builder()
///     .packet_id(42u16)
///     .reason_code(mqtt::result_code::PubackReasonCode::Success)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&puback).unwrap();
/// // JSON will contain: {"type":"puback","packet_id":42,"reason_code":"Success"}
/// ```
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> Serialize
    for GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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

        let mut state = serializer.serialize_struct("puback", field_count)?;
        state.serialize_field("type", PacketType::Puback.as_str())?;
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

/// `Display` implementation for `GenericPuback`.
///
/// Formats the PUBACK packet as JSON for human-readable output.
/// This is useful for logging, debugging, and diagnostic purposes.
///
/// # Output Format
///
/// The output is JSON formatted and includes all packet fields.
/// If serialization fails, an error message is displayed instead.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let puback = mqtt::packet::v5_0::Puback::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// println!("{}", puback);
/// // Output: {"type":"puback","packet_id":123}
/// ```
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Display
    for GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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

/// `Debug` implementation for `GenericPuback`.
///
/// Uses the same JSON formatting as the `Display` implementation
/// to provide consistent debug output across the library.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let puback = mqtt::packet::v5_0::Puback::builder()
///     .packet_id(456u16)
///     .build()
///     .unwrap();
///
/// println!("{:?}", puback);
/// // Output: {"type":"puback","packet_id":456}
/// ```
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Debug
    for GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// `GenericPacketTrait` implementation for `GenericPuback`.
///
/// Provides the core packet interface methods required by the MQTT protocol
/// library. This trait enables the PUBACK packet to be used generically
/// alongside other packet types in the protocol processing pipeline.
///
/// # Trait Methods
///
/// - `size()`: Returns the total packet size in bytes
/// - `to_buffers()`: Converts the packet to I/O slices for transmission
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericPacketTrait for GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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

/// `GenericPacketDisplay` implementation for `GenericPuback`.
///
/// Provides formatted display capabilities for the PUBACK packet
/// that can be used by the generic packet display system.
/// This enables consistent formatting across all packet types.
///
/// # Trait Methods
///
/// - `fmt_debug()`: Debug formatting (JSON output)
/// - `fmt_display()`: Display formatting (JSON output)
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericPacketDisplay for GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
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

/// Validates properties for PUBACK packets according to MQTT v5.0 specification.
///
/// This function ensures that only valid properties are included in PUBACK packets
/// and that property usage follows the protocol constraints.
///
/// # MQTT v5.0 PUBACK Property Rules
///
/// PUBACK packets may only contain the following properties:
/// - `ReasonString`: Human readable string for diagnostics (max 1 per packet)
/// - `UserProperty`: Application-specific name-value pairs (unlimited)
///
/// # Parameters
///
/// * `props` - A slice of properties to validate
///
/// # Returns
///
/// - `Ok(())` if all properties are valid for PUBACK packets
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
/// // Valid properties - empty
/// let props = vec![];
/// validate_puback_properties(&props).unwrap();
///
/// // Valid properties - reason string and user properties
/// let props = vec![
///     mqtt::packet::Property::ReasonString("Processing failed".to_string()),
///     mqtt::packet::Property::UserProperty(("key".to_string(), "value".to_string())),
/// ];
/// validate_puback_properties(&props).unwrap();
/// ```
fn validate_puback_properties<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>(
    props: &GenericProperties<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
) -> Result<(), MqttError> {
    let mut count_reason_string = 0;
    for prop in props {
        match prop {
            GenericProperty::GenericReasonString(_) => count_reason_string += 1,
            GenericProperty::GenericUserProperty(_) => {}
            _ => return Err(MqttError::ProtocolError),
        }
    }
    if count_reason_string > 1 {
        return Err(MqttError::ProtocolError);
    }
    Ok(())
}
