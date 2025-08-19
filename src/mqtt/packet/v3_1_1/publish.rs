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

use alloc::sync::Arc;
use alloc::vec::Vec;
use core::fmt;
use core::mem;
use derive_builder::Builder;
#[cfg(feature = "std")]
use std::io::IoSlice;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use getset::{CopyGetters, Getters};

use crate::mqtt::common::{GenericArcPayload, IntoPayload};
use crate::mqtt::packet::json_bin_encode::escape_binary_json_string;
use crate::mqtt::packet::mqtt_string::GenericMqttString;
use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::qos::Qos;
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::result_code::MqttError;

/// MQTT 3.1.1 PUBLISH packet representation
///
/// The PUBLISH packet is used to transport application messages from a client to the server
/// or from the server to a client. It is the primary packet type for delivering messages
/// in MQTT and supports Quality of Service (QoS) levels 0, 1, and 2.
///
/// According to MQTT 3.1.1 specification section 3.3, the PUBLISH packet contains:
/// - Fixed header with packet type, flags (DUP, QoS, RETAIN), and remaining length
/// - Variable header with topic name and packet identifier (for QoS > 0)
/// - Payload containing the application message data
///
/// # Fixed Header Flags
///
/// The PUBLISH packet uses the following fixed header flags:
/// - **Bit 0**: RETAIN flag - if set, the server retains the message for future subscribers
/// - **Bits 1-2**: QoS level (0, 1, or 2) - determines delivery guarantee
/// - **Bit 3**: DUP flag - indicates this is a duplicate message (QoS > 0 only)
/// - **Bits 4-7**: Packet type (0011 for PUBLISH)
///
/// # Quality of Service (QoS)
///
/// - **QoS 0**: At most once delivery - fire and forget, no packet identifier required
/// - **QoS 1**: At least once delivery - requires packet identifier and PUBACK response
/// - **QoS 2**: Exactly once delivery - requires packet identifier and PUBREC/PUBREL/PUBCOMP sequence
///
/// # Topic Names
///
/// Topic names in PUBLISH packets must not contain wildcard characters (+ or #) as these
/// are reserved for subscription filters. The topic name is encoded as an MQTT String
/// in the variable header.
///
/// # RETAIN Flag
///
/// When the RETAIN flag is set, the server stores the message and delivers it to future
/// subscribers that match the topic filter. Only one retained message per topic is stored.
///
/// # DUP Flag
///
/// The DUP flag is used to indicate that this might be a re-delivery of an earlier attempt
/// to send the packet. This flag is only meaningful for QoS > 0 messages and should be
/// set when re-transmitting a PUBLISH packet.
///
/// # Generic Type Parameters
///
/// - `PacketIdType`: The type used for packet identifiers (usually u16, but can be u32 for broker clusters)
/// - `STRING_BUFFER_SIZE`: Stack buffer size for topic names in bytes (default: 32)
/// - `PAYLOAD_BUFFER_SIZE`: Stack buffer size for message payloads in bytes (default: 32)
///
/// # Stack Buffer Optimization
///
/// This implementation uses Small String/Buffer Optimization (SSO) to store small data on the stack
/// rather than the heap, improving performance for typical MQTT messages. When data exceeds the
/// buffer size, it automatically falls back to heap allocation.
///
/// # Custom Buffer Sizes
///
/// For applications that need different performance characteristics, you can define custom
/// type aliases with specific buffer sizes:
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::packet::v3_1_1::GenericPublish;
///
/// // For high-throughput applications with larger messages
/// type LargePublish = GenericPublish<u16, 128, 512>;
///
/// // For memory-constrained environments
/// type SmallPublish = GenericPublish<u16, 16, 16>;
///
/// // For broker clusters with extended packet IDs
/// type ClusterPublish = GenericPublish<u32, 64, 256>;
///
/// // Usage is identical to the standard Publish type
/// let publish = LargePublish::builder()
///     .topic_name("sensors/temperature/detailed/location")
///     .payload(&large_sensor_data)
///     .build()
///     .unwrap();
/// ```
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::Qos;
///
/// // Create a simple QoS 0 PUBLISH message using the standard type
/// let publish = mqtt::packet::v3_1_1::Publish::builder()
///     .topic_name("sensor/temperature")
///     .unwrap()
///     .qos(Qos::AtMostOnce)
///     .payload(b"23.5")
///     .build()
///     .unwrap();
///
/// assert_eq!(publish.topic_name(), "sensor/temperature");
/// assert_eq!(publish.qos(), Qos::AtMostOnce);
/// assert_eq!(publish.payload().as_slice(), b"23.5");
/// assert!(!publish.retain());
/// assert!(!publish.dup());
/// assert_eq!(publish.packet_id(), None);
///
/// // Create a QoS 1 PUBLISH message with retain flag
/// let publish = mqtt::packet::v3_1_1::Publish::builder()
///     .topic_name("device/status")
///     .unwrap()
///     .qos(Qos::AtLeastOnce)
///     .packet_id(123)
///     .retain(true)
///     .payload(b"online")
///     .build()
///     .unwrap();
///
/// assert_eq!(publish.qos(), Qos::AtLeastOnce);
/// assert!(publish.retain());
/// assert_eq!(publish.packet_id(), Some(123));
///
/// // Serialize to bytes for network transmission
/// let total_size = publish.size();
/// let buffer = publish.to_continuous_buffer();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericPublish<
    PacketIdType,
    const STRING_BUFFER_SIZE: usize = 32,
    const PAYLOAD_BUFFER_SIZE: usize = 32,
