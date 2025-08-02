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

/// MQTT v3.1.1 PINGRESP packet representation
///
/// The PINGRESP packet is a heartbeat response packet sent by an MQTT server to a client
/// in response to a PINGREQ packet. This packet acknowledges that the server has received
/// the client's keep-alive request and confirms that the connection is still active.
///
/// According to MQTT v3.1.1 specification (<https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html>),
/// the PINGRESP packet:
/// - Has no variable header
/// - Has no payload
/// - Has a remaining length of 0
/// - Must be sent by the server in response to a PINGREQ packet
///
/// # Protocol Information
///
/// - **Packet Type**: 13 (0xD0)
/// - **Remaining Length**: 0 (no variable header or payload)
/// - **Direction**: Server to Client
/// - **QoS Level**: Not applicable (control packet)
///
/// # Keep Alive Mechanism
///
/// The PINGRESP packet is used to:
/// - Acknowledge the client's PINGREQ packet
/// - Confirm to the client that the server is alive and responsive
/// - Maintain the network connection's keep-alive state
/// - Prevent connection timeouts due to network inactivity
///
/// In MQTT v3.1.1, the keep-alive mechanism works as follows:
/// 1. Client sends PINGREQ when no other packets have been sent within the keep-alive interval
/// 2. Server must respond with PINGRESP to acknowledge the keep-alive
/// 3. If client doesn't receive PINGRESP within a reasonable time, connection is considered broken
///
/// # Timing Requirements
///
/// According to the MQTT v3.1.1 specification:
/// - The server should send a PINGRESP packet immediately upon receiving a PINGREQ
/// - The response should be sent with minimal delay to ensure timely keep-alive acknowledgment
/// - Failure to respond may result in the client closing the connection
///
/// # Wire Format
///
/// The PINGRESP packet consists of only the fixed header:
/// ```text
/// Bit     7 6 5 4   3 2 1 0
/// Byte 1  1 1 0 1   0 0 0 0  (0xD0 - Packet Type and Flags)
/// Byte 2  0 0 0 0   0 0 0 0  (0x00 - Remaining Length)
/// ```
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Create a PINGRESP packet
/// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
///
/// assert_eq!(pingresp.packet_type(), mqtt::packet::packet_type::PacketType::Pingresp);
/// assert_eq!(pingresp.size(), 2); // Fixed header (1 byte) + remaining length (1 byte)
///
/// // Build using builder pattern
/// let pingresp = mqtt::packet::v3_1_1::Pingresp::builder()
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = pingresp.to_buffers();
/// assert_eq!(buffers.len(), 2); // Fixed header + remaining length
/// ```
#[derive(PartialEq, Eq, Builder, Clone)]
#[builder(derive(Debug), pattern = "owned", build_fn(skip))]
pub struct Pingresp {
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
}

impl Pingresp {
    /// Creates a new PINGRESP packet
    ///
    /// This method creates a PINGRESP packet with the standard fixed header
    /// and zero remaining length, as specified by the MQTT v3.1.1 protocol.
    /// The PINGRESP packet is typically created by the server in response
    /// to a PINGREQ packet from the client to acknowledge the keep-alive mechanism.
    ///
    /// # Returns
    ///
    /// A new `Pingresp` instance ready for transmission
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
    /// assert_eq!(pingresp.size(), 2);
    /// assert_eq!(pingresp.packet_type(), mqtt::packet::packet_type::PacketType::Pingresp);
    /// ```
    pub fn new() -> Self {
        Self {
            fixed_header: [FixedHeader::Pingresp.as_u8()],
            remaining_length: VariableByteInteger::from_u32(0).unwrap(),
        }
    }

    /// Creates a new builder for constructing a PINGRESP packet
    ///
    /// The builder pattern provides a consistent interface for packet creation,
    /// even though PINGRESP packets have no configurable parameters in MQTT v3.1.1.
    /// This maintains consistency with other packet types in the library.
    ///
    /// # Returns
    ///
    /// A `PingrespBuilder` instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingresp = mqtt::packet::v3_1_1::Pingresp::builder()
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(pingresp.size(), 2);
    /// ```
    pub fn builder() -> PingrespBuilder {
        PingrespBuilder::default()
    }

