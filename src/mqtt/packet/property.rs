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

use crate::mqtt::packet::escape_binary_json_string;
use crate::mqtt::packet::mqtt_binary::MqttBinary;
use crate::mqtt::packet::mqtt_string::MqttString;
use crate::mqtt::packet::DecodeResult;
use crate::mqtt::packet::VariableByteInteger;
use crate::mqtt::result_code::MqttError;
use alloc::{string::String, vec::Vec};
use core::convert::{TryFrom, TryInto};
use core::fmt;
use num_enum::TryFromPrimitive;
use serde::ser::SerializeStruct;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
#[cfg(feature = "std")]
use std::io::IoSlice;

/// MQTT v5.0 Property Identifiers
///
/// This enum represents all property identifiers defined in the MQTT v5.0 specification.
/// Properties are used to extend MQTT packets with additional metadata and control information.
///
/// Each property has a unique identifier (1-42) and is associated with specific packet types.
/// Properties provide enhanced functionality such as message expiry, user properties,
/// authentication data, and various server capabilities.
///
/// # Specification Reference
///
/// See [MQTT v5.0 Properties](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901029)
/// for detailed information about each property.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let property_id = mqtt::packet::PropertyId::MessageExpiryInterval;
/// assert_eq!(property_id.as_u8(), 2);
/// assert_eq!(property_id.as_str(), "message_expiry_interval");
/// ```
#[derive(Deserialize, PartialEq, Eq, Copy, Clone, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PropertyId {
    /// Indicates the format of the payload in PUBLISH packets (0=binary, 1=UTF-8)
    PayloadFormatIndicator = 1,
    /// Message expiry interval in seconds
    MessageExpiryInterval = 2,
    /// Content type of the application message
    ContentType = 3,
    /// Topic name for response messages
    ResponseTopic = 8,
    /// Correlation data for request/response messaging
    CorrelationData = 9,
    /// Subscription identifier for matching subscriptions
    SubscriptionIdentifier = 11,
    /// Session expiry interval in seconds
    SessionExpiryInterval = 17,
    /// Client identifier assigned by the server
    AssignedClientIdentifier = 18,
    /// Keep alive time assigned by the server
    ServerKeepAlive = 19,
    /// Authentication method name
    AuthenticationMethod = 21,
    /// Authentication data
    AuthenticationData = 22,
    /// Request problem information flag
    RequestProblemInformation = 23,
    /// Will delay interval in seconds
    WillDelayInterval = 24,
    /// Request response information flag
    RequestResponseInformation = 25,
    /// Response information string
    ResponseInformation = 26,
    /// Server reference for redirection
    ServerReference = 28,
    /// Human readable reason string
    ReasonString = 31,
    /// Maximum number of concurrent PUBLISH packets
    ReceiveMaximum = 33,
    /// Maximum topic alias value
    TopicAliasMaximum = 34,
    /// Topic alias value
    TopicAlias = 35,
    /// Maximum QoS level supported
    MaximumQos = 36,
    /// Retain availability flag
    RetainAvailable = 37,
    /// User-defined property key-value pair
    UserProperty = 38,
    /// Maximum packet size
    MaximumPacketSize = 39,
    /// Wildcard subscription availability flag
    WildcardSubscriptionAvailable = 40,
    /// Subscription identifier availability flag
    SubscriptionIdentifierAvailable = 41,
    /// Shared subscription availability flag
    SharedSubscriptionAvailable = 42,
}

impl PropertyId {
    /// Get the numeric identifier of the property
    ///
    /// Returns the property identifier as defined in the MQTT v5.0 specification.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let prop = mqtt::packet::PropertyId::MessageExpiryInterval;
    /// assert_eq!(prop.as_u8(), 2);
    /// ```
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    /// Get the string representation of the property identifier
    ///
    /// Returns a human-readable string name for the property, suitable for
    /// serialization and debugging purposes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let prop = mqtt::packet::PropertyId::ContentType;
    /// assert_eq!(prop.as_str(), "content_type");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            PropertyId::PayloadFormatIndicator => "payload_format_indicator",
            PropertyId::MessageExpiryInterval => "message_expiry_interval",
            PropertyId::ContentType => "content_type",
            PropertyId::ResponseTopic => "response_topic",
            PropertyId::CorrelationData => "correlation_data",
            PropertyId::SubscriptionIdentifier => "subscription_identifier",
            PropertyId::SessionExpiryInterval => "session_expiry_interval",
            PropertyId::AssignedClientIdentifier => "assigned_client_identifier",
            PropertyId::ServerKeepAlive => "server_keep_alive",
            PropertyId::AuthenticationMethod => "authentication_method",
            PropertyId::AuthenticationData => "authentication_data",
            PropertyId::RequestProblemInformation => "request_problem_information",
            PropertyId::WillDelayInterval => "will_delay_interval",
            PropertyId::RequestResponseInformation => "request_response_information",
            PropertyId::ResponseInformation => "response_information",
            PropertyId::ServerReference => "server_reference",
            PropertyId::ReasonString => "reason_string",
            PropertyId::ReceiveMaximum => "receive_maximum",
            PropertyId::TopicAliasMaximum => "topic_alias_maximum",
            PropertyId::TopicAlias => "topic_alias",
            PropertyId::MaximumQos => "maximum_qos",
            PropertyId::RetainAvailable => "retain_available",
            PropertyId::UserProperty => "user_property",
            PropertyId::MaximumPacketSize => "maximum_packet_size",
            PropertyId::WildcardSubscriptionAvailable => "wildcard_subscription_available",
            PropertyId::SubscriptionIdentifierAvailable => "subscription_identifier_available",
            PropertyId::SharedSubscriptionAvailable => "shared_subscription_available",
        }
    }
}

impl Serialize for PropertyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl fmt::Display for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

impl fmt::Debug for PropertyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Payload Format Indicator values
///
/// Specifies the format of the payload in PUBLISH packets.
/// This helps receivers interpret the payload data correctly.
///
/// # Specification Reference
///
/// See [Payload Format Indicator](https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901111)
/// in the MQTT v5.0 specification.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let format = mqtt::packet::PayloadFormat::String;
/// assert_eq!(format as u8, 1);
/// ```
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PayloadFormat {
    /// Payload is unspecified bytes (binary data)
    Binary = 0,
    /// Payload is UTF-8 encoded character data
    String = 1,
}
impl fmt::Display for PayloadFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            PayloadFormat::Binary => "binary",
            PayloadFormat::String => "string",
        };
        write!(f, "{s}")
    }
}

/// Trait for calculating the encoded size of property values
///
/// This trait provides a method to determine how many bytes a property value
/// will occupy when encoded according to the MQTT v5.0 specification.
pub trait PropertySize {
    /// Calculate the encoded size of the property value in bytes
    ///
    /// Returns the number of bytes required to encode this value in the MQTT wire format.
    fn size(&self) -> usize;
}

/// Implementation of PropertySize for u8 values
impl PropertySize for u8 {
    fn size(&self) -> usize {
        1
    }
}

/// Implementation of PropertySize for u16 values (big-endian encoding)
impl PropertySize for u16 {
    fn size(&self) -> usize {
        2
    }
}

/// Implementation of PropertySize for u32 values (big-endian encoding)
impl PropertySize for u32 {
    fn size(&self) -> usize {
        4
    }
}
/// Implementation of PropertySize for String values (UTF-8 string with 2-byte length prefix)
impl PropertySize for String {
    fn size(&self) -> usize {
        2 + self.len()
    }
}

/// Implementation of PropertySize for `Vec<u8>` values (binary data with 2-byte length prefix)
impl PropertySize for Vec<u8> {
    fn size(&self) -> usize {
        2 + self.len()
    }
}

/// Implementation of PropertySize for VariableByteInteger values
/// Variable byte integers use 1-4 bytes depending on the value
impl PropertySize for VariableByteInteger {
    fn size(&self) -> usize {
        match self.to_u32() {
            0..=0x7F => 1,
            0x80..=0x3FFF => 2,
            0x4000..=0x1F_FFFF => 3,
            _ => 4,
        }
    }
}

