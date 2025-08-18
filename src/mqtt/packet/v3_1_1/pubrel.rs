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
use crate::mqtt::result_code::MqttError;
use crate::mqtt::result_code::PubrelReasonCode;

/// A PUBREL packet for MQTT v3.1.1 protocol.
///
/// The PUBREL packet is the third packet in the QoS 2 PUBLISH message exchange sequence.
/// It is sent in response to a PUBREC packet and triggers the final PUBCOMP response.
/// This packet instructs the receiver to release any stored state for the specified
/// packet identifier and complete the QoS 2 delivery sequence.
///
/// # MQTT v3.1.1 Specification
///
/// According to the MQTT v3.1.1 specification (Section 3.6), the PUBREL packet:
/// - Is sent by the sender of a PUBLISH packet with QoS 2 in response to a PUBREC
/// - Contains the same Packet Identifier as the original PUBLISH and PUBREC packets
/// - Has a fixed header with packet type 6 (0110) and reserved bits set to 0010
/// - May optionally include a reason code indicating the result of the release operation
/// - Must be acknowledged with a PUBCOMP packet
/// - Has a fixed header format where bits 3,2,1,0 of byte 1 are reserved and must be set to 0,0,1,0
///
/// # QoS 2 Message Flow
///
/// The PUBREL packet is part of the four-packet QoS 2 handshake:
/// 1. PUBLISH (QoS 2) -> 2. PUBREC -> 3. **PUBREL** -> 4. PUBCOMP
///
/// # Generic Support
///
/// This struct supports generic packet identifiers through the `PacketIdType` parameter,
/// allowing for extended packet ID ranges (e.g., u32) for broker clustering scenarios.
/// The standard type alias `Pubrel` uses `u16` packet identifiers as per MQTT specification.
///
/// # Examples
///
/// Creating a basic PUBREL packet:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubrel.packet_id(), 123u16);
/// ```
///
/// Creating a PUBREL with reason code:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
///     .packet_id(456u16)
///     .reason_code(mqtt::result_code::PubrelReasonCode::Success)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubrel.packet_id(), 456u16);
/// assert_eq!(pubrel.reason_code(), Some(mqtt::result_code::PubrelReasonCode::Success));
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericPubrel<PacketIdType>
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
}

/// Type alias for PUBREL packet with standard u16 packet identifiers.
///
/// This is the standard PUBREL packet type that most applications should use,
/// conforming to the MQTT v3.1.1 specification's u16 packet identifier requirement.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
/// ```
pub type Pubrel = GenericPubrel<u16>;

