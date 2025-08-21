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

use crate::mqtt::mqtt_internal::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::mqtt_internal::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::mqtt_internal::packet::GenericPacketDisplay;
use crate::mqtt::mqtt_internal::packet::GenericPacketTrait;
use crate::mqtt::mqtt_internal::packet::IsPacketId;
use crate::mqtt::mqtt_internal::result_code::MqttError;
use crate::mqtt::mqtt_internal::result_code::PubcompReasonCode;

/// A PUBCOMP packet for MQTT v3.1.1 protocol.
///
/// The PUBCOMP packet is the fourth and final packet in the QoS 2 PUBLISH message flow.
/// It is sent by the receiver of a PUBREL packet to complete the QoS 2 message delivery
/// guarantee, acknowledging that the message processing has been completed.
///
/// # MQTT v3.1.1 Specification
///
/// According to the MQTT v3.1.1 specification (<https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html>),
/// the PUBCOMP packet:
/// - Is the final packet in the QoS 2 PUBLISH message handshake
/// - Is sent by the receiver in response to a PUBREL packet from the sender
/// - Contains the same Packet Identifier as the original PUBLISH packet
/// - Completes the QoS 2 message delivery flow: PUBLISH -> PUBREC -> PUBREL -> PUBCOMP
/// - Has no payload and only contains the packet identifier
/// - Unlike MQTT v5.0, it does not support reason codes in the basic v3.1.1 specification
///
/// # Generic Support
///
/// This struct supports generic packet identifiers through the `PacketIdType` parameter,
/// allowing for extended packet ID ranges (e.g., u32) for broker clustering scenarios
/// where packet ID exhaustion needs to be prevented. The standard type alias `Pubcomp`
/// uses `u16` packet identifiers as per MQTT v3.1.1 specification.
///
/// # QoS 2 Message Flow
///
/// The PUBCOMP packet completes the exactly-once delivery guarantee:
/// 1. Sender sends PUBLISH packet with QoS 2
/// 2. Receiver responds with PUBREC packet (acknowledging receipt)
/// 3. Sender responds with PUBREL packet (requesting processing)
/// 4. Receiver responds with PUBCOMP packet (confirming completion) - **this packet**
///
/// After sending PUBCOMP, the receiver can safely discard the message state as the
/// QoS 2 delivery guarantee has been fulfilled.
///
/// # Packet Structure
///
/// The PUBCOMP packet consists of:
/// - Fixed header (1 byte): packet type and flags
/// - Remaining length (1-4 bytes): length of variable header
/// - Variable header: packet identifier (2 bytes for standard u16)
/// - No payload
///
/// # Examples
///
/// Creating a basic PUBCOMP packet:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubcomp.packet_id(), 123u16);
/// assert_eq!(pubcomp.reason_code(), None);
/// ```
///
/// Parsing a PUBCOMP packet from bytes:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let data = [0x00, 0x7B]; // packet ID = 123
/// let (pubcomp, consumed) = mqtt::packet::v3_1_1::Pubcomp::parse(&data).unwrap();
/// assert_eq!(pubcomp.packet_id(), 123u16);
/// assert_eq!(consumed, 2);
/// ```
///
/// Converting to transmission buffers:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
///     .packet_id(456u16)
///     .build()
///     .unwrap();
///
/// let buffers = pubcomp.to_buffers();
/// let total_size = pubcomp.size();
/// assert!(total_size >= 4); // Fixed header + remaining length + packet ID
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
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
}

/// Type alias for PUBCOMP packet with standard u16 packet identifiers.
///
/// This is the standard PUBCOMP packet type that most applications should use,
/// conforming to the MQTT v3.1.1 specification's u16 packet identifier requirement.
/// The packet identifier must be in the range 1-65535 (non-zero).
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
/// ```

