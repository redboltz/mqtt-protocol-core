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

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::result_code::MqttError;
use crate::mqtt::result_code::SubackReturnCode;

/// MQTT v3.1.1 SUBACK packet representation with generic packet ID support
///
/// The SUBACK packet is sent by the MQTT broker in response to a SUBSCRIBE packet
/// from a client. It indicates the maximum QoS level that was granted for each subscription
/// or provides a failure code indicating why the subscription failed.
///
/// According to MQTT v3.1.1 specification, the SUBACK packet contains:
/// - Fixed header with packet type and remaining length
/// - Variable header with packet identifier
/// - Payload containing return codes for each topic filter subscription
///
/// # Packet Structure
///
/// ```text
/// SUBACK Packet Structure (MQTT v3.1.1):
/// +----------------+
/// | Fixed Header   |  - Packet type (0x90) and remaining length
/// +----------------+
/// | Packet ID      |  - 2 bytes (or PacketIdType::Buffer size)
/// +----------------+
/// | Return Codes   |  - One or more return codes (1 byte each)
/// +----------------+
/// ```
///
/// # Return Codes
///
/// Each return code in the SUBACK packet corresponds to a topic filter in the original
/// SUBSCRIBE packet and indicates the maximum QoS level granted or an error:
///
/// **Success codes (QoS levels):**
/// - `0x00` Maximum QoS 0 - Subscription granted with maximum QoS 0
/// - `0x01` Maximum QoS 1 - Subscription granted with maximum QoS 1  
/// - `0x02` Maximum QoS 2 - Subscription granted with maximum QoS 2
///
/// **Error codes:**
/// - `0x80` Failure - Subscription failed
///
/// Unlike MQTT v5.0, MQTT v3.1.1 has a simple return code system with only four possible values.
/// The server may grant a lower QoS level than requested by the client.
///
/// # Generic Packet ID Support
///
/// This implementation supports generic packet ID types through the `PacketIdType` parameter.
/// While MQTT specification uses 16-bit packet IDs, this allows for extended packet IDs
/// (e.g., 32-bit) for broker cluster implementations to prevent packet ID exhaustion.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
///
/// // Create a SUBACK with successful subscriptions
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(42u16)
///     .return_codes(vec![
///         SubackReturnCode::SuccessMaximumQos1,
///         SubackReturnCode::SuccessMaximumQos2,
///     ])
///     .build()
///     .unwrap();
///
/// assert_eq!(suback.packet_id(), 42);
/// assert_eq!(suback.return_codes().len(), 2);
///
/// // Create SUBACK with mixed success and error codes
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(100u16)
///     .return_codes(vec![
///         SubackReturnCode::SuccessMaximumQos1,
///         SubackReturnCode::Failure,
///         SubackReturnCode::SuccessMaximumQos0,
///     ])
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = suback.to_buffers();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericSuback<PacketIdType>
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
    return_codes_buf: Vec<u8>,
}

/// Standard MQTT v3.1.1 SUBACK packet with 16-bit packet IDs
///
/// This is a type alias for `GenericSuback<u16>` that provides the standard MQTT v3.1.1
/// SUBACK packet implementation using 16-bit packet identifiers as defined in the
/// MQTT specification.
///
/// Most applications should use this type unless they specifically need extended
/// packet ID support for broker cluster implementations.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
///
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(42u16)
///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
///     .build()
///     .unwrap();
/// ```
pub type Suback = GenericSuback<u16>;

