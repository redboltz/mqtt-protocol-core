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
use getset::{CopyGetters, Getters};

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
use crate::mqtt::packet::{
    Properties, PropertiesParse, PropertiesSize, PropertiesToBuffers, Property,
};
use crate::mqtt::result_code::DisconnectReasonCode;
use crate::mqtt::result_code::MqttError;

/// MQTT 5.0 DISCONNECT packet representation
///
/// The DISCONNECT packet is sent by either the client or the server to indicate
/// graceful disconnection from the MQTT connection. In MQTT 5.0, both the client
/// and server can send DISCONNECT packets to cleanly terminate the connection.
///
/// According to MQTT 5.0 specification, the DISCONNECT packet contains:
/// - Fixed header with packet type and remaining length
/// - Variable header with reason code and properties (both optional)
/// - No payload
///
/// # Reason Codes
///
/// The reason code indicates why the connection is being terminated:
/// - `0x00` Normal disconnection - Clean disconnect without error
/// - `0x04` Disconnect with Will Message - Normal disconnect, publish Will Message
/// - `0x80` Unspecified error
/// - `0x81` Malformed packet
/// - `0x82` Protocol error
/// - `0x83` Implementation specific error
/// - `0x87` Not authorized
/// - `0x89` Server busy
/// - `0x8B` Server shutting down
/// - `0x8D` Keep alive timeout
/// - `0x8E` Session taken over
/// - `0x8F` Topic filter invalid
/// - `0x90` Topic name invalid
/// - `0x93` Receive maximum exceeded
/// - `0x94` Topic alias invalid
/// - `0x95` Packet too large
/// - `0x96` Message rate too high
/// - `0x97` Quota exceeded
/// - `0x98` Administrative action
/// - `0x99` Payload format invalid
/// - `0x9A` Retain not supported
/// - `0x9B` QoS not supported
/// - `0x9C` Use another server
/// - `0x9D` Server moved
/// - `0x9E` Shared subscriptions not supported
/// - `0x9F` Connection rate exceeded
/// - `0xA0` Maximum connect time
/// - `0xA1` Subscription identifiers not supported
/// - `0xA2` Wildcard subscriptions not supported
///
/// # Properties
///
/// MQTT 5.0 DISCONNECT packets can include the following properties:
/// - **Session Expiry Interval**: Overrides the session expiry interval set in CONNECT
/// - **Reason String**: Human-readable string describing the reason for disconnect
/// - **User Properties**: Key-value pairs for application-specific metadata
/// - **Server Reference**: Alternative server for client to connect to
///
/// # Packet Structure Rules
///
/// - If no reason code is present, the remaining length is 0
/// - If reason code is present but no properties, remaining length is 1
/// - If properties are present, reason code must also be present
/// - Properties must not contain duplicate entries except for User Properties
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
///
/// // Create a simple normal disconnection
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .build()
///     .unwrap();
///
/// // Create disconnect with reason code
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .reason_code(DisconnectReasonCode::NormalDisconnection)
///     .build()
///     .unwrap();
///
/// assert_eq!(disconnect.reason_code(), Some(DisconnectReasonCode::NormalDisconnection));
///
/// // Create disconnect with reason code and properties
/// let props = vec![
///     mqtt::packet::ReasonString::new("Session timeout").unwrap().into(),
///     mqtt::packet::UserProperty::new("client_id", "device123").unwrap().into(),
/// ];
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .reason_code(DisconnectReasonCode::KeepAliveTimeout)
///     .props(props)
///     .build()
///     .unwrap();
///
/// // Serialize to bytes for network transmission
/// let buffers = disconnect.to_buffers();
/// let size = disconnect.size();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct Disconnect {
    #[builder(private)]
    fixed_header: [u8; 1],
    #[builder(private)]
    remaining_length: VariableByteInteger,
    #[builder(private)]
    reason_code_buf: Option<[u8; 1]>,
    #[builder(private)]
    property_length: Option<VariableByteInteger>,

    /// Optional MQTT 5.0 properties for the DISCONNECT packet
    ///
    /// Properties provide additional metadata about the disconnection.
    /// Valid properties for DISCONNECT packets include:
    /// - Session Expiry Interval
    /// - Reason String
    /// - User Properties
    /// - Server Reference
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Option<Properties>,
}