macro_rules! mqtt_property_common {
    ($name:ident, $id:expr, $ty:ty) => {
        #[derive(Debug, PartialEq, Eq, Clone)]
        pub struct $name {
            id_bytes: [u8; 1],
            value: $ty,
        }

        impl $name {
            /// Returns the PropertyId of this property.
            ///
            /// # Returns
            ///
            /// The PropertyId enum value.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = Property::new(...);
            /// let id = prop.id();
            /// ```
            pub fn id(&self) -> PropertyId {
                $id
            }
        }

        impl From<$name> for Property {
            fn from(v: $name) -> Self {
                Property::$name(v)
            }
        }
    };
}

macro_rules! mqtt_property_binary {
    ($name:ident, $id:expr) => {
        mqtt_property_common!($name, $id, MqttBinary);

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let escaped = escape_binary_json_string(self.val());

                let mut state = serializer.serialize_struct(stringify!($name), 2)?;
                state.serialize_field("id", &($id as u8))?;
                state.serialize_field("val", &escaped)?;
                state.end()
            }
        }

        impl $name {
            /// Creates a new binary property with the given value.
            ///
            /// # Parameters
            ///
            /// * `v` - The binary value to set (can be any type that converts to bytes)
            ///
            /// # Returns
            ///
            /// * `Ok(Self)` - Successfully created property
            /// * `Err(MqttError)` - If the binary data is invalid or too large
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = CorrelationData::new(b"correlation-123").unwrap();
            /// ```
            pub fn new<T>(v: T) -> Result<Self, MqttError>
            where
                T: TryInto<MqttBinary, Error = MqttError>,
            {
                let binary = v.try_into()?;

                Ok(Self {
                    id_bytes: [$id as u8],
                    value: binary,
                })
            }

            /// Parses a binary property from the given byte slice.
            ///
            /// # Parameters
            ///
            /// * `bytes` - The byte slice to parse from
            ///
            /// # Returns
            ///
            /// * `Ok((Self, usize))` - The parsed property and number of bytes consumed
            /// * `Err(MqttError)` - If parsing fails
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let data = &[0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];
            /// let (prop, consumed) = CorrelationData::parse(data).unwrap();
            /// assert_eq!(consumed, 7);
            /// ```
            pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
                let (mqtt_binary, consumed) = MqttBinary::decode(bytes)?;
                Ok((
                    Self {
                        id_bytes: [$id as u8],
                        value: mqtt_binary,
                    },
                    consumed,
                ))
            }

            /// Converts the property to I/O slices for efficient transmission.
            ///
            /// # Returns
            ///
            /// A vector of I/O slices containing the property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = CorrelationData::new(b"data").unwrap();
            /// let buffers = prop.to_buffers();
            /// ```
            #[cfg(feature = "std")]
            pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
                let mut result = vec![IoSlice::new(&self.id_bytes)];
                let mut binary_bufs = self.value.to_buffers();
                result.append(&mut binary_bufs);
                result
            }

            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = CorrelationData::new(b"data").unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = Property::new(...).unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            pub fn to_continuous_buffer(&self) -> Vec<u8> {
                let mut buf = Vec::new();
                buf.extend_from_slice(&self.id_bytes);
                buf.append(&mut self.value.to_continuous_buffer());
                buf
            }

            /// Returns the binary value of this property.
            ///
            /// # Returns
            ///
            /// A reference to the binary data as a byte slice.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = CorrelationData::new(b"hello").unwrap();
            /// assert_eq!(prop.val(), b"hello");
            /// ```
            pub fn val(&self) -> &[u8] {
                self.value.as_slice()
            }

            /// Returns the total size of this property in bytes.
            ///
            /// This includes the property ID (1 byte) plus the binary data size.
            ///
            /// # Returns
            ///
            /// The total size in bytes.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = CorrelationData::new(b"hello").unwrap();
            /// assert_eq!(prop.size(), 8); // 1 (ID) + 2 (length) + 5 (data)
            /// ```
            pub fn size(&self) -> usize {
                1 + self.value.size() // ID + MqttBinary size
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match escape_binary_json_string(self.val()) {
                    Some(escaped) => write!(
                        f,
                        "{{\"id\": \"{}\", \"value\": \"{}\"}}",
                        self.id(),
                        escaped
                    ),
                    None => write!(
                        f,
                        "{{\"id\": \"{}\", \"value\": \"{:?}\"}}",
                        self.id(),
                        self.val()
                    ),
                }
            }
        }
    };
}

macro_rules! mqtt_property_string {
    ($name:ident, $id:expr) => {
        mqtt_property_common!($name, $id, MqttString);

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut s = serializer.serialize_struct(stringify!($name), 2)?;
                s.serialize_field("id", &($id as u8))?;
                s.serialize_field("val", self.val())?;
                s.end()
            }
        }

        impl $name {
            /// Creates a new string property with the given value.
            ///
            /// # Parameters
            ///
            /// * `s` - The string value to set
            ///
            /// # Returns
            ///
            /// * `Ok(Self)` - Successfully created property
            /// * `Err(MqttError)` - If the string is invalid or too long
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ContentType::new("application/json").unwrap();
            /// ```
            pub fn new<T>(s: T) -> Result<Self, MqttError>
            where
                T: TryInto<MqttString, Error = MqttError>,
            {
                let value = s.try_into()?;

                Ok(Self {
                    id_bytes: [$id as u8],
                    value,
                })
            }

            /// Parses a string property from the given byte slice.
            ///
            /// # Parameters
            ///
            /// * `bytes` - The byte slice to parse from
            ///
            /// # Returns
            ///
            /// * `Ok((Self, usize))` - The parsed property and number of bytes consumed
            /// * `Err(MqttError)` - If parsing fails
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let data = &[0x00, 0x05, b'h', b'e', b'l', b'l', b'o'];
            /// let (prop, consumed) = ContentType::parse(data).unwrap();
            /// assert_eq!(consumed, 7);
            /// ```
            pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
                let (mqtt_string, consumed) = MqttString::decode(bytes)?;
                Ok((
                    Self {
                        id_bytes: [$id as u8],
                        value: mqtt_string,
                    },
                    consumed,
                ))
            }

            /// Converts the property to I/O slices for efficient transmission.
            ///
            /// # Returns
            ///
            /// A vector of I/O slices containing the property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ContentType::new("text/plain").unwrap();
            /// let buffers = prop.to_buffers();
            /// ```
            #[cfg(feature = "std")]
            pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
                let mut result = vec![IoSlice::new(&self.id_bytes)];
                let mut string_bufs = self.value.to_buffers();
                result.append(&mut string_bufs);
                result
            }

            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = Property::new(...).unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            pub fn to_continuous_buffer(&self) -> Vec<u8> {
                let mut buf = Vec::new();
                buf.extend_from_slice(&self.id_bytes);
                buf.append(&mut self.value.to_continuous_buffer());
                buf
            }

            /// Returns the string value of this property.
            ///
            /// # Returns
            ///
            /// A reference to the string value.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ContentType::new("application/json").unwrap();
            /// assert_eq!(prop.val(), "application/json");
            /// ```
            pub fn val(&self) -> &str {
                self.value.as_str()
            }

            /// Returns the total size of this property in bytes.
            ///
            /// This includes the property ID (1 byte) plus the string data size.
            ///
            /// # Returns
            ///
            /// The total size in bytes.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ContentType::new("hello").unwrap();
            /// assert_eq!(prop.size(), 8); // 1 (ID) + 2 (length) + 5 (data)
            /// ```
            pub fn size(&self) -> usize {
                1 + self.value.size() // ID + MqttString size
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{{\"id\": \"{}\", \"value\": \"{}\"}}",
                    self.id(),
                    self.val()
                )
            }
        }
    };
}

