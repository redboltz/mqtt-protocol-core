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

use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};

use derive_builder::Builder;
use getset::{CopyGetters, Getters};

use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::json_bin_encode::escape_binary_json_string;
use crate::mqtt::packet::mqtt_binary::MqttBinary;
use crate::mqtt::packet::mqtt_string::MqttString;
use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};

use crate::mqtt::packet::qos::Qos;
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::{
    Properties, PropertiesParse, PropertiesSize, PropertiesToBuffers, Property,
};
use crate::mqtt::result_code::MqttError;

/// MQTT 5.0 CONNECT packet representation
///
/// The CONNECT packet is the first packet sent by a client to the MQTT server (broker)
/// to establish an MQTT connection. It contains information about the client's identity,
/// session preferences, authentication credentials, and last will testament.
///
/// According to MQTT 5.0 specification, the CONNECT packet contains:
/// - Fixed header with packet type and remaining length
/// - Variable header with protocol name, version, connect flags, keep alive, and properties
/// - Payload with client identifier, will properties/topic/payload (if set), username, and password (if set)
///
/// # Protocol Information
///
/// - **Protocol Name**: Always "MQTT" (4 bytes)
/// - **Protocol Version**: 0x05 for MQTT 5.0
///
/// # Connect Flags
///
/// The connect flags byte contains several important flags:
/// - **Bit 0**: Reserved (must be 0)
/// - **Bit 1**: Clean Start flag - if set, the server discards any existing session state
/// - **Bit 2**: Will flag - indicates presence of will message
/// - **Bits 3-4**: Will QoS level (0, 1, or 2)
/// - **Bit 5**: Will Retain flag - if set, will message is retained
/// - **Bit 6**: Password flag - indicates presence of password
/// - **Bit 7**: User Name flag - indicates presence of user name
///
/// # Properties
///
/// MQTT 5.0 CONNECT packets can include various properties:
/// - Session Expiry Interval
/// - Receive Maximum
/// - Maximum Packet Size
/// - Topic Alias Maximum
/// - Request Response Information
/// - Request Problem Information
/// - User Properties
/// - Authentication Method
/// - Authentication Data
///
/// # Will Properties
///
/// If a will message is present, it can include its own properties:
/// - Will Delay Interval
/// - Payload Format Indicator
/// - Message Expiry Interval
/// - Content Type
/// - Response Topic
/// - Correlation Data
/// - User Properties
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
///
/// // Create a basic CONNECT packet with client ID
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("my-client-123")
///     .clean_start(true)
///     .build()
///     .unwrap();
///
/// assert_eq!(connect.client_id(), "my-client-123");
/// assert!(connect.clean_start());
/// assert_eq!(connect.protocol_version(), 5);
///
/// // Create CONNECT with will message and credentials
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("client-with-will")
///     .clean_start(false)
///     .will_message("my/status", b"offline", Qos::AtLeastOnce, true)
///     .user_name("username")
///     .password(b"password")
///     .keep_alive(60)
///     .build()
///     .unwrap();
///
/// assert_eq!(connect.will_topic(), Some("my/status"));
/// assert_eq!(connect.will_payload(), Some(b"offline".as_slice()));
/// assert_eq!(connect.will_qos(), Qos::AtLeastOnce);
/// assert!(connect.will_retain());
/// assert_eq!(connect.user_name(), Some("username"));
/// assert_eq!(connect.keep_alive(), 60);
///
/// // Serialize to bytes for network transmission
/// let buffers = connect.to_buffers();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct Connect {
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    protocol_name: [u8; 6],
    #[builder(private)]
    protocol_version_buf: [u8; 1],
    #[builder(private)]
    connect_flags_buf: [u8; 1],
    #[builder(private)]
    keep_alive_buf: [u8; 2],
    #[builder(private)]
    property_length: VariableByteInteger,

    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Properties,

    #[builder(private)]
    client_id_buf: MqttString,

    #[builder(private)]
    will_property_length: VariableByteInteger,
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub will_props: Properties,
    #[builder(private)]
    will_topic_buf: MqttString,
    #[builder(private)]
    will_payload_buf: MqttBinary,

    #[builder(private)]
    user_name_buf: MqttString,
    #[builder(private)]
    password_buf: MqttBinary,
}

impl Connect {
    /// Creates a new builder for constructing a CONNECT packet
    ///
    /// # Returns
    ///
    /// A `ConnectBuilder` instance with default values
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("my-client")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> ConnectBuilder {
        ConnectBuilder::default()
    }

    /// Returns the packet type for CONNECT packets
    ///
    /// # Returns
    ///
    /// `PacketType::Connect` (value 1)
    pub fn packet_type() -> PacketType {
        PacketType::Connect
    }

