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

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use derive_builder::Builder;

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::result_code::MqttError;

/// MQTT v3.1.1 PINGREQ packet representation
///
/// The PINGREQ packet is a heartbeat packet sent by an MQTT client to the server
/// to keep the connection alive and ensure that the connection is still active.
/// This packet is part of the keep-alive mechanism in MQTT protocol.
///
/// According to MQTT v3.1.1 specification, the PINGREQ packet:
/// - Has no variable header
/// - Has no payload
/// - Has a remaining length of 0
/// - Is typically sent by the client at intervals specified by the Keep Alive value
///
/// # Protocol Information
///
/// - **Packet Type**: 12 (0xC0)
/// - **Remaining Length**: 0 (no variable header or payload)
/// - **Direction**: Client to Server
/// - **Specification**: [MQTT v3.1.1 Section 3.12](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718081)
///
/// # Keep Alive Mechanism
///
/// The PINGREQ packet is used to:
/// - Indicate to the server that the client is alive
/// - Exercise the network to confirm connectivity
/// - Ensure that the network connection is active
///
/// The server must respond with a PINGRESP packet when it receives a PINGREQ.
/// If the client does not receive a PINGRESP within a reasonable time,
/// it should close the network connection.
///
/// # Timing
///
/// The client should send a PINGREQ packet when:
/// - The Keep Alive time period has elapsed since the last packet was sent
/// - No other packets need to be sent during the Keep Alive period
///
/// # Wire Format
///
/// The PINGREQ packet consists of only a fixed header:
/// ```text
/// +----------+
/// |  Fixed   |
/// | Header   |
/// +----------+
/// ```
///
/// Where the fixed header is:
/// - Byte 1: 0xC0 (Packet Type = 12, Flags = 0)
/// - Byte 2: 0x00 (Remaining Length = 0)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Create a PINGREQ packet
/// let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
///
/// assert_eq!(pingreq.packet_type(), mqtt::packet::packet_type::PacketType::Pingreq);
/// assert_eq!(pingreq.size(), 2); // Fixed header (1 byte) + remaining length (1 byte)
///
/// // Build using builder pattern
/// let pingreq = mqtt::packet::v3_1_1::Pingreq::builder()
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = pingreq.to_buffers();
/// assert_eq!(buffers.len(), 2); // Fixed header + remaining length
/// ```
#[derive(PartialEq, Eq, Builder, Clone)]
#[builder(derive(Debug), pattern = "owned", build_fn(skip))]
pub struct Pingreq {
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
}

impl Pingreq {
    /// Creates a new PINGREQ packet
    ///
    /// This method creates a PINGREQ packet with the standard fixed header
    /// and zero remaining length, as specified by the MQTT v3.1.1 protocol.
    ///
    /// # Returns
    ///
    /// A new `Pingreq` instance ready for transmission
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
    /// assert_eq!(pingreq.size(), 2);
    /// ```
    pub fn new() -> Self {
        Self {
            fixed_header: [FixedHeader::Pingreq.as_u8()],
            remaining_length: VariableByteInteger::from_u32(0).unwrap(),
        }
    }