macro_rules! mqtt_property_string_pair {
    ($name:ident, $id:expr) => {
        mqtt_property_common!($name, $id, (MqttString, MqttString));

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut s = serializer.serialize_struct(stringify!($name), 3)?;
                s.serialize_field("id", &($id as u8))?;
                s.serialize_field("key", self.key())?;
                s.serialize_field("val", self.val())?;
                s.end()
            }
        }

        impl $name {
            /// Creates a new string pair property with the given key and value.
            ///
            /// # Parameters
            ///
            /// * `key` - The key string
            /// * `val` - The value string
            ///
            /// # Returns
            ///
            /// * `Ok(Self)` - Successfully created property
            /// * `Err(MqttError)` - If either string is invalid or too long
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = UserProperty::new("name", "value").unwrap();
            /// ```
            pub fn new<K, V>(key: K, val: V) -> Result<Self, MqttError>
            where
                K: TryInto<MqttString, Error = MqttError>,
                V: TryInto<MqttString, Error = MqttError>,
            {
                let key_mqtt = key.try_into()?;
                let val_mqtt = val.try_into()?;

                Ok(Self {
                    id_bytes: [$id as u8],
                    value: (key_mqtt, val_mqtt),
                })
            }

            /// Parses a string pair property from the given byte slice.
            ///
            /// # Parameters
            ///
            /// * `bytes` - The byte slice to parse from
            ///
            /// # Returns
            ///
            /// * `Ok((Self, usize))` - The parsed property and number of bytes consumed
            /// * `Err(MqttError)` - If parsing fails
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let data = &[0x00, 0x03, b'k', b'e', b'y', 0x00, 0x05, b'v', b'a', b'l', b'u', b'e'];
            /// let (prop, consumed) = UserProperty::parse(data).unwrap();
            /// assert_eq!(consumed, 12);
            /// ```
            pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
                let (key, key_consumed) = MqttString::decode(bytes)?;
                let (val, val_consumed) = MqttString::decode(&bytes[key_consumed..])?;

                Ok((
                    Self {
                        id_bytes: [$id as u8],
                        value: (key, val),
                    },
                    key_consumed + val_consumed,
                ))
            }

            /// Converts the property to I/O slices for efficient transmission.
            ///
            /// # Returns
            ///
            /// A vector of I/O slices containing the property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = UserProperty::new("name", "value").unwrap();
            /// let buffers = prop.to_buffers();
            /// ```
            #[cfg(feature = "std")]
            pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
                let mut result = vec![IoSlice::new(&self.id_bytes)];
                let mut key_bufs = self.value.0.to_buffers();
                let mut val_bufs = self.value.1.to_buffers();

                result.append(&mut key_bufs);
                result.append(&mut val_bufs);
                result
            }

            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = UserProperty::new("key", "value").unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            pub fn to_continuous_buffer(&self) -> Vec<u8> {
                let mut buf = Vec::new();
                buf.extend_from_slice(&self.id_bytes);
                buf.append(&mut self.value.0.to_continuous_buffer());
                buf.append(&mut self.value.1.to_continuous_buffer());
                buf
            }

            /// Returns the key string of this property.
            ///
            /// # Returns
            ///
            /// A reference to the key string.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = UserProperty::new("name", "value").unwrap();
            /// assert_eq!(prop.key(), "name");
            /// ```
            pub fn key(&self) -> &str {
                self.value.0.as_str()
            }

            /// Returns the value string of this property.
            ///
            /// # Returns
            ///
            /// A reference to the value string.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = UserProperty::new("name", "value").unwrap();
            /// assert_eq!(prop.val(), "value");
            /// ```
            pub fn val(&self) -> &str {
                self.value.1.as_str()
            }

            /// Returns the total size of this property in bytes.
            ///
            /// This includes the property ID (1 byte) plus both key and value string sizes.
            ///
            /// # Returns
            ///
            /// The total size in bytes.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = UserProperty::new("key", "value").unwrap();
            /// assert_eq!(prop.size(), 13); // 1 (ID) + 2 (key len) + 3 (key) + 2 (val len) + 5 (val)
            /// ```
            pub fn size(&self) -> usize {
                1 + self.value.0.size() + self.value.1.size() // ID + key size + value size
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{{\"id\": \"{}\", \"key\": \"{}\", \"val\": \"{}\"}}",
                    self.id(),
                    self.key(),
                    self.val()
                )
            }
        }
    };
}

macro_rules! mqtt_property_u8_custom_new {
    ($name:ident, $id:expr, $validator:expr) => {
        mqtt_property_common!($name, $id, [u8; 1]);

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut s = serializer.serialize_struct(stringify!($name), 2)?;
                s.serialize_field("id", &($id as u8))?;
                s.serialize_field("val", &self.val())?;
                s.end()
            }
        }

        impl $name {
            /// Parses a u8 property from the given byte slice.
            ///
            /// # Parameters
            ///
            /// * `bytes` - The byte slice to parse from
            ///
            /// # Returns
            ///
            /// * `Ok((Self, usize))` - The parsed property and number of bytes consumed
            /// * `Err(MqttError)` - If parsing fails or validation fails
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let data = &[42];
            /// let (prop, consumed) = PayloadFormatIndicator::parse(data).unwrap();
            /// assert_eq!(consumed, 1);
            /// ```
            pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
                if bytes.len() < 1 {
                    return Err(MqttError::MalformedPacket);
                }
                if let Some(validator) = $validator {
                    validator(bytes[0])?;
                }
                Ok((
                    Self {
                        id_bytes: [$id as u8],
                        value: [bytes[0]],
                    },
                    1,
                ))
            }

            /// Converts the property to I/O slices for efficient transmission.
            ///
            /// # Returns
            ///
            /// A vector of I/O slices containing the property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = PayloadFormatIndicator::new(1).unwrap();
            /// let buffers = prop.to_buffers();
            /// ```
            #[cfg(feature = "std")]
            pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
                vec![IoSlice::new(&self.id_bytes), IoSlice::new(&self.value)]
            }

            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = Property::new(...).unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            pub fn to_continuous_buffer(&self) -> Vec<u8> {
                let mut buf = Vec::new();
                buf.extend_from_slice(&self.id_bytes);
                buf.extend_from_slice(&self.value);
                buf
            }

            /// Returns the u8 value of this property.
            ///
            /// # Returns
            ///
            /// The u8 value.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = PayloadFormatIndicator::new(1).unwrap();
            /// assert_eq!(prop.val(), 1);
            /// ```
            pub fn val(&self) -> u8 {
                self.value[0]
            }

            /// Returns the total size of this property in bytes.
            ///
            /// This includes the property ID (1 byte) plus the u8 value (1 byte).
            ///
            /// # Returns
            ///
            /// The total size in bytes (always 2 for u8 properties).
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = PayloadFormatIndicator::new(1).unwrap();
            /// assert_eq!(prop.size(), 2);
            /// ```
            pub fn size(&self) -> usize {
                1 + self.value.len()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{{\"id\": \"{}\", \"value\": {}}}",
                    self.id(),
                    self.val()
                )
            }
        }
    };
}

macro_rules! mqtt_property_u8 {
    ($name:ident, $id:expr, $validator:expr) => {
        mqtt_property_u8_custom_new!($name, $id, $validator);

        impl $name {
            /// Creates a new u8 property with the given value.
            ///
            /// # Parameters
            ///
            /// * `v` - The u8 value to set
            ///
            /// # Returns
            ///
            /// * `Ok(Self)` - Successfully created property
            /// * `Err(MqttError)` - If the value fails validation
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = PayloadFormatIndicator::new(1).unwrap();
            /// ```
            pub fn new(v: u8) -> Result<Self, MqttError> {
                if let Some(validator) = $validator {
                    validator(v)?;
                }
                Ok(Self {
                    id_bytes: [$id as u8],
                    value: [v],
                })
            }
        }
    };
}

