use num_enum::TryFromPrimitive;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
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

/// MQTT Control Packet Type enumeration
///
/// Represents the MQTT packet types as defined in the MQTT specification.
/// These correspond to bits 7-4 of the Fixed Header's first byte.
/// Each variant maps to its numeric value as defined in the MQTT protocol.
///
/// Supports both MQTT v3.1.1 and v5.0, with `Auth` being exclusive to v5.0.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::packet::PacketType;
///
/// let packet_type = PacketType::Connect;
/// assert_eq!(packet_type.as_u8(), 1);
/// assert_eq!(packet_type.as_str(), "connect");
/// ```
#[derive(Deserialize, PartialEq, Eq, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum PacketType {
    /// Client connection request packet
    Connect = 1,
    /// Server connection acknowledgment packet
    Connack = 2,
    /// Publish message packet (QoS 0, 1, 2)
    Publish = 3,
    /// Publish acknowledgment packet (QoS 1)
    Puback = 4,
    /// Publish received packet (QoS 2, step 1)
    Pubrec = 5,
    /// Publish release packet (QoS 2, step 2)
    Pubrel = 6,
    /// Publish complete packet (QoS 2, step 3)
    Pubcomp = 7,
    /// Client subscription request packet
    Subscribe = 8,
    /// Server subscription acknowledgment packet
    Suback = 9,
    /// Client unsubscription request packet
    Unsubscribe = 10,
    /// Server unsubscription acknowledgment packet
    Unsuback = 11,
    /// Ping request packet (keep-alive)
    Pingreq = 12,
    /// Ping response packet (keep-alive)
    Pingresp = 13,
    /// Disconnect notification packet
    Disconnect = 14,
    /// Authentication exchange packet (MQTT v5.0 only)
    Auth = 15,
}

/// MQTT Fixed Header first byte enumeration
///
/// Represents the complete first byte of the MQTT Fixed Header, which includes
/// both the packet type (bits 7-4) and packet-specific flags (bits 3-0).
/// Each variant contains the full byte value with appropriate flags/reserved bits set.
///
/// # Packet-Specific Flags
///
/// - **PUBLISH**: Contains DUP, QoS, and RETAIN flags (bits 3-0)
/// - **PUBREL, SUBSCRIBE, UNSUBSCRIBE**: Have reserved bits set as required by spec
/// - **Other packets**: Flags/reserved bits are typically zero
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::packet::FixedHeader;
///
/// let header = FixedHeader::Connect;
/// assert_eq!(header.as_u8(), 0x10);
/// assert_eq!(header.packet_type(), PacketType::Connect);
/// ```
#[derive(Deserialize, PartialEq, Eq, Copy, Clone, TryFromPrimitive)]
#[repr(u8)]
pub enum FixedHeader {
    /// CONNECT packet header (0x10)
    Connect = 0x10,
    /// CONNACK packet header (0x20)
    Connack = 0x20,
    /// PUBLISH packet header (0x30) - flags for DUP, QoS, RETAIN in lower bits
    Publish = 0x30,
    /// PUBACK packet header (0x40)
    Puback = 0x40,
    /// PUBREC packet header (0x50)
    Pubrec = 0x50,
    /// PUBREL packet header (0x62) - reserved bits set as required
    Pubrel = 0x62,
    /// PUBCOMP packet header (0x70)
    Pubcomp = 0x70,
    /// SUBSCRIBE packet header (0x82) - reserved bits set as required
    Subscribe = 0x82,
    /// SUBACK packet header (0x90)
    Suback = 0x90,
    /// UNSUBSCRIBE packet header (0xa2) - reserved bits set as required
    Unsubscribe = 0xa2,
    /// UNSUBACK packet header (0xb0)
    Unsuback = 0xb0,
    /// PINGREQ packet header (0xc0)
    Pingreq = 0xc0,
    /// PINGRESP packet header (0xd0)
    Pingresp = 0xd0,
    /// DISCONNECT packet header (0xe0)
    Disconnect = 0xe0,
    /// AUTH packet header (0xf0) - MQTT v5.0 only
    Auth = 0xf0,
}