> where
    PacketIdType: IsPacketId,
{
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    topic_name_buf: GenericMqttString<STRING_BUFFER_SIZE>,
    #[builder(private)]
    packet_id_buf: Option<PacketIdType::Buffer>,

    #[builder(private)]
    payload_buf: GenericArcPayload<PAYLOAD_BUFFER_SIZE>,
}

/// Standard PUBLISH packet with default settings
///
/// This is the most commonly used PUBLISH packet type for standard MQTT 3.1.1
/// applications. It uses u16 packet identifiers and default buffer sizes
/// (32 bytes for both topic names and payloads).
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::packet::v3_1_1::Publish;
/// use mqtt_protocol_core::mqtt::packet::Qos;
///
/// let publish = Publish::builder()
///     .topic_name("my/topic")
///     .qos(Qos::AtLeastOnce)
///     .packet_id(42)
///     .payload(b"Hello, MQTT!")
///     .build()
///     .unwrap();
/// ```
pub type Publish = GenericPublish<u16>;

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize>
    GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a PUBLISH packet
    ///
    /// The builder pattern allows for flexible construction of PUBLISH packets
    /// with various combinations of QoS levels, flags, and content.
    ///
    /// # Returns
    ///
    /// A `GenericPublishBuilder` instance with default values
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("sensors/temperature")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(1)
    ///     .retain(true)
    ///     .payload(b"25.3")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericPublishBuilder<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
    {
        GenericPublishBuilder::<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::default()
    }

    /// Returns the packet type for PUBLISH packets
    ///
    /// This is always `PacketType::Publish` for PUBLISH packet instances.
    ///
    /// # Returns
    ///
    /// `PacketType::Publish`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::packet_type::PacketType;
    ///
    /// assert_eq!(mqtt::packet::v3_1_1::Publish::packet_type(), PacketType::Publish);
    /// ```
    pub const fn packet_type() -> PacketType {
        PacketType::Publish
    }

    /// Returns the packet identifier if present
    ///
    /// The packet identifier is only present for QoS 1 and QoS 2 PUBLISH packets.
    /// For QoS 0 packets, this method returns `None`.
    ///
    /// # Returns
    ///
    /// - `Some(PacketIdType)` - The packet identifier for QoS > 0 packets
    /// - `None` - For QoS 0 packets or if no packet ID was set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// // QoS 0 packet has no packet ID
    /// let publish_qos0 = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::AtMostOnce)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(publish_qos0.packet_id(), None);
    ///
    /// // QoS 1 packet has packet ID
    /// let publish_qos1 = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(123)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(publish_qos1.packet_id(), Some(123));
    /// ```
    pub fn packet_id(&self) -> Option<PacketIdType> {
        self.packet_id_buf
            .as_ref()
            .map(|buf| PacketIdType::from_buffer(buf.as_ref()))
    }

    /// Returns the Quality of Service level for this PUBLISH packet
    ///
    /// The QoS level determines the delivery guarantee for the message:
    /// - QoS 0: At most once delivery (fire and forget)
    /// - QoS 1: At least once delivery (requires PUBACK)
    /// - QoS 2: Exactly once delivery (requires PUBREC/PUBREL/PUBCOMP)
    ///
    /// # Returns
    ///
    /// The `Qos` level extracted from the fixed header flags
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::ExactlyOnce)
    ///     .packet_id(42)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(publish.qos(), Qos::ExactlyOnce);
    /// ```
    pub fn qos(&self) -> Qos {
        let qos_value = (self.fixed_header[0] >> 1) & 0b0000_0011;
        match qos_value {
            0 => Qos::AtMostOnce,
            1 => Qos::AtLeastOnce,
            2 => Qos::ExactlyOnce,
            _ => unreachable!("Invalid QoS value"),
        }
    }

    /// Returns the DUP flag value
    ///
    /// The DUP flag indicates whether this packet is a duplicate of an earlier
    /// transmission attempt. This flag is only meaningful for QoS > 0 packets
    /// and should be set during re-transmission scenarios.
    ///
    /// # Returns
    ///
    /// `true` if the DUP flag is set, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(1)
    ///     .dup(true)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(publish.dup());
    /// ```
    pub fn dup(&self) -> bool {
        (self.fixed_header[0] & 0b0000_1000) != 0
    }

    /// Returns the RETAIN flag value
    ///
    /// The RETAIN flag indicates that the server should store this message
    /// and deliver it to future subscribers that match the topic filter.
    /// Only one retained message per topic is stored by the server.
    ///
    /// # Returns
    ///
    /// `true` if the RETAIN flag is set, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("device/status")
    ///     .unwrap()
    ///     .retain(true)
    ///     .payload(b"online")
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(publish.retain());
    /// ```
    pub fn retain(&self) -> bool {
        (self.fixed_header[0] & 0b0000_0001) != 0
    }

    /// Sets the DUP flag and returns the modified packet
    ///
    /// This method allows modifying the DUP flag after packet creation,
    /// which is useful when implementing re-transmission logic for QoS > 0 packets.
    ///
    /// # Parameters
    ///
    /// * `dup` - `true` to set the DUP flag, `false` to clear it
    ///
    /// # Returns
    ///
    /// The modified packet with the DUP flag updated
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(1)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(!publish.dup());
    ///
    /// let publish_with_dup = publish.set_dup(true);
    /// assert!(publish_with_dup.dup());
    /// ```
    pub fn set_dup(mut self, dup: bool) -> Self {
        if dup {
            self.fixed_header[0] |= 0b0000_1000;
        } else {
            self.fixed_header[0] &= !0b0000_1000;
        }
        self
    }

    /// Returns the topic name for this PUBLISH packet
    ///
    /// The topic name identifies the information channel to which the payload
    /// is published. Topic names are UTF-8 encoded strings and must not contain
    /// wildcard characters (+ or #).
    ///
    /// # Returns
    ///
    /// A string slice containing the topic name
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("sensors/temperature/room1")
    ///     .unwrap()
    ///     .payload(b"22.5")
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(publish.topic_name(), "sensors/temperature/room1");
    /// ```
    pub fn topic_name(&self) -> &str {
        self.topic_name_buf.as_str()
    }

    /// Returns a reference to the message payload
    ///
    /// The payload contains the application message that is being published.
    /// It can be any binary data and is not interpreted by the MQTT protocol.
    ///
    /// # Returns
    ///
    /// A reference to the `ArcPayload` containing the message data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let message_data = b"Hello, MQTT World!";
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("greetings")
    ///     .unwrap()
    ///     .payload(message_data)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(publish.payload().as_slice(), message_data);
    /// ```
    pub fn payload(&self) -> &GenericArcPayload<PAYLOAD_BUFFER_SIZE> {
        &self.payload_buf
    }

    /// Returns the total size of the packet in bytes
    ///
    /// This includes the fixed header, variable header, and payload.
    /// The size is useful for buffer allocation and network transmission planning.
    ///
    /// # Returns
    ///
    /// The total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test")
    ///     .unwrap()
    ///     .payload(b"data")
    ///     .build()
    ///     .unwrap();
    ///
    /// let size = publish.size();
    /// assert!(size > 0);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// represent the complete PUBLISH packet in wire format.
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
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .payload(b"test data")
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = publish.to_buffers();
    /// // Use with vectored write: socket.write_vectored(&buffers)?;
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.append(&mut self.topic_name_buf.to_buffers());
        if let Some(buf) = &self.packet_id_buf {
            bufs.push(IoSlice::new(buf.as_ref()));
        }
        if self.payload_buf.len() > 0 {
            bufs.push(IoSlice::new(self.payload_buf.as_slice()));
        }
        bufs
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is an alternative to [`to_buffers()`] and is compatible with
    /// no-std environments where vectored I/O may not be available.
    ///
    /// The returned buffer contains the complete PUBLISH packet serialized according
    /// to the MQTT v3.1.1 protocol specification, including fixed header, variable
    /// header, and payload.
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
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .payload(b"test data")
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = publish.to_continuous_buffer();
    /// // buffer contains all packet bytes
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.append(&mut self.topic_name_buf.to_continuous_buffer());
        if let Some(packet_id_buf) = &self.packet_id_buf {
            buf.extend_from_slice(packet_id_buf.as_ref());
        }
        if self.payload_buf.len() > 0 {
            buf.extend_from_slice(self.payload_buf.as_slice());
        }
        buf
    }

    /// Parses a PUBLISH packet from raw bytes
    ///
    /// This method deserializes a PUBLISH packet from its wire format representation.
    /// It validates the packet structure according to MQTT 3.1.1 specification and
    /// returns a parsed packet instance along with the number of bytes consumed.
    ///
    /// # Parameters
    ///
    /// * `flags` - The fixed header flags byte containing QoS, DUP, and RETAIN flags
    /// * `data_arc` - Arc-wrapped byte slice containing the packet variable header and payload
    ///
    /// # Returns
    ///
    /// * `Ok((GenericPublish, usize))` - Successfully parsed packet and bytes consumed
    /// * `Err(MqttError)` - Parse error if the packet is malformed
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - QoS level is 3 (invalid)
    /// - Topic name is invalid UTF-8 or too long
    /// - Packet ID is missing for QoS > 0 packets
    /// - Insufficient data for required fields
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use alloc::sync::Arc;
    ///
    /// // Raw packet data (example - actual format would be binary)
    /// let packet_data = Arc::new([/* packet bytes */]);
    /// let flags = 0b0010; // QoS 1, no DUP, no RETAIN
    ///
    /// match mqtt::packet::v3_1_1::Publish::parse(flags, packet_data) {
    ///     Ok((publish, consumed)) => {
    ///         println!("Parsed packet with {} bytes consumed", consumed);
    ///         println!("Topic: {}", publish.topic_name());
    ///     }
    ///     Err(e) => println!("Parse error: {:?}", e),
    /// }
    /// ```
    pub fn parse(flags: u8, data_arc: Arc<[u8]>) -> Result<(Self, usize), MqttError> {
        let fixed_header_byte = FixedHeader::Publish as u8 | (flags & 0b0000_1111);

        let qos_value = (flags >> 1) & 0b0000_0011;
        if qos_value == 3 {
            return Err(MqttError::MalformedPacket);
        }

        let mut cursor = 0;

        let (topic_name, consumed) =
            GenericMqttString::<STRING_BUFFER_SIZE>::decode(&data_arc[cursor..])?;
        cursor += consumed;

        let qos = match qos_value {
            0 => Qos::AtMostOnce,
            1 => Qos::AtLeastOnce,
            2 => Qos::ExactlyOnce,
            _ => unreachable!(),
        };

        let packet_id_buf = if qos != Qos::AtMostOnce {
            let buffer_size = core::mem::size_of::<<PacketIdType as IsPacketId>::Buffer>();
            if data_arc.len() < cursor + buffer_size {
                return Err(MqttError::MalformedPacket);
            }
            let mut buf = PacketIdType::Buffer::default();
            buf.as_mut()
                .copy_from_slice(&data_arc[cursor..cursor + buffer_size]);
            cursor += buffer_size;
            Some(buf)
        } else {
            None
        };

        let payload_len = data_arc.len() - cursor;
        let payload = if payload_len > 0 {
            GenericArcPayload::<PAYLOAD_BUFFER_SIZE>::new(data_arc.clone(), cursor, payload_len)
        } else {
            GenericArcPayload::<PAYLOAD_BUFFER_SIZE>::default()
        };

        let remaining_size = topic_name.size()
            + packet_id_buf
                .as_ref()
                .map_or(0, |_| mem::size_of::<PacketIdType>())
            + payload_len;

        let publish = GenericPublish {
            fixed_header: [fixed_header_byte],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            topic_name_buf: topic_name,
            packet_id_buf,
            payload_buf: payload,
        };

        Ok((publish, data_arc.len()))
    }
}

