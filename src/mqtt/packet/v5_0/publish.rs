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

use alloc::string::String;
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

use crate::mqtt::packet::json_bin_encode::escape_binary_json_string;
use crate::mqtt::packet::mqtt_string::MqttString;
use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::property::PropertiesToContinuousBuffer;
use crate::mqtt::packet::qos::Qos;
use crate::mqtt::packet::topic_alias_send::TopicAliasType;
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::IsPacketId;
#[cfg(feature = "std")]
use crate::mqtt::packet::PropertiesToBuffers;
use crate::mqtt::packet::{Properties, PropertiesParse, PropertiesSize, Property};
use crate::mqtt::result_code::MqttError;
use crate::mqtt::{ArcPayload, IntoPayload};

/// MQTT 5.0 PUBLISH packet representation
///
/// The PUBLISH packet is used to transport application messages from a client to the server
/// or from the server to a client. It is the primary packet type for delivering messages
/// in MQTT and supports Quality of Service (QoS) levels 0, 1, and 2.
///
/// According to MQTT 5.0 specification, the PUBLISH packet contains:
/// - Fixed header with packet type, flags (DUP, QoS, RETAIN), and remaining length
/// - Variable header with topic name, packet identifier (for QoS > 0), and properties
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
/// # Topic Names and Topic Aliases
///
/// MQTT 5.0 introduces Topic Aliases to reduce packet size for frequently used topics.
/// The topic name can be replaced with a numeric alias after the first use, allowing
/// for more efficient transmission of messages on the same topic.
///
/// # Properties
///
/// MQTT 5.0 PUBLISH packets can include various properties:
/// - Payload Format Indicator - indicates if payload is UTF-8 text or binary
/// - Message Expiry Interval - message expiration time in seconds
/// - Topic Alias - numeric alias for the topic name
/// - Response Topic - topic for response messages
/// - Correlation Data - correlation data for request/response flows
/// - User Properties - custom key-value pairs
/// - Subscription Identifier - identifier matching subscription (server to client only)
/// - Content Type - MIME type of the payload
///
/// # Generic Type Parameter
///
/// The `PacketIdType` generic parameter allows using packet identifiers larger than
/// the standard u16, which can be useful for broker clusters to avoid packet ID
/// exhaustion when extending the MQTT protocol.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
///
/// // Create a simple QoS 0 PUBLISH message
/// let publish = mqtt::packet::v5_0::Publish::builder()
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
/// let publish = mqtt::packet::v5_0::Publish::builder()
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
/// let buffers = publish.to_buffers();
/// let total_size = publish.size();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericPublish<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    topic_name_buf: MqttString,
    #[builder(private)]
    packet_id_buf: Option<PacketIdType::Buffer>,
    #[builder(private)]
    property_length: VariableByteInteger,

    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Properties,

    #[builder(private)]
    payload_buf: ArcPayload,

    #[builder(private)]
    #[getset(get_copy = "pub")]
    topic_name_extracted: bool,
}

/// Type alias for PUBLISH packet with standard u16 packet identifiers
///
/// This is the most commonly used PUBLISH packet type for standard MQTT 5.0
/// implementations that use 16-bit packet identifiers as specified in the protocol.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
///
/// let publish = mqtt::packet::v5_0::Publish::builder()
///     .topic_name("my/topic")
///     .unwrap()
///     .qos(Qos::AtLeastOnce)
///     .packet_id(42)
///     .payload(b"Hello, MQTT!")
///     .build()
///     .unwrap();
/// ```
pub type Publish = GenericPublish<u16>;