    /// Returns the MQTT protocol name
    ///
    /// For MQTT 5.0, this is always "MQTT"
    ///
    /// # Returns
    ///
    /// The protocol name as a string slice
    pub fn protocol_name(&self) -> &str {
        "MQTT"
    }

    /// Returns the MQTT protocol version
    ///
    /// For MQTT 5.0, this is always 5 (0x05)
    ///
    /// # Returns
    ///
    /// The protocol version number
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version_buf[0]
    }

    /// Returns the Clean Start flag value
    ///
    /// When true, the server discards any existing session state for this client
    /// and starts a new clean session. When false, the server attempts to resume
    /// an existing session if one exists.
    ///
    /// # Returns
    ///
    /// `true` if Clean Start is enabled, `false` otherwise
    pub fn clean_start(&self) -> bool {
        (self.connect_flags_buf[0] & 0b0000_0010) != 0
    }

    /// Returns whether a will message is present
    ///
    /// The will message is published by the server when the client disconnects
    /// unexpectedly or fails to send keep-alive messages.
    ///
    /// # Returns
    ///
    /// `true` if a will message is configured, `false` otherwise
    pub fn will_flag(&self) -> bool {
        (self.connect_flags_buf[0] & 0b0000_0100) != 0
    }

    /// Returns the QoS level for the will message
    ///
    /// The will QoS specifies the quality of service level to use when
    /// publishing the will message. Only valid when `will_flag()` returns true.
    ///
    /// # Returns
    ///
    /// The QoS level (`AtMostOnce`, `AtLeastOnce`, or `ExactlyOnce`)
    pub fn will_qos(&self) -> Qos {
        let qos_bits = (self.connect_flags_buf[0] >> 3) & 0x03;
        Qos::try_from(qos_bits).unwrap_or(Qos::AtMostOnce)
    }

    /// Returns whether the will message should be retained
    ///
    /// When true, the will message is stored by the server and delivered
    /// to new subscribers on the will topic. Only valid when `will_flag()` returns true.
    ///
    /// # Returns
    ///
    /// `true` if the will message should be retained, `false` otherwise
    pub fn will_retain(&self) -> bool {
        (self.connect_flags_buf[0] & 0b0010_0000) != 0
    }

    /// Returns whether a password is present
    ///
    /// According to MQTT specification, if password flag is set,
    /// the user name flag must also be set.
    ///
    /// # Returns
    ///
    /// `true` if a password is included, `false` otherwise
    pub fn password_flag(&self) -> bool {
        (self.connect_flags_buf[0] & 0b0100_0000) != 0
    }

    /// Returns whether a user name is present
    ///
    /// # Returns
    ///
    /// `true` if a user name is included, `false` otherwise
    pub fn user_name_flag(&self) -> bool {
        (self.connect_flags_buf[0] & 0b1000_0000) != 0
    }

    /// Returns the keep alive interval in seconds
    ///
    /// The keep alive interval specifies the maximum time interval between
    /// control packets sent by the client. A value of 0 disables keep alive.
    ///
    /// # Returns
    ///
    /// Keep alive interval in seconds
    pub fn keep_alive(&self) -> u16 {
        u16::from_be_bytes(self.keep_alive_buf)
    }

    /// Returns the client identifier
    ///
    /// The client identifier uniquely identifies the client to the server.
    /// It must be unique across all clients connected to the server.
    ///
    /// # Returns
    ///
    /// The client identifier as a string slice
    pub fn client_id(&self) -> &str {
        self.client_id_buf.as_str()
    }

    /// Returns the will topic if a will message is configured
    ///
    /// The will topic specifies where the will message should be published
    /// when the client disconnects unexpectedly.
    ///
    /// # Returns
    ///
    /// `Some(&str)` containing the will topic if will flag is set, `None` otherwise
    pub fn will_topic(&self) -> Option<&str> {
        if self.will_flag() {
            Some(self.will_topic_buf.as_str())
        } else {
            None
        }
    }

    /// Returns the will message payload if a will message is configured
    ///
    /// The will payload contains the message content to be published
    /// when the client disconnects unexpectedly.
    ///
    /// # Returns
    ///
    /// `Some(&[u8])` containing the will payload if will flag is set, `None` otherwise
    pub fn will_payload(&self) -> Option<&[u8]> {
        if self.will_flag() {
            Some(self.will_payload_buf.as_slice())
        } else {
            None
        }
    }

    /// Returns the user name if present
    ///
    /// The user name is used for authentication with the MQTT server.
    ///
    /// # Returns
    ///
    /// `Some(&str)` containing the user name if user name flag is set, `None` otherwise
    pub fn user_name(&self) -> Option<&str> {
        if self.user_name_flag() {
            Some(self.user_name_buf.as_str())
        } else {
            None
        }
    }

    /// Returns the password if present
    ///
    /// The password is used for authentication with the MQTT server.
    /// According to the MQTT specification, password can only be present
    /// if user name is also present.
    ///
    /// # Returns
    ///
    /// `Some(&[u8])` containing the password if password flag is set, `None` otherwise
    pub fn password(&self) -> Option<&[u8]> {
        if self.password_flag() {
            Some(self.password_buf.as_slice())
        } else {
            None
        }
    }

    /// Returns the total size of the packet in bytes
    ///
    /// This includes the fixed header, variable header, and payload.
    ///
    /// # Returns
    ///
    /// Total packet size in bytes
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the packet to a vector of I/O slices for efficient network transmission
    ///
    /// This method creates a zero-copy representation of the packet by returning
    /// references to the internal buffers. This is more efficient than copying
    /// all data into a single buffer.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` references that can be used with vectored I/O operations
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("client-123")
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = connect.to_buffers();
    /// // Use buffers for vectored write operations
    /// ```
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(&self.protocol_name));
        bufs.push(IoSlice::new(&self.protocol_version_buf));
        bufs.push(IoSlice::new(&self.connect_flags_buf));
        bufs.push(IoSlice::new(&self.keep_alive_buf));
        bufs.push(IoSlice::new(self.property_length.as_bytes()));
        bufs.extend(self.props.to_buffers());

        bufs.extend(self.client_id_buf.to_buffers());

        if self.will_flag() {
            bufs.push(IoSlice::new(self.will_property_length.as_bytes()));
            bufs.extend(self.will_props.to_buffers());
            bufs.extend(self.will_topic_buf.to_buffers());
            bufs.extend(self.will_payload_buf.to_buffers());
        }

        if self.user_name_flag() {
            bufs.extend(self.user_name_buf.to_buffers());
        }

        if self.password_flag() {
            bufs.extend(self.password_buf.to_buffers());
        }

        bufs
    }

    /// Parses a CONNECT packet from raw bytes
    ///
    /// This method parses the variable header and payload of a CONNECT packet
    /// according to the MQTT 5.0 specification. It validates the protocol name,
    /// version, and flag consistency.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes containing the variable header and payload (excluding fixed header)
    ///
    /// # Returns
    ///
    /// * `Ok((Connect, usize))` - The parsed packet and number of bytes consumed
    /// * `Err(MqttError)` - If parsing fails due to malformed data or protocol violations
    ///
    /// # Errors
    ///
    /// * `MqttError::MalformedPacket` - Invalid packet structure
    /// * `MqttError::ProtocolError` - Protocol specification violation
    /// * `MqttError::UnsupportedProtocolVersion` - Wrong protocol version
    /// * `MqttError::ClientIdentifierNotValid` - Invalid client identifier
    /// * `MqttError::BadUserNameOrPassword` - Invalid credentials
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Assuming 'packet_data' contains a valid CONNECT packet payload
    /// match mqtt::packet::v5_0::Connect::parse(&packet_data) {
    ///     Ok((connect, bytes_consumed)) => {
    ///         println!("Client ID: {}", connect.client_id());
    ///         println!("Consumed {} bytes", bytes_consumed);
    ///     }
    ///     Err(e) => println!("Parse error: {:?}", e),
    /// }
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;

        // Protocol Name (should be "MQTT")
        if data.len() < cursor + 6 {
            return Err(MqttError::MalformedPacket);
        }
        let protocol_name = [
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
            data[cursor + 4],
            data[cursor + 5],
        ];
        cursor += 6;
        if &protocol_name[2..] != b"MQTT" || protocol_name[0] != 0x00 || protocol_name[1] != 0x04 {
            return Err(MqttError::ProtocolError);
        }

        // Protocol Version
        if data.len() < cursor + 1 {
            return Err(MqttError::MalformedPacket);
        }
        let protocol_version = data[cursor];
        let protocol_version_buf = [protocol_version];
        cursor += 1;
        if protocol_version != 0x05 {
            return Err(MqttError::UnsupportedProtocolVersion);
        }

        // Connect Flags
        if data.len() < cursor + 1 {
            return Err(MqttError::MalformedPacket);
        }
        let connect_flags = data[cursor];
        let connect_flags_buf = [connect_flags];
        cursor += 1;

        // Keep Alive
        if data.len() < cursor + 2 {
            return Err(MqttError::MalformedPacket);
        }
        let keep_alive_buf = [data[cursor], data[cursor + 1]];
        cursor += 2;

        // Properties
        let (props, consumed) = Properties::parse(&data[cursor..])?;
        cursor += consumed;
        validate_connect_properties(&props)?;
        let property_length = VariableByteInteger::from_u32(props.size() as u32).unwrap();

        // Client Identifier
        let (client_id_buf, consumed) =
            MqttString::decode(&data[cursor..]).map_err(|_| MqttError::ClientIdentifierNotValid)?;
        cursor += consumed;

        // Will Properties and Messages (if will flag is set)
        let will_flag = (connect_flags & 0b0000_0100) != 0;
        let mut will_props = Properties::new();
        let mut will_property_length = VariableByteInteger::from_u32(0).unwrap();
        let mut will_topic_buf = MqttString::default();
        let mut will_payload_buf = MqttBinary::default();

        if will_flag {
            // Will Properties
            let (w_props, consumed) = Properties::parse(&data[cursor..])?;
            cursor += consumed;
            validate_will_properties(&w_props)?;
            will_props = w_props;
            will_property_length = VariableByteInteger::from_u32(will_props.size() as u32).unwrap();

            // Will Topic
            let (w_topic, consumed) = MqttString::decode(&data[cursor..])?;
            cursor += consumed;
            will_topic_buf = w_topic;

            // Will Payload
            let (w_payload, consumed) = MqttBinary::decode(&data[cursor..])?;
            cursor += consumed;
            will_payload_buf = w_payload;
        }

        // User Name (if user name flag is set)
        let user_name_flag = (connect_flags & 0b1000_0000) != 0;
        let mut user_name_buf = MqttString::default();
        if user_name_flag {
            let (uname, consumed) = MqttString::decode(&data[cursor..])
                .map_err(|_| MqttError::BadUserNameOrPassword)?;
            cursor += consumed;
            user_name_buf = uname;
        }

        // Password (if password flag is set)
        let password_flag = (connect_flags & 0b0100_0000) != 0;
        let mut password_buf = MqttBinary::default();
        if password_flag {
            let (pwd, consumed) = MqttBinary::decode(&data[cursor..])
                .map_err(|_| MqttError::BadUserNameOrPassword)?;
            cursor += consumed;
            password_buf = pwd;
        }

        // Validate flags consistency
        if password_flag && !user_name_flag {
            return Err(MqttError::ProtocolError);
        }

        let connect = Connect {
            fixed_header: [FixedHeader::Connect as u8],
            remaining_length: VariableByteInteger::from_u32(cursor as u32).unwrap(),
            protocol_name,
            protocol_version_buf,
            connect_flags_buf,
            keep_alive_buf,
            property_length,
            props,
            client_id_buf,
            will_property_length,
            will_props,
            will_topic_buf,
            will_payload_buf,
            user_name_buf,
            password_buf,
        };

        Ok((connect, cursor))
    }
}