impl<PacketIdType> GenericSuback<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Create a new GenericSubackBuilder for constructing SUBACK packets
    ///
    /// Returns a builder instance that allows setting the various fields of a SUBACK packet
    /// in a fluent interface style. The builder ensures all required fields are set before
    /// creating the final packet.
    ///
    /// # Returns
    ///
    /// A new `GenericSubackBuilder<PacketIdType>` instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::GenericSuback::<u16>::builder()
    ///     .packet_id(42u16)
    ///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericSubackBuilder<PacketIdType> {
        GenericSubackBuilder::<PacketIdType>::default()
    }

    /// Get the packet type for SUBACK packets
    ///
    /// Returns the constant packet type identifier for SUBACK packets.
    /// This is always `PacketType::Suback` for SUBACK packets.
    ///
    /// # Returns
    ///
    /// The packet type `PacketType::Suback`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::packet_type::PacketType;
    ///
    /// assert_eq!(mqtt::packet::v3_1_1::Suback::packet_type(), PacketType::Suback);
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Suback
    }

    /// Get the packet identifier from the SUBACK packet
    ///
    /// Returns the packet identifier that matches the SUBSCRIBE packet this SUBACK
    /// is responding to. The packet identifier is used to correlate the SUBACK
    /// with the original SUBSCRIBE request.
    ///
    /// # Returns
    ///
    /// The packet identifier as `PacketIdType`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::Suback::builder()
    ///     .packet_id(1234u16)
    ///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(suback.packet_id(), 1234);
    /// ```
    pub fn packet_id(&self) -> PacketIdType {
        PacketIdType::from_buffer(self.packet_id_buf.as_ref())
    }

    /// Get the return codes from the SUBACK packet
    ///
    /// Returns a vector of return codes indicating the result of each subscription
    /// request in the original SUBSCRIBE packet. Each return code corresponds to
    /// a topic filter in the SUBSCRIBE packet, in the same order.
    ///
    /// Invalid return code bytes are converted to `SubackReturnCode::Failure`
    /// to maintain packet integrity.
    ///
    /// # Returns
    ///
    /// A `Vec<SubackReturnCode>` containing the subscription results
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::Suback::builder()
    ///     .packet_id(42u16)
    ///     .return_codes(vec![
    ///         SubackReturnCode::SuccessMaximumQos1,
    ///         SubackReturnCode::Failure,
    ///         SubackReturnCode::SuccessMaximumQos0,
    ///     ])
    ///     .build()
    ///     .unwrap();
    ///
    /// let codes = suback.return_codes();
    /// assert_eq!(codes.len(), 3);
    /// assert_eq!(codes[0], SubackReturnCode::SuccessMaximumQos1);
    /// assert_eq!(codes[1], SubackReturnCode::Failure);
    /// assert_eq!(codes[2], SubackReturnCode::SuccessMaximumQos0);
    /// ```
    pub fn return_codes(&self) -> Vec<SubackReturnCode> {
        self.return_codes_buf
            .iter()
            .map(|&byte| SubackReturnCode::try_from(byte).unwrap_or(SubackReturnCode::Failure))
            .collect()
    }

    /// Parse a SUBACK packet from raw bytes
    ///
    /// Parses the variable header and payload of a SUBACK packet from the provided
    /// byte buffer. The fixed header should already be parsed before calling this method.
    ///
    /// # Arguments
    ///
    /// * `data` - The raw bytes containing the SUBACK packet variable header and payload
    ///
    /// # Returns
    ///
    /// Returns a tuple containing:
    /// - The parsed `GenericSuback` instance
    /// - The number of bytes consumed during parsing
    ///
    /// # Errors
    ///
    /// Returns `MqttError` if:
    /// - The packet is malformed (insufficient bytes, invalid packet ID, invalid return codes)
    /// - The packet violates protocol rules (no return codes provided)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse SUBACK packet from network data
    /// let data = &[0x00, 0x10, 0x00]; // packet_id=16, granted QoS 0
    /// let (suback, consumed) = mqtt::packet::v3_1_1::Suback::parse(data).unwrap();
    ///
    /// assert_eq!(suback.packet_id(), 16);
    /// assert_eq!(consumed, 3);
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

        let mut return_codes_buf = Vec::new();
        while cursor < data.len() {
            let _return_code =
                SubackReturnCode::try_from(data[cursor]).map_err(|_| MqttError::MalformedPacket)?;
            return_codes_buf.push(data[cursor]);
            cursor += 1;
        }

        if return_codes_buf.is_empty() {
            return Err(MqttError::ProtocolError);
        }

        let remaining_size = buffer_size + return_codes_buf.len();
        let remaining_length = VariableByteInteger::from_u32(remaining_size as u32).unwrap();

        let suback = GenericSuback {
            fixed_header: [FixedHeader::Suback as u8],
            remaining_length,
            packet_id_buf,
            return_codes_buf,
        };

        Ok((suback, cursor))
    }

    /// Calculate the total size of the SUBACK packet in bytes
    ///
    /// Returns the total number of bytes required to represent this SUBACK packet
    /// when serialized for network transmission. This includes the fixed header,
    /// variable header, and all return codes.
    ///
    /// # Returns
    ///
    /// The total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::Suback::builder()
    ///     .packet_id(42u16)
    ///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = suback.size();
    /// assert!(size > 0);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Convert the SUBACK packet to I/O buffers for efficient network transmission
    ///
    /// Returns a vector of `IoSlice` references that can be used with vectored I/O
    /// operations for efficient network transmission without copying data. The buffers
    /// represent the complete SUBACK packet in wire format.
    ///
    /// The returned buffers contain the packet in the following order:
    /// 1. Fixed header (packet type and remaining length)
    /// 2. Packet identifier
    /// 3. Return codes
    ///
    /// # Returns
    ///
    /// A `Vec<IoSlice<'_>>` containing references to the packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::Suback::builder()
    ///     .packet_id(42u16)
    ///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = suback.to_buffers();
    /// // Use buffers for vectored I/O operations
    /// // let bytes_written = socket.write_vectored(&buffers)?;
    /// ```
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(self.packet_id_buf.as_ref()));

        if !self.return_codes_buf.is_empty() {
            bufs.push(IoSlice::new(&self.return_codes_buf));
        }

        bufs
    }
}