impl<PacketIdType> GenericPublish<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Creates a new builder for constructing a PUBLISH packet
    ///
    /// The builder pattern allows for flexible construction of PUBLISH packets
    /// with various combinations of properties, QoS levels, and content.
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
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("sensors/temperature")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(1)
    ///     .retain(true)
    ///     .payload(b"25.3")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericPublishBuilder<PacketIdType> {
        GenericPublishBuilder::<PacketIdType>::default()
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
    /// assert_eq!(mqtt::packet::v5_0::Publish::packet_type(), PacketType::Publish);
    /// ```
    pub fn packet_type() -> PacketType {
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
    /// - `None` - For QoS 0 packets
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// // QoS 0 packet has no packet ID
    /// let qos0_publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("topic")
    ///     .unwrap()
    ///     .qos(Qos::AtMostOnce)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(qos0_publish.packet_id(), None);
    ///
    /// // QoS 1 packet has packet ID
    /// let qos1_publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(42)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(qos1_publish.packet_id(), Some(42));
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
    /// - QoS 1: At least once delivery (acknowledged)
    /// - QoS 2: Exactly once delivery (assured)
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
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("topic")
    ///     .unwrap()
    ///     .qos(Qos::ExactlyOnce)
    ///     .packet_id(1)
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

    /// Returns the DUP (duplicate) flag status
    ///
    /// The DUP flag indicates whether this PUBLISH packet is a duplicate of an earlier
    /// packet that may not have been acknowledged. This flag is only meaningful for
    /// QoS 1 and QoS 2 packets and should be ignored for QoS 0 packets.
    ///
    /// # Returns
    ///
    /// - `true` if this is a duplicate packet
    /// - `false` if this is the first transmission attempt
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("topic")
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

    /// Returns the RETAIN flag status
    ///
    /// The RETAIN flag indicates whether the server should retain this message
    /// for future subscribers to the topic. When a client subscribes to a topic
    /// with a retained message, it will immediately receive the retained message.
    ///
    /// # Returns
    ///
    /// - `true` if the message should be retained by the server
    /// - `false` if the message should not be retained
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("device/status")
    ///     .unwrap()
    ///     .qos(Qos::AtMostOnce)
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

    /// Sets the DUP (duplicate) flag and returns the modified packet
    ///
    /// This method is typically used when retransmitting a QoS 1 or QoS 2 PUBLISH
    /// packet that may not have been acknowledged. The DUP flag helps the receiver
    /// identify potential duplicate messages.
    ///
    /// # Parameters
    ///
    /// - `dup`: Whether to set the DUP flag (`true`) or clear it (`false`)
    ///
    /// # Returns
    ///
    /// The modified `GenericPublish` instance with the DUP flag updated
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("topic")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(1)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert!(!publish.dup());
    ///
    /// let dup_publish = publish.set_dup(true);
    /// assert!(dup_publish.dup());
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
    /// is published. In MQTT 5.0, the topic name may be empty if a TopicAlias
    /// property is used instead.
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
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("sensors/temperature/room1")
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(publish.topic_name(), "sensors/temperature/room1");
    /// ```
    pub fn topic_name(&self) -> &str {
        self.topic_name_buf.as_str()
    }

    /// Returns a reference to the payload data
    ///
    /// The payload contains the application message data being published.
    /// It can be any binary data up to the maximum message size allowed
    /// by the MQTT implementation.
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
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("greetings")
    ///     .unwrap()
    ///     .payload(message_data)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(publish.payload().as_slice(), message_data);
    /// assert_eq!(publish.payload().len(), message_data.len());
    /// ```
    pub fn payload(&self) -> &ArcPayload {
        &self.payload_buf
    }

    /// Remove TopicAlias property and add topic name
    ///
    /// This method is used for store regulation - it sets the topic name and removes
    /// any TopicAlias property from the packet properties. This is typically done
    /// when storing messages where the topic alias needs to be resolved to the
    /// actual topic name for persistence.
    ///
    /// The method validates that the current topic name is empty (as expected when
    /// using topic aliases) and that the new topic name doesn't contain wildcard
    /// characters, which are not allowed in PUBLISH packets.
    ///
    /// # Parameters
    ///
    /// - `topic`: The topic name to set, replacing any topic alias
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` - The modified packet with topic name set and topic alias removed
    /// - `Err(MqttError)` - If the topic name is invalid or the packet state is incorrect
    ///
    /// # Errors
    ///
    /// - `MqttError::TopicNameInvalid` - If the current topic name is not empty
    /// - `MqttError::MalformedPacket` - If the topic contains wildcard characters
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Assume we have a publish packet with a topic alias
    /// let publish_with_alias = // ... created with topic alias
    ///
    /// let publish_with_topic = publish_with_alias
    ///     .remove_topic_alias_add_topic("sensors/temperature".to_string())
    ///     .unwrap();
    ///
    /// assert_eq!(publish_with_topic.topic_name(), "sensors/temperature");
    /// // Topic alias property has been removed from properties
    /// ```
    pub fn remove_topic_alias_add_topic(mut self, topic: String) -> Result<Self, MqttError> {
        // Check that topic_name is currently empty (for store regulation)
        if !self.topic_name_buf.as_str().is_empty() {
            return Err(MqttError::TopicNameInvalid);
        }

        // Validate topic name (no wildcards allowed in PUBLISH)
        if topic.contains('#') || topic.contains('+') {
            return Err(MqttError::MalformedPacket);
        }

        // Set the topic name
        self.topic_name_buf = MqttString::new(topic)?;

        // Remove TopicAlias property if present
        self.props
            .retain(|prop| !matches!(prop, crate::mqtt::packet::Property::TopicAlias(_)));

        // Recalculate property_length and remaining_length
        self.recalculate_lengths();

        Ok(self)
    }

    /// Remove TopicAlias property
    ///
    /// This method removes any TopicAlias property from the packet properties
    /// while keeping the topic name unchanged. This is useful when you want to
    /// send a packet with the full topic name instead of using a topic alias,
    /// or when converting packets for systems that don't support topic aliases.
    ///
    /// The method automatically recalculates the property_length and remaining_length
    /// to reflect the removal of the TopicAlias property.
    ///
    /// # Returns
    ///
    /// The modified `GenericPublish` instance with TopicAlias property removed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Assume we have a publish packet with both topic name and topic alias
    /// let publish_with_alias = // ... created with topic alias
    ///
    /// let publish_without_alias = publish_with_alias.remove_topic_alias();
    ///
    /// // Topic name remains the same, but TopicAlias property is removed
    /// assert_eq!(publish_without_alias.topic_name(), "sensors/temperature");
    /// // props() no longer contains TopicAlias property
    /// ```
    pub fn remove_topic_alias(mut self) -> Self {
        self.props
            .retain(|prop| !matches!(prop, crate::mqtt::packet::Property::TopicAlias(_)));

        // Recalculate property_length and remaining_length
        self.recalculate_lengths();
        self
    }

    /// Add extracted topic name for TopicAlias resolution
    ///
    /// This method is used during packet reception to restore topic names from
    /// TopicAlias mappings. It's typically called by connection handling code
    /// when a PUBLISH packet is received with a TopicAlias property but no topic name.
    ///
    /// The method requires the current topic name to be empty (as expected when
    /// receiving a packet with only a topic alias) and validates that the resolved
    /// topic name doesn't contain wildcard characters. It keeps the TopicAlias
    /// property intact and marks the topic name as extracted.
    ///
    /// # Parameters
    ///
    /// - `topic`: The resolved topic name from the topic alias mapping
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` - The modified packet with topic name set and extraction flag marked
    /// - `Err(MqttError)` - If the topic name is invalid or the packet state is incorrect
    ///
    /// # Errors
    ///
    /// - `MqttError::TopicNameInvalid` - If the current topic name is not empty
    /// - `MqttError::MalformedPacket` - If the topic contains wildcard characters
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Assume we received a packet with topic alias but no topic name
    /// let publish_with_alias_only = // ... received packet
    ///
    /// let publish_with_extracted_topic = publish_with_alias_only
    ///     .add_extracted_topic_name("sensors/temperature".to_string())
    ///     .unwrap();
    ///
    /// assert_eq!(publish_with_extracted_topic.topic_name(), "sensors/temperature");
    /// assert!(publish_with_extracted_topic.topic_name_extracted());
    /// ```
    pub fn add_extracted_topic_name(mut self, topic: &str) -> Result<Self, MqttError> {
        // Check that topic_name is currently empty
        if !self.topic_name_buf.as_str().is_empty() {
            return Err(MqttError::TopicNameInvalid);
        }

        // Validate topic name (no wildcards allowed in PUBLISH)
        if topic.contains('#') || topic.contains('+') {
            return Err(MqttError::MalformedPacket);
        }

        // Set the topic name
        self.topic_name_buf = MqttString::new(topic)?;

        // Mark that topic name was extracted
        self.topic_name_extracted = true;

        // Recalculate remaining_length (property_length stays the same)
        self.recalculate_lengths();

        Ok(self)
    }

    /// Remove topic name and add TopicAlias property
    ///
    /// This method replaces the topic name with a TopicAlias property, setting the
    /// topic name to an empty string and adding the TopicAlias property to the
    /// packet properties. This is useful for reducing packet size when sending
    /// multiple messages to the same topic.
    ///
    /// The method removes any existing TopicAlias property before adding the new one
    /// to ensure only one TopicAlias property exists. It automatically recalculates
    /// the property_length and remaining_length to reflect these changes.
    ///
    /// # Parameters
    ///
    /// - `topic_alias`: The numeric topic alias to use instead of the topic name
    ///
    /// # Returns
    ///
    /// The modified `GenericPublish` instance with empty topic name and TopicAlias property
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let publish_with_topic = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("sensors/temperature/room1")
    ///     .unwrap()
    ///     .payload(b"23.5")
    ///     .build()
    ///     .unwrap();
    ///
    /// let publish_with_alias = publish_with_topic.remove_topic_add_topic_alias(42);
    ///
    /// assert_eq!(publish_with_alias.topic_name(), "");
    /// // props() now contains TopicAlias property with value 42
    /// ```
    pub fn remove_topic_add_topic_alias(mut self, topic_alias: TopicAliasType) -> Self {
        // Set topic name to empty string
        self.topic_name_buf = MqttString::new("").unwrap();

        // Add TopicAlias property to the end of properties
        let topic_alias_property =
            Property::TopicAlias(crate::mqtt::packet::TopicAlias::new(topic_alias).unwrap());

        // Remove existing TopicAlias property if present
        self.props
            .retain(|prop| !matches!(prop, Property::TopicAlias(_)));
        // Add new TopicAlias property at the end
        self.props.push(topic_alias_property);

        // Recalculate lengths
        self.recalculate_lengths();

        self
    }

    /// Recalculate property_length and remaining_length after modifications
    ///
    /// This internal method recalculates the variable header lengths after
    /// modifications to properties, topic name, or payload. It ensures that
    /// the packet maintains correct length fields for proper serialization.
    ///
    /// The method calculates:
    /// - Property length based on the current properties
    /// - Remaining length including topic name, packet ID (if QoS > 0), properties, and payload
    fn recalculate_lengths(&mut self) {
        // Calculate property length
        let props_size: usize = self.props.size();
        self.property_length = VariableByteInteger::from_u32(props_size as u32).unwrap();

        // Calculate remaining length
        let mut remaining_size = self.topic_name_buf.size();

        // Add packet ID size if QoS > 0
        if self.qos() != crate::mqtt::packet::qos::Qos::AtMostOnce {
            remaining_size += self
                .packet_id_buf
                .as_ref()
                .map_or(0, |_| core::mem::size_of::<PacketIdType>());
        }

        // Add property length size
        remaining_size += self.property_length.size();
        remaining_size += props_size;

        // Add payload size
        remaining_size += self.payload_buf.len();

        self.remaining_length = VariableByteInteger::from_u32(remaining_size as u32).unwrap();
    }

    /// Returns the total size of the PUBLISH packet in bytes
    ///
    /// This includes the fixed header, variable header, and payload.
    /// The size calculation accounts for:
    /// - Fixed header (1 byte)
    /// - Remaining length field (1-4 bytes)
    /// - All variable header and payload data
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
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("test")
    ///     .unwrap()
    ///     .payload(b"hello")
    ///     .build()
    ///     .unwrap();
    ///
    /// let packet_size = publish.size();
    /// println!("PUBLISH packet size: {} bytes", packet_size);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the PUBLISH packet to a vector of I/O slices for efficient transmission
    ///
    /// This method creates a vector of `IoSlice` references that can be used with
    /// vectored I/O operations (like `write_vectored`) for efficient network transmission
    /// without copying the packet data.
    ///
    /// The buffers are arranged in the correct MQTT packet order:
    /// 1. Fixed header
    /// 2. Remaining length
    /// 3. Topic name
    /// 4. Packet identifier (if QoS > 0)
    /// 5. Property length
    /// 6. Properties
    /// 7. Payload
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` references representing the complete packet
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use std::io::Write;
    ///
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("test/topic")
    ///     .unwrap()
    ///     .payload(b"test payload")
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = publish.to_buffers();
    /// // Use with vectored I/O: stream.write_vectored(&buffers)
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
        bufs.push(IoSlice::new(self.property_length.as_bytes()));
        bufs.append(&mut self.props.to_buffers());
        if self.payload_buf.len() > 0 {
            bufs.push(IoSlice::new(self.payload_buf.as_slice()));
        }
        bufs
    }

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is compatible with no-std environments and provides an alternative
    /// to [`to_buffers()`] when vectored I/O is not needed.
    ///
    /// The returned buffer contains the complete PUBLISH packet serialized according
    /// to the MQTT v5.0 protocol specification, including fixed header, remaining
    /// length, topic name, packet identifier, properties, and payload.
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
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("sensors/temperature")
    ///     .qos(mqtt::qos::QualityOfService::AtLeastOnce)
    ///     .packet_id(1u16)
    ///     .payload(b"23.5")
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
        buf.extend_from_slice(self.property_length.as_bytes());
        buf.append(&mut self.props.to_continuous_buffer());
        if self.payload_buf.len() > 0 {
            buf.extend_from_slice(self.payload_buf.as_slice());
        }
        buf
    }

    /// Parses a PUBLISH packet from raw bytes
    ///
    /// This method deserializes PUBLISH packet data from a byte buffer, validating
    /// the packet structure and extracting all components including topic name,
    /// packet identifier, properties, and payload.
    ///
    /// The parsing process:
    /// 1. Validates QoS flags (QoS 3 is invalid)
    /// 2. Extracts topic name
    /// 3. Extracts packet identifier (if QoS > 0)
    /// 4. Parses properties (if present)
    /// 5. Extracts payload (remaining bytes)
    /// 6. Validates property constraints
    ///
    /// # Parameters
    ///
    /// - `flags`: The fixed header flags byte containing QoS, DUP, and RETAIN flags
    /// - `data_arc`: Shared reference to the packet data bytes
    ///
    /// # Returns
    ///
    /// - `Ok((GenericPublish, usize))` - The parsed packet and total bytes consumed
    /// - `Err(MqttError)` - If the packet is malformed or invalid
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - If QoS is 3, insufficient data, or invalid structure
    /// - `MqttError::ProtocolError` - If properties are invalid for PUBLISH packets
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use alloc::sync::Arc;
    ///
    /// let packet_data: Arc<[u8]> = // ... raw packet bytes
    /// let flags = 0x00; // QoS 0, no DUP, no RETAIN
    ///
    /// match mqtt::packet::v5_0::Publish::parse(flags, packet_data) {
    ///     Ok((publish, consumed)) => {
    ///         println!("Parsed PUBLISH: topic={}, payload_len={}",
    ///                  publish.topic_name(), publish.payload().len());
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

        let (topic_name, consumed) = MqttString::decode(&data_arc[cursor..])?;
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

        let (property_length, props) = if cursor < data_arc.len() {
            let (props, consumed) = Properties::parse(&data_arc[cursor..])?;
            cursor += consumed;
            validate_publish_properties(&props)?;
            let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();
            (prop_len, props)
        } else {
            (VariableByteInteger::from_u32(0).unwrap(), Properties::new())
        };

        let payload_len = data_arc.len() - cursor;
        let payload = if payload_len > 0 {
            ArcPayload::new(data_arc.clone(), cursor, payload_len)
        } else {
            ArcPayload::default()
        };

        let remaining_size = topic_name.size()
            + packet_id_buf
                .as_ref()
                .map_or(0, |_| mem::size_of::<PacketIdType>())
            + property_length.size()
            + props.size()
            + payload_len;

        let publish = GenericPublish {
            fixed_header: [fixed_header_byte],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            topic_name_buf: topic_name,
            packet_id_buf,
            property_length,
            props,
            payload_buf: payload,
            topic_name_extracted: false,
        };

        Ok((publish, data_arc.len()))
    }
}

/// Builder implementation for constructing PUBLISH packets
///
/// The builder provides a fluent interface for constructing PUBLISH packets
/// with validation of packet constraints and automatic calculation of lengths.
impl<PacketIdType> GenericPublishBuilder<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Sets the topic name for the PUBLISH packet
    ///
    /// The topic name identifies the information channel to which the payload
    /// is published. Topic names cannot contain wildcard characters (+ or #)
    /// in PUBLISH packets, as these are only allowed in SUBSCRIBE packets.
    ///
    /// # Parameters
    ///
    /// - `topic`: The topic name as any type that can be referenced as a string
    ///
    /// # Returns
    ///
    /// - `Ok(Self)` - The builder with topic name set
    /// - `Err(MqttError)` - If the topic name is invalid
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - If the topic contains wildcard characters
    /// - Other MQTT string validation errors
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("sensors/temperature/room1")
    ///     .unwrap();
    /// ```
    pub fn topic_name<T: AsRef<str>>(mut self, topic: T) -> Result<Self, MqttError> {
        let mqtt_str = MqttString::new(topic)?;
        if mqtt_str.as_str().contains('#') || mqtt_str.as_str().contains('+') {
            return Err(MqttError::MalformedPacket);
        }
        self.topic_name_buf = Some(mqtt_str);
        Ok(self)
    }

    /// Sets the Quality of Service level for the PUBLISH packet
    ///
    /// The QoS level determines the delivery guarantee:
    /// - QoS 0: At most once delivery (no packet identifier required)
    /// - QoS 1: At least once delivery (packet identifier required)
    /// - QoS 2: Exactly once delivery (packet identifier required)
    ///
    /// # Parameters
    ///
    /// - `qos`: The Quality of Service level
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
    /// let builder = mqtt::packet::v5_0::Publish::builder()
    ///     .qos(Qos::AtLeastOnce);
    /// ```
    pub fn qos(mut self, qos: Qos) -> Self {
        let mut header = self.fixed_header.unwrap_or([FixedHeader::Publish as u8]);
        header[0] &= !0b0000_0110; // Clear the QoS bits
        header[0] |= (qos as u8) << 1;
        self.fixed_header = Some(header);
        self
    }

    /// Sets the DUP (duplicate) flag for the PUBLISH packet
    ///
    /// The DUP flag indicates whether this PUBLISH packet is a duplicate of
    /// an earlier packet. This flag is only meaningful for QoS 1 and QoS 2
    /// packets and should be set when retransmitting.
    ///
    /// # Parameters
    ///
    /// - `dup`: Whether this is a duplicate packet transmission
    ///
    /// # Returns
    ///
    /// The builder with DUP flag set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v5_0::Publish::builder()
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
    /// The RETAIN flag indicates whether the server should retain this message
    /// for future subscribers. When set, the server stores the message and
    /// delivers it to any future subscribers to the topic.
    ///
    /// # Parameters
    ///
    /// - `retain`: Whether the message should be retained by the server
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
    /// let builder = mqtt::packet::v5_0::Publish::builder()
    ///     .retain(true);
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
    /// and must be unique within the scope of the client session. It is used
    /// to match acknowledgment packets (PUBACK, PUBREC, etc.) with the original
    /// PUBLISH packet.
    ///
    /// # Parameters
    ///
    /// - `id`: The packet identifier (must be non-zero for QoS > 0)
    ///
    /// # Returns
    ///
    /// The builder with packet identifier set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v5_0::Publish::builder()
    ///     .packet_id(42);
    /// ```
    pub fn packet_id(mut self, id: PacketIdType) -> Self {
        self.packet_id_buf = Some(Some(id.to_buffer()));
        self
    }

    /// Sets the payload data for the PUBLISH packet
    ///
    /// The payload contains the application message data being published.
    /// It can be any binary data that implements `IntoPayload`, including
    /// byte slices, vectors, and strings.
    ///
    /// # Parameters
    ///
    /// - `data`: The payload data that can be converted into an `ArcPayload`
    ///
    /// # Returns
    ///
    /// The builder with payload data set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v5_0::Publish::builder()
    ///     .payload(b"Hello, MQTT!");
    ///
    /// let builder2 = mqtt::packet::v5_0::Publish::builder()
    ///     .payload("Temperature: 23.5 degrees C".to_string());
    /// ```
    pub fn payload<T>(mut self, data: T) -> Self
    where
        T: IntoPayload,
    {
        self.payload_buf = Some(data.into_payload());
        self
    }

    /// Validates the builder state before building the PUBLISH packet
    ///
    /// This method performs comprehensive validation of the packet configuration:
    /// - Validates topic name and TopicAlias property constraints
    /// - Ensures packet identifier requirements match QoS level
    /// - Validates payload size limits
    /// - Validates properties for PUBLISH packet constraints
    ///
    /// # Returns
    ///
    /// - `Ok(())` - If the packet configuration is valid
    /// - `Err(MqttError)` - If any validation constraint is violated
    ///
    /// # Errors
    ///
    /// Various `MqttError` variants depending on the specific validation failure
    fn validate(&self) -> Result<(), MqttError> {
        let property_validation = if let Some(props) = &self.props {
            validate_publish_properties(props)?
        } else {
            PropertyValidation::ValidWithoutTopicAlias
        };
        let has_topic_alias = property_validation == PropertyValidation::ValidWithTopicAlias;

        if self.topic_name_buf.is_none() {
            if !has_topic_alias {
                return Err(MqttError::MalformedPacket);
            }
        } else if let Some(topic) = &self.topic_name_buf {
            if topic.as_str().is_empty() && !has_topic_alias {
                return Err(MqttError::MalformedPacket);
            }
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

        if let Some(props) = &self.props {
            validate_publish_properties(props)?;
        }

        Ok(())
    }

    /// Builds the PUBLISH packet from the current builder state
    ///
    /// This method validates the builder configuration and constructs the final
    /// PUBLISH packet. It automatically calculates all length fields and
    /// organizes the packet components according to MQTT 5.0 specification.
    ///
    /// # Returns
    ///
    /// - `Ok(GenericPublish<PacketIdType>)` - The constructed PUBLISH packet
    /// - `Err(MqttError)` - If validation fails or packet construction is invalid
    ///
    /// # Errors
    ///
    /// Any validation errors from the `validate()` method
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let publish = mqtt::packet::v5_0::Publish::builder()
    ///     .topic_name("sensors/temperature")
    ///     .unwrap()
    ///     .qos(Qos::AtLeastOnce)
    ///     .packet_id(1)
    ///     .payload(b"23.5")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(self) -> Result<GenericPublish<PacketIdType>, MqttError> {
        self.validate()?;

        let topic_name_buf = self.topic_name_buf.unwrap_or(MqttString::new("").unwrap());
        let fixed_header = self.fixed_header.unwrap_or([FixedHeader::Publish as u8]);
        let packet_id_buf = self.packet_id_buf.flatten();
        let props = self.props.unwrap_or(Properties::new());
        let props_size: usize = props.size();
        let property_length = VariableByteInteger::from_u32(props_size as u32).unwrap();
        let payload = self.payload_buf.unwrap_or_else(ArcPayload::default);

        let mut remaining = topic_name_buf.size();
        if (fixed_header[0] >> 1) & 0b0000_0011 != 0 && packet_id_buf.is_some() {
            remaining += mem::size_of::<PacketIdType>();
        }
        remaining += property_length.size() + props_size;
        remaining += payload.len();
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(GenericPublish {
            fixed_header,
            remaining_length,
            topic_name_buf,
            packet_id_buf,
            property_length,
            props,
            payload_buf: payload,
            topic_name_extracted: false,
        })
    }
}