impl<PacketIdType> GenericPubrel<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a PUBREL packet.
    ///
    /// # Returns
    ///
    /// A new `GenericPubrelBuilder` instance for building PUBREL packets.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v3_1_1::Pubrel::builder();
    /// let pubrel = builder
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericPubrelBuilder<PacketIdType> {
        GenericPubrelBuilder::<PacketIdType>::default()
    }

    /// Returns the packet type for PUBREL packets.
    ///
    /// # Returns
    ///
    /// Always returns `PacketType::Pubrel`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let packet_type = mqtt::packet::v3_1_1::Pubrel::packet_type();
    /// assert_eq!(packet_type, mqtt::packet::packet_type::PacketType::Pubrel);
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Pubrel
    }

    /// Returns the packet identifier of this PUBREL packet.
    ///
    /// The packet identifier must match the identifier used in the original
    /// PUBLISH and PUBREC packets in the QoS 2 message exchange sequence.
    /// According to MQTT v3.1.1 specification, the packet identifier must be
    /// the same as the one used in the corresponding PUBLISH packet.
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
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(1337u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrel.packet_id(), 1337u16);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Returns the reason code of this PUBREL packet, if present.
    ///
    /// In MQTT v3.1.1, the reason code is an optional field that can be included
    /// to provide additional information about the packet identifier release operation.
    /// If no reason code is present, it implies successful processing
    /// (equivalent to `PubrelReasonCode::Success`).
    ///
    /// Note: Reason codes are primarily a feature of MQTT v5.0, but this implementation
    /// provides backward compatibility support for extended MQTT v3.1.1 implementations.
    ///
    /// Available reason codes:
    /// - `Success`: The packet identifier has been released successfully
    /// - `PacketIdentifierNotFound`: The specified packet identifier was not found
    ///
    /// # Returns
    ///
    /// An `Option<PubrelReasonCode>` containing the reason code if present,
    /// or `None` if no reason code was included in the packet.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(123u16)
    ///     .reason_code(mqtt::result_code::PubrelReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrel.reason_code(),
    ///            Some(mqtt::result_code::PubrelReasonCode::Success));
    /// ```
    pub fn reason_code(&self) -> Option<PubrelReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| PubrelReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of this PUBREL packet in bytes.
    ///
    /// This includes the fixed header (1 byte), remaining length field,
    /// variable header (packet identifier), and optional reason code.
    /// The calculation follows the MQTT v3.1.1 specification format.
    ///
    /// # Returns
    ///
    /// The total packet size in bytes as a `usize`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = pubrel.size();
    /// // Minimum size: 1 (fixed header) + 1 (remaining length) + 2 (packet id)
    /// assert!(size >= 4);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts this PUBREL packet into a vector of I/O slices for efficient network transmission.
    ///
    /// This method creates zero-copy I/O slices that can be used with vectored I/O operations
    /// for efficient packet transmission over network connections. The slices are ordered
    /// according to the MQTT v3.1.1 packet structure.
    ///
    /// # Returns
    ///
    /// A `Vec<IoSlice<'_>>` containing slices of the packet data ready for transmission.
    /// The slices are ordered as: fixed header, remaining length, packet identifier, optional reason code.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = pubrel.to_buffers();
    /// // Can be used with vectored write operations
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

        bufs
    }

    /// Converts the PUBREL packet into a continuous buffer for no-std environments.
    ///
    /// This method serializes the entire packet into a single contiguous byte vector,
    /// which is suitable for no-std environments where IoSlice is not available.
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the complete packet data.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = pubrel.to_continuous_buffer();
    /// // Use buffer for writing to network streams
    /// ```
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.extend_from_slice(self.packet_id_buf.as_ref());
        if let Some(rc_buf) = &self.reason_code_buf {
            buf.extend_from_slice(rc_buf);
        }
        buf
    }

    /// Parses a PUBREL packet from raw bytes.
    ///
    /// This method deserializes a PUBREL packet from its wire format according to
    /// the MQTT v3.1.1 specification. It validates the packet structure and ensures
    /// all components are correctly formatted.
    ///
    /// # Parameters
    ///
    /// * `data` - The raw byte slice containing the PUBREL packet data (excluding fixed header)
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok((GenericPubrel<PacketIdType>, usize))` - The parsed packet and number of bytes consumed
    /// - `Err(MqttError)` - Parsing error if the packet is malformed
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - The packet identifier is zero (invalid per MQTT v3.1.1 specification)
    /// - The reason code is invalid (when present)
    /// - The packet is truncated or malformed
    /// - The packet data is shorter than the minimum required length
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let packet_data = &[0x00, 0x01]; // Packet ID = 1
    /// match mqtt::packet::v3_1_1::Pubrel::parse(packet_data) {
    ///     Ok((pubrel, consumed)) => {
    ///         assert_eq!(pubrel.packet_id(), 1u16);
    ///         assert_eq!(consumed, 2);
    ///     }
    ///     Err(e) => panic!("Parse error: {:?}", e),
    /// }
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
            let _ = PubrelReasonCode::try_from(rc).map_err(|_| MqttError::MalformedPacket)?;
            cursor += 1;
            Some([rc])
        } else {
            None
        };

        let remaining_size = buffer_size + reason_code_buf.as_ref().map_or(0, |_| 1);

        let pubrel = GenericPubrel {
            fixed_header: [FixedHeader::Pubrel.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            packet_id_buf,
            reason_code_buf,
        };

        Ok((pubrel, cursor))
    }
}

/// Builder implementation for constructing PUBREL packets.
///
/// This implementation provides methods for setting packet-specific fields
/// during the construction process using the builder pattern.
impl<PacketIdType> GenericPubrelBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the PUBREL packet.
    ///
    /// The packet identifier must match the identifier used in the original
    /// PUBLISH and PUBREC packets in the QoS 2 message exchange sequence.
    /// The packet identifier cannot be zero as per MQTT v3.1.1 specification.
    ///
    /// # Parameters
    ///
    /// * `id` - The packet identifier to use. Must be non-zero.
    ///
    /// # Returns
    ///
    /// The updated builder instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrel.packet_id(), 123u16);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Sets the reason code for the PUBREL packet.
    ///
    /// The reason code provides additional information about the packet identifier
    /// release operation. Including a reason code is optional in MQTT v3.1.1.
    ///
    /// Note: Reason codes are primarily a feature of MQTT v5.0, but this implementation
    /// provides support for extended MQTT v3.1.1 implementations.
    ///
    /// Available reason codes:
    /// - `Success`: The packet identifier has been released successfully
    /// - `PacketIdentifierNotFound`: The specified packet identifier was not found
    ///
    /// # Parameters
    ///
    /// * `rc` - The reason code to include in the packet.
    ///
    /// # Returns
    ///
    /// The updated builder instance for method chaining.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(456u16)
    ///     .reason_code(mqtt::result_code::PubrelReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrel.reason_code(),
    ///            Some(mqtt::result_code::PubrelReasonCode::Success));
    /// ```
    pub fn reason_code(mut self, rc: PubrelReasonCode) -> Self {
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

        Ok(())
    }

    /// Validates the builder configuration and constructs the PUBREL packet.
    ///
    /// This method performs comprehensive validation of the packet configuration
    /// before constructing the final PUBREL packet. It ensures all MQTT v3.1.1
    /// specification requirements are met.
    ///
    /// # Returns
    ///
    /// A `Result` containing:
    /// - `Ok(GenericPubrel<PacketIdType>)` - The constructed and validated PUBREL packet
    /// - `Err(MqttError)` - Validation error if the configuration is invalid
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - No packet identifier was set (required field)
    /// - The packet identifier is zero (invalid per MQTT v3.1.1 specification)
    /// - The reason code is invalid (when present)
    ///
    /// # Examples
    ///
    /// Successful packet construction:
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .packet_id(42u16)
    ///     .reason_code(mqtt::result_code::PubrelReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    /// ```
    ///
    /// Construction failure due to missing packet ID:
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let result = mqtt::packet::v3_1_1::Pubrel::builder()
    ///     .build();
    ///
    /// assert!(result.is_err());
    /// ```
    pub fn build(self) -> Result<GenericPubrel<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let reason_code_buf = self.reason_code_buf.flatten();

        let mut remaining = mem::size_of::<PacketIdType>();
        // add reason code if present
        if reason_code_buf.is_some() {
            remaining += 1;
        }
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericPubrel {
            fixed_header: [FixedHeader::Pubrel.as_u8()],
            remaining_length,
            packet_id_buf,
            reason_code_buf,
        })
    }
}