/// Builder implementation for constructing PUBLISH packets
///
/// The builder provides a fluent interface for constructing PUBLISH packets
/// with various configurations. It validates the packet configuration before
/// building and ensures MQTT 3.1.1 protocol compliance.
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize>
    GenericPublishBuilder<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
where
    PacketIdType: IsPacketId,
{
    /// Sets the topic name for the PUBLISH packet
    ///
    /// The topic name identifies the information channel to which the payload
    /// is published. Topic names must be valid UTF-8 strings and cannot contain
    /// wildcard characters (+ or #) which are reserved for subscription filters.
    ///
    /// # Parameters
    ///
    /// * `topic` - The topic name as a string reference
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with topic name set
    /// * `Err(MqttError)` - If topic name is invalid or contains wildcards
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - Topic name contains wildcard characters (+ or #)
    /// - Topic name is not valid UTF-8
    /// - Topic name exceeds maximum length
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("sensors/temperature/room1")
    ///     .unwrap();
    ///
    /// // This would fail due to wildcard
    /// // let invalid = mqtt::packet::v3_1_1::Publish::builder()
    /// //     .topic_name("sensors/+/temperature")
    /// //     .unwrap();
    /// ```
    pub fn topic_name<T: AsRef<str>>(mut self, topic: T) -> Result<Self, MqttError> {
        let mqtt_str = GenericMqttString::<STRING_BUFFER_SIZE>::new(topic)?;
        if mqtt_str.as_str().contains('#') || mqtt_str.as_str().contains('+') {
            return Err(MqttError::MalformedPacket);
        }
        self.topic_name_buf = Some(mqtt_str);
        Ok(self)
    }

    /// Sets the Quality of Service level for the PUBLISH packet
    ///
    /// The QoS level determines the delivery guarantee for the message:
    /// - QoS 0: At most once delivery (fire and forget)
    /// - QoS 1: At least once delivery (requires acknowledgment)
    /// - QoS 2: Exactly once delivery (requires handshake)
    ///
    /// # Parameters
    ///
    /// * `qos` - The desired QoS level
    ///
    /// # Returns
    ///
    /// The builder with QoS level set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let builder = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce);
    /// ```
    pub fn qos(mut self, qos: Qos) -> Self {
        let mut header = self.fixed_header.unwrap_or([FixedHeader::Publish as u8]);
        header[0] &= !0b0000_0110; // Clear the QoS bits
        header[0] |= (qos as u8) << 1;
        self.fixed_header = Some(header);
        self
    }

    /// Sets the DUP flag for the PUBLISH packet
    ///
    /// The DUP flag indicates whether this packet is a duplicate of an earlier
    /// transmission attempt. This flag is only meaningful for QoS > 0 packets
    /// and should be set when re-transmitting a packet.
    ///
    /// # Parameters
    ///
    /// * `dup` - `true` to set the DUP flag, `false` to clear it
    ///
    /// # Returns
    ///
    /// The builder with DUP flag set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let builder = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(1)
    ///     .dup(true);
    /// ```
    pub fn dup(mut self, dup: bool) -> Self {
        let mut header = self.fixed_header.unwrap_or([FixedHeader::Publish as u8]);
        if dup {
            header[0] |= 0b0000_1000;
        } else {
            header[0] &= !0b0000_1000;
        }
        self.fixed_header = Some(header);
        self
    }

    /// Sets the RETAIN flag for the PUBLISH packet
    ///
    /// The RETAIN flag indicates that the server should store this message
    /// and deliver it to future subscribers that match the topic filter.
    /// Only one retained message per topic is stored by the server.
    ///
    /// # Parameters
    ///
    /// * `retain` - `true` to set the RETAIN flag, `false` to clear it
    ///
    /// # Returns
    ///
    /// The builder with RETAIN flag set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("device/status")
    ///     .unwrap()
    ///     .retain(true)
    ///     .payload(b"online");
    /// ```
    pub fn retain(mut self, retain: bool) -> Self {
        let mut header = self.fixed_header.unwrap_or([FixedHeader::Publish as u8]);
        if retain {
            header[0] |= 0b00000001;
        } else {
            header[0] &= !0b00000001;
        }
        self.fixed_header = Some(header);
        self
    }

    /// Sets the packet identifier for the PUBLISH packet
    ///
    /// The packet identifier is required for QoS 1 and QoS 2 PUBLISH packets
    /// and must be a non-zero value. It is used to match the packet with its
    /// corresponding acknowledgment packets (PUBACK, PUBREC, etc.).
    ///
    /// # Parameters
    ///
    /// * `id` - The packet identifier (must be non-zero for QoS > 0)
    ///
    /// # Returns
    ///
    /// The builder with packet ID set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let builder = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(123);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(Some(id.to_buffer()));
        self
    }

    /// Sets the payload for the PUBLISH packet
    ///
    /// The payload contains the application message data that is being published.
    /// It can be any binary data and is not interpreted by the MQTT protocol.
    /// The payload can be empty for some use cases.
    ///
    /// # Parameters
    ///
    /// * `data` - The payload data implementing `IntoPayload`
    ///
    /// # Returns
    ///
    /// The builder with payload set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // String payload
    /// let builder1 = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("text/message")
    ///     .unwrap()
    ///     .payload("Hello, World!");
    ///
    /// // Binary payload
    /// let builder2 = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("binary/data")
    ///     .unwrap()
    ///     .payload(vec![0x01, 0x02, 0x03, 0x04]);
    ///
    /// // Byte slice payload
    /// let builder3 = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("sensor/reading")
    ///     .unwrap()
    ///     .payload(b"25.3");
    /// ```
    pub fn payload<T>(mut self, data: T) -> Self
    where
        T: IntoPayload<PAYLOAD_BUFFER_SIZE>,
    {
        self.payload_buf = Some(data.into_payload());
        self
    }

    fn validate(&self) -> Result<(), MqttError> {
        if self.topic_name_buf.is_none()
            || self.topic_name_buf.as_ref().unwrap().as_str().is_empty()
        {
            return Err(MqttError::MalformedPacket);
        }

        if let Some(header) = &self.fixed_header {
            let qos_value = (header[0] >> 1) & 0b0000_0011;
            let qos = match qos_value {
                0 => Qos::AtMostOnce,
                1 => Qos::AtLeastOnce,
                2 => Qos::ExactlyOnce,
                _ => return Err(MqttError::MalformedPacket),
            };

            if qos == Qos::AtMostOnce {
                if self.packet_id_buf.is_some() && self.packet_id_buf.as_ref().unwrap().is_some() {
                    return Err(MqttError::MalformedPacket);
                }
            } else {
                if self.packet_id_buf.is_none() || self.packet_id_buf.as_ref().unwrap().is_none() {
                    return Err(MqttError::MalformedPacket);
                }
                if let Some(Some(packet_id_buf)) = &self.packet_id_buf {
                    let packet_id = PacketIdType::from_buffer(packet_id_buf.as_ref());
                    if packet_id.is_zero() {
                        return Err(MqttError::MalformedPacket);
                    }
                }
            }
        } else if self.packet_id_buf.is_some() && self.packet_id_buf.as_ref().unwrap().is_some() {
            return Err(MqttError::MalformedPacket);
        }

        if let Some(payload) = &self.payload_buf {
            if payload.len() > 268435455 {
                return Err(MqttError::MalformedPacket);
            }
        }

        Ok(())
    }

    /// Builds and validates the PUBLISH packet
    ///
    /// This method constructs the final PUBLISH packet from the builder configuration.
    /// It performs comprehensive validation to ensure the packet complies with
    /// MQTT 3.1.1 specification requirements.
    ///
    /// # Returns
    ///
    /// * `Ok(GenericPublish<PacketIdType>)` - Successfully built and validated packet
    /// * `Err(MqttError)` - Validation error if packet configuration is invalid
    ///
    /// # Errors
    ///
    /// Returns `MqttError::MalformedPacket` if:
    /// - Topic name is empty or not set
    /// - QoS 0 packet has a packet ID (should be None)
    /// - QoS > 0 packet is missing a packet ID
    /// - Packet ID is zero for QoS > 0 packets
    /// - Payload exceeds maximum size (268,435,455 bytes)
    /// - Invalid QoS level (3)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// // Valid QoS 0 packet
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("sensors/temperature")
    ///     .unwrap()
    ///     .qos(Qos::AtMostOnce)
    ///     .payload(b"25.3")
    ///     .build()
    ///     .unwrap();
    ///
    /// // Valid QoS 1 packet with packet ID
    /// let publish = mqtt::packet::v3_1_1::Publish::builder()
    ///     .topic_name("device/status")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(123)
    ///     .payload(b"online")
    ///     .build()
    ///     .unwrap();
    ///
    /// // This would fail - QoS 1 packet without packet ID
    /// // let invalid = mqtt::packet::v3_1_1::Publish::builder()
    /// //     .topic_name("test/topic")
    /// //     .unwrap()
    /// //     .qos(Qos::AtLeastOnce)
    /// //     .build()
    /// //     .unwrap();
    /// ```
    pub fn build(
        self,
    ) -> Result<GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>, MqttError>
    {
        self.validate()?;

        let topic_name_buf = self.topic_name_buf.unwrap();
        let fixed_header = self.fixed_header.unwrap_or([FixedHeader::Publish as u8]);
        let packet_id_buf = self.packet_id_buf.flatten();
        let payload = self
            .payload_buf
            .unwrap_or_else(|| GenericArcPayload::<PAYLOAD_BUFFER_SIZE>::default());

        let mut remaining = topic_name_buf.size();
        if (fixed_header[0] >> 1) & 0b0000_0011 != 0 && packet_id_buf.is_some() {
            remaining += mem::size_of::<PacketIdType>();
        }
        remaining += payload.len();
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericPublish {
            fixed_header,
            remaining_length,
            topic_name_buf,
            packet_id_buf,
            payload_buf: payload,
        })
    }
}