/// Builder implementation for `GenericSuback`
///
/// Provides a fluent interface for constructing SUBACK packets with proper validation.
/// The builder ensures all required fields are set and validates the packet structure
/// before creating the final packet instance.
impl<PacketIdType> GenericSubackBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Set the packet identifier for the SUBACK packet
    ///
    /// The packet identifier must match the packet identifier from the original
    /// SUBSCRIBE packet that this SUBACK is responding to. The packet identifier
    /// cannot be zero.
    ///
    /// # Arguments
    ///
    /// * `id` - The packet identifier of type `PacketIdType`
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::Suback::builder()
    ///     .packet_id(42u16)
    ///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(id.to_buffer());
        self
    }

    /// Set the return codes for the SUBACK packet
    ///
    /// The return codes indicate the result of each subscription request in the original
    /// SUBSCRIBE packet. Each return code corresponds to a topic filter in the SUBSCRIBE
    /// packet, in the same order. At least one return code must be provided.
    ///
    /// # Arguments
    ///
    /// * `codes` - A vector of `SubackReturnCode` values indicating subscription results
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::Suback::builder()
    ///     .packet_id(42u16)
    ///     .return_codes(vec![
    ///         SubackReturnCode::SuccessMaximumQos1,
    ///         SubackReturnCode::Failure,
    ///         SubackReturnCode::SuccessMaximumQos0,
    ///     ])
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn return_codes(mut self, codes: Vec<SubackReturnCode>) -> Self {
        let return_codes_buf: Vec<u8> = codes.iter().map(|&rc| rc as u8).collect();
        self.return_codes_buf = Some(return_codes_buf);
        self
    }

    /// Validate the builder state before constructing the packet
    ///
    /// Performs comprehensive validation of all builder fields to ensure the
    /// resulting SUBACK packet will be valid according to MQTT v3.1.1 specification.
    ///
    /// # Validation Rules
    ///
    /// - Packet identifier must be set and non-zero
    /// - At least one return code must be provided
    ///
    /// # Returns
    ///
    /// `Ok(())` if validation passes, `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - Missing or invalid packet identifier
    /// - `MqttError::ProtocolError` - Missing return codes
    fn validate(&self) -> Result<(), MqttError> {
        if self.packet_id_buf.is_none() {
            return Err(MqttError::MalformedPacket);
        }

        let packet_id_bytes = self.packet_id_buf.as_ref().unwrap().as_ref();
        let all_zeros = packet_id_bytes.iter().all(|&b| b == 0);
        if all_zeros {
            return Err(MqttError::MalformedPacket);
        }

        if self
            .return_codes_buf
            .as_ref()
            .map_or(true, |r| r.is_empty())
        {
            return Err(MqttError::ProtocolError);
        }

        Ok(())
    }

    /// Build the final SUBACK packet
    ///
    /// Validates all builder fields and constructs the final `GenericSuback` instance.
    /// This method consumes the builder and returns either a valid SUBACK packet
    /// or an error if validation fails.
    ///
    /// The method automatically calculates the remaining length field based on the
    /// provided data.
    ///
    /// # Returns
    ///
    /// `Ok(GenericSuback<PacketIdType>)` containing the constructed packet,
    /// or `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - Missing or invalid packet identifier
    /// - `MqttError::ProtocolError` - Missing return codes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
    ///
    /// let suback = mqtt::packet::v3_1_1::Suback::builder()
    ///     .packet_id(42u16)
    ///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(suback.packet_id(), 42);
    /// ```
    pub fn build(self) -> Result<GenericSuback<PacketIdType>, MqttError> {
        self.validate()?;

        let packet_id_buf = self.packet_id_buf.unwrap();
        let return_codes_buf = self.return_codes_buf.unwrap_or_default();

        let packet_id_size = mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
        let return_codes_size = return_codes_buf.len();

        let remaining = packet_id_size + return_codes_size;
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericSuback {
            fixed_header: [FixedHeader::Suback as u8],
            remaining_length,
            packet_id_buf,
            return_codes_buf,
        })
    }
}