/// Serde serialization implementation for PUBLISH packets
///
/// Provides JSON serialization support for PUBLISH packets, converting
/// all packet fields to a structured JSON representation. Binary payload
/// data is handled appropriately with escape sequences when necessary.
impl<PacketIdType> Serialize for GenericPublish<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    /// Serializes the PUBLISH packet to the given serializer
    ///
    /// Creates a structured representation including packet type, topic name,
    /// QoS level, flags, packet identifier, properties, and payload data.
    /// Binary payload data is escaped appropriately for JSON representation.
    ///
    /// # Parameters
    ///
    /// - `serializer`: The serde serializer to write to
    ///
    /// # Returns
    ///
    /// Serializer result containing the serialized packet data
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = 9; // type, topic_name, qos, retain, dup, extracted, packet_id, props, payload

        let mut state = serializer.serialize_struct("publish", field_count)?;
        state.serialize_field("type", PacketType::Publish.as_str())?;
        state.serialize_field("topic_name", &self.topic_name_buf)?;
        state.serialize_field("qos", &self.qos())?;
        state.serialize_field("retain", &self.retain())?;
        state.serialize_field("dup", &self.dup())?;
        state.serialize_field("extracted", &self.topic_name_extracted())?;
        state.serialize_field("packet_id", &self.packet_id())?;
        state.serialize_field("props", &self.props)?;

        let payload_data = self.payload_buf.as_slice();
        match escape_binary_json_string(payload_data) {
            Some(escaped) => state.serialize_field("payload", &escaped)?,
            None => state.serialize_field("payload", &payload_data)?,
        }

        state.end()
    }
}

