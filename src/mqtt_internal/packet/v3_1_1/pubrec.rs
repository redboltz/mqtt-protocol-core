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
use crate::mqtt_internal::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt_internal::packet::GenericPacketDisplay;
use crate::mqtt_internal::packet::GenericPacketTrait;
use crate::mqtt_internal::packet::IsPacketId;
use crate::mqtt_internal::result_code::MqttError;
use crate::mqtt_internal::result_code::PubrecReasonCode;

/// A PUBREC packet for MQTT v3.1.1 protocol.
///
/// The PUBREC packet is the response to a PUBLISH packet with QoS level 2 and is the second
/// step in the four-part handshake for QoS 2 message delivery. The complete QoS 2 flow is:
/// 1. PUBLISH (sender -> receiver)
/// 2. **PUBREC** (receiver -> sender) - this packet
/// 3. PUBREL (sender -> receiver)
/// 4. PUBCOMP (receiver -> sender)
///
/// # MQTT v3.1.1 Specification
///
/// According to the MQTT v3.1.1 specification (<https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html>),
/// the PUBREC packet:
/// - Is sent by the receiver of a PUBLISH packet with QoS 2
/// - Contains the same Packet Identifier as the PUBLISH packet being acknowledged
/// - Indicates that the message has been received but not yet processed
/// - Has a fixed header with packet type 5 (0101) and remaining length
/// - Contains only a packet identifier in the variable header (no payload)
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
/// # MQTT v3.1.1 vs v5.0 Differences
///
/// Unlike MQTT v5.0, the v3.1.1 PUBREC packet:
/// - Does not support reason codes in the standard specification
/// - Does not support properties
/// - Has a simpler structure with only packet identifier
/// - This implementation optionally supports reason codes for extended compatibility
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
///
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubrec.packet_id(), 123u16);
/// assert_eq!(pubrec.reason_code(), None);
/// ```
///
/// Creating a PUBREC with optional reason code (extended feature):
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
///     .packet_id(456u16)
///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
///     .build()
///     .unwrap();
///
/// assert_eq!(pubrec.packet_id(), 456u16);
/// assert_eq!(pubrec.reason_code(), Some(mqtt::result_code::PubrecReasonCode::Success));
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
}

/// Type alias for PUBREC packet with standard u16 packet identifiers.
///
/// This is the standard PUBREC packet type that most applications should use,
/// conforming to the MQTT v3.1.1 specification's u16 packet identifier requirement.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
///     .packet_id(1u16)
///     .build()
///     .unwrap();
/// ```

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
    /// let builder = mqtt::packet::v3_1_1::Pubrec::builder();
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
    /// assert_eq!(mqtt::packet::v3_1_1::Pubrec::packet_type(), mqtt::packet::packet_type::PacketType::Pubrec);
    /// ```
    pub const fn packet_type() -> PacketType {
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
    /// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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
    /// **Note**: Reason codes are not part of the standard MQTT v3.1.1 specification
    /// but are supported as an optional extension for enhanced compatibility and debugging.
    ///
    /// In standard MQTT v3.1.1, the absence of a reason code implies successful receipt.
    ///
    /// # Returns
    ///
    /// - `Some(PubrecReasonCode)` if a reason code was included in the packet (non-standard)
    /// - `None` if no reason code was present (standard v3.1.1 behavior)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// // Standard PUBREC without reason code
    /// let pubrec1 = mqtt::packet::v3_1_1::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(pubrec1.reason_code(), None);
    ///
    /// // Extended PUBREC with reason code
    /// let pubrec2 = mqtt::packet::v3_1_1::Pubrec::builder()
    ///     .packet_id(456u16)
    ///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(pubrec2.reason_code(), Some(mqtt::result_code::PubrecReasonCode::Success));
    /// ```
    pub fn reason_code(&self) -> Option<PubrecReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| PubrecReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of this PUBREC packet in bytes.
    ///
    /// This includes the fixed header, remaining length field, and all variable
    /// header components. For MQTT v3.1.1, this typically includes:
    /// - Fixed header (1 byte)
    /// - Remaining length field (1-4 bytes)
    /// - Packet identifier (2 bytes for u16)
    /// - Optional reason code (1 byte, if present)
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
    /// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = pubrec.size();
    /// // Standard v3.1.1 size: fixed header (1) + remaining length (1) + packet_id (2) = 4 bytes
    /// assert_eq!(size, 4);
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
    /// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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

        bufs
    }

    /// Converts this PUBREC packet into a continuous buffer for no-std environments.
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
    /// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = pubrec.to_continuous_buffer();
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

    /// Parses a PUBREC packet from raw byte data.
    ///
    /// This method parses the variable header of a PUBREC packet from the provided
    /// byte slice. The fixed header should have already been parsed and removed from the data.
    ///
    /// For MQTT v3.1.1, the standard format includes only a packet identifier.
    /// This implementation also supports parsing an optional reason code for extended compatibility.
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
    /// - The packet data is too short to contain a packet identifier
    /// - The reason code (if present) is invalid
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Standard v3.1.1 PUBREC with packet_id = 1
    /// let packet_data = &[0x00, 0x01];
    /// let (pubrec, consumed) = mqtt::packet::v3_1_1::Pubrec::parse(packet_data).unwrap();
    /// assert_eq!(pubrec.packet_id(), 1u16);
    /// assert_eq!(consumed, 2);
    /// assert_eq!(pubrec.reason_code(), None);
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

        let remaining_size = buffer_size + reason_code_buf.as_ref().map_or(0, |_| 1);

        let pubrec = GenericPubrec {
            fixed_header: [FixedHeader::Pubrec.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            packet_id_buf,
            reason_code_buf,
        };

        Ok((pubrec, cursor))
    }
}