impl<PacketIdType> GenericPubcomp<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a PUBCOMP packet.
    ///
    /// The builder provides a fluent interface for creating PUBCOMP packets.
    /// At minimum, a packet identifier must be set before building.
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
    /// let builder = mqtt::packet::v3_1_1::Pubcomp::builder();
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
    /// This is a constant function that always returns `PacketType::Pubcomp`.
    /// It can be used to identify the packet type without creating an instance.
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
    /// let packet_type = mqtt::packet::v3_1_1::Pubcomp::packet_type();
    /// assert_eq!(packet_type, mqtt::packet::packet_type::PacketType::Pubcomp);
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Pubcomp
    }

    /// Returns the packet identifier of this PUBCOMP packet.
    ///
    /// The packet identifier must match the packet identifier of the original
    /// PUBLISH packet in the QoS 2 message flow. This ensures that the completion
    /// acknowledgment is correctly associated with the original message.
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
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
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
    /// Note: MQTT v3.1.1 specification does not define reason codes for PUBCOMP packets.
    /// This field is included for forward compatibility and will typically return `None`
    /// for standard v3.1.1 implementations. Some extended implementations may include
    /// reason codes as a non-standard extension.
    ///
    /// # Returns
    ///
    /// An `Option<PubcompReasonCode>` containing the reason code if present,
    /// or `None` for standard MQTT v3.1.1 packets.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// // Standard v3.1.1 PUBCOMP packets do not include reason codes
    /// assert_eq!(pubcomp.reason_code(), None);
    /// ```
    pub fn reason_code(&self) -> Option<PubcompReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| PubcompReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of the PUBCOMP packet in bytes.
    ///
    /// This includes the fixed header, remaining length field, and packet identifier.
    /// For standard MQTT v3.1.1 PUBCOMP packets, this is typically 4 bytes:
    /// - 1 byte fixed header
    /// - 1 byte remaining length
    /// - 2 bytes packet identifier
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
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = pubcomp.size();
    /// assert_eq!(size, 4); // Fixed header + remaining length + packet ID
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the PUBCOMP packet into a vector of I/O slices for efficient transmission.
    ///
    /// This method prepares the packet data for network transmission by organizing
    /// it into a series of byte slices that can be sent without additional copying.
    /// The slices are ordered as: fixed header, remaining length, packet identifier,
    /// and optional reason code (if present).
    ///
    /// # Returns
    ///
    /// A `Vec<IoSlice<'_>>` containing the packet data organized as I/O slices.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = pubcomp.to_buffers();
    /// assert_eq!(buffers.len(), 3); // Fixed header, remaining length, packet ID
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

    /// Converts the PUBCOMP packet into a continuous buffer for no-std environments.
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
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = pubcomp.to_continuous_buffer();
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

    /// Parses a PUBCOMP packet from raw bytes.
    ///
    /// This method deserializes a byte array into a PUBCOMP packet structure,
    /// validating the packet format and extracting the packet identifier and
    /// any optional reason code. The input data should not include the fixed
    /// header and remaining length, only the variable header content.
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
    /// - The packet identifier is zero (invalid per MQTT specification)
    /// - The data is too short to contain a valid packet identifier
    /// - The reason code is invalid (if present)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse a minimal PUBCOMP packet (packet ID only)
    /// let data = [0x00, 0x01]; // packet ID = 1
    /// let (pubcomp, consumed) = mqtt::packet::v3_1_1::Pubcomp::parse(&data).unwrap();
    /// assert_eq!(pubcomp.packet_id(), 1u16);
    /// assert_eq!(consumed, 2);
    /// assert_eq!(pubcomp.reason_code(), None);
    /// ```
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse PUBCOMP with packet ID 0x1234
    /// let data = [0x12, 0x34]; // packet ID = 0x1234 (4660)
    /// let (pubcomp, consumed) = mqtt::packet::v3_1_1::Pubcomp::parse(&data).unwrap();
    /// assert_eq!(pubcomp.packet_id(), 4660u16);
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
            let _ = PubcompReasonCode::try_from(rc).map_err(|_| MqttError::MalformedPacket)?;
            cursor += 1;
            Some([rc])
        } else {
            None
        };

        let remaining_size = buffer_size + reason_code_buf.as_ref().map_or(0, |_| 1);

        let pubcomp = GenericPubcomp {
            fixed_header: [FixedHeader::Pubcomp.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            packet_id_buf,
            reason_code_buf,
        };

        Ok((pubcomp, cursor))
    }
}