macro_rules! mqtt_property_u16 {
    ($name:ident, $id:expr, $validator:expr) => {
        mqtt_property_common!($name, $id, [u8; 2]);

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut s = serializer.serialize_struct(stringify!($name), 2)?;
                s.serialize_field("id", &($id as u8))?;
                s.serialize_field("val", &self.val())?;
                s.end()
            }
        }

        impl $name {
            /// Creates a new u16 property with the given value.
            ///
            /// # Parameters
            ///
            /// * `v` - The u16 value to set
            ///
            /// # Returns
            ///
            /// * `Ok(Self)` - Successfully created property
            /// * `Err(MqttError)` - If the value fails validation
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ServerKeepAlive::new(60).unwrap();
            /// ```
            pub fn new(v: u16) -> Result<Self, MqttError> {
                if let Some(validator) = $validator {
                    validator(v)?;
                }
                Ok(Self {
                    id_bytes: [$id as u8],
                    value: v.to_be_bytes(),
                })
            }

            /// Parses a u16 property from the given byte slice.
            ///
            /// # Parameters
            ///
            /// * `bytes` - The byte slice to parse from
            ///
            /// # Returns
            ///
            /// * `Ok((Self, usize))` - The parsed property and number of bytes consumed
            /// * `Err(MqttError)` - If parsing fails or validation fails
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let data = &[0x00, 0x3C]; // 60 in big-endian
            /// let (prop, consumed) = ServerKeepAlive::parse(data).unwrap();
            /// assert_eq!(consumed, 2);
            /// ```
            pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
                if bytes.len() < 2 {
                    return Err(MqttError::MalformedPacket);
                }
                let v = u16::from_be_bytes([bytes[0], bytes[1]]);
                if let Some(validator) = $validator {
                    validator(v)?;
                }
                Ok((
                    Self {
                        id_bytes: [$id as u8],
                        value: bytes[..2].try_into().unwrap(),
                    },
                    2,
                ))
            }

            /// Converts the property to I/O slices for efficient transmission.
            ///
            /// # Returns
            ///
            /// A vector of I/O slices containing the property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ServerKeepAlive::new(60).unwrap();
            /// let buffers = prop.to_buffers();
            /// ```
            #[cfg(feature = "std")]
            pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
                vec![IoSlice::new(&self.id_bytes), IoSlice::new(&self.value)]
            }

            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = Property::new(...).unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            pub fn to_continuous_buffer(&self) -> Vec<u8> {
                let mut buf = Vec::new();
                buf.extend_from_slice(&self.id_bytes);
                buf.extend_from_slice(&self.value);
                buf
            }

            /// Returns the u16 value of this property.
            ///
            /// # Returns
            ///
            /// The u16 value.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ServerKeepAlive::new(60).unwrap();
            /// assert_eq!(prop.val(), 60);
            /// ```
            pub fn val(&self) -> u16 {
                u16::from_be_bytes([self.value[0], self.value[1]])
            }

            /// Returns the total size of this property in bytes.
            ///
            /// This includes the property ID (1 byte) plus the u16 value (2 bytes).
            ///
            /// # Returns
            ///
            /// The total size in bytes (always 3 for u16 properties).
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = ServerKeepAlive::new(60).unwrap();
            /// assert_eq!(prop.size(), 3);
            /// ```
            pub fn size(&self) -> usize {
                1 + self.value.len()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{{\"id\": \"{}\", \"value\": {}}}",
                    self.id(),
                    self.val()
                )
            }
        }
    };
}

macro_rules! mqtt_property_u32 {
    ($name:ident, $id:expr, $validator:expr) => {
        mqtt_property_common!($name, $id, [u8; 4]);

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut s = serializer.serialize_struct(stringify!($name), 2)?;
                s.serialize_field("id", &($id as u8))?;
                s.serialize_field("value", &self.val())?;
                s.end()
            }
        }

        impl $name {
            /// Creates a new u32 property with the given value.
            ///
            /// # Parameters
            ///
            /// * `v` - The u32 value to set
            ///
            /// # Returns
            ///
            /// * `Ok(Self)` - Successfully created property
            /// * `Err(MqttError)` - If the value fails validation
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = MessageExpiryInterval::new(300).unwrap();
            /// ```
            pub fn new(v: u32) -> Result<Self, MqttError> {
                if let Some(validator) = $validator {
                    validator(v)?;
                }
                Ok(Self {
                    id_bytes: [$id as u8],
                    value: v.to_be_bytes(),
                })
            }

            /// Parses a u32 property from the given byte slice.
            ///
            /// # Parameters
            ///
            /// * `bytes` - The byte slice to parse from
            ///
            /// # Returns
            ///
            /// * `Ok((Self, usize))` - The parsed property and number of bytes consumed
            /// * `Err(MqttError)` - If parsing fails or validation fails
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let data = &[0x00, 0x00, 0x01, 0x2C]; // 300 in big-endian
            /// let (prop, consumed) = MessageExpiryInterval::parse(data).unwrap();
            /// assert_eq!(consumed, 4);
            /// ```
            pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
                if bytes.len() < 4 {
                    return Err(MqttError::MalformedPacket);
                }
                let v = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                if let Some(validator) = $validator {
                    validator(v)?;
                }
                Ok((
                    Self {
                        id_bytes: [$id as u8],
                        value: bytes[..4].try_into().unwrap(),
                    },
                    4,
                ))
            }

            /// Converts the property to I/O slices for efficient transmission.
            ///
            /// # Returns
            ///
            /// A vector of I/O slices containing the property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = MessageExpiryInterval::new(300).unwrap();
            /// let buffers = prop.to_buffers();
            /// ```
            #[cfg(feature = "std")]
            pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
                vec![IoSlice::new(&self.id_bytes), IoSlice::new(&self.value)]
            }

            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = Property::new(...).unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            pub fn to_continuous_buffer(&self) -> Vec<u8> {
                let mut buf = Vec::new();
                buf.extend_from_slice(&self.id_bytes);
                buf.extend_from_slice(&self.value);
                buf
            }

            /// Returns the u32 value of this property.
            ///
            /// # Returns
            ///
            /// The u32 value.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = MessageExpiryInterval::new(300).unwrap();
            /// assert_eq!(prop.val(), 300);
            /// ```
            pub fn val(&self) -> u32 {
                u32::from_be_bytes([self.value[0], self.value[1], self.value[2], self.value[3]])
            }

            /// Returns the total size of this property in bytes.
            ///
            /// This includes the property ID (1 byte) plus the u32 value (4 bytes).
            ///
            /// # Returns
            ///
            /// The total size in bytes (always 5 for u32 properties).
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = MessageExpiryInterval::new(300).unwrap();
            /// assert_eq!(prop.size(), 5);
            /// ```
            pub fn size(&self) -> usize {
                1 + self.value.len()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{{\"id\": \"{}\", \"value\": {}}}",
                    self.id(),
                    self.val()
                )
            }
        }
    };
}

macro_rules! mqtt_property_variable_integer {
    ($name:ident, $id:expr, $validator:expr) => {
        mqtt_property_common!($name, $id, VariableByteInteger);

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let mut s = serializer.serialize_struct(stringify!($name), 2)?;
                s.serialize_field("id", &($id as u8))?;
                s.serialize_field("val", &self.val())?;
                s.end()
            }
        }

        impl $name {
            /// Creates a new variable integer property with the given value.
            ///
            /// # Parameters
            ///
            /// * `v` - The u32 value to set (encoded as variable byte integer)
            ///
            /// # Returns
            ///
            /// * `Ok(Self)` - Successfully created property
            /// * `Err(MqttError)` - If the value fails validation or is out of range
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = SubscriptionIdentifier::new(42).unwrap();
            /// ```
            pub fn new(v: u32) -> Result<Self, MqttError> {
                let vbi = VariableByteInteger::from_u32(v).ok_or(MqttError::ValueOutOfRange)?;
                if let Some(validator) = $validator {
                    validator(v)?;
                }
                Ok(Self {
                    id_bytes: [$id as u8],
                    value: vbi,
                })
            }

            /// Parses a variable integer property from the given byte slice.
            ///
            /// # Parameters
            ///
            /// * `bytes` - The byte slice to parse from
            ///
            /// # Returns
            ///
            /// * `Ok((Self, usize))` - The parsed property and number of bytes consumed
            /// * `Err(MqttError)` - If parsing fails or validation fails
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let data = &[0x2A]; // 42 as variable byte integer
            /// let (prop, consumed) = SubscriptionIdentifier::parse(data).unwrap();
            /// assert_eq!(consumed, 1);
            /// ```
            pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
                match VariableByteInteger::decode_stream(bytes) {
                    DecodeResult::Ok(vbi, len) => {
                        if let Some(validator) = $validator {
                            validator(vbi.to_u32())?;
                        }
                        Ok((
                            Self {
                                id_bytes: [$id as u8],
                                value: vbi,
                            },
                            len,
                        ))
                    }
                    DecodeResult::Incomplete => Err(MqttError::InsufficientBytes),
                    DecodeResult::Err(_) => Err(MqttError::InsufficientBytes),
                }
            }

            /// Converts the property to I/O slices for efficient transmission.
            ///
            /// # Returns
            ///
            /// A vector of I/O slices containing the property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = SubscriptionIdentifier::new(42).unwrap();
            /// let buffers = prop.to_buffers();
            /// ```
            #[cfg(feature = "std")]
            pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
                vec![
                    IoSlice::new(&self.id_bytes),
                    IoSlice::new(&self.value.as_bytes()),
                ]
            }

            /// Converts the property to a continuous buffer.
            ///
            /// # Returns
            ///
            /// A byte vector containing the complete property data.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = Property::new(...).unwrap();
            /// let buffer = prop.to_continuous_buffer();
            /// ```
            pub fn to_continuous_buffer(&self) -> Vec<u8> {
                let mut buf = Vec::new();
                buf.extend_from_slice(&self.id_bytes);
                buf.append(&mut self.value.to_continuous_buffer());
                buf
            }

            /// Returns the u32 value of this property.
            ///
            /// # Returns
            ///
            /// The u32 value encoded as a variable byte integer.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = SubscriptionIdentifier::new(42).unwrap();
            /// assert_eq!(prop.val(), 42);
            /// ```
            pub fn val(&self) -> u32 {
                self.value.to_u32()
            }

            /// Returns the total size of this property in bytes.
            ///
            /// This includes the property ID (1 byte) plus the variable byte integer size.
            ///
            /// # Returns
            ///
            /// The total size in bytes.
            ///
            /// # Examples
            ///
            /// ```ignore
            /// let prop = SubscriptionIdentifier::new(42).unwrap();
            /// assert_eq!(prop.size(), 2); // 1 (ID) + 1 (value < 128)
            /// ```
            pub fn size(&self) -> usize {
                1 + self.value.size()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{{\"id\": \"{}\", \"value\": {}}}",
                    self.id(),
                    self.val()
                )
            }
        }
    };
}

