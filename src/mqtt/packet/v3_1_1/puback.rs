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
use crate::mqtt::result_code::PubackReasonCode;

/// A PUBACK packet for MQTT v3.1.1 protocol.
///
/// The PUBACK packet is the response to a PUBLISH packet with QoS level 1.
/// This packet acknowledges the receipt of a PUBLISH packet and may optionally
/// contain a reason code to provide information about the acknowledgment status.
///
/// # MQTT v3.1.1 Specification
///
/// According to the MQTT v3.1.1 specification (<https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html>),
/// the PUBACK packet:
/// - Is sent by the receiver of a PUBLISH packet with QoS 1
/// - Contains the same Packet Identifier as the PUBLISH packet being acknowledged
/// - Has a fixed header with packet type 4 (0100 in binary)
/// - Contains a 2-byte packet identifier in the variable header
/// - May optionally include a single-byte reason code (implementation-specific extension)
/// - Has no payload
///
/// # Packet Structure
///
/// ```text
/// Fixed Header:
/// +--------+--------+
/// | Type=4 | Flags  |  (Packet Type = 4, Flags = 0000)
/// +--------+--------+
/// |  Remaining Length |  (Variable length encoding)
/// +-------------------+
///
/// Variable Header:
/// +--------+--------+
/// |   Packet ID MSB  |
/// +--------+--------+
/// |   Packet ID LSB  |
/// +--------+--------+
/// | Reason Code (opt)|  (Optional, implementation-specific)
/// +--------+--------+
/// ```
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
///
/// let puback = mqtt::packet::v3_1_1::Puback::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(puback.packet_id(), 123u16);
/// assert_eq!(puback.reason_code(), None);
/// ```
///
/// Creating a PUBACK with reason code:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let puback = mqtt::packet::v3_1_1::Puback::builder()
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
pub struct GenericPuback<PacketIdType>
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

/// Type alias for PUBACK packet with standard u16 packet identifiers.
///
/// This is the standard PUBACK packet type that most applications should use,
/// conforming to the MQTT v3.1.1 specification's u16 packet identifier requirement.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let puback = mqtt::packet::v3_1_1::Puback::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
/// ```

impl<PacketIdType> GenericPuback<PacketIdType>
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
    /// let builder = mqtt::packet::v3_1_1::Puback::builder();
    /// let puback = builder
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericPubackBuilder<PacketIdType> {
        GenericPubackBuilder::<PacketIdType>::default()
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
    /// let packet_type = mqtt::packet::v3_1_1::Puback::packet_type();
    /// assert_eq!(packet_type, mqtt::packet::packet_type::PacketType::Puback);
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Puback
    }

    /// Returns the packet identifier of this PUBACK packet.
    ///
    /// The packet identifier must match the packet identifier of the
    /// PUBLISH packet that this PUBACK is acknowledging. According to
    /// MQTT v3.1.1 specification, this identifier is a 16-bit value
    /// that must be non-zero.
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
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
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
    /// In MQTT v3.1.1, reason codes are not part of the standard specification
    /// but may be included as an implementation-specific extension. If no reason
    /// code is present, it typically implies successful acknowledgment.
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
    /// // PUBACK without reason code (standard v3.1.1)
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(puback.reason_code(), None);
    ///
    /// // PUBACK with reason code (implementation extension)
    /// let puback_with_reason = mqtt::packet::v3_1_1::Puback::builder()
    ///     .packet_id(2u16)
    ///     .reason_code(mqtt::result_code::PubackReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(puback_with_reason.reason_code(),
    ///            Some(mqtt::result_code::PubackReasonCode::Success));
    /// ```
    pub fn reason_code(&self) -> Option<PubackReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| PubackReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of the PUBACK packet in bytes.
    ///
    /// This includes the fixed header, remaining length field, packet identifier,
    /// and optional reason code.
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
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
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

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// represent the complete PUBACK packet in wire format.
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
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = puback.to_buffers();
    /// // Use with vectored write: socket.write_vectored(&buffers)?;
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

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is compatible with no-std environments and provides an alternative
    /// to [`to_buffers()`] when vectored I/O is not needed.
    ///
    /// The returned buffer contains the complete PUBACK packet serialized according
    /// to the MQTT v3.1.1 protocol specification, including fixed header, remaining
    /// length, packet identifier, and optional reason code.
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
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = puback.to_continuous_buffer();
    /// // buffer contains all packet bytes
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
        buf
    }

    /// Parses a PUBACK packet from raw bytes.
    ///
    /// This method deserializes a byte array into a PUBACK packet structure,
    /// validating the packet format and extracting all components including
    /// packet identifier and optional reason code.
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
    /// - The packet identifier is zero (invalid according to MQTT v3.1.1)
    /// - The data is too short to contain a valid packet identifier
    /// - The reason code is invalid (if present)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse PUBACK packet data with packet ID = 1
    /// let data = &[0x00, 0x01]; // packet_id = 1
    /// let (puback, consumed) = mqtt::packet::v3_1_1::Puback::parse(data).unwrap();
    ///
    /// assert_eq!(puback.packet_id(), 1u16);
    /// assert_eq!(consumed, 2);
    /// assert_eq!(puback.reason_code(), None);
    ///
    /// // Parse PUBACK packet data with packet ID and reason code
    /// let data_with_reason = &[0x00, 0x02, 0x00]; // packet_id = 2, reason = Success
    /// let (puback_with_reason, consumed) = mqtt::packet::v3_1_1::Puback::parse(data_with_reason).unwrap();
    ///
    /// assert_eq!(puback_with_reason.packet_id(), 2u16);
    /// assert_eq!(consumed, 3);
    /// assert_eq!(puback_with_reason.reason_code(),
    ///            Some(mqtt::result_code::PubackReasonCode::Success));
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

        let remaining_size = buffer_size + reason_code_buf.as_ref().map_or(0, |_| 1);

        let puback = GenericPuback {
            fixed_header: [FixedHeader::Puback.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            packet_id_buf,
            reason_code_buf,
        };

        Ok((puback, cursor))
    }
}