/// Display trait implementation for PUBLISH packets
///
/// Provides human-readable JSON string representation of PUBLISH packets
/// using the Serialize implementation.
impl<PacketIdType> fmt::Display for GenericPublish<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    /// Formats the PUBLISH packet as a JSON string
    ///
    /// # Parameters
    ///
    /// - `f`: The formatter to write to
    ///
    /// # Returns
    ///
    /// Formatting result with JSON representation or error message
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Debug trait implementation for PUBLISH packets
///
/// Uses the same JSON representation as Display for consistent debugging output.
impl<PacketIdType> fmt::Debug for GenericPublish<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    /// Formats the PUBLISH packet for debug output
    ///
    /// Delegates to the Display implementation for consistent output format.
    ///
    /// # Parameters
    ///
    /// - `f`: The formatter to write to
    ///
    /// # Returns
    ///
    /// Formatting result with JSON representation
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Generic packet trait implementation for PUBLISH packets
///
/// Provides the common packet interface used by the MQTT protocol implementation
/// for size calculation and buffer generation.
impl<PacketIdType> GenericPacketTrait for GenericPublish<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    /// Returns the total packet size in bytes
    ///
    /// Delegates to the packet's own size method.
    ///
    /// # Returns
    ///
    /// Total packet size including all headers and payload
    fn size(&self) -> usize {
        self.size()
    }

    /// Converts to I/O slices for efficient transmission
    ///
    /// Delegates to the packet's own to_buffers method.
    ///
    /// # Returns
    ///
    /// Vector of I/O slices representing the complete packet
    #[cfg(feature = "std")]
    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }

    fn to_continuous_buffer(&self) -> Vec<u8> {
        self.to_continuous_buffer()
    }
}

