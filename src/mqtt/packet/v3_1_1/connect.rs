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
use derive_builder::Builder;
#[cfg(feature = "std")]
use std::io::IoSlice;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use getset::{CopyGetters, Getters};

use crate::mqtt::packet::json_bin_encode::escape_binary_json_string;
use crate::mqtt::packet::mqtt_binary::GenericMqttBinary;
use crate::mqtt::packet::mqtt_string::GenericMqttString;
use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::qos::Qos;
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::result_code::MqttError;

/// MQTT v3.1.1 CONNECT packet representation
///
/// The CONNECT packet is the first packet sent by a client to the MQTT server (broker)
/// to establish an MQTT connection. It contains information about the client's identity,
/// session preferences, authentication credentials, and last will testament.
///
/// According to MQTT v3.1.1 specification (<https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html>),
/// the CONNECT packet contains:
/// - Fixed header with packet type and remaining length
/// - Variable header with protocol name, version, connect flags, and keep alive
/// - Payload with client identifier, will topic/message (if set), username, and password (if set)
///
/// # Protocol Information
///
/// - **Protocol Name**: Always "MQTT" (4 bytes)
/// - **Protocol Version**: 0x04 for MQTT v3.1.1
///
/// # Connect Flags
///
/// The connect flags byte contains several important flags:
/// - **Bit 0**: Reserved (must be 0)
/// - **Bit 1**: Clean Session flag - if set, the server discards any existing session state
/// - **Bit 2**: Will flag - indicates presence of will message
/// - **Bits 3-4**: Will QoS level (0, 1, or 2)
/// - **Bit 5**: Will Retain flag - if set, will message is retained
/// - **Bit 6**: Password flag - indicates presence of password
/// - **Bit 7**: User Name flag - indicates presence of user name
///
/// # Session Management
///
/// The Clean Session flag controls session state handling:
/// - **Clean Session = true**: Server discards previous session state and starts fresh
/// - **Clean Session = false**: Server attempts to resume existing session if present
///
/// # Will Message
///
/// The will message is published by the server when the client disconnects unexpectedly
/// or fails to send keep-alive messages within the specified interval.
///
/// # Keep Alive
///
/// The keep alive timer specifies the maximum time interval (in seconds) between
/// control packet transmissions. A value of 0 disables the keep alive mechanism.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
///
/// // Create a basic CONNECT packet with client ID
/// let connect = mqtt::packet::v3_1_1::Connect::builder()
///     .client_id("my-client-123")
///     .clean_session(true)
///     .build()
///     .unwrap();
///
/// assert_eq!(connect.client_id(), "my-client-123");
/// assert!(connect.clean_session());
/// assert_eq!(connect.protocol_version(), 4);
///
/// // Create CONNECT with will message and credentials
/// let connect = mqtt::packet::v3_1_1::Connect::builder()
///     .client_id("client-with-will")
///     .clean_session(false)
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
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct GenericConnect<
    const STRING_BUFFER_SIZE: usize = 32,
    const BINARY_BUFFER_SIZE: usize = 32,
