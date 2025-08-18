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
use derive_builder::Builder;
#[cfg(feature = "std")]
use std::io::IoSlice;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::result_code::MqttError;

/// MQTT v3.1.1 DISCONNECT packet representation
///
/// The DISCONNECT packet is sent by the client to indicate graceful disconnection
/// from the MQTT broker. In MQTT v3.1.1, only the client can send DISCONNECT packets
/// to cleanly terminate the connection.
///
/// According to the MQTT v3.1.1 specification
/// (<https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html>),
/// the DISCONNECT packet contains:
/// - Fixed header with packet type (14) and remaining length (0)
/// - No variable header
/// - No payload
///
/// # Protocol Behavior
///
/// When a client sends a DISCONNECT packet:
/// - The client must not send any more packets on that connection
/// - The broker must discard any Will Message associated with the connection
/// - The broker should close the network connection
/// - The session state is preserved if Clean Session was set to 0 in CONNECT
///
/// # Packet Structure
///
/// The DISCONNECT packet in MQTT v3.1.1 is extremely simple:
/// - Fixed header: 2 bytes (packet type + remaining length of 0)
/// - Variable header: None
/// - Payload: None
///
/// The remaining length is always 0 for DISCONNECT packets in v3.1.1.
///
/// # Will Message Handling
///
/// A key difference from ungraceful disconnection is that when a client sends
/// a DISCONNECT packet, the broker MUST discard any Will Message for that client
/// without publishing it. This allows clients to disconnect cleanly without
/// triggering their Will Message.
///
/// # Network Connection
///
/// After sending a DISCONNECT packet, the client should close the network connection.
/// The broker, upon receiving a DISCONNECT packet, should close the network connection
/// to the client.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Create a DISCONNECT packet
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
/// assert_eq!(disconnect.size(), 2); // Fixed header (1) + remaining length (1)
///
/// // Using the builder pattern
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::builder()
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = disconnect.to_buffers();
/// let size = disconnect.size();
///
/// // Parse from received bytes
/// let data = []; // No variable header or payload data
/// let (parsed_disconnect, consumed) = mqtt::packet::v3_1_1::Disconnect::parse(&data).unwrap();
/// assert_eq!(consumed, 0); // No bytes consumed from variable header/payload
/// ```
#[derive(PartialEq, Eq, Builder, Clone)]
#[builder(no_std, derive(Debug), pattern = "owned", build_fn(skip))]
pub struct Disconnect {
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
}

impl Disconnect {
    /// Creates a new DISCONNECT packet with default configuration
    ///
    /// This is the simplest way to create a DISCONNECT packet for MQTT v3.1.1.
    /// The packet is created with the standard fixed header and zero remaining length.
    ///
    /// # Returns
    ///
    /// A new `Disconnect` instance ready for transmission
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
    /// assert_eq!(disconnect.size(), 2); // Fixed header + remaining length
    /// ```
    pub fn new() -> Self {
        Self {
            fixed_header: [FixedHeader::Disconnect.as_u8()],
            remaining_length: VariableByteInteger::from_u32(0).unwrap(),
        }
    }