/// Builder implementation for constructing PUBCOMP packets.
///
/// The builder provides a fluent interface for creating PUBCOMP packets with
/// the required packet identifier and optional reason code. All PUBCOMP packets
/// require a non-zero packet identifier to be valid.
///
/// # Required Fields
///
/// - `packet_id`: Must be non-zero and match the original PUBLISH packet identifier
///
/// # Optional Fields
///
/// - `reason_code`: Not part of standard MQTT v3.1.1 but included for compatibility
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// // Build a basic PUBCOMP packet
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
///     .packet_id(42u16)
///     .build()
///     .unwrap();
/// ```
impl<PacketIdType> GenericPubcompBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the packet identifier for the PUBCOMP packet.
    ///
    /// The packet identifier must match the packet identifier of the original
    /// PUBLISH packet in the QoS 2 message flow. This ensures proper message
    /// flow correlation. The identifier must be non-zero as per MQTT specification.
    ///
    /// # Parameters
    ///
    /// * `id` - The packet identifier to set (must be non-zero)
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
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
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
    /// Note: MQTT v3.1.1 specification does not define reason codes for PUBCOMP packets.
    /// This method is provided for forward compatibility and non-standard extensions.
    /// Standard v3.1.1 implementations should not use reason codes.
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
    /// // Note: This is non-standard for MQTT v3.1.1
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
    ///     .packet_id(1u16)
    ///     .reason_code(mqtt::result_code::PubcompReasonCode::Success)
    ///     .build()
    ///     .unwrap();
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

        Ok(())
    }

    /// Builds the PUBCOMP packet from the configured builder.
    ///
    /// This method performs validation of the packet configuration and constructs
    /// the final PUBCOMP packet. The packet identifier must be set and non-zero
    /// before calling build.
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
    /// - The packet identifier is zero (invalid per MQTT specification)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Build a valid PUBCOMP packet
    /// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
    ///     .packet_id(42u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubcomp.packet_id(), 42u16);
    /// ```
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // This will fail because packet ID is required
    /// let result = mqtt::packet::v3_1_1::Pubcomp::builder()
    ///     .build();
    /// assert!(result.is_err());
    /// ```
    pub fn build(self) -> Result<GenericPubcomp<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let reason_code_buf = self.reason_code_buf.flatten();

        let mut remaining = mem::size_of::<PacketIdType>();
        // add reason code if present
        if reason_code_buf.is_some() {
            remaining += 1;
        }
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericPubcomp {
            fixed_header: [FixedHeader::Pubcomp.as_u8()],
            remaining_length,
            packet_id_buf,
            reason_code_buf,
        })
    }
}

/// Serialize implementation for PUBCOMP packets.
///
/// This implementation allows PUBCOMP packets to be serialized to JSON format
/// for debugging, logging, and testing purposes. The serialization includes
/// the packet type, packet identifier, and optional reason code.
///
/// # Serialized Format
///
/// The JSON format includes:
/// - `type`: Always "pubcomp"
/// - `packet_id`: The packet identifier
/// - `reason_code`: The reason code (if present, non-standard for v3.1.1)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use serde_json;
///
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
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

        let mut state = serializer.serialize_struct("pubcomp", field_count)?;
        state.serialize_field("type", PacketType::Pubcomp.as_str())?;
        state.serialize_field("packet_id", &self.packet_id())?;
        if self.reason_code_buf.is_some() {
            state.serialize_field("reason_code", &self.reason_code())?;
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
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
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
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
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
/// enabling generic packet handling and processing across different
/// packet implementations.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
///
/// // Use generic packet interface
/// let packet_size = pubcomp.size();
/// let buffers = pubcomp.to_buffers();
/// assert!(packet_size >= 4); // Minimum size for PUBCOMP
/// ```
impl<PacketIdType> GenericPacketTrait for GenericPubcomp<PacketIdType>
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

/// GenericPacketDisplay implementation for PUBCOMP packets.
///
/// This trait provides unified display formatting for all MQTT packet types,
/// enabling consistent output across different packet implementations.
/// Both debug and display formatting produce JSON output for consistency.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
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
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}