> {
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
    client_id_buf: GenericMqttString<STRING_BUFFER_SIZE>,

    #[builder(private)]
    will_topic_buf: GenericMqttString<STRING_BUFFER_SIZE>,
    #[builder(private)]
    will_payload_buf: GenericMqttBinary<BINARY_BUFFER_SIZE>,

    #[builder(private)]
    user_name_buf: GenericMqttString<STRING_BUFFER_SIZE>,
    #[builder(private)]
    password_buf: GenericMqttBinary<BINARY_BUFFER_SIZE>,
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
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
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("my-client")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> GenericConnectBuilder<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE> {
        GenericConnectBuilder::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::default()
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
    /// For MQTT v3.1.1, this is always "MQTT"
    ///
    /// # Returns
    ///
    /// The protocol name as a string slice
    pub fn protocol_name(&self) -> &str {
        "MQTT"
    }

    /// Returns the MQTT protocol version
    ///
    /// For MQTT v3.1.1, this is always 4 (0x04)
    ///
    /// # Returns
    ///
    /// The protocol version number
    pub fn protocol_version(&self) -> u8 {
        self.protocol_version_buf[0]
    }

    /// Returns the Clean Session flag value
    ///
    /// When true, the server discards any existing session state for this client
    /// and starts a new clean session. When false, the server attempts to resume
    /// an existing session if one exists.
    ///
    /// # Returns
    ///
    /// `true` if Clean Session is enabled, `false` otherwise
    pub fn clean_session(&self) -> bool {
        (self.connect_flags_buf[0] & 0b0000_0010) != 0
    }

    /// Alias for `clean_session()` for compatibility with MQTT 5.0 terminology
    ///
    /// In MQTT v3.1.1, this is equivalent to the Clean Session flag.
    /// The name "clean_start" is used in MQTT 5.0 but has the same meaning.
    ///
    /// # Returns
    ///
    /// `true` if Clean Session is enabled, `false` otherwise
    pub fn clean_start(&self) -> bool {
        self.clean_session()
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
    /// The keep alive timer specifies the maximum time interval between
    /// control packet transmissions. A value of 0 disables the keep alive mechanism.
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
    /// Each client connecting to a server has a unique client identifier.
    ///
    /// # Returns
    ///
    /// The client identifier as a string slice
    pub fn client_id(&self) -> &str {
        self.client_id_buf.as_str()
    }

    /// Returns the will topic if a will message is present
    ///
    /// The will topic specifies where the will message should be published
    /// when the client disconnects unexpectedly.
    ///
    /// # Returns
    ///
    /// `Some(topic)` if will flag is set, `None` otherwise
    pub fn will_topic(&self) -> Option<&str> {
        if self.will_flag() {
            Some(self.will_topic_buf.as_str())
        } else {
            None
        }
    }

    /// Returns the will message payload if a will message is present
    ///
    /// The will payload contains the message content to be published
    /// when the client disconnects unexpectedly.
    ///
    /// # Returns
    ///
    /// `Some(payload)` if will flag is set, `None` otherwise
    pub fn will_payload(&self) -> Option<&[u8]> {
        if self.will_flag() {
            Some(self.will_payload_buf.as_slice())
        } else {
            None
        }
    }

    /// Returns the user name if present
    ///
    /// The user name is used for authentication purposes.
    ///
    /// # Returns
    ///
    /// `Some(username)` if user name flag is set, `None` otherwise
    pub fn user_name(&self) -> Option<&str> {
        if self.user_name_flag() {
            Some(self.user_name_buf.as_str())
        } else {
            None
        }
    }

    /// Returns the password if present
    ///
    /// The password is used for authentication purposes and can contain binary data.
    /// According to MQTT specification, password can only be present if user name is also present.
    ///
    /// # Returns
    ///
    /// `Some(password)` if password flag is set, `None` otherwise
    pub fn password(&self) -> Option<&[u8]> {
        if self.password_flag() {
            Some(self.password_buf.as_slice())
        } else {
            None
        }
    }

    /// Returns the total size of the packet in bytes
    ///
    /// This includes the fixed header, remaining length field, and all payload data.
    ///
    /// # Returns
    ///
    /// Total packet size in bytes
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the packet to a vector of I/O slices for efficient network transmission
    ///
    /// This method creates a zero-copy representation of the packet as I/O slices,
    /// which can be used with vectored I/O operations for efficient transmission.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` containing all packet data
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("client")
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = connect.to_buffers();
    /// // Use buffers for vectored I/O
    /// ```
    #[cfg(feature = "std")]
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        bufs.push(IoSlice::new(&self.protocol_name));
        bufs.push(IoSlice::new(&self.protocol_version_buf));
        bufs.push(IoSlice::new(&self.connect_flags_buf));
        bufs.push(IoSlice::new(&self.keep_alive_buf));

        bufs.extend(self.client_id_buf.to_buffers());

        if self.will_flag() {
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

    /// Converts the CONNECT packet into a continuous buffer for no-std environments.
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
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("client")
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffer = connect.to_continuous_buffer();
    /// // Use buffer for writing to network streams
    /// ```
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        buf.extend_from_slice(&self.protocol_name);
        buf.extend_from_slice(&self.protocol_version_buf);
        buf.extend_from_slice(&self.connect_flags_buf);
        buf.extend_from_slice(&self.keep_alive_buf);

        buf.append(&mut self.client_id_buf.to_continuous_buffer());

        if self.will_flag() {
            buf.append(&mut self.will_topic_buf.to_continuous_buffer());
            buf.append(&mut self.will_payload_buf.to_continuous_buffer());
        }

        if self.user_name_flag() {
            buf.append(&mut self.user_name_buf.to_continuous_buffer());
        }

        if self.password_flag() {
            buf.append(&mut self.password_buf.to_continuous_buffer());
        }

        buf
    }

    /// Parses a CONNECT packet from raw bytes
    ///
    /// This method deserializes a CONNECT packet from the provided byte slice,
    /// validating the protocol format and extracting all packet fields.
    ///
    /// # Parameters
    ///
    /// * `data` - The raw packet data starting from the variable header
    ///            (excluding the fixed header bytes that were already processed)
    ///
    /// # Returns
    ///
    /// * `Ok((connect, consumed))` - The parsed CONNECT packet and number of bytes consumed
    /// * `Err(MqttError)` - Parse error with specific error type:
    ///   - `MalformedPacket` - Invalid packet format
    ///   - `ProtocolError` - Protocol violation (e.g., invalid protocol name)
    ///   - `UnsupportedProtocolVersion` - Unsupported protocol version
    ///   - `ClientIdentifierNotValid` - Invalid client identifier format
    ///   - `BadUserNameOrPassword` - Invalid username or password format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse CONNECT packet from bytes
    /// let packet_data = &[/* packet bytes */];
    /// match mqtt::packet::v3_1_1::Connect::parse(packet_data) {
    ///     Ok((connect, consumed)) => {
    ///         println!("Client ID: {}", connect.client_id());
    ///         println!("Consumed {} bytes", consumed);
    ///     }
    ///     Err(e) => println!("Parse error: {:?}", e),
    /// }
    /// ```
    pub fn parse(
        data: &[u8],
    ) -> Result<
        (
            GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
            usize,
        ),
        MqttError,
    > {
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
        if protocol_version != 0x04 {
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

        // Client Identifier
        let (client_id_buf, consumed) =
            GenericMqttString::<STRING_BUFFER_SIZE>::decode(&data[cursor..])
                .map_err(|_| MqttError::ClientIdentifierNotValid)?;
        cursor += consumed;

        // Will  Messages (if will flag is set)
        let will_flag = (connect_flags & 0b0000_0100) != 0;
        let mut will_topic_buf = GenericMqttString::<STRING_BUFFER_SIZE>::default();
        let mut will_payload_buf = GenericMqttBinary::<BINARY_BUFFER_SIZE>::default();

        if will_flag {
            // Will Topic
            let (w_topic, consumed) =
                GenericMqttString::<STRING_BUFFER_SIZE>::decode(&data[cursor..])?;
            cursor += consumed;
            will_topic_buf = w_topic;

            // Will Payload
            let (w_payload, consumed) =
                GenericMqttBinary::<BINARY_BUFFER_SIZE>::decode(&data[cursor..])?;
            cursor += consumed;
            will_payload_buf = w_payload;
        }

        // User Name (if user name flag is set)
        let user_name_flag = (connect_flags & 0b1000_0000) != 0;
        let mut user_name_buf = GenericMqttString::<STRING_BUFFER_SIZE>::default();
        if user_name_flag {
            let (uname, consumed) =
                GenericMqttString::<STRING_BUFFER_SIZE>::decode(&data[cursor..])
                    .map_err(|_| MqttError::BadUserNameOrPassword)?;
            cursor += consumed;
            user_name_buf = uname;
        }

        // Password (if password flag is set)
        let password_flag = (connect_flags & 0b0100_0000) != 0;
        let mut password_buf = GenericMqttBinary::<BINARY_BUFFER_SIZE>::default();
        if password_flag {
            let (pwd, consumed) = GenericMqttBinary::<BINARY_BUFFER_SIZE>::decode(&data[cursor..])
                .map_err(|_| MqttError::BadUserNameOrPassword)?;
            cursor += consumed;
            password_buf = pwd;
        }

        // Validate flags consistency
        if password_flag && !user_name_flag {
            return Err(MqttError::ProtocolError);
        }

        let connect = GenericConnect {
            fixed_header: [FixedHeader::Connect as u8],
            remaining_length: VariableByteInteger::from_u32(cursor as u32).unwrap(),
            protocol_name,
            protocol_version_buf,
            connect_flags_buf,
            keep_alive_buf,
            client_id_buf,
            will_topic_buf,
            will_payload_buf,
            user_name_buf,
            password_buf,
        };

        Ok((connect, cursor))
    }
}

/// Builder for constructing MQTT v3.1.1 CONNECT packets
///
/// The `ConnectBuilder` provides a fluent interface for creating CONNECT packets
/// with various configuration options. All setter methods can be chained together
/// to build the final packet.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::qos::Qos;
///
/// // Build a complete CONNECT packet
/// let connect = mqtt::packet::v3_1_1::Connect::builder()
///     .client_id("my-device")
///     .clean_session(true)
///     .keep_alive(300)
///     .will_message("device/status", b"offline", Qos::AtLeastOnce, true)
///     .user_name("username")
///     .password(b"password")
///     .build()
///     .unwrap();
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize>
    GenericConnectBuilder<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    /// Sets the client identifier
    ///
    /// The client identifier uniquely identifies the client to the server.
    /// Each client connecting to a server must have a unique client identifier.
    /// The client identifier must be a valid UTF-8 string.
    ///
    /// # Parameters
    ///
    /// * `id` - The client identifier string
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with client ID set
    /// * `Err(MqttError)` - If the client ID is invalid UTF-8 or too long
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let builder = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("device-001")
    ///     .unwrap();
    /// ```
    pub fn client_id(mut self, id: impl AsRef<str>) -> Result<Self, MqttError> {
        let mqtt_str = GenericMqttString::<STRING_BUFFER_SIZE>::new(id.as_ref())?;
        self.client_id_buf = Some(mqtt_str);
        Ok(self)
    }

    /// Sets the Clean Session flag
    ///
    /// When set to true, the server discards any existing session state for this client
    /// and starts a new clean session. When false, the server attempts to resume
    /// an existing session if one exists.
    ///
    /// # Parameters
    ///
    /// * `clean` - Whether to enable clean session mode
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .clean_session(true)  // Start with clean session
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn clean_session(mut self, clean: bool) -> Self {
        let mut flags = self.connect_flags_buf.unwrap_or([0])[0];
        if clean {
            flags |= 0b0000_0010;
        } else {
            flags &= !0b0000_0010;
        }
        self.connect_flags_buf = Some([flags]);
        self
    }

    /// Alias for `clean_session()` for compatibility with MQTT 5.0 terminology
    ///
    /// In MQTT v3.1.1, this is equivalent to the Clean Session flag.
    /// The name "clean_start" is used in MQTT 5.0 but has the same meaning.
    ///
    /// # Parameters
    ///
    /// * `clean` - Whether to enable clean session mode
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn clean_start(self, clean: bool) -> Self {
        self.clean_session(clean)
    }

    /// Sets the will message (Last Will and Testament)
    ///
    /// The will message is published by the server when the client disconnects
    /// unexpectedly or fails to send keep-alive messages within the specified interval.
    /// This allows clients to notify other clients about unexpected disconnections.
    ///
    /// # Parameters
    ///
    /// * `topic` - The topic where the will message should be published
    /// * `payload` - The will message payload (can be binary data)
    /// * `qos` - Quality of Service level for the will message
    /// * `retain` - Whether the will message should be retained by the server
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
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("device")
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
        let will_topic = GenericMqttString::<STRING_BUFFER_SIZE>::new(topic.as_ref())?;
        let will_payload = GenericMqttBinary::<BINARY_BUFFER_SIZE>::new(payload.as_ref().to_vec())?;

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
    /// The user name is used for client authentication. It must be a valid UTF-8 string.
    /// If password authentication is required, the password should be set using `password()`.
    ///
    /// # Parameters
    ///
    /// * `name` - The user name string
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with user name set
    /// * `Err(MqttError)` - If the user name is invalid UTF-8 or too long
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("device")
    ///     .user_name("myusername")
    ///     .password(b"mypassword")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn user_name(mut self, name: impl AsRef<str>) -> Result<Self, MqttError> {
        let mqtt_str = GenericMqttString::<STRING_BUFFER_SIZE>::new(name.as_ref())?;
        self.user_name_buf = Some(mqtt_str);

        let mut flags = self.connect_flags_buf.unwrap_or([0b0000_0010])[0];
        flags |= 0b1000_0000; // User name flag
        self.connect_flags_buf = Some([flags]);
        Ok(self)
    }

    /// Sets the password for authentication
    ///
    /// The password is used for client authentication and can contain binary data.
    /// According to MQTT specification, password can only be set if user name is also set.
    /// The password field is treated as binary data and does not need to be valid UTF-8.
    ///
    /// # Parameters
    ///
    /// * `pwd` - The password as bytes (can be binary data)
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` - Builder with password set
    /// * `Err(MqttError)` - If the password data is too long
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Text password
    /// let connect1 = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("device")
    ///     .user_name("user")
    ///     .password(b"mypassword")
    ///     .build()
    ///     .unwrap();
    ///
    /// // Binary password
    /// let binary_pwd = vec![0x01, 0x02, 0x03, 0x04];
    /// let connect2 = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("device")
    ///     .user_name("user")
    ///     .password(&binary_pwd)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn password(mut self, pwd: impl AsRef<[u8]>) -> Result<Self, MqttError> {
        let mqtt_bin = GenericMqttBinary::<BINARY_BUFFER_SIZE>::new(pwd.as_ref().to_vec())?;
        self.password_buf = Some(mqtt_bin);

        let mut flags = self.connect_flags_buf.unwrap_or([0b0000_0010])[0];
        flags |= 0b0100_0000; // Password flag
        self.connect_flags_buf = Some([flags]);
        Ok(self)
    }

    /// Sets the keep alive interval
    ///
    /// The keep alive timer specifies the maximum time interval (in seconds) between
    /// control packet transmissions. If no control packets are sent within this interval,
    /// the client must send a PINGREQ packet. A value of 0 disables the keep alive mechanism.
    ///
    /// # Parameters
    ///
    /// * `seconds` - Keep alive interval in seconds (0-65535, where 0 = disabled)
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // 5 minute keep alive
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("device")
    ///     .keep_alive(300)
    ///     .build()
    ///     .unwrap();
    ///
    /// // Disable keep alive
    /// let connect_no_keepalive = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("device")
    ///     .keep_alive(0)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn keep_alive(mut self, seconds: u16) -> Self {
        self.keep_alive_buf = Some(seconds.to_be_bytes());
        self
    }

    /// Validates the builder configuration
    ///
    /// This method checks that the builder configuration is valid according to
    /// MQTT v3.1.1 specification rules:
    /// - Password flag can only be set if user name flag is also set
    /// - If will flag is set, both will topic and will payload must be present
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Configuration is valid
    /// * `Err(MqttError::ProtocolError)` - Protocol violation detected
    /// * `Err(MqttError::MalformedPacket)` - Incomplete will message configuration
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

        Ok(())
    }

    /// Builds the final CONNECT packet
    ///
    /// This method validates the builder configuration and constructs the final
    /// CONNECT packet. All required fields are populated with defaults if not explicitly set.
    ///
    /// # Default Values
    ///
    /// - Client ID: Empty string (some brokers allow this)
    /// - Clean Session: `true`
    /// - Keep Alive: 0 (disabled)
    /// - No will message
    /// - No authentication credentials
    ///
    /// # Returns
    ///
    /// * `Ok(Connect)` - Successfully built CONNECT packet
    /// * `Err(MqttError)` - Validation error or configuration issue
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Minimal CONNECT packet
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .build()
    ///     .unwrap();
    ///
    /// // Full CONNECT packet
    /// let connect = mqtt::packet::v3_1_1::Connect::builder()
    ///     .client_id("my-device")
    ///     .clean_session(true)
    ///     .keep_alive(300)
    ///     .user_name("username")
    ///     .password(b"password")
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn build(
        self,
    ) -> Result<GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>, MqttError> {
        self.validate()?;

        let protocol_name = [0x00, 0x04, b'M', b'Q', b'T', b'T'];
        let protocol_version_buf = [0x04];
        let connect_flags_buf = self.connect_flags_buf.unwrap_or([0b0000_0010]); // Default: clean_start = true
        let connect_flags = connect_flags_buf[0];
        let keep_alive_buf = self.keep_alive_buf.unwrap_or([0, 0]);

        let client_id_buf = self.client_id_buf.unwrap_or_default();

        let will_flag = (connect_flags & 0b0000_0100) != 0;
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
        remaining += client_id_buf.size(); // client identifier

        if will_flag {
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

        Ok(GenericConnect {
            fixed_header: [FixedHeader::Connect as u8],
            remaining_length,
            protocol_name,
            protocol_version_buf,
            connect_flags_buf,
            keep_alive_buf,
            client_id_buf,
            will_topic_buf,
            will_payload_buf,
            user_name_buf,
            password_buf,
        })
    }
}

/// Implementation of the Serialize trait for CONNECT packets
///
/// This implementation provides JSON serialization for CONNECT packets,
/// which is useful for logging, debugging, and API responses. The password
/// field is masked for security reasons.
///
/// # Security Note
///
/// The password field is always serialized as "*****" to prevent
/// accidental exposure of sensitive authentication data in logs.
///
/// # JSON Format
///
/// The serialized JSON contains the following fields:
/// - `type`: Always "connect"
/// - `client_id`: The client identifier
/// - `clean_start`: The clean session flag
/// - `keep_alive`: Keep alive interval in seconds
/// - `user_name`: Username (if present)
/// - `password`: Always "*****" (if present)
/// - `will_qos`: Will message QoS (if will message present)
/// - `will_retain`: Will retain flag (if will message present)
/// - `will_topic`: Will topic (if will message present)
/// - `will_payload`: Will payload as escaped binary string (if will message present)
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> Serialize
    for GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 5; // type, client_id, clean_start, keep_alive

        if self.user_name_flag() {
            field_count += 1;
        }
        if self.password_flag() {
            field_count += 1;
        }

        if self.will_flag() {
            field_count += 4; // will_qos, will_retain, will_topic, will_payload
        }

        let mut state = serializer.serialize_struct("Connect", field_count)?;
        state.serialize_field("type", "connect")?;
        state.serialize_field("client_id", &self.client_id())?;
        state.serialize_field("clean_start", &self.clean_start())?;
        state.serialize_field("keep_alive", &self.keep_alive())?;

        if self.user_name_flag() {
            state.serialize_field("user_name", &self.user_name())?;
        }

        if self.password_flag() {
            state.serialize_field("password", "*****")?;
        }

        if self.will_flag() {
            state.serialize_field("will_qos", &self.will_qos())?;
            state.serialize_field("will_retain", &self.will_retain())?;
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

/// Implementation of the Display trait for CONNECT packets
///
/// This implementation formats the CONNECT packet as a JSON string for
/// human-readable output. It uses the Serialize implementation internally.
///
/// # Output Format
///
/// The output is a JSON representation of the packet. If serialization fails,
/// an error JSON object is returned instead.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let connect = mqtt::packet::v3_1_1::Connect::builder()
///     .client_id("device")
///     .build()
///     .unwrap();
///
/// println!("{}", connect);
/// // Output: {"type":"connect","client_id":"device","clean_start":true,"keep_alive":0}
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Display
    for GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Implementation of the Debug trait for CONNECT packets
///
/// This implementation uses the same format as Display, providing
/// a JSON representation of the packet for debugging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let connect = mqtt::packet::v3_1_1::Connect::builder()
///     .client_id("device")
///     .build()
///     .unwrap();
///
/// println!("{:?}", connect);
/// // Output: {"type":"connect","client_id":"device","clean_start":true,"keep_alive":0}
/// ```
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> fmt::Debug
    for GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// Implementation of GenericPacketTrait for CONNECT packets
///
/// This trait provides a generic interface for packet operations,
/// allowing CONNECT packets to be used polymorphically with other packet types.
///
/// The implementation delegates to the specific CONNECT packet methods.
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> GenericPacketTrait
    for GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    /// Returns the total size of the packet in bytes
    ///
    /// Delegates to the `Connect::size()` method.
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

/// Implementation of GenericPacketDisplay for CONNECT packets
///
/// This trait provides a generic interface for packet display operations,
/// allowing CONNECT packets to be formatted uniformly with other packet types.
///
/// The implementation delegates to the standard Debug and Display traits.
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> GenericPacketDisplay
    for GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    /// Formats the packet using the Debug trait
    ///
    /// Delegates to the `Debug::fmt()` implementation.
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    /// Formats the packet using the Display trait
    ///
    /// Delegates to the `Display::fmt()` implementation.
    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}