/// Display trait implementation for GenericSuback
///
/// Provides a human-readable JSON representation of the SUBACK packet.
/// The display format includes the packet type, packet ID, and return codes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
///
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(42u16)
///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
///     .build()
///     .unwrap();
///
/// println!("{}", suback); // Prints JSON representation
/// ```
impl<PacketIdType> fmt::Display for GenericSuback<PacketIdType>
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

/// Debug trait implementation for GenericSuback
///
/// Provides a debug representation of the SUBACK packet using the same JSON format
/// as the Display trait. This ensures consistent output for logging and debugging.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
///
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(42u16)
///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
///     .build()
///     .unwrap();
///
/// println!("{:?}", suback); // Prints JSON representation
/// ```
impl<PacketIdType> fmt::Debug for GenericSuback<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Serialize trait implementation for GenericSuback
///
/// Provides JSON serialization support for SUBACK packets. The serialized format
/// includes the packet type, packet ID, and return codes.
///
/// The serialized structure contains:
/// - `type`: Always "suback"
/// - `packet_id`: The packet identifier
/// - `return_codes`: Array of return codes (only if non-empty)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
///
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(42u16)
///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&suback).unwrap();
/// // json contains: {"type":"suback","packet_id":42,"return_codes":["SuccessMaximumQos1"]}
/// ```
impl<PacketIdType> Serialize for GenericSuback<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 2; // type and packet_id are always present

        if !self.return_codes_buf.is_empty() {
            field_count += 1;
        }

        let mut state = serializer.serialize_struct("Suback", field_count)?;

        state.serialize_field("type", "suback")?;
        state.serialize_field("packet_id", &self.packet_id())?;

        if !self.return_codes_buf.is_empty() {
            state.serialize_field("return_codes", &self.return_codes())?;
        }

        state.end()
    }
}

/// GenericPacketTrait implementation for GenericSuback
///
/// Provides the standard packet interface methods for SUBACK packets.
/// This trait allows SUBACK packets to be used polymorphically with other
/// MQTT packet types.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;
/// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
///
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(42u16)
///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
///     .build()
///     .unwrap();
///
/// // Use trait methods
/// let size = suback.size();
/// let buffers = suback.to_buffers();
/// ```
impl<PacketIdType> GenericPacketTrait for GenericSuback<PacketIdType>
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

/// GenericPacketDisplay implementation for GenericSuback
///
/// Provides standardized display formatting for SUBACK packets through the
/// GenericPacketDisplay trait. This allows consistent formatting across
/// different packet types in the library.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketDisplay;
/// use mqtt_protocol_core::mqtt::result_code::SubackReturnCode;
///
/// let suback = mqtt::packet::v3_1_1::Suback::builder()
///     .packet_id(42u16)
///     .return_codes(vec![SubackReturnCode::SuccessMaximumQos1])
///     .build()
///     .unwrap();
///
/// // Use trait methods for consistent formatting
/// println!("{}", format_args!("{}", suback));
/// ```
impl<PacketIdType> GenericPacketDisplay for GenericSuback<PacketIdType>
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