impl Disconnect {
    /// Creates a new builder for constructing a DISCONNECT packet
    ///
    /// The builder pattern allows for flexible construction of DISCONNECT packets
    /// with optional reason codes and properties.
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
    /// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> DisconnectBuilder {
        DisconnectBuilder::default()
    }

    /// Returns the packet type for DISCONNECT packets
    ///
    /// This is always `PacketType::Disconnect` (14) for DISCONNECT packets.
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
    /// assert_eq!(mqtt::packet::v5_0::Disconnect::packet_type(), PacketType::Disconnect);
    /// ```
    pub fn packet_type() -> PacketType {
        PacketType::Disconnect
    }

    /// Returns the reason code for the disconnection
    ///
    /// The reason code indicates why the connection is being terminated.
    /// If no reason code is present in the packet, `None` is returned,
    /// which implies normal disconnection (0x00).
    ///
    /// # Returns
    ///
    /// - `Some(DisconnectReasonCode)` if a reason code is present
    /// - `None` if no reason code is present (implies normal disconnection)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
    ///
    /// // Disconnect without explicit reason code
    /// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(disconnect.reason_code(), None);
    ///
    /// // Disconnect with reason code
    /// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
    ///     .reason_code(DisconnectReasonCode::ServerShuttingDown)
    ///     .build()
    ///     .unwrap();
    /// assert_eq!(disconnect.reason_code(), Some(DisconnectReasonCode::ServerShuttingDown));
    /// ```
    pub fn reason_code(&self) -> Option<DisconnectReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| DisconnectReasonCode::try_from(buf[0]).ok())
    }

    /// Returns the total size of the DISCONNECT packet in bytes
    ///
    /// This includes the fixed header (1 byte), remaining length field,
    /// optional reason code (1 byte), optional property length field,
    /// and optional properties.
    ///
    /// # Returns
    ///
    /// The total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
    ///
    /// // Simple disconnect (2 bytes: fixed header + remaining length 0)
    /// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
    ///     .build()
    ///     .unwrap();
    /// let size = disconnect.size();
    ///
    /// // Disconnect with reason code (4 bytes: fixed header + remaining length 1 + reason code + property length 0)
    /// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
    ///     .reason_code(DisconnectReasonCode::NormalDisconnection)
    ///     .build()
    ///     .unwrap();
    /// let size_with_reason = disconnect.size();
    /// assert!(size_with_reason > size);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Converts the DISCONNECT packet to a vector of byte slices for network transmission
    ///
    /// This method creates a zero-copy representation of the packet as `IoSlice` buffers,
    /// which can be efficiently written to the network using vectored I/O operations.
    ///
    /// The buffers are ordered as follows:
    /// 1. Fixed header (1 byte)
    /// 2. Remaining length (1-4 bytes)
    /// 3. Reason code (1 byte, if present)
    /// 4. Property length (1-4 bytes, if properties are present)
    /// 5. Properties (variable length, if present)
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` containing the packet data ready for transmission
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
    ///
    /// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
    ///     .reason_code(DisconnectReasonCode::NormalDisconnection)
    ///     .build()
    ///     .unwrap();
    ///
    /// let buffers = disconnect.to_buffers();
    /// // Can be used with vectored I/O operations like write_vectored
    /// ```
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut bufs = Vec::new();
        bufs.push(IoSlice::new(&self.fixed_header));
        bufs.push(IoSlice::new(self.remaining_length.as_bytes()));
        if let Some(buf) = &self.reason_code_buf {
            bufs.push(IoSlice::new(buf));
        }
        if let Some(pl) = &self.property_length {
            bufs.push(IoSlice::new(pl.as_bytes()));
        }
        if let Some(ref props) = self.props {
            bufs.append(&mut props.to_buffers());
        }

        bufs
    }

    /// Parses a DISCONNECT packet from byte data
    ///
    /// This method parses the variable header portion of a DISCONNECT packet,
    /// extracting the optional reason code and properties. The fixed header
    /// should have been parsed separately before calling this method.
    ///
    /// # Parameters
    ///
    /// * `data` - Byte slice containing the variable header data
    ///
    /// # Returns
    ///
    /// * `Ok((Disconnect, usize))` - The parsed packet and number of bytes consumed
    /// * `Err(MqttError)` - If the packet is malformed or contains invalid data
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - If the packet structure is invalid
    /// - `MqttError::ProtocolError` - If properties violate MQTT 5.0 rules
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Parse disconnect with reason code and empty properties
    /// let data = [0x00, 0x00]; // Normal disconnection + empty properties
    /// let (disconnect, consumed) = mqtt::packet::v5_0::Disconnect::parse(&data).unwrap();
    /// assert_eq!(consumed, 2);
    ///
    /// // Parse empty disconnect (no reason code, no properties)
    /// let data = [];
    /// let (disconnect, consumed) = mqtt::packet::v5_0::Disconnect::parse(&data).unwrap();
    /// assert_eq!(consumed, 0);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;

        // reason_code
        let reason_code_buf = if cursor < data.len() {
            let rc = data[cursor];
            let _ = DisconnectReasonCode::try_from(rc).map_err(|_| MqttError::MalformedPacket)?;
            cursor += 1;
            Some([rc])
        } else {
            None
        };

        // properties
        let (property_length, props) = if reason_code_buf.is_some() && cursor < data.len() {
            let (props, consumed) = Properties::parse(&data[cursor..])?;
            cursor += consumed;
            validate_disconnect_properties(&props)?;
            let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

            (Some(prop_len), Some(props))
        } else {
            (None, None)
        };

        let remaining_size = reason_code_buf.as_ref().map_or(0, |_| 1)
            + property_length.as_ref().map_or(0, |pl| pl.size())
            + props.as_ref().map_or(0, |ps| ps.size());

        let disconnect = Disconnect {
            fixed_header: [FixedHeader::Disconnect.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            reason_code_buf,
            property_length,
            props,
        };

        Ok((disconnect, cursor))
    }
}