/// Builder implementation for `GenericPuback`.
///
/// Provides methods for constructing PUBACK packets with validation.
/// The builder ensures that all required fields are set and validates
/// the packet structure according to MQTT v3.1.1 specification.
impl<PacketIdType> GenericPubackBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the PUBACK packet.
    ///
    /// The packet identifier must match the packet identifier of the
    /// PUBLISH packet that this PUBACK is acknowledging. According to
    /// MQTT v3.1.1 specification, the packet identifier must be non-zero.
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
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
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
    /// In MQTT v3.1.1, reason codes are not part of the standard specification
    /// but may be included as an implementation-specific extension. Setting a
    /// reason code is optional and provides additional information about the
    /// acknowledgment status.
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
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// // Basic PUBACK
    /// let puback = mqtt::packet::v3_1_1::Puback::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// // PUBACK with reason code
    /// let puback_with_reason = mqtt::packet::v3_1_1::Puback::builder()
    ///     .packet_id(2u16)
    ///     .reason_code(mqtt::result_code::PubackReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<GenericPuback<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let reason_code_buf = self.reason_code_buf.flatten();

        let mut remaining = mem::size_of::<PacketIdType>();
        // add reason code if present
        if reason_code_buf.is_some() {
            remaining += 1;
        }
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericPuback {
            fixed_header: [FixedHeader::Puback.as_u8()],
            remaining_length,
            packet_id_buf,
            reason_code_buf,
        })
    }
}

/// `Serialize` implementation for `GenericPuback`.
///
/// Serializes the PUBACK packet into a structured format suitable for JSON output
/// or other serialization formats. The serialization includes the packet type,
/// packet identifier, and optional reason code.
///
/// # Serialized Fields
///
/// - `type`: Always "puback"
/// - `packet_id`: The packet identifier
/// - `reason_code`: Present only if a reason code was set
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let puback = mqtt::packet::v3_1_1::Puback::builder()
///     .packet_id(42u16)
///     .reason_code(mqtt::result_code::PubackReasonCode::Success)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&puback).unwrap();
/// // JSON will contain: {"type":"puback","packet_id":42,"reason_code":"Success"}
/// ```
impl<PacketIdType> Serialize for GenericPuback<PacketIdType>
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

        let mut state = serializer.serialize_struct("puback", field_count)?;
        state.serialize_field("type", PacketType::Puback.as_str())?;
        state.serialize_field("packet_id", &self.packet_id())?;
        if self.reason_code_buf.is_some() {
            state.serialize_field("reason_code", &self.reason_code())?;
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
/// let puback = mqtt::packet::v3_1_1::Puback::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// println!("{}", puback);
/// // Output: {"type":"puback","packet_id":123}
/// ```
impl<PacketIdType> fmt::Display for GenericPuback<PacketIdType>
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
/// let puback = mqtt::packet::v3_1_1::Puback::builder()
///     .packet_id(456u16)
///     .build()
///     .unwrap();
///
/// println!("{:?}", puback);
/// // Output: {"type":"puback","packet_id":456}
/// ```
impl<PacketIdType> fmt::Debug for GenericPuback<PacketIdType>
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
impl<PacketIdType> GenericPacketTrait for GenericPuback<PacketIdType>
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
impl<PacketIdType> GenericPacketDisplay for GenericPuback<PacketIdType>
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