/// Builder for constructing MQTT 5.0 CONNECT packets
///
/// The `ConnectBuilder` provides a fluent interface for creating `Connect` packets
/// with proper validation and default values. It handles the complex MQTT protocol
/// requirements and flag dependencies automatically.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
///
/// // Basic connection
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("my-client")
///     .build()
///     .unwrap();
///
/// // Connection with all features
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("full-client")
///     .clean_start(false)
///     .keep_alive(30)
///     .will_message("status/offline", b"client disconnected", Qos::AtLeastOnce, true)
///     .user_name("user123")
///     .password(b"secret")
///     .build()
///     .unwrap();
/// ```
impl ConnectBuilder {
    /// Sets the client identifier
    ///
    /// The client identifier uniquely identifies the client to the server.
    /// It must be a valid UTF-8 string and should be unique across all clients.
    ///
    /// # Arguments
    ///
    /// * `id` - The client identifier string
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with client ID set
    /// * `Err(MqttError)` - If the client ID is invalid (e.g., too long)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("unique-client-123")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn client_id(mut self, id: impl AsRef<str>) -> Result<Self, MqttError> {
        let mqtt_str = MqttString::new(id.as_ref())?;
        self.client_id_buf = Some(mqtt_str);
        Ok(self)
    }

    /// Sets the Clean Start flag
    ///
    /// When Clean Start is true, the server discards any existing session state
    /// for this client and starts a new clean session. When false, the server
    /// attempts to resume an existing session if one exists.
    ///
    /// # Arguments
    ///
    /// * `clean` - Whether to start a clean session
    ///
    /// # Returns
    ///
    /// Builder with Clean Start flag set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Clean session (default)
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("client")
    ///     .clean_start(true)
    ///     .build()
    ///     .unwrap();
    ///
    /// // Resume existing session
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("client")
    ///     .clean_start(false)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn clean_start(mut self, clean: bool) -> Self {
        let mut flags = self.connect_flags_buf.unwrap_or([0])[0];
        if clean {
            flags |= 0b0000_0010;
        } else {
            flags &= !0b0000_0010;
        }
        self.connect_flags_buf = Some([flags]);
        self
    }

    /// Sets the will message (Last Will and Testament)
    ///
    /// The will message is published by the server when the client disconnects
    /// unexpectedly or fails to send keep-alive messages. All parameters must
    /// be provided together.
    ///
    /// # Arguments
    ///
    /// * `topic` - Topic where the will message should be published
    /// * `payload` - Will message payload bytes
    /// * `qos` - Quality of Service level for the will message
    /// * `retain` - Whether the will message should be retained
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with will message configured
    /// * `Err(MqttError)` - If topic or payload is invalid
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("client-with-will")
    ///     .will_message("device/status", b"offline", Qos::AtLeastOnce, true)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn will_message(
        mut self,
        topic: impl AsRef<str>,
        payload: impl AsRef<[u8]>,
        qos: Qos,
        retain: bool,
    ) -> Result<Self, MqttError> {
        let will_topic = MqttString::new(topic.as_ref())?;
        let will_payload = MqttBinary::new(payload.as_ref().to_vec())?;

        self.will_topic_buf = Some(will_topic);
        self.will_payload_buf = Some(will_payload);

        let mut flags = self.connect_flags_buf.unwrap_or([0b0000_0010])[0];
        flags |= 0b0000_0100; // Will flag
        flags |= (qos as u8) << 3; // Will QoS
        if retain {
            flags |= 0b0010_0000; // Will retain
        }
        self.connect_flags_buf = Some([flags]);
        Ok(self)
    }

    /// Sets the user name for authentication
    ///
    /// The user name is used for client authentication with the MQTT server.
    /// Setting a user name automatically sets the user name flag in the connect flags.
    ///
    /// # Arguments
    ///
    /// * `name` - The user name string
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with user name set
    /// * `Err(MqttError)` - If the user name is invalid (e.g., too long)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("authenticated-client")
    ///     .user_name("my_username")
    ///     .password(b"my_password")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn user_name(mut self, name: impl AsRef<str>) -> Result<Self, MqttError> {
        let mqtt_str = MqttString::new(name.as_ref())?;
        self.user_name_buf = Some(mqtt_str);

        let mut flags = self.connect_flags_buf.unwrap_or([0b0000_0010])[0];
        flags |= 0b1000_0000; // User name flag
        self.connect_flags_buf = Some([flags]);
        Ok(self)
    }

    /// Sets the password for authentication
    ///
    /// The password is used for client authentication with the MQTT server.
    /// According to MQTT specification, a password can only be set if a user name
    /// is also provided. Setting a password automatically sets the password flag.
    ///
    /// # Arguments
    ///
    /// * `pwd` - The password bytes
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with password set
    /// * `Err(MqttError)` - If the password is invalid (e.g., too long)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("secure-client")
    ///     .user_name("username")
    ///     .password(b"secure_password_123")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn password(mut self, pwd: impl AsRef<[u8]>) -> Result<Self, MqttError> {
        let mqtt_bin = MqttBinary::new(pwd.as_ref().to_vec())?;
        self.password_buf = Some(mqtt_bin);

        let mut flags = self.connect_flags_buf.unwrap_or([0b0000_0010])[0];
        flags |= 0b0100_0000; // Password flag
        self.connect_flags_buf = Some([flags]);
        Ok(self)
    }

    /// Sets the keep alive interval in seconds
    ///
    /// The keep alive interval specifies the maximum time between control packets
    /// sent by the client. If set to 0, keep alive is disabled. The server may
    /// disconnect the client if it doesn't receive any packets within 1.5 times
    /// the keep alive interval.
    ///
    /// # Arguments
    ///
    /// * `seconds` - Keep alive interval in seconds (0 to disable)
    ///
    /// # Returns
    ///
    /// Builder with keep alive interval set
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // 60 second keep alive
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("client-with-keepalive")
    ///     .keep_alive(60)
    ///     .build()
    ///     .unwrap();
    ///
    /// // Disable keep alive
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("client-no-keepalive")
    ///     .keep_alive(0)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn keep_alive(mut self, seconds: u16) -> Self {
        self.keep_alive_buf = Some(seconds.to_be_bytes());
        self
    }

    /// Validates the builder state for MQTT protocol compliance
    ///
    /// This method checks various MQTT protocol requirements:
    /// - Password flag can only be set if user name flag is also set
    /// - Will message components must be consistent
    /// - Properties must be valid for CONNECT packets
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all validations pass
    /// * `Err(MqttError)` - If any validation fails
    ///
    /// # Errors
    ///
    /// * `MqttError::ProtocolError` - MQTT protocol violation
    /// * `MqttError::MalformedPacket` - Inconsistent packet structure
    fn validate(&self) -> Result<(), MqttError> {
        let flags = self.connect_flags_buf.unwrap_or([0b0000_0010])[0];

        // Password flag set but user name flag not set
        if (flags & 0b0100_0000) != 0 && (flags & 0b1000_0000) == 0 {
            return Err(MqttError::ProtocolError);
        }

        // Will flag validation
        let will_flag = (flags & 0b0000_0100) != 0;
        if will_flag {
            // As long as using will_message() setter, they both must be set
            if self.will_topic_buf.is_none() || self.will_payload_buf.is_none() {
                return Err(MqttError::MalformedPacket);
            }
        }

        if let Some(ref props) = self.props {
            validate_connect_properties(props)?;
        }

        if let Some(ref will_props) = self.will_props {
            validate_will_properties(will_props)?;
        }

        Ok(())
    }

    /// Builds the final CONNECT packet
    ///
    /// Validates all fields and constructs the complete CONNECT packet with
    /// properly calculated lengths and flags. This method consumes the builder.
    ///
    /// # Returns
    ///
    /// * `Ok(Connect)` - The constructed CONNECT packet
    /// * `Err(MqttError)` - If validation fails or packet construction fails
    ///
    /// # Errors
    ///
    /// * `MqttError::ProtocolError` - MQTT protocol violation
    /// * `MqttError::MalformedPacket` - Invalid packet structure
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::packet::qos::Qos;
    ///
    /// let connect = mqtt::packet::v5_0::Connect::builder()
    ///     .client_id("example-client")
    ///     .clean_start(true)
    ///     .keep_alive(60)
    ///     .will_message("status", b"offline", Qos::AtMostOnce, false)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(connect.client_id(), "example-client");
    /// assert_eq!(connect.keep_alive(), 60);
    /// ```
    pub fn build(self) -> Result<Connect, MqttError> {
        self.validate()?;

        let protocol_name = [0x00, 0x04, b'M', b'Q', b'T', b'T'];
        let protocol_version_buf = [0x05];
        let connect_flags_buf = self.connect_flags_buf.unwrap_or([0b0000_0010]); // Default: clean_start = true
        let connect_flags = connect_flags_buf[0];
        let keep_alive_buf = self.keep_alive_buf.unwrap_or([0, 0]);
        let props = self.props.unwrap_or_else(Properties::new);
        let property_length = VariableByteInteger::from_u32(props.size() as u32).unwrap();

        let client_id_buf = self.client_id_buf.unwrap_or_default();

        let will_flag = (connect_flags & 0b0000_0100) != 0;
        let will_props = self.will_props.unwrap_or_else(Properties::new);
        let will_property_length = VariableByteInteger::from_u32(will_props.size() as u32).unwrap();
        let will_topic_buf = self.will_topic_buf.unwrap_or_default();
        let will_payload_buf = self.will_payload_buf.unwrap_or_default();

        let user_name_buf = self.user_name_buf.unwrap_or_default();
        let password_buf = self.password_buf.unwrap_or_default();

        // Calculate remaining length
        let mut remaining = 0;
        remaining += 6; // protocol name
        remaining += 1; // protocol version
        remaining += 1; // connect flags
        remaining += 2; // keep alive
        remaining += property_length.size() + props.size(); // properties
        remaining += client_id_buf.size(); // client identifier

        if will_flag {
            remaining += will_property_length.size() + will_props.size(); // will properties
            remaining += will_topic_buf.size(); // will topic
            remaining += will_payload_buf.size(); // will payload
        }

        if (connect_flags & 0b1000_0000) != 0 {
            remaining += user_name_buf.size(); // user name
        }

        if (connect_flags & 0b0100_0000) != 0 {
            remaining += password_buf.size(); // password
        }

        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(Connect {
            fixed_header: [FixedHeader::Connect as u8],
            remaining_length,
            protocol_name,
            protocol_version_buf,
            connect_flags_buf,
            keep_alive_buf,
            property_length,
            props,
            client_id_buf,
            will_property_length,
            will_props,
            will_topic_buf,
            will_payload_buf,
            user_name_buf,
            password_buf,
        })
    }
}