impl PacketType {
    /// Convert the packet type to its numeric value
    ///
    /// Returns the packet type as a `u8` value according to the MQTT specification.
    ///
    /// # Returns
    ///
    /// The numeric packet type value (1-15)
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Convert the packet type to its string representation
    ///
    /// Returns a lowercase string representation of the packet type,
    /// suitable for logging, debugging, or serialization.
    ///
    /// # Returns
    ///
    /// A static string slice with the packet type name
    pub fn as_str(&self) -> &'static str {
        match self {
            PacketType::Connect => "connect",
            PacketType::Connack => "connack",
            PacketType::Publish => "publish",
            PacketType::Puback => "puback",
            PacketType::Pubrec => "pubrec",
            PacketType::Pubrel => "pubrel",
            PacketType::Pubcomp => "pubcomp",
            PacketType::Subscribe => "subscribe",
            PacketType::Suback => "suback",
            PacketType::Unsubscribe => "unsubscribe",
            PacketType::Unsuback => "unsuback",
            PacketType::Pingreq => "pingreq",
            PacketType::Pingresp => "pingresp",
            PacketType::Disconnect => "disconnect",
            PacketType::Auth => "auth",
        }
    }

    /// Convert the packet type to its corresponding `FixedHeader`
    ///
    /// Creates a `FixedHeader` value with the packet type and appropriate
    /// default flags/reserved bits as specified by the MQTT protocol.
    ///
    /// # Returns
    ///
    /// The corresponding `FixedHeader` with default flags
    pub fn to_fixed_header(self) -> FixedHeader {
        match self {
            PacketType::Connect => FixedHeader::Connect,
            PacketType::Connack => FixedHeader::Connack,
            PacketType::Publish => FixedHeader::Publish,
            PacketType::Puback => FixedHeader::Puback,
            PacketType::Pubrec => FixedHeader::Pubrec,
            PacketType::Pubrel => FixedHeader::Pubrel,
            PacketType::Pubcomp => FixedHeader::Pubcomp,
            PacketType::Subscribe => FixedHeader::Subscribe,
            PacketType::Suback => FixedHeader::Suback,
            PacketType::Unsubscribe => FixedHeader::Unsubscribe,
            PacketType::Unsuback => FixedHeader::Unsuback,
            PacketType::Pingreq => FixedHeader::Pingreq,
            PacketType::Pingresp => FixedHeader::Pingresp,
            PacketType::Disconnect => FixedHeader::Disconnect,
            PacketType::Auth => FixedHeader::Auth,
        }
    }
}

impl FixedHeader {
    /// Convert the fixed header to its byte value
    ///
    /// Returns the complete first byte of the MQTT Fixed Header,
    /// including both packet type and flags/reserved bits.
    ///
    /// # Returns
    ///
    /// The fixed header byte value
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Extract the packet type from the fixed header
    ///
    /// Extracts the packet type from bits 7-4 of the fixed header byte,
    /// discarding the flags/reserved bits in the lower 4 bits.
    ///
    /// # Returns
    ///
    /// The `PacketType` extracted from the header, or `PacketType::Connect` as fallback
    pub fn packet_type(self) -> PacketType {
        let type_bits = (self as u8) >> 4;
        PacketType::try_from(type_bits).unwrap_or(PacketType::Connect)
    }

    /// Convert the fixed header to its string representation
    ///
    /// Returns the string representation of the underlying packet type.
    /// This is equivalent to calling `self.packet_type().as_str()`.
    ///
    /// # Returns
    ///
    /// A static string slice with the packet type name
    pub fn as_str(&self) -> &'static str {
        self.packet_type().as_str()
    }
}

/// Serialize `PacketType` as a string
impl Serialize for PacketType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Display `PacketType` as JSON string
impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Debug `PacketType` using Display implementation
impl fmt::Debug for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Serialize `FixedHeader` as a string
impl Serialize for FixedHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Display `FixedHeader` as JSON string
impl fmt::Display for FixedHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Debug `FixedHeader` using Display implementation
impl fmt::Debug for FixedHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