/// Builder for constructing PUBREC packets.
///
/// The `GenericPubrecBuilder` provides a fluent interface for building PUBREC packets
/// with various optional components. All PUBREC packets require a packet identifier,
/// while reason codes are optional and represent an extension to the standard v3.1.1 protocol.
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
/// // Standard MQTT v3.1.1 PUBREC packet
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
///     .packet_id(123u16)
///     .build()
///     .unwrap();
///
/// // Extended PUBREC with reason code
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
///     .packet_id(456u16)
///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
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
    /// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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
    /// **Note**: This is an extension to the standard MQTT v3.1.1 specification.
    /// Reason codes are not defined in the original v3.1.1 protocol but are
    /// supported for enhanced compatibility and debugging capabilities.
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
    /// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
    ///     .packet_id(123u16)
    ///     .reason_code(mqtt::result_code::PubrecReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(pubrec.reason_code(), Some(mqtt::result_code::PubrecReasonCode::Success));
    /// ```
    pub fn reason_code(mut self, rc: PubrecReasonCode) -> Self {
        self.reason_code_buf = Some(Some([rc as u8]));
        self
    }

    /// Validates the builder configuration.
    ///
    /// This internal method ensures that:
    /// - A packet identifier has been set and is non-zero
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

    /// Builds the PUBREC packet from the configured parameters.
    ///
    /// This method validates the builder configuration and constructs a complete
    /// PUBREC packet. It calculates the remaining length and arranges all components
    /// according to the MQTT v3.1.1 specification (with optional extensions).
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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::prelude::*;
    ///
    /// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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

        let mut remaining = mem::size_of::<PacketIdType>();
        // add reason code if present
        if reason_code_buf.is_some() {
            remaining += 1;
        }
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericPubrec {
            fixed_header: [FixedHeader::Pubrec.as_u8()],
            remaining_length,
            packet_id_buf,
            reason_code_buf,
        })
    }
}

/// Implementation of `Serialize` trait for JSON serialization of PUBREC packets.
///
/// This implementation serializes a PUBREC packet into a JSON object containing
/// the packet type, packet identifier, and optional reason code. The serialization
/// format is suitable for debugging, logging, and API responses.
///
/// # Serialized Fields
///
/// - `type`: Always "pubrec" (string)
/// - `packet_id`: The packet identifier (number)
/// - `reason_code`: The reason code (optional, only if present)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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

        let mut state = serializer.serialize_struct("pubrec", field_count)?;
        state.serialize_field("type", PacketType::Pubrec.as_str())?;
        state.serialize_field("packet_id", &self.packet_id())?;
        if self.reason_code_buf.is_some() {
            state.serialize_field("reason_code", &self.reason_code())?;
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
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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
/// let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
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