/// Implements JSON serialization for CONNECT packets
///
/// Provides structured JSON representation of the packet contents,
/// useful for debugging, logging, and API responses. Sensitive data
/// like passwords are masked for security.
///
/// # JSON Format
///
/// The serialized JSON includes:
/// - `type`: Always "connect"
/// - `client_id`: Client identifier
/// - `clean_start`: Clean start flag
/// - `keep_alive`: Keep alive interval
/// - `props`: MQTT properties (if present)
/// - `user_name`: User name (if present)
/// - `password`: Always "*****" (masked for security)
/// - `will_qos`: Will QoS level (if will message present)
/// - `will_retain`: Will retain flag (if will message present)
/// - `will_props`: Will properties (if present)
/// - `will_topic`: Will topic (if will message present)
/// - `will_payload`: Will payload as escaped binary string (if will message present)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use serde_json;
///
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("test-client")
///     .user_name("user")
///     .password(b"secret")
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&connect).unwrap();
/// // JSON will contain masked password: "password": "*****"
/// ```
impl Serialize for Connect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 5; // type, client_id, clean_start, keep_alive, props

        if self.user_name_flag() {
            field_count += 1;
        }

        if self.password_flag() {
            field_count += 1;
        }

        if self.will_flag() {
            field_count += 4; // will_qos, will_retain, will_topic, will_payload
            if !self.will_props.is_empty() {
                field_count += 1; // will_props
            }
        }

        let mut state = serializer.serialize_struct("Connect", field_count)?;
        state.serialize_field("type", "connect")?;
        state.serialize_field("client_id", &self.client_id())?;
        state.serialize_field("clean_start", &self.clean_start())?;
        state.serialize_field("keep_alive", &self.keep_alive())?;

        if !self.props.is_empty() {
            state.serialize_field("props", &self.props)?;
        }

        if self.user_name_flag() {
            state.serialize_field("user_name", &self.user_name())?;
        }

        if self.password_flag() {
            state.serialize_field("password", "*****")?;
        }

        if self.will_flag() {
            state.serialize_field("will_qos", &self.will_qos())?;
            state.serialize_field("will_retain", &self.will_retain())?;
            if !self.will_props.is_empty() {
                state.serialize_field("will_props", &self.will_props)?;
            }
            state.serialize_field("will_topic", &self.will_topic())?;

            // Format will_payload as binary string for JSON
            if let Some(payload) = self.will_payload() {
                match escape_binary_json_string(payload) {
                    Some(escaped) => state.serialize_field("will_payload", &escaped)?,
                    None => state.serialize_field("will_payload", &payload)?,
                }
            }
        }

        state.end()
    }
}