    /// Creates a new builder for constructing a PINGREQ packet
    ///
    /// The builder pattern provides a consistent interface for packet creation,
    /// even though PINGREQ packets have no configurable parameters.
    ///
    /// # Returns
    ///
    /// A `PingreqBuilder` instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingreq = mqtt::packet::v3_1_1::Pingreq::builder()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> PingreqBuilder {
        PingreqBuilder::default()
    }

    /// Returns the packet type for PINGREQ packets
    ///
    /// # Returns
    ///
    /// `PacketType::Pingreq` (value 12)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// assert_eq!(
    ///     mqtt::packet::v3_1_1::Pingreq::packet_type(),
    ///     mqtt::packet::packet_type::PacketType::Pingreq
    /// );
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Pingreq
    }

    /// Returns the total size of the PINGREQ packet in bytes
    ///
    /// The size includes the fixed header (1 byte) and the remaining length field (1 byte).
    /// Since PINGREQ has no variable header or payload, the total size is always 2 bytes.
    ///
    /// # Returns
    ///
    /// The packet size in bytes (always 2 for PINGREQ)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
    /// assert_eq!(pingreq.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the PINGREQ packet to a vector of I/O slices for efficient network transmission
    ///
    /// This method provides zero-copy serialization by returning references to the
    /// internal packet data as I/O slices, which can be used directly with vectored I/O operations.
    ///
    /// # Returns
    ///
    /// A vector containing:
    /// - Fixed header slice (1 byte)
    /// - Remaining length slice (1 byte)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
    /// let buffers = pingreq.to_buffers();
    /// assert_eq!(buffers.len(), 2);
    ///
    /// // Can be used with vectored write operations
    /// // stream.write_vectored(&buffers).await?;
    /// ```
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        vec![
            IoSlice::new(&self.fixed_header),
            IoSlice::new(self.remaining_length.as_bytes()),
        ]
    }

    /// Parses a PINGREQ packet from raw bytes
    ///
    /// Since PINGREQ packets have no variable header or payload, this method
    /// simply creates a new PINGREQ packet instance. The data parameter is
    /// not used but is kept for consistency with other packet types.
    ///
    /// # Parameters
    ///
    /// * `_data` - Raw byte data (unused for PINGREQ)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The created `Pingreq` instance
    /// - The number of bytes consumed (always 0 for PINGREQ)
    ///
    /// # Errors
    ///
    /// This method always succeeds for PINGREQ packets.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let data = &[];
    /// let (pingreq, consumed) = mqtt::packet::v3_1_1::Pingreq::parse(data).unwrap();
    /// assert_eq!(consumed, 0);
    /// assert_eq!(pingreq.size(), 2);
    /// ```
    pub fn parse(_data: &[u8]) -> Result<(Self, usize), MqttError> {
        // PINGREQ packet has no variable header or payload
        let remaining_length = VariableByteInteger::from_u32(0).unwrap();

        let pingreq = Pingreq {
            fixed_header: [FixedHeader::Pingreq.as_u8()],
            remaining_length,
        };

        Ok((pingreq, 0))
    }
}

impl PingreqBuilder {
    /// Builds a PINGREQ packet from the builder
    ///
    /// Since PINGREQ packets have no configurable parameters, this method
    /// simply creates a standard PINGREQ packet.
    ///
    /// # Returns
    ///
    /// A `Result` containing the built `Pingreq` packet on success
    ///
    /// # Errors
    ///
    /// This method always succeeds for PINGREQ packets.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingreq = mqtt::packet::v3_1_1::Pingreq::builder()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<Pingreq, MqttError> {
        let remaining_length = VariableByteInteger::from_u32(0).unwrap();

        Ok(Pingreq {
            fixed_header: [FixedHeader::Pingreq.as_u8()],
            remaining_length,
        })
    }
}

/// Implements JSON serialization for PINGREQ packets
///
/// This implementation allows PINGREQ packets to be serialized to JSON format,
/// which is useful for debugging, logging, and protocol analysis.
///
/// The serialized format includes:
/// - `type`: The packet type as a string ("pingreq")
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use serde_json;
///
/// let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
/// let json = serde_json::to_string(&pingreq).unwrap();
/// assert!(json.contains("pingreq"));
/// ```
impl Serialize for Pingreq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("pingreq", 1)?;
        state.serialize_field("type", PacketType::Pingreq.as_str())?;
        state.end()
    }
}

/// Implements display formatting for PINGREQ packets
///
/// This implementation provides a JSON representation of the PINGREQ packet
/// for human-readable output, debugging, and logging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
/// println!("PINGREQ: {}", pingreq);
/// // Output: {"type":"pingreq"}
/// ```
impl fmt::Display for Pingreq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Implements debug formatting for PINGREQ packets
///
/// This implementation uses the same format as `Display` to provide
/// consistent output for debugging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
/// println!("Debug: {:?}", pingreq);
/// // Output: {"type":"pingreq"}
/// ```
impl fmt::Debug for Pingreq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Implements the generic packet trait for PINGREQ packets
///
/// This trait provides a common interface for all MQTT packet types,
/// allowing them to be used polymorphically in generic contexts.
///
/// # Methods
///
/// - `size()`: Returns the packet size in bytes
/// - `to_buffers()`: Returns I/O slices for network transmission
impl GenericPacketTrait for Pingreq {
    fn size(&self) -> usize {
        self.size()
    }

    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }
}

/// Implements the generic packet display trait for PINGREQ packets
///
/// This trait provides a common interface for formatting MQTT packets,
/// allowing them to be displayed consistently across different packet types.
///
/// # Methods
///
/// - `fmt_debug()`: Debug formatting
/// - `fmt_display()`: Display formatting
impl GenericPacketDisplay for Pingreq {
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