    /// Creates a new builder for constructing a DISCONNECT packet
    ///
    /// The builder pattern provides a consistent interface across all packet types,
    /// even though DISCONNECT packets in v3.1.1 have no configurable parameters.
    /// This method is provided for API consistency.
    ///
    /// # Returns
    ///
    /// A new `DisconnectBuilder` instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let disconnect = mqtt::packet::v3_1_1::Disconnect::builder()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> DisconnectBuilder {
        DisconnectBuilder::default()
    }

    /// Returns the packet type for DISCONNECT packets
    ///
    /// This is always `PacketType::Disconnect` (14) for DISCONNECT packets.
    /// This method is useful for packet identification and routing.
    ///
    /// # Returns
    ///
    /// `PacketType::Disconnect`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::packet_type::PacketType;
    ///
    /// assert_eq!(mqtt::packet::v3_1_1::Disconnect::packet_type(), PacketType::Disconnect);
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Disconnect
    }

    /// Returns the total size of the DISCONNECT packet in bytes
    ///
    /// For MQTT v3.1.1 DISCONNECT packets, this is always 2 bytes:
    /// - 1 byte for the fixed header (packet type and flags)
    /// - 1 byte for the remaining length (always 0)
    ///
    /// # Returns
    ///
    /// The total packet size in bytes (always 2 for v3.1.1 DISCONNECT)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
    /// assert_eq!(disconnect.size(), 2);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the DISCONNECT packet to a vector of byte slices for network transmission
    ///
    /// This method creates a zero-copy representation of the packet as `IoSlice` buffers,
    /// which can be efficiently written to the network using vectored I/O operations.
    ///
    /// For MQTT v3.1.1 DISCONNECT packets, the buffers contain:
    /// 1. Fixed header (1 byte) - packet type and flags
    /// 2. Remaining length (1 byte) - always 0x00
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` containing the packet data ready for transmission
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
    /// let buffers = disconnect.to_buffers();
    /// assert_eq!(buffers.len(), 2); // Fixed header + remaining length
    /// // Can be used with vectored I/O operations like write_vectored
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        vec![
            IoSlice::new(&self.fixed_header),
            IoSlice::new(self.remaining_length.as_bytes()),
        ]
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is compatible with no-std environments and provides an alternative
    /// to [`to_buffers()`] when vectored I/O is not needed.
    ///
    /// For MQTT v3.1.1 DISCONNECT packets, the buffer contains:
    /// 1. Fixed header (1 byte) - packet type and flags
    /// 2. Remaining length (1 byte) - always 0x00
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
    /// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
    /// let buffer = disconnect.to_continuous_buffer();
    /// assert_eq!(buffer.len(), 2); // Fixed header + remaining length
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf
    }

    /// Parses a DISCONNECT packet from byte data
    ///
    /// This method parses the variable header portion of a DISCONNECT packet.
    /// Since MQTT v3.1.1 DISCONNECT packets have no variable header or payload,
    /// this method always succeeds and consumes 0 bytes from the input data.
    ///
    /// The fixed header should have been parsed separately before calling this method.
    ///
    /// # Parameters
    ///
    /// * `_data` - Byte slice containing the variable header data (unused for v3.1.1)
    ///
    /// # Returns
    ///
    /// * `Ok((Disconnect, usize))` - The parsed packet and number of bytes consumed (always 0)
    /// * This method never returns an error for valid MQTT v3.1.1 implementations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse DISCONNECT with no variable header data
    /// let data = []; // No variable header or payload for v3.1.1
    /// let (disconnect, consumed) = mqtt::packet::v3_1_1::Disconnect::parse(&data).unwrap();
    /// assert_eq!(consumed, 0);
    /// assert_eq!(disconnect.size(), 2);
    ///
    /// // The method ignores any data passed to it
    /// let data_with_extra = [0x01, 0x02, 0x03];
    /// let (disconnect, consumed) = mqtt::packet::v3_1_1::Disconnect::parse(&data_with_extra).unwrap();
    /// assert_eq!(consumed, 0); // Still consumes 0 bytes
    /// ```
    pub fn parse(_data: &[u8]) -> Result<(Self, usize), MqttError> {
        // DISCONNECT packet has no variable header or payload in v3.1.1
        let remaining_length = VariableByteInteger::from_u32(0).unwrap();

        let disconnect = Disconnect {
            fixed_header: [FixedHeader::Disconnect.as_u8()],
            remaining_length,
        };

        Ok((disconnect, 0))
    }
}