/// Implements Display trait for CONNECT packets
///
/// Formats the packet as JSON string for human-readable output.
/// This is useful for logging, debugging, and displaying packet information.
/// On serialization errors, returns an error JSON object.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("display-client")
///     .build()
///     .unwrap();
///
/// println!("CONNECT packet: {}", connect);
/// // Output: {"type":"connect","client_id":"display-client",...}
/// ```
impl fmt::Display for Connect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(e) => write!(f, "{{\"error\": \"{}\"}}", e),
        }
    }
}

/// Implements Debug trait for CONNECT packets
///
/// Uses the same JSON representation as Display for consistent debugging output.
/// This provides structured, readable information about the packet contents.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("debug-client")
///     .build()
///     .unwrap();
///
/// println!("{:?}", connect);
/// // Output: {"type":"connect","client_id":"debug-client",...}
/// ```
impl fmt::Debug for Connect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Implements generic packet behavior for CONNECT packets
///
/// This trait provides common functionality shared across all MQTT packet types,
/// enabling polymorphic handling of different packet types through a common interface.
/// It provides methods for getting packet size and converting to I/O buffers.
///
/// # Purpose
///
/// This implementation allows CONNECT packets to be used polymorphically with other
/// MQTT packet types in generic contexts, such as packet processing pipelines.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;
///
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("generic-client")
///     .build()
///     .unwrap();
///
/// // Use as generic packet
/// let size = connect.size();
/// let buffers = connect.to_buffers();
/// ```
impl GenericPacketTrait for Connect {
    fn size(&self) -> usize {
        self.size()
    }

    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }
}