/// Builder for constructing DISCONNECT packets
///
/// The `DisconnectBuilder` follows the builder pattern to allow flexible
/// construction of DISCONNECT packets with optional components.
///
/// # Validation Rules
///
/// - Properties can only be set if a reason code is also present
/// - Properties must contain only valid DISCONNECT properties
/// - No duplicate properties are allowed (except User Properties)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
///
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .reason_code(DisconnectReasonCode::ServerShuttingDown)
///     .props(vec![
///         mqtt::packet::ReasonString::new("Maintenance").unwrap().into()
///     ])
///     .build()
///     .unwrap();
/// ```
impl DisconnectBuilder {
    /// Sets the reason code for the DISCONNECT packet
    ///
    /// The reason code indicates why the connection is being terminated.
    /// Setting a reason code is required if you want to include properties.
    ///
    /// # Parameters
    ///
    /// * `rc` - The disconnect reason code
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
    ///
    /// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
    ///     .reason_code(DisconnectReasonCode::KeepAliveTimeout)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn reason_code(mut self, rc: DisconnectReasonCode) -> Self {
        self.reason_code_buf = Some(Some([rc as u8]));
        self
    }

    /// Validates the builder configuration before building the packet
    ///
    /// This method ensures that the packet configuration follows MQTT 5.0 rules:
    /// - Properties can only be present if a reason code is also present
    /// - Properties must be valid for DISCONNECT packets
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the configuration is valid
    /// * `Err(MqttError)` if validation fails
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - If properties are set without a reason code
    /// - `MqttError::ProtocolError` - If properties contain invalid entries
    fn validate(&self) -> Result<(), MqttError> {
        if self.reason_code_buf.is_none() && self.props.is_some() {
            return Err(MqttError::MalformedPacket);
        }
        if self.props.is_some() {
            let inner_option = self.props.as_ref().unwrap();
            let props = inner_option
                .as_ref()
                .expect("INTERNAL ERRORS: props was set with None value, this should never happen");
            validate_disconnect_properties(props)?;
        }
        Ok(())
    }

    /// Builds the DISCONNECT packet from the configured parameters
    ///
    /// This method validates the builder configuration and constructs the final
    /// DISCONNECT packet. It calculates the remaining length and organizes
    /// all components according to MQTT 5.0 specification.
    ///
    /// # Returns
    ///
    /// * `Ok(Disconnect)` - The constructed packet
    /// * `Err(MqttError)` - If the configuration is invalid
    ///
    /// # Errors
    ///
    /// - `MqttError::MalformedPacket` - If the packet structure would be invalid
    /// - `MqttError::ProtocolError` - If properties violate MQTT 5.0 rules
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
    ///
    /// let result = mqtt::packet::v5_0::Disconnect::builder()
    ///     .reason_code(DisconnectReasonCode::NormalDisconnection)
    ///     .build();
    ///
    /// match result {
    ///     Ok(disconnect) => println!("Packet built successfully"),
    ///     Err(e) => println!("Build failed: {:?}", e),
    /// }
    /// ```
    pub fn build(self) -> Result<Disconnect, MqttError> {
        self.validate()?;

        let reason_code_buf = self.reason_code_buf.flatten();
        let props = self.props.flatten();
        let props_size: usize = props.as_ref().map_or(0, |p| p.size());
        // property_length only if properties are present
        let property_length = if props.is_some() {
            Some(VariableByteInteger::from_u32(props_size as u32).unwrap())
        } else {
            None
        };

        let mut remaining = 0;
        // add reason code if present
        if reason_code_buf.is_some() {
            remaining += 1;
        }
        // add properties if present
        if let Some(ref pl) = property_length {
            remaining += pl.size() + props_size;
        }
        let remaining_length = VariableByteInteger::from_u32(remaining as u32).unwrap();

        Ok(Disconnect {
            fixed_header: [FixedHeader::Disconnect.as_u8()],
            remaining_length,
            reason_code_buf,
            property_length,
            props,
        })
    }
}