/// Serde serialization implementation for PUBLISH packets
///
/// Serializes the PUBLISH packet to a structured format (typically JSON)
/// containing all packet fields including type, topic name, QoS, flags,
/// packet ID, and payload. Binary payload data is escaped as needed for
/// text-based serialization formats.
///
/// # Serialized Fields
///
/// - `type`: Always "publish"
/// - `topic_name`: The topic name string
/// - `qos`: QoS level (0, 1, or 2)
/// - `retain`: Boolean retain flag
/// - `dup`: Boolean DUP flag
/// - `packet_id`: Optional packet identifier (null for QoS 0)
/// - `payload`: Payload data (escaped if binary)
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize> Serialize
    for GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 6; // type, topic_name, qos, retain, dup, packet_id

        field_count += 1; // payload

        let mut state = serializer.serialize_struct("publish", field_count)?;
        state.serialize_field("type", PacketType::Publish.as_str())?;
        state.serialize_field("topic_name", &self.topic_name_buf)?;
        state.serialize_field("qos", &self.qos())?;
        state.serialize_field("retain", &self.retain())?;
        state.serialize_field("dup", &self.dup())?;
        state.serialize_field("packet_id", &self.packet_id())?;

        let payload_data = self.payload_buf.as_slice();
        match escape_binary_json_string(payload_data) {
            Some(escaped) => state.serialize_field("payload", &escaped)?,
            None => state.serialize_field("payload", &payload_data)?,
        }

        state.end()
    }
}