/// Implements generic packet display behavior for CONNECT packets
///
/// This trait provides standardized display formatting methods that work
/// across different MQTT packet types. This allows for uniform handling of packets
/// across different MQTT packet types. This enables uniform display handling in
/// generic packet processing contexts.
///
/// # Purpose
///
/// Enables consistent display behavior when working with collections of different
/// packet types or in generic packet processing scenarios.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketDisplay;
/// use std::fmt::Write;
///
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_id("display-generic")
///     .build()
///     .unwrap();
///
/// let mut output = String::new();
/// write!(&mut output, "{}", connect).unwrap();
/// ```
impl GenericPacketDisplay for Connect {
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Validates CONNECT packet properties according to MQTT 5.0 specification
///
/// This function ensures that properties in a CONNECT packet are valid and
/// appear at most once (except for User Properties which can appear multiple times).
///
/// # Valid CONNECT Properties
///
/// - Session Expiry Interval (max 1)
/// - Receive Maximum (max 1)
/// - Maximum Packet Size (max 1)
/// - Topic Alias Maximum (max 1)
/// - Request Response Information (max 1)
/// - Request Problem Information (max 1)
/// - User Property (multiple allowed)
/// - Authentication Method (max 1)
/// - Authentication Data (max 1)
///
/// # Arguments
///
/// * `props` - Properties to validate
///
/// # Returns
///
/// * `Ok(())` - If all properties are valid
/// * `Err(MqttError::ProtocolError)` - If invalid properties are found or duplicates exist
///
/// # Errors
///
/// Returns `MqttError::ProtocolError` if:
/// - Invalid property types are present
/// - Required-unique properties appear more than once
fn validate_connect_properties(props: &Properties) -> Result<(), MqttError> {
    let mut count_session_expiry_interval = 0;
    let mut count_receive_maximum = 0;
    let mut count_maximum_packet_size = 0;
    let mut count_topic_alias_maximum = 0;
    let mut count_request_response_information = 0;
    let mut count_request_problem_information = 0;
    let mut count_authentication_method = 0;
    let mut count_authentication_data = 0;

    for prop in props {
        match prop {
            Property::SessionExpiryInterval(_) => count_session_expiry_interval += 1,
            Property::ReceiveMaximum(_) => count_receive_maximum += 1,
            Property::MaximumPacketSize(_) => count_maximum_packet_size += 1,
            Property::TopicAliasMaximum(_) => count_topic_alias_maximum += 1,
            Property::RequestResponseInformation(_) => count_request_response_information += 1,
            Property::RequestProblemInformation(_) => count_request_problem_information += 1,
            Property::UserProperty(_) => {}
            Property::AuthenticationMethod(_) => count_authentication_method += 1,
            Property::AuthenticationData(_) => count_authentication_data += 1,
            _ => return Err(MqttError::ProtocolError),
        }
    }

    if count_session_expiry_interval > 1
        || count_receive_maximum > 1
        || count_maximum_packet_size > 1
        || count_topic_alias_maximum > 1
        || count_request_response_information > 1
        || count_request_problem_information > 1
        || count_authentication_method > 1
        || count_authentication_data > 1
    {
        return Err(MqttError::ProtocolError);
    }

    Ok(())
}

/// Validates will message properties according to MQTT 5.0 specification
///
/// This function ensures that properties in a will message are valid and
/// appear at most once (except for User Properties which can appear multiple times).
///
/// # Valid Will Properties
///
/// - Will Delay Interval (max 1)
/// - Payload Format Indicator (max 1)
/// - Message Expiry Interval (max 1)
/// - Content Type (max 1)
/// - Response Topic (max 1)
/// - Correlation Data (max 1)
/// - User Property (multiple allowed)
///
/// # Arguments
///
/// * `props` - Will properties to validate
///
/// # Returns
///
/// * `Ok(())` - If all properties are valid
/// * `Err(MqttError::ProtocolError)` - If invalid properties are found or duplicates exist
///
/// # Errors
///
/// Returns `MqttError::ProtocolError` if:
/// - Invalid property types are present for will messages
/// - Required-unique properties appear more than once
fn validate_will_properties(props: &Properties) -> Result<(), MqttError> {
    let mut count_will_delay_interval = 0;
    let mut count_payload_format_indicator = 0;
    let mut count_message_expiry_interval = 0;
    let mut count_content_type = 0;
    let mut count_response_topic = 0;
    let mut count_correlation_data = 0;

    for prop in props {
        match prop {
            Property::WillDelayInterval(_) => count_will_delay_interval += 1,
            Property::PayloadFormatIndicator(_) => count_payload_format_indicator += 1,
            Property::MessageExpiryInterval(_) => count_message_expiry_interval += 1,
            Property::ContentType(_) => count_content_type += 1,
            Property::ResponseTopic(_) => count_response_topic += 1,
            Property::CorrelationData(_) => count_correlation_data += 1,
            Property::UserProperty(_) => {}
            _ => return Err(MqttError::ProtocolError),
        }
    }

    if count_will_delay_interval > 1
        || count_payload_format_indicator > 1
        || count_message_expiry_interval > 1
        || count_content_type > 1
        || count_response_topic > 1
        || count_correlation_data > 1
    {
        return Err(MqttError::ProtocolError);
    }

    Ok(())
}