/// Implements JSON serialization for DISCONNECT packets
///
/// This implementation converts the DISCONNECT packet to a JSON representation
/// suitable for debugging, logging, or API responses. The serialization includes
/// the packet type, optional reason code, and optional properties.
///
/// # JSON Structure
///
/// ```json
/// {
///   "type": "disconnect",
///   "reason_code": "NormalDisconnection",  // Optional
///   "props": [ ... ]                        // Optional
/// }
/// ```
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
///
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .reason_code(DisconnectReasonCode::ServerShuttingDown)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&disconnect).unwrap();
/// println!("DISCONNECT packet: {}", json);
/// ```
impl Serialize for Disconnect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut field_count = 1; // type

        if self.reason_code_buf.is_some() {
            field_count += 1; // reason_code
        }

        if self.props.is_some() {
            field_count += 1; // props
        }

        let mut state = serializer.serialize_struct("disconnect", field_count)?;
        state.serialize_field("type", PacketType::Disconnect.as_str())?;
        if self.reason_code_buf.is_some() {
            state.serialize_field("reason_code", &self.reason_code())?;
        }
        if let Some(props) = &self.props {
            state.serialize_field("props", props)?;
        }

        state.end()
    }
}

/// Implements `Display` trait for DISCONNECT packets
///
/// This provides a human-readable JSON representation of the packet,
/// making it useful for debugging and logging purposes.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::DisconnectReasonCode;
///
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .reason_code(DisconnectReasonCode::NormalDisconnection)
///     .build()
///     .unwrap();
///
/// println!("Packet: {}", disconnect);
/// // Output: {"type":"disconnect","reason_code":"NormalDisconnection"}
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
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .build()
///     .unwrap();
///
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
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;
///
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .build()
///     .unwrap();
///
/// // Use through the generic trait
/// let size = disconnect.size();
/// let buffers = disconnect.to_buffers();
/// ```
impl GenericPacketTrait for Disconnect {
    fn size(&self) -> usize {
        self.size()
    }

    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        self.to_buffers()
    }
}

/// Implements the generic packet display trait for DISCONNECT packets
///
/// This trait provides a common display interface for all MQTT packet types,
/// enabling consistent formatting across different packet implementations.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::packet::GenericPacketDisplay;
///
/// let disconnect = mqtt::packet::v5_0::Disconnect::builder()
///     .build()
///     .unwrap();
///
/// // Format through the generic trait
/// println!("{}", disconnect);
/// ```
impl GenericPacketDisplay for Disconnect {
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

/// Validates that properties are appropriate for DISCONNECT packets
///
/// This function ensures that only valid properties are included in DISCONNECT packets
/// and that no duplicates exist (except for User Properties which can appear multiple times).
///
/// # Valid Properties for DISCONNECT
///
/// - **Session Expiry Interval**: Maximum one occurrence
/// - **Reason String**: Maximum one occurrence  
/// - **User Property**: Multiple occurrences allowed
/// - **Server Reference**: Maximum one occurrence
///
/// # Parameters
///
/// * `props` - Slice of properties to validate
///
/// # Returns
///
/// * `Ok(())` if all properties are valid
/// * `Err(MqttError::ProtocolError)` if invalid or duplicate properties are found
///
/// # Errors
///
/// - `MqttError::ProtocolError` - If properties contain invalid entries or duplicates
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let props = vec![
///     mqtt::packet::ReasonString::new("Timeout").unwrap().into(),
///     mqtt::packet::UserProperty::new("client", "device1").unwrap().into(),
/// ];
///
/// // This is called internally during packet construction
/// // validate_disconnect_properties(&props).unwrap();
/// ```
fn validate_disconnect_properties(props: &[Property]) -> Result<(), MqttError> {
    let mut count_session_expiry_interval = 0;
    let mut count_reason_string = 0;
    let mut count_server_reference = 0;
    for prop in props {
        match prop {
            Property::SessionExpiryInterval(_) => count_session_expiry_interval += 1,
            Property::ReasonString(_) => count_reason_string += 1,
            Property::UserProperty(_) => {}
            Property::ServerReference(_) => count_server_reference += 1,
            _ => return Err(MqttError::ProtocolError),
        }
    }
    if count_session_expiry_interval > 1 {
        return Err(MqttError::ProtocolError);
    }
    if count_reason_string > 1 {
        return Err(MqttError::ProtocolError);
    }
    if count_server_reference > 1 {
        return Err(MqttError::ProtocolError);
    }
    Ok(())
}