    /// Returns the packet type for PINGRESP packets
    ///
    /// This is a constant function that returns the MQTT packet type identifier
    /// for PINGRESP packets as defined in the MQTT v3.1.1 specification.
    ///
    /// # Returns
    ///
    /// `PacketType::Pingresp` (value 13, 0xD0 in the fixed header)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// assert_eq!(
    ///     mqtt::packet::v3_1_1::Pingresp::packet_type(),
    ///     mqtt::packet::packet_type::PacketType::Pingresp
    /// );
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Pingresp
    }

    /// Returns the total size of the PINGRESP packet in bytes
    ///
    /// The size includes the fixed header (1 byte) and the remaining length field (1 byte).
    /// Since PINGRESP has no variable header or payload in MQTT v3.1.1, the total size
    /// is always 2 bytes.
    ///
    /// # Returns
    ///
    /// The packet size in bytes (always 2 for PINGRESP in MQTT v3.1.1)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
    /// assert_eq!(pingresp.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the PINGRESP packet to a vector of I/O slices for efficient network transmission
    ///
    /// This method provides zero-copy serialization by returning references to the
    /// internal packet data as I/O slices, which can be used directly with vectored I/O operations.
    /// This is particularly useful for high-performance network implementations.
    ///
    /// # Returns
    ///
    /// A vector containing:
    /// - Fixed header slice (1 byte containing 0xD0)
    /// - Remaining length slice (1 byte containing 0x00)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
    /// let buffers = pingresp.to_buffers();
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

    /// Parses a PINGRESP packet from raw bytes
    ///
    /// Since PINGRESP packets have no variable header or payload in MQTT v3.1.1,
    /// this method simply creates a new PINGRESP packet instance. The data parameter
    /// is not used but is kept for consistency with other packet parsing methods.
    ///
    /// This method is typically called by the packet parser after determining
    /// that the packet type is PINGRESP from the fixed header.
    ///
    /// # Parameters
    ///
    /// * `_data` - Raw byte data (unused for PINGRESP as it has no payload)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - The created `Pingresp` instance
    /// - The number of bytes consumed from the data (always 0 for PINGRESP)
    ///
    /// # Errors
    ///
    /// This method always succeeds for PINGRESP packets in MQTT v3.1.1.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let data = &[];
    /// let (pingresp, consumed) = mqtt::packet::v3_1_1::Pingresp::parse(data).unwrap();
    /// assert_eq!(consumed, 0);
    /// assert_eq!(pingresp.size(), 2);
    /// ```
    pub fn parse(_data: &[u8]) -> Result<(Self, usize), MqttError> {
        // PINGRESP packet has no variable header or payload
        let remaining_length = VariableByteInteger::from_u32(0).unwrap();

        let pingresp = Pingresp {
            fixed_header: [FixedHeader::Pingresp.as_u8()],
            remaining_length,
        };

        Ok((pingresp, 0))
    }
}

impl PingrespBuilder {
    /// Builds a PINGRESP packet from the builder
    ///
    /// Since PINGRESP packets have no configurable parameters in MQTT v3.1.1,
    /// this method simply creates a standard PINGRESP packet with the appropriate
    /// fixed header and zero remaining length.
    ///
    /// # Returns
    ///
    /// A `Result` containing the built `Pingresp` packet on success
    ///
    /// # Errors
    ///
    /// This method always succeeds for PINGRESP packets in MQTT v3.1.1.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let pingresp = mqtt::packet::v3_1_1::Pingresp::builder()
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(pingresp.size(), 2);
    /// ```
    pub fn build(self) -> Result<Pingresp, MqttError> {
        let remaining_length = VariableByteInteger::from_u32(0).unwrap();

        Ok(Pingresp {
            fixed_header: [FixedHeader::Pingresp.as_u8()],
            remaining_length,
        })
    }
}

/// Implements JSON serialization for PINGRESP packets
///
/// This implementation allows PINGRESP packets to be serialized to JSON format,
/// which is useful for debugging, logging, and protocol analysis. The JSON format
/// provides a human-readable representation of the packet structure.
///
/// The serialized format includes:
/// - `type`: The packet type as a string ("pingresp")
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use serde_json;
///
/// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
/// let json = serde_json::to_string(&pingresp).unwrap();
/// assert!(json.contains("pingresp"));
/// ```
impl Serialize for Pingresp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("pingresp", 1)?;
        state.serialize_field("type", PacketType::Pingresp.as_str())?;
        state.end()
    }
}

/// Implements display formatting for PINGRESP packets
///
/// This implementation provides a JSON representation of the PINGRESP packet
/// for human-readable output, debugging, and logging purposes. The display
/// format is consistent across all packet types in the library.
///
/// # Output Format
///
/// The display format is a JSON object containing the packet type information.
/// If serialization fails, an error message is displayed instead.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
/// println!("PINGRESP: {}", pingresp);
/// // Output: {"type":"pingresp"}
/// ```
impl fmt::Display for Pingresp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Implements debug formatting for PINGRESP packets
///
/// This implementation uses the same format as `Display` to provide
/// consistent output for debugging purposes. This ensures that debug
/// and display representations are identical for PINGRESP packets.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
/// println!("Debug: {:?}", pingresp);
/// // Output: {"type":"pingresp"}
/// ```
impl fmt::Debug for Pingresp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Implements the generic packet trait for PINGRESP packets
///
/// This trait provides a common interface for all MQTT packet types,
/// allowing them to be used polymorphically in generic contexts such
/// as packet serialization, transmission, and size calculation.
///
/// # Methods
///
/// - `size()`: Returns the packet size in bytes
/// - `to_buffers()`: Returns I/O slices for network transmission
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
/// let generic_packet: &dyn mqtt::packet::GenericPacketTrait = &pingresp;
/// assert_eq!(generic_packet.size(), 2);
/// ```
impl GenericPacketTrait for Pingresp {
    fn size(&self) -> usize {
        self.size()
    }

    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }
}

/// Implements the generic packet display trait for PINGRESP packets
///
/// This trait provides a common interface for formatting MQTT packets,
/// allowing them to be displayed consistently across different packet types.
/// It supports both debug and display formatting modes.
///
/// # Methods
///
/// - `fmt_debug()`: Debug formatting (same as `Debug` trait)
/// - `fmt_display()`: Display formatting (same as `Display` trait)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::prelude::*;
///
/// let pingresp = mqtt::packet::v3_1_1::Pingresp::new();
/// let generic_display: &dyn mqtt::packet::GenericPacketDisplay = &pingresp;
/// println!("{}", format!("{:?}", generic_display));
/// ```
impl GenericPacketDisplay for Pingresp {
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