/// Builder for constructing DISCONNECT packets
///
/// The `DisconnectBuilder` provides a consistent builder interface for DISCONNECT packets,
/// even though MQTT v3.1.1 DISCONNECT packets have no configurable parameters.
/// This builder is provided for API consistency across all packet types.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Build a DISCONNECT packet using the builder pattern
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::builder()
///     .build()
///     .unwrap();
///
/// assert_eq!(disconnect.size(), 2);
/// ```
impl DisconnectBuilder {
    /// Builds the DISCONNECT packet from the configured parameters
    ///
    /// Since MQTT v3.1.1 DISCONNECT packets have no configurable parameters,
    /// this method always creates the same packet structure regardless of
    /// the builder state. The packet is created with:
    /// - Fixed header with DISCONNECT packet type (14)
    /// - Remaining length of 0
    /// - No variable header or payload
    ///
    /// # Returns
    ///
    /// * `Ok(Disconnect)` - The constructed packet (always succeeds)
    /// * This method never returns an error for MQTT v3.1.1 DISCONNECT packets
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let result = mqtt::packet::v3_1_1::Disconnect::builder()
    ///     .build();
    ///
    /// match result {
    ///     Ok(disconnect) => {
    ///         println!("DISCONNECT packet built successfully");
    ///         assert_eq!(disconnect.size(), 2);
    ///     }
    ///     Err(e) => {
    ///         // This should never happen for v3.1.1 DISCONNECT
    ///         println!("Build failed: {:?}", e);
    ///     }
    /// }
    /// ```
    pub fn build(self) -> Result<Disconnect, MqttError> {
        let remaining_length = VariableByteInteger::from_u32(0).unwrap();

        Ok(Disconnect {
            fixed_header: [FixedHeader::Disconnect.as_u8()],
            remaining_length,
        })
    }
}

/// Implements JSON serialization for DISCONNECT packets
///
/// This implementation converts the DISCONNECT packet to a JSON representation
/// suitable for debugging, logging, or API responses. Since MQTT v3.1.1 DISCONNECT
/// packets have no configurable parameters, the serialization only includes the packet type.
///
/// # JSON Structure
///
/// ```json
/// {
///   "type": "disconnect"
/// }
/// ```
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
/// let json = serde_json::to_string(&disconnect).unwrap();
/// println!("DISCONNECT packet: {}", json);
/// // Output: {"type":"disconnect"}
/// ```
impl Serialize for Disconnect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("disconnect", 1)?;
        state.serialize_field("type", PacketType::Disconnect.as_str())?;
        state.end()
    }
}

/// Implements `Display` trait for DISCONNECT packets
///
/// This provides a human-readable JSON representation of the packet,
/// making it useful for debugging and logging purposes. The output
/// is the same as the JSON serialization.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
/// println!("Packet: {}", disconnect);
/// // Output: {"type":"disconnect"}
/// ```
impl fmt::Display for Disconnect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Implements `Debug` trait for DISCONNECT packets
///
/// This provides the same output as the `Display` implementation,
/// showing the JSON representation for debugging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
/// println!("{:?}", disconnect);
/// // Output: {"type":"disconnect"}
/// ```
impl fmt::Debug for Disconnect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Implements the generic packet trait for DISCONNECT packets
///
/// This trait provides a common interface for all MQTT packet types,
/// allowing them to be used polymorphically in packet processing code.
/// The implementation delegates to the specific DISCONNECT methods.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;
///
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
///
/// // Use through the generic trait
/// let size = disconnect.size();
/// let buffers = disconnect.to_buffers();
/// assert_eq!(size, 2);
/// assert_eq!(buffers.len(), 2);
/// ```
impl GenericPacketTrait for Disconnect {
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

/// Implements the generic packet display trait for DISCONNECT packets
///
/// This trait provides a common display interface for all MQTT packet types,
/// enabling consistent formatting across different packet implementations.
/// The implementation delegates to the standard `Debug` and `Display` traits.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketDisplay;
///
/// let disconnect = mqtt::packet::v3_1_1::Disconnect::new();
///
/// // Format through the generic trait
/// println!("{}", disconnect); // Uses fmt_display
/// println!("{:?}", disconnect); // Uses fmt_debug
/// ```
impl GenericPacketDisplay for Disconnect {
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}