type U16Validator = fn(u16) -> Result<(), MqttError>;
type U32Validator = fn(u32) -> Result<(), MqttError>;

mqtt_property_u8_custom_new!(
    PayloadFormatIndicator,
    PropertyId::PayloadFormatIndicator,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
impl PayloadFormatIndicator {
    /// Creates a new PayloadFormatIndicator property.
    ///
    /// # Parameters
    ///
    /// * `v` - The PayloadFormat enum value
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Successfully created property
    /// * `Err(MqttError)` - If creation fails
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let prop = PayloadFormatIndicator::new(PayloadFormat::Utf8String).unwrap();
    /// ```
    pub fn new(v: PayloadFormat) -> Result<Self, MqttError> {
        Ok(Self {
            id_bytes: [PropertyId::PayloadFormatIndicator.as_u8(); 1],
            value: [v as u8],
        })
    }
}

mqtt_property_u32!(
    MessageExpiryInterval,
    PropertyId::MessageExpiryInterval,
    None::<U32Validator>
);
mqtt_property_string!(ContentType, PropertyId::ContentType);
mqtt_property_string!(ResponseTopic, PropertyId::ResponseTopic);
mqtt_property_binary!(CorrelationData, PropertyId::CorrelationData);
mqtt_property_variable_integer!(
    SubscriptionIdentifier,
    PropertyId::SubscriptionIdentifier,
    Some(|v| {
        if v == 0 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u32!(
    SessionExpiryInterval,
    PropertyId::SessionExpiryInterval,
    None::<U32Validator>
);
mqtt_property_string!(
    AssignedClientIdentifier,
    PropertyId::AssignedClientIdentifier
);
mqtt_property_u16!(
    ServerKeepAlive,
    PropertyId::ServerKeepAlive,
    None::<U16Validator>
);
mqtt_property_string!(AuthenticationMethod, PropertyId::AuthenticationMethod);
mqtt_property_binary!(AuthenticationData, PropertyId::AuthenticationData);
mqtt_property_u8!(
    RequestProblemInformation,
    PropertyId::RequestProblemInformation,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u32!(
    WillDelayInterval,
    PropertyId::WillDelayInterval,
    None::<U32Validator>
);
mqtt_property_u8!(
    RequestResponseInformation,
    PropertyId::RequestResponseInformation,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_string!(ResponseInformation, PropertyId::ResponseInformation);
mqtt_property_string!(ServerReference, PropertyId::ServerReference);
mqtt_property_string!(ReasonString, PropertyId::ReasonString);
mqtt_property_u16!(
    ReceiveMaximum,
    PropertyId::ReceiveMaximum,
    Some(|v| {
        if v == 0 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u16!(
    TopicAliasMaximum,
    PropertyId::TopicAliasMaximum,
    None::<U16Validator>
);
mqtt_property_u16!(
    TopicAlias,
    PropertyId::TopicAlias,
    Some(|v| {
        if v == 0 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u8!(
    MaximumQos,
    PropertyId::MaximumQos,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u8!(
    RetainAvailable,
    PropertyId::RetainAvailable,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_string_pair!(UserProperty, PropertyId::UserProperty);
mqtt_property_u32!(
    MaximumPacketSize,
    PropertyId::MaximumPacketSize,
    Some(|v| {
        if v == 0 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u8!(
    WildcardSubscriptionAvailable,
    PropertyId::WildcardSubscriptionAvailable,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u8!(
    SubscriptionIdentifierAvailable,
    PropertyId::SubscriptionIdentifierAvailable,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);
mqtt_property_u8!(
    SharedSubscriptionAvailable,
    PropertyId::SharedSubscriptionAvailable,
    Some(|v| {
        if v > 1 {
            Err(MqttError::ProtocolError)
        } else {
            Ok(())
        }
    })
);

/// MQTT v5.0 Property enum
///
/// This enum represents all possible MQTT v5.0 properties that can be included
/// in various packet types. Each variant wraps a specific property type with
/// its associated data and validation rules.
///
/// Properties provide extensibility to MQTT packets, allowing clients and servers
/// to communicate additional metadata, control flow information, and authentication data.
///
/// # Usage
///
/// Properties are typically collected in a `Vec<Property>` and included in
/// MQTT packets during construction or parsing.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Create a message expiry property
/// let expiry = mqtt::packet::MessageExpiryInterval::new(3600).unwrap();
/// let property = mqtt::packet::Property::MessageExpiryInterval(expiry);
///
/// // Create user property
/// let user_prop = mqtt::packet::UserProperty::new("key", "value").unwrap();
/// let property = mqtt::packet::Property::UserProperty(user_prop);
/// ```
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Property {
    PayloadFormatIndicator(PayloadFormatIndicator),
    MessageExpiryInterval(MessageExpiryInterval),
    ContentType(ContentType),
    ResponseTopic(ResponseTopic),
    CorrelationData(CorrelationData),
    SubscriptionIdentifier(SubscriptionIdentifier),
    SessionExpiryInterval(SessionExpiryInterval),
    AssignedClientIdentifier(AssignedClientIdentifier),
    ServerKeepAlive(ServerKeepAlive),
    AuthenticationMethod(AuthenticationMethod),
    AuthenticationData(AuthenticationData),
    RequestProblemInformation(RequestProblemInformation),
    WillDelayInterval(WillDelayInterval),
    RequestResponseInformation(RequestResponseInformation),
    ResponseInformation(ResponseInformation),
    ServerReference(ServerReference),
    ReasonString(ReasonString),
    ReceiveMaximum(ReceiveMaximum),
    TopicAliasMaximum(TopicAliasMaximum),
    TopicAlias(TopicAlias),
    MaximumQos(MaximumQos),
    RetainAvailable(RetainAvailable),
    UserProperty(UserProperty),
    MaximumPacketSize(MaximumPacketSize),
    WildcardSubscriptionAvailable(WildcardSubscriptionAvailable),
    SubscriptionIdentifierAvailable(SubscriptionIdentifierAvailable),
    SharedSubscriptionAvailable(SharedSubscriptionAvailable),
}

impl fmt::Display for Property {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Property::PayloadFormatIndicator(p) => write!(f, "{p}"),
            Property::MessageExpiryInterval(p) => write!(f, "{p}"),
            Property::ContentType(p) => write!(f, "{p}"),
            Property::ResponseTopic(p) => write!(f, "{p}"),
            Property::CorrelationData(p) => write!(f, "{p}"),
            Property::SubscriptionIdentifier(p) => write!(f, "{p}"),
            Property::SessionExpiryInterval(p) => write!(f, "{p}"),
            Property::AssignedClientIdentifier(p) => write!(f, "{p}"),
            Property::ServerKeepAlive(p) => write!(f, "{p}"),
            Property::AuthenticationMethod(p) => write!(f, "{p}"),
            Property::AuthenticationData(p) => write!(f, "{p}"),
            Property::RequestProblemInformation(p) => write!(f, "{p}"),
            Property::WillDelayInterval(p) => write!(f, "{p}"),
            Property::RequestResponseInformation(p) => write!(f, "{p}"),
            Property::ResponseInformation(p) => write!(f, "{p}"),
            Property::ServerReference(p) => write!(f, "{p}"),
            Property::ReasonString(p) => write!(f, "{p}"),
            Property::ReceiveMaximum(p) => write!(f, "{p}"),
            Property::TopicAliasMaximum(p) => write!(f, "{p}"),
            Property::TopicAlias(p) => write!(f, "{p}"),
            Property::MaximumQos(p) => write!(f, "{p}"),
            Property::RetainAvailable(p) => write!(f, "{p}"),
            Property::UserProperty(p) => write!(f, "{p}"),
            Property::MaximumPacketSize(p) => write!(f, "{p}"),
            Property::WildcardSubscriptionAvailable(p) => write!(f, "{p}"),
            Property::SubscriptionIdentifierAvailable(p) => write!(f, "{p}"),
            Property::SharedSubscriptionAvailable(p) => write!(f, "{p}"),
        }
    }
}

/// Trait for accessing property values in a type-safe manner
///
/// This trait provides methods to extract values from `Property` enum variants
/// without having to match on each variant explicitly. Methods return `Option`
/// to handle cases where the property type doesn't match the requested type.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let prop = mqtt::packet::Property::MessageExpiryInterval(
///     mqtt::packet::MessageExpiryInterval::new(3600).unwrap()
/// );
///
/// // Extract the u32 value
/// if let Some(interval) = prop.as_u32() {
///     println!("Message expires in {} seconds", interval);
/// }
/// ```
pub trait PropertyValueAccess {
    /// Extract u8 value from byte-based properties
    ///
    /// Returns `Some(u8)` for properties that store single-byte values,
    /// `None` for other property types.
    fn as_u8(&self) -> Option<u8>;

    /// Extract u16 value from two-byte properties
    ///
    /// Returns `Some(u16)` for properties that store two-byte values,
    /// `None` for other property types.
    fn as_u16(&self) -> Option<u16>;

    /// Extract u32 value from four-byte properties
    ///
    /// Returns `Some(u32)` for properties that store four-byte values,
    /// `None` for other property types.
    fn as_u32(&self) -> Option<u32>;

    /// Extract string value from string-based properties
    ///
    /// Returns `Some(&str)` for properties that store UTF-8 strings,
    /// `None` for other property types.
    fn as_str(&self) -> Option<&str>;

    /// Extract binary data from binary-based properties
    ///
    /// Returns `Some(&[u8])` for properties that store binary data,
    /// `None` for other property types.
    fn as_bytes(&self) -> Option<&[u8]>;

    /// Extract key-value pair from UserProperty
    ///
    /// Returns `Some((key, value))` for UserProperty, `None` for other property types.
    fn as_key_value(&self) -> Option<(&str, &str)>;
}

impl PropertyValueAccess for Property {
    fn as_u8(&self) -> Option<u8> {
        match self {
            // All property types that return u8
            Property::PayloadFormatIndicator(p) => Some(p.val()),
            Property::MaximumQos(p) => Some(p.val()),
            Property::RetainAvailable(p) => Some(p.val()),
            Property::RequestProblemInformation(p) => Some(p.val()),
            Property::RequestResponseInformation(p) => Some(p.val()),
            Property::WildcardSubscriptionAvailable(p) => Some(p.val()),
            Property::SubscriptionIdentifierAvailable(p) => Some(p.val()),
            Property::SharedSubscriptionAvailable(p) => Some(p.val()),
            _ => None,
        }
    }

    fn as_u16(&self) -> Option<u16> {
        match self {
            // All property types that return u16
            Property::TopicAlias(p) => Some(p.val()),
            Property::ReceiveMaximum(p) => Some(p.val()),
            Property::TopicAliasMaximum(p) => Some(p.val()),
            Property::ServerKeepAlive(p) => Some(p.val()),
            _ => None,
        }
    }

    fn as_u32(&self) -> Option<u32> {
        match self {
            // All property types that return u32
            Property::MessageExpiryInterval(p) => Some(p.val()),
            Property::SessionExpiryInterval(p) => Some(p.val()),
            Property::WillDelayInterval(p) => Some(p.val()),
            Property::MaximumPacketSize(p) => Some(p.val()),
            Property::SubscriptionIdentifier(p) => Some(p.val()),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            // All property types that return strings
            Property::ContentType(p) => Some(p.val()),
            Property::ResponseTopic(p) => Some(p.val()),
            Property::AssignedClientIdentifier(p) => Some(p.val()),
            Property::AuthenticationMethod(p) => Some(p.val()),
            Property::ResponseInformation(p) => Some(p.val()),
            Property::ServerReference(p) => Some(p.val()),
            Property::ReasonString(p) => Some(p.val()),
            _ => None,
        }
    }

    fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            // Property types that return binary data
            Property::CorrelationData(p) => Some(p.val()),
            Property::AuthenticationData(p) => Some(p.val()),
            _ => None,
        }
    }

    fn as_key_value(&self) -> Option<(&str, &str)> {
        match self {
            // Property types that return key-value pairs
            Property::UserProperty(p) => Some((p.key(), p.val())),
            _ => None,
        }
    }
}

impl Property {
    /// Get the property identifier for this property
    ///
    /// Returns the `PropertyId` that corresponds to this property type.
    /// This is useful for determining the property type without matching
    /// on the enum variant.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let prop = mqtt::packet::Property::MessageExpiryInterval(
    ///     mqtt::packet::MessageExpiryInterval::new(3600).unwrap()
    /// );
    /// assert_eq!(prop.id(), mqtt::packet::PropertyId::MessageExpiryInterval);
    /// ```
    pub fn id(&self) -> PropertyId {
        match self {
            Property::PayloadFormatIndicator(p) => p.id(),
            Property::MessageExpiryInterval(p) => p.id(),
            Property::ContentType(p) => p.id(),
            Property::ResponseTopic(p) => p.id(),
            Property::CorrelationData(p) => p.id(),
            Property::SubscriptionIdentifier(p) => p.id(),
            Property::SessionExpiryInterval(p) => p.id(),
            Property::AssignedClientIdentifier(p) => p.id(),
            Property::ServerKeepAlive(p) => p.id(),
            Property::AuthenticationMethod(p) => p.id(),
            Property::AuthenticationData(p) => p.id(),
            Property::RequestProblemInformation(p) => p.id(),
            Property::WillDelayInterval(p) => p.id(),
            Property::RequestResponseInformation(p) => p.id(),
            Property::ResponseInformation(p) => p.id(),
            Property::ServerReference(p) => p.id(),
            Property::ReasonString(p) => p.id(),
            Property::ReceiveMaximum(p) => p.id(),
            Property::TopicAliasMaximum(p) => p.id(),
            Property::TopicAlias(p) => p.id(),
            Property::MaximumQos(p) => p.id(),
            Property::RetainAvailable(p) => p.id(),
            Property::UserProperty(p) => p.id(),
            Property::MaximumPacketSize(p) => p.id(),
            Property::WildcardSubscriptionAvailable(p) => p.id(),
            Property::SubscriptionIdentifierAvailable(p) => p.id(),
            Property::SharedSubscriptionAvailable(p) => p.id(),
        }
    }

    /// Get the encoded size of this property in bytes
    ///
    /// Returns the total number of bytes required to encode this property
    /// in the MQTT wire format, including the property identifier and
    /// any length prefixes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let prop = mqtt::packet::Property::MessageExpiryInterval(
    ///     mqtt::packet::MessageExpiryInterval::new(3600).unwrap()
    /// );
    /// let size = prop.size(); // 1 byte ID + 4 bytes value = 5 bytes
    /// ```
    pub fn size(&self) -> usize {
        match self {
            Property::PayloadFormatIndicator(p) => p.size(),
            Property::MessageExpiryInterval(p) => p.size(),
            Property::ContentType(p) => p.size(),
            Property::ResponseTopic(p) => p.size(),
            Property::CorrelationData(p) => p.size(),
            Property::SubscriptionIdentifier(p) => p.size(),
            Property::SessionExpiryInterval(p) => p.size(),
            Property::AssignedClientIdentifier(p) => p.size(),
            Property::ServerKeepAlive(p) => p.size(),
            Property::AuthenticationMethod(p) => p.size(),
            Property::AuthenticationData(p) => p.size(),
            Property::RequestProblemInformation(p) => p.size(),
            Property::WillDelayInterval(p) => p.size(),
            Property::RequestResponseInformation(p) => p.size(),
            Property::ResponseInformation(p) => p.size(),
            Property::ServerReference(p) => p.size(),
            Property::ReasonString(p) => p.size(),
            Property::ReceiveMaximum(p) => p.size(),
            Property::TopicAliasMaximum(p) => p.size(),
            Property::TopicAlias(p) => p.size(),
            Property::MaximumQos(p) => p.size(),
            Property::RetainAvailable(p) => p.size(),
            Property::UserProperty(p) => p.size(),
            Property::MaximumPacketSize(p) => p.size(),
            Property::WildcardSubscriptionAvailable(p) => p.size(),
            Property::SubscriptionIdentifierAvailable(p) => p.size(),
            Property::SharedSubscriptionAvailable(p) => p.size(),
        }
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// include the property identifier and the encoded property value.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let prop = mqtt::packet::Property::ContentType(
    ///     mqtt::packet::ContentType::new("application/json").unwrap()
    /// );
    /// let buffers = prop.to_buffers();
    /// // Can be used with vectored write operations
    /// // socket.write_vectored(&buffers)?;
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        match self {
            Property::PayloadFormatIndicator(p) => p.to_buffers(),
            Property::MessageExpiryInterval(p) => p.to_buffers(),
            Property::ContentType(p) => p.to_buffers(),
            Property::ResponseTopic(p) => p.to_buffers(),
            Property::CorrelationData(p) => p.to_buffers(),
            Property::SubscriptionIdentifier(p) => p.to_buffers(),
            Property::SessionExpiryInterval(p) => p.to_buffers(),
            Property::AssignedClientIdentifier(p) => p.to_buffers(),
            Property::ServerKeepAlive(p) => p.to_buffers(),
            Property::AuthenticationMethod(p) => p.to_buffers(),
            Property::AuthenticationData(p) => p.to_buffers(),
            Property::RequestProblemInformation(p) => p.to_buffers(),
            Property::WillDelayInterval(p) => p.to_buffers(),
            Property::RequestResponseInformation(p) => p.to_buffers(),
            Property::ResponseInformation(p) => p.to_buffers(),
            Property::ServerReference(p) => p.to_buffers(),
            Property::ReasonString(p) => p.to_buffers(),
            Property::ReceiveMaximum(p) => p.to_buffers(),
            Property::TopicAliasMaximum(p) => p.to_buffers(),
            Property::TopicAlias(p) => p.to_buffers(),
            Property::MaximumQos(p) => p.to_buffers(),
            Property::RetainAvailable(p) => p.to_buffers(),
            Property::UserProperty(p) => p.to_buffers(),
            Property::MaximumPacketSize(p) => p.to_buffers(),
            Property::WildcardSubscriptionAvailable(p) => p.to_buffers(),
            Property::SubscriptionIdentifierAvailable(p) => p.to_buffers(),
            Property::SharedSubscriptionAvailable(p) => p.to_buffers(),
        }
    }

    /// Create a continuous buffer containing the complete property data
    ///
    /// Returns a vector containing all property bytes in a single continuous buffer.
    /// This method is compatible with no-std environments and provides an alternative
    /// to [`to_buffers()`] when vectored I/O is not needed.
    ///
    /// The returned buffer includes the property identifier and the encoded property value.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let prop = mqtt::packet::Property::ContentType(
    ///     mqtt::packet::ContentType::new("application/json").unwrap()
    /// );
    /// let buffer = prop.to_continuous_buffer();
    /// // buffer contains all property bytes
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        match self {
            Property::PayloadFormatIndicator(p) => p.to_continuous_buffer(),
            Property::MessageExpiryInterval(p) => p.to_continuous_buffer(),
            Property::ContentType(p) => p.to_continuous_buffer(),
            Property::ResponseTopic(p) => p.to_continuous_buffer(),
            Property::CorrelationData(p) => p.to_continuous_buffer(),
            Property::SubscriptionIdentifier(p) => p.to_continuous_buffer(),
            Property::SessionExpiryInterval(p) => p.to_continuous_buffer(),
            Property::AssignedClientIdentifier(p) => p.to_continuous_buffer(),
            Property::ServerKeepAlive(p) => p.to_continuous_buffer(),
            Property::AuthenticationMethod(p) => p.to_continuous_buffer(),
            Property::AuthenticationData(p) => p.to_continuous_buffer(),
            Property::RequestProblemInformation(p) => p.to_continuous_buffer(),
            Property::WillDelayInterval(p) => p.to_continuous_buffer(),
            Property::RequestResponseInformation(p) => p.to_continuous_buffer(),
            Property::ResponseInformation(p) => p.to_continuous_buffer(),
            Property::ServerReference(p) => p.to_continuous_buffer(),
            Property::ReasonString(p) => p.to_continuous_buffer(),
            Property::ReceiveMaximum(p) => p.to_continuous_buffer(),
            Property::TopicAliasMaximum(p) => p.to_continuous_buffer(),
            Property::TopicAlias(p) => p.to_continuous_buffer(),
            Property::MaximumQos(p) => p.to_continuous_buffer(),
            Property::RetainAvailable(p) => p.to_continuous_buffer(),
            Property::UserProperty(p) => p.to_continuous_buffer(),
            Property::MaximumPacketSize(p) => p.to_continuous_buffer(),
            Property::WildcardSubscriptionAvailable(p) => p.to_continuous_buffer(),
            Property::SubscriptionIdentifierAvailable(p) => p.to_continuous_buffer(),
            Property::SharedSubscriptionAvailable(p) => p.to_continuous_buffer(),
        }
    }

    /// Parse a property from a byte sequence
    ///
    /// Decodes a single MQTT property from a byte buffer according to the MQTT v5.0
    /// specification. The buffer must start with a property identifier byte followed
    /// by the property value in the appropriate format.
    ///
    /// # Parameters
    ///
    /// * `bytes` - Byte buffer containing the encoded property data
    ///
    /// # Returns
    ///
    /// * `Ok((Property, bytes_consumed))` - Successfully parsed property and number of bytes consumed
    /// * `Err(MqttError::MalformedPacket)` - If the buffer is too short, contains an invalid property ID, or malformed property data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Buffer: [property_id, property_data...]
    /// let buffer = &[0x02, 0x00, 0x00, 0x0E, 0x10]; // MessageExpiryInterval = 3600
    /// let (property, consumed) = mqtt::packet::Property::parse(buffer).unwrap();
    ///
    /// match property {
    ///     mqtt::packet::Property::MessageExpiryInterval(prop) => {
    ///         assert_eq!(prop.val(), 3600);
    ///     }
    ///     _ => panic!("Wrong property type"),
    /// }
    /// assert_eq!(consumed, 5);
    /// ```
    pub fn parse(bytes: &[u8]) -> Result<(Self, usize), MqttError> {
        if bytes.is_empty() {
            return Err(MqttError::MalformedPacket);
        }

        let id = PropertyId::try_from(bytes[0]).map_err(|_| MqttError::MalformedPacket)?;

        let (prop, len) = match id {
            PropertyId::PayloadFormatIndicator => {
                let (p, l) = PayloadFormatIndicator::parse(&bytes[1..])?;
                (Self::PayloadFormatIndicator(p), l + 1)
            }
            PropertyId::MessageExpiryInterval => {
                let (p, l) = MessageExpiryInterval::parse(&bytes[1..])?;
                (Self::MessageExpiryInterval(p), l + 1)
            }
            PropertyId::ContentType => {
                let (p, l) = ContentType::parse(&bytes[1..])?;
                (Self::ContentType(p), l + 1)
            }
            PropertyId::ResponseTopic => {
                let (p, l) = ResponseTopic::parse(&bytes[1..])?;
                (Self::ResponseTopic(p), l + 1)
            }
            PropertyId::CorrelationData => {
                let (p, l) = CorrelationData::parse(&bytes[1..])?;
                (Self::CorrelationData(p), l + 1)
            }
            PropertyId::SubscriptionIdentifier => {
                let (p, l) = SubscriptionIdentifier::parse(&bytes[1..])?;
                (Self::SubscriptionIdentifier(p), l + 1)
            }
            PropertyId::SessionExpiryInterval => {
                let (p, l) = SessionExpiryInterval::parse(&bytes[1..])?;
                (Self::SessionExpiryInterval(p), l + 1)
            }
            PropertyId::AssignedClientIdentifier => {
                let (p, l) = AssignedClientIdentifier::parse(&bytes[1..])?;
                (Self::AssignedClientIdentifier(p), l + 1)
            }
            PropertyId::ServerKeepAlive => {
                let (p, l) = ServerKeepAlive::parse(&bytes[1..])?;
                (Self::ServerKeepAlive(p), l + 1)
            }
            PropertyId::AuthenticationMethod => {
                let (p, l) = AuthenticationMethod::parse(&bytes[1..])?;
                (Self::AuthenticationMethod(p), l + 1)
            }
            PropertyId::AuthenticationData => {
                let (p, l) = AuthenticationData::parse(&bytes[1..])?;
                (Self::AuthenticationData(p), l + 1)
            }
            PropertyId::RequestProblemInformation => {
                let (p, l) = RequestProblemInformation::parse(&bytes[1..])?;
                (Self::RequestProblemInformation(p), l + 1)
            }
            PropertyId::WillDelayInterval => {
                let (p, l) = WillDelayInterval::parse(&bytes[1..])?;
                (Self::WillDelayInterval(p), l + 1)
            }
            PropertyId::RequestResponseInformation => {
                let (p, l) = RequestResponseInformation::parse(&bytes[1..])?;
                (Self::RequestResponseInformation(p), l + 1)
            }
            PropertyId::ResponseInformation => {
                let (p, l) = ResponseInformation::parse(&bytes[1..])?;
                (Self::ResponseInformation(p), l + 1)
            }
            PropertyId::ServerReference => {
                let (p, l) = ServerReference::parse(&bytes[1..])?;
                (Self::ServerReference(p), l + 1)
            }
            PropertyId::ReasonString => {
                let (p, l) = ReasonString::parse(&bytes[1..])?;
                (Self::ReasonString(p), l + 1)
            }
            PropertyId::ReceiveMaximum => {
                let (p, l) = ReceiveMaximum::parse(&bytes[1..])?;
                (Self::ReceiveMaximum(p), l + 1)
            }
            PropertyId::TopicAliasMaximum => {
                let (p, l) = TopicAliasMaximum::parse(&bytes[1..])?;
                (Self::TopicAliasMaximum(p), l + 1)
            }
            PropertyId::TopicAlias => {
                let (p, l) = TopicAlias::parse(&bytes[1..])?;
                (Self::TopicAlias(p), l + 1)
            }
            PropertyId::MaximumQos => {
                let (p, l) = MaximumQos::parse(&bytes[1..])?;
                (Self::MaximumQos(p), l + 1)
            }
            PropertyId::RetainAvailable => {
                let (p, l) = RetainAvailable::parse(&bytes[1..])?;
                (Self::RetainAvailable(p), l + 1)
            }
            PropertyId::UserProperty => {
                let (p, l) = UserProperty::parse(&bytes[1..])?;
                (Self::UserProperty(p), l + 1)
            }
            PropertyId::MaximumPacketSize => {
                let (p, l) = MaximumPacketSize::parse(&bytes[1..])?;
                (Self::MaximumPacketSize(p), l + 1)
            }
            PropertyId::WildcardSubscriptionAvailable => {
                let (p, l) = WildcardSubscriptionAvailable::parse(&bytes[1..])?;
                (Self::WildcardSubscriptionAvailable(p), l + 1)
            }
            PropertyId::SubscriptionIdentifierAvailable => {
                let (p, l) = SubscriptionIdentifierAvailable::parse(&bytes[1..])?;
                (Self::SubscriptionIdentifierAvailable(p), l + 1)
            }
            PropertyId::SharedSubscriptionAvailable => {
                let (p, l) = SharedSubscriptionAvailable::parse(&bytes[1..])?;
                (Self::SharedSubscriptionAvailable(p), l + 1)
            }
        };

        Ok((prop, len))
    }
}

/// Collection of MQTT properties
///
/// This type alias represents a collection of MQTT v5.0 properties that can be
/// included in various packet types. Properties are stored as a vector to preserve
/// order and allow multiple instances of certain property types (like UserProperty).
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let mut properties = mqtt::packet::Properties::new();
///
/// // Add a message expiry interval
/// let expiry = mqtt::packet::MessageExpiryInterval::new(3600).unwrap();
/// properties.push(mqtt::packet::Property::MessageExpiryInterval(expiry));
///
/// // Add user-defined properties
/// let user_prop = mqtt::packet::UserProperty::new("app", "myapp").unwrap();
/// properties.push(mqtt::packet::Property::UserProperty(user_prop));
/// ```
pub type Properties = Vec<Property>;

/// Trait for converting properties collection to continuous buffer
///
/// This trait provides functionality to convert a collection of properties
/// into a single continuous buffer compatible with no-std environments.
pub trait PropertiesToContinuousBuffer {
    /// Convert properties to continuous buffer
    ///
    /// Returns a vector containing all property bytes in a single continuous buffer.
    fn to_continuous_buffer(&self) -> Vec<u8>;
}

/// Trait for converting properties collection to I/O buffers
///
/// This trait provides functionality to convert a collection of properties
/// into IoSlice buffers suitable for efficient network I/O operations.
#[cfg(feature = "std")]
pub trait PropertiesToBuffers {
    /// Convert properties to IoSlice buffers for vectored I/O
    ///
    /// Returns a vector of IoSlice objects that can be used with
    /// vectored write operations for zero-copy network transmission.
    fn to_buffers(&self) -> Vec<IoSlice<'_>>;
}

/// Implementation of PropertiesToContinuousBuffer for Properties
///
/// Concatenates continuous buffers from all properties in the collection.
impl PropertiesToContinuousBuffer for Properties {
    fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut result = Vec::new();

        for prop in self {
            result.append(&mut prop.to_continuous_buffer());
        }

        result
    }
}

/// Implementation of PropertiesToBuffers for Properties
///
/// Concatenates IoSlice buffers from all properties in the collection.
#[cfg(feature = "std")]
impl PropertiesToBuffers for Properties {
    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut result = Vec::new();

        for prop in self {
            result.append(&mut prop.to_buffers());
        }

        result
    }
}

/// Trait for calculating the total encoded size of properties collection
///
/// This trait provides functionality to calculate the total number of bytes
/// required to encode a collection of properties in the MQTT wire format.
pub trait PropertiesSize {
    /// Calculate the total encoded size of all properties in bytes
    ///
    /// Returns the sum of the encoded sizes of all properties in the collection.
    fn size(&self) -> usize;
}

/// Implementation of PropertiesSize for Properties
///
/// Calculates the total size by summing the encoded size of each property.
impl PropertiesSize for Properties {
    fn size(&self) -> usize {
        self.iter().map(|prop| prop.size()).sum()
    }
}

/// Trait for parsing properties collection from byte data
///
/// This trait provides functionality to parse a collection of MQTT properties
/// from a byte buffer according to the MQTT v5.0 specification format.
pub trait PropertiesParse {
    /// Parse properties collection from byte data
    ///
    /// Parses properties from a byte buffer that contains a variable-length integer
    /// indicating the properties length, followed by the encoded properties.
    ///
    /// # Parameters
    ///
    /// * `data` - Byte buffer containing the encoded properties data
    ///
    /// # Returns
    ///
    /// * `Ok((Properties, bytes_consumed))` - Successfully parsed properties and bytes consumed
    /// * `Err(MqttError)` - If the buffer is malformed or contains invalid property data
    fn parse(data: &[u8]) -> Result<(Self, usize), MqttError>
    where
        Self: Sized;
}

/// Implementation of PropertiesParse for Properties
///
/// Parses properties according to MQTT v5.0 specification format:
/// - Variable-length integer indicating properties length
/// - Sequence of encoded properties
impl PropertiesParse for Properties {
    fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        if data.is_empty() {
            return Err(MqttError::MalformedPacket);
        }

        let (prop_len, consumed) = match VariableByteInteger::decode_stream(data) {
            DecodeResult::Ok(vbi, cons) => (vbi, cons),
            _ => return Err(MqttError::MalformedPacket),
        };

        let mut cursor = consumed;
        let mut props = Properties::new();

        if prop_len.to_u32() == 0 {
            return Ok((props, cursor));
        }

        let props_end = cursor + prop_len.to_u32() as usize;
        if props_end > data.len() {
            return Err(MqttError::MalformedPacket);
        }

        while cursor < props_end {
            let (p, c) = Property::parse(&data[cursor..props_end])?;
            props.push(p);
            cursor += c;
        }

        Ok((props, cursor))
    }
}