/// Serialize implementation for PUBREL packets.
///
/// This implementation allows PUBREL packets to be serialized into various formats
/// (JSON, YAML, etc.) using the serde framework. The serialization includes all
/// relevant packet fields in a structured format.
///
/// # Serialized Fields
///
/// - `type`: Always "pubrel" to identify the packet type
/// - `packet_id`: The packet identifier value
/// - `reason_code`: The reason code (if present)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use serde_json;
///
/// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
///     .packet_id(123u16)
///     .reason_code(mqtt::result_code::PubrelReasonCode::Success)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&pubrel).unwrap();
/// // Result: {"type":"pubrel","packet_id":123,"reason_code":"Success"}
/// ```
impl<PacketIdType> Serialize for GenericPubrel<PacketIdType>
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

        let mut state = serializer.serialize_struct("pubrel", field_count)?;
        state.serialize_field("type", PacketType::Pubrel.as_str())?;
        state.serialize_field("packet_id", &self.packet_id())?;
        if self.reason_code_buf.is_some() {
            state.serialize_field("reason_code", &self.reason_code())?;
        }

        state.end()
    }
}

/// Display trait implementation for PUBREL packets.
///
/// This implementation provides a human-readable string representation of the
/// PUBREL packet in JSON format. This is useful for debugging, logging, and
/// diagnostic purposes.
///
/// # Format
///
/// The output format is a JSON string containing:
/// - `type`: The packet type ("pubrel")
/// - `packet_id`: The packet identifier
/// - `reason_code`: The reason code (if present)
///
/// If serialization fails, an error message is returned in JSON format.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
///     .packet_id(42u16)
///     .build()
///     .unwrap();
///
/// println!("{}", pubrel);
/// // Output: {"type":"pubrel","packet_id":42}
/// ```
impl<PacketIdType> fmt::Display for GenericPubrel<PacketIdType>
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

/// Debug trait implementation for PUBREL packets.
///
/// This implementation provides debug formatting that delegates to the Display
/// implementation, ensuring consistent output format for both debugging and
/// display purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// println!("{:?}", pubrel);
/// // Output: {"type":"pubrel","packet_id":123}
/// ```
impl<PacketIdType> fmt::Debug for GenericPubrel<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Generic packet trait implementation for PUBREL packets.
///
/// This implementation provides the generic packet interface required by the
/// MQTT protocol library framework. It enables PUBREL packets to be used
/// polymorphically with other packet types in the system.
///
/// # Methods
///
/// - `size()`: Returns the total packet size in bytes
/// - `to_buffers()`: Returns I/O slices for efficient transmission
impl<PacketIdType> GenericPacketTrait for GenericPubrel<PacketIdType>
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

/// Generic packet display trait implementation for PUBREL packets.
///
/// This implementation provides generic display formatting capabilities
/// that can be used by the MQTT protocol library framework for consistent
/// packet representation across different display contexts.
///
/// # Methods
///
/// - `fmt_debug()`: Provides debug formatting via the Debug trait
/// - `fmt_display()`: Provides display formatting via the Display trait
impl<PacketIdType> GenericPacketDisplay for GenericPubrel<PacketIdType>
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