/// Display implementation for PUBLISH packets
///
/// Formats the PUBLISH packet as a JSON string for human-readable output.
/// This implementation uses the Serialize trait to convert the packet to JSON.
/// If serialization fails, an error message is displayed instead.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
///
/// let publish = mqtt::packet::v3_1_1::Publish::builder()
///     .topic_name("test/topic")
///     .unwrap()
///     .qos(Qos::AtLeastOnce)
///     .packet_id(42)
///     .payload(b"Hello")
///     .build()
///     .unwrap();
///
/// println!("{}", publish);
/// // Output: {"type":"publish","topic_name":"test/topic","qos":1,...}
/// ```
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize> fmt::Display
    for GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
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

/// Debug implementation for PUBLISH packets
///
/// Uses the same JSON formatting as the Display implementation to provide
/// consistent debug output. This makes debugging easier by showing all
/// packet fields in a structured format.
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize> fmt::Debug
    for GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// GenericPacketTrait implementation for PUBLISH packets
///
/// Provides common packet functionality including size calculation and
/// buffer conversion for network transmission. This trait is used by
/// the generic packet handling infrastructure.
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize>
    GenericPacketTrait for GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
where
    PacketIdType: IsPacketId,
{
    /// Returns the total packet size in bytes
    fn size(&self) -> usize {
        self.size()
    }

    /// Converts the packet to I/O slices for efficient transmission
    #[cfg(feature = "std")]
    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }

    /// Create a continuous buffer containing the complete packet data
    fn to_continuous_buffer(&self) -> Vec<u8> {
        self.to_continuous_buffer()
    }
}

/// GenericPacketDisplay implementation for PUBLISH packets
///
/// Provides unified display and debug formatting through the generic
/// packet display trait. This enables consistent packet formatting
/// across different packet types in the library.
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize>
    GenericPacketDisplay for GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>
where
    PacketIdType: IsPacketId + Serialize,
{
    /// Formats the packet for debug output
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    /// Formats the packet for display output
    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}