/// Generic packet display trait implementation for PUBLISH packets
///
/// Provides the display interface used by the generic packet handling system.
impl<PacketIdType> GenericPacketDisplay for GenericPublish<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    /// Formats the packet for debug display
    ///
    /// # Parameters
    ///
    /// - `f`: The formatter to write to
    ///
    /// # Returns
    ///
    /// Formatting result
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    /// Formats the packet for display
    ///
    /// # Parameters
    ///
    /// - `f`: The formatter to write to
    ///
    /// # Returns
    ///
    /// Formatting result
    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

/// Internal enumeration for property validation results
///
/// Used during PUBLISH packet validation to track whether the packet
/// contains a TopicAlias property, which affects topic name requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyValidation {
    /// Properties are valid and include a TopicAlias property
    ValidWithTopicAlias,
    /// Properties are valid and do not include a TopicAlias property
    ValidWithoutTopicAlias,
}

/// Validates properties for PUBLISH packet compliance
///
/// Ensures that PUBLISH packet properties conform to MQTT 5.0 specification
/// requirements. Validates that:
/// - Only valid properties for PUBLISH packets are present
/// - Each property appears at most once (except UserProperty)
/// - Property values are within valid ranges
///
/// # Parameters
///
/// - `props`: The properties to validate
///
/// # Returns
///
/// - `Ok(PropertyValidation)` - Validation result indicating if TopicAlias is present
/// - `Err(MqttError::ProtocolError)` - If properties are invalid
///
/// # Valid PUBLISH Properties
///
/// - PayloadFormatIndicator (0 or 1 only)
/// - MessageExpiryInterval
/// - TopicAlias (1-65535)
/// - ResponseTopic
/// - CorrelationData
/// - UserProperty (multiple allowed)
/// - SubscriptionIdentifier (server to client only)
/// - ContentType
fn validate_publish_properties(props: &[Property]) -> Result<PropertyValidation, MqttError> {
    let mut count_payload_format = 0;
    let mut count_expiry = 0;
    let mut count_topic_alias = 0;
    let mut count_response_topic = 0;
    let mut count_correlation_data = 0;
    let mut count_content_type = 0;

    for prop in props {
        match prop {
            Property::ContentType(_) => count_content_type += 1,
            Property::CorrelationData(_) => count_correlation_data += 1,
            Property::MessageExpiryInterval(_) => count_expiry += 1,
            Property::PayloadFormatIndicator(_) => count_payload_format += 1,
            Property::ResponseTopic(_) => count_response_topic += 1,
            Property::SubscriptionIdentifier(_) => {}
            Property::TopicAlias(_) => count_topic_alias += 1,
            Property::UserProperty(_) => {}
            _ => return Err(MqttError::ProtocolError),
        }
    }

    if count_payload_format > 1
        || count_expiry > 1
        || count_topic_alias > 1
        || count_response_topic > 1
        || count_correlation_data > 1
        || count_content_type > 1
    {
        return Err(MqttError::ProtocolError);
    }

    if count_topic_alias > 0 {
        Ok(PropertyValidation::ValidWithTopicAlias)
    } else {
        Ok(PropertyValidation::ValidWithoutTopicAlias)
    }
}
