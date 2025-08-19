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

use crate::mqtt::packet::packet_type::{FixedHeader, PacketType};
use crate::mqtt::packet::property::PropertiesToContinuousBuffer;
use crate::mqtt::packet::variable_byte_integer::VariableByteInteger;
use crate::mqtt::packet::GenericPacketDisplay;
use crate::mqtt::packet::GenericPacketTrait;
#[cfg(feature = "std")]
use crate::mqtt::packet::PropertiesToBuffers;
use crate::mqtt::packet::{Properties, PropertiesParse, PropertiesSize, Property};
use crate::mqtt::result_code::AuthReasonCode;
use crate::mqtt::result_code::MqttError;

/// MQTT v5.0 AUTH packet representation
///
/// The AUTH packet is sent from the Client to the Server or from the Server to the Client
/// as part of an extended authentication exchange, such as challenge/response authentication.
/// It is also used by the Client or Server to perform re-authentication during a connection.
///
/// The AUTH packet enables enhanced authentication flows beyond the basic username/password
/// mechanism provided in the CONNECT packet. It supports:
/// - SASL-based authentication mechanisms
/// - Challenge/response authentication flows
/// - Re-authentication during an active connection
/// - Custom authentication methods
///
/// # Packet Structure
///
/// The AUTH packet consists of:
/// - Fixed Header (1 byte): Packet type (0xF0) and flags
/// - Variable Header:
///   - Reason Code (1 byte, optional)
///   - Properties (variable length, optional if reason code present)
///
/// # Authentication Flow
///
/// Enhanced authentication is a challenge/response style authentication that can extend
/// beyond the initial connection. The flow typically involves:
/// 1. Client sends CONNECT with Authentication Method property
/// 2. Server responds with CONNACK containing Continue Authentication reason code
/// 3. Multiple AUTH packet exchanges between client and server
/// 4. Final CONNACK with Success or failure reason code
///
/// # Re-authentication
///
/// Either the Client or Server can initiate re-authentication at any time during the
/// connection by sending an AUTH packet with a reason code of Re-authenticate.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
/// use mqtt_protocol_core::mqtt::packet::Properties;
/// use mqtt_protocol_core::mqtt::packet::Property;
///
/// // Create an AUTH packet for continue authentication
/// let auth = mqtt::packet::v5_0::Auth::builder()
///     .reason_code(AuthReasonCode::ContinueAuthentication)
///     .props(Properties::from_vec(vec![
///         Property::AuthenticationMethod("SCRAM-SHA-256".into()),
///         Property::AuthenticationData(b"challenge_data".to_vec()),
///     ]))
///     .build()
///     .unwrap();
///
/// // Create an AUTH packet for re-authentication
/// let reauth = mqtt::packet::v5_0::Auth::builder()
///     .reason_code(AuthReasonCode::ReAuthenticate)
///     .build()
///     .unwrap();
/// ```
#[derive(PartialEq, Eq, Builder, Clone, Getters, CopyGetters)]
#[builder(no_std, derive(Debug), pattern = "owned", setter(into), build_fn(skip))]
pub struct Auth {
    /// Fixed header containing packet type and flags
    #[builder(private)]
    fixed_header: [u8; 1],
    /// Remaining length of the variable header and payload
    #[builder(private)]
    remaining_length: VariableByteInteger,
    /// Reason code buffer (1 byte if present)
    #[builder(private)]
    reason_code_buf: Option<[u8; 1]>,
    /// Property length as variable byte integer
    #[builder(private)]
    property_length: Option<VariableByteInteger>,

    /// MQTT v5.0 properties for the AUTH packet
    ///
    /// Valid properties for AUTH packets include:
    /// - Authentication Method: Specifies the authentication method being used
    /// - Authentication Data: Contains method-specific authentication data
    /// - Reason String: Human-readable string describing the reason
    /// - User Property: Application-specific key-value pairs
    #[builder(setter(into, strip_option))]
    #[getset(get = "pub")]
    pub props: Option<Properties>,
}

impl Auth {
    /// Create a new AuthBuilder for constructing AUTH packets
    ///
    /// Returns a builder instance that can be used to configure and construct
    /// an AUTH packet with the desired properties and reason code.
    ///
    /// # Returns
    ///
    /// * `AuthBuilder` - A new builder instance
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
    ///
    /// let auth = mqtt::packet::v5_0::Auth::builder()
    ///     .reason_code(AuthReasonCode::ContinueAuthentication)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> AuthBuilder {
        AuthBuilder::default()
    }

    /// Get the packet type for AUTH packets
    ///
    /// Returns the fixed packet type identifier for AUTH packets as defined
    /// in the MQTT v5.0 specification.
    ///
    /// # Returns
    ///
    /// * `PacketType::Auth` - The AUTH packet type constant
    pub fn packet_type() -> PacketType {
        PacketType::Auth
    }

    /// Get the reason code from the AUTH packet
    ///
    /// Extracts and parses the reason code from the packet buffer. The reason code
    /// indicates the purpose or result of the authentication operation.
    ///
    /// # Returns
    ///
    /// * `Some(AuthReasonCode)` - The parsed reason code if present and valid
    /// * `None` - If no reason code is present in the packet
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
    ///
    /// let auth = mqtt::packet::v5_0::Auth::builder()
    ///     .reason_code(AuthReasonCode::Success)
    ///     .build()
    ///     .unwrap();
    ///
    /// assert_eq!(auth.reason_code(), Some(AuthReasonCode::Success));
    /// ```
    pub fn reason_code(&self) -> Option<AuthReasonCode> {
        self.reason_code_buf
            .as_ref()
            .and_then(|buf| AuthReasonCode::try_from(buf[0]).ok())
    }

    /// Calculate the total size of the AUTH packet in bytes
    ///
    /// Returns the complete size of the packet including the fixed header,
    /// remaining length field, reason code (if present), property length
    /// (if present), and all properties.
    ///
    /// # Returns
    ///
    /// * `usize` - Total packet size in bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let auth = mqtt::packet::v5_0::Auth::builder().build().unwrap();
    /// let size = auth.size();
    /// println!("AUTH packet size: {} bytes", size);
    /// ```
    pub fn size(&self) -> usize {
        1 + self.remaining_length.size() + self.remaining_length.to_u32() as usize
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// represent the complete AUTH packet in wire format.
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
    /// let auth = mqtt::packet::v5_0::Auth::builder().build().unwrap();
    /// let buffers = auth.to_buffers();
    /// // Use with vectored write: socket.write_vectored(&buffers)?;
    /// ```
    #[cfg(feature = "std")]
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

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method provides an alternative to `to_buffers()` for no-std environments
    /// where vectored I/O is not available.
    ///
    /// The returned buffer contains the complete AUTH packet serialized according
    /// to the MQTT v5.0 protocol specification, including fixed header, remaining
    /// length, optional reason code, and optional properties.
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
    /// let auth = mqtt::packet::v5_0::Auth::builder().build().unwrap();
    /// let buffer = auth.to_continuous_buffer();
    /// // buffer contains all packet bytes
    /// ```
    ///
    /// [`to_buffers()`]: #method.to_buffers
    pub fn to_continuous_buffer(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.fixed_header);
        buf.extend_from_slice(self.remaining_length.as_bytes());
        if let Some(rc_buf) = &self.reason_code_buf {
            buf.extend_from_slice(rc_buf);
        }
        if let Some(pl) = &self.property_length {
            buf.extend_from_slice(pl.as_bytes());
        }
        if let Some(ref props) = self.props {
            buf.append(&mut props.to_continuous_buffer());
        }
        buf
    }

    /// Parse an AUTH packet from raw bytes
    ///
    /// Parses the variable header portion of an AUTH packet from the provided byte buffer.
    /// This function expects the fixed header to have already been parsed and removed.
    ///
    /// The parsing process validates:
    /// - Reason code validity (if present)
    /// - Property structure and content
    /// - Required property relationships (e.g., Authentication Data requires Authentication Method)
    /// - Protocol compliance for reason code and property combinations
    ///
    /// # Parameters
    ///
    /// * `data` - Byte slice containing the AUTH packet variable header (excluding fixed header)
    ///
    /// # Returns
    ///
    /// * `Ok((Auth, usize))` - Successfully parsed AUTH packet and number of bytes consumed
    /// * `Err(MqttError::MalformedPacket)` - If the packet structure is invalid
    /// * `Err(MqttError::ProtocolError)` - If the packet violates MQTT protocol rules
    ///
    /// # Protocol Rules
    ///
    /// - If reason code is not Success, Authentication Method property is required
    /// - Authentication Data property requires Authentication Method property
    /// - Properties must not contain duplicates (except User Property)
    /// - Only specific properties are allowed in AUTH packets
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let data = &[0x00, 0x00]; // Minimal AUTH packet: Success reason code + empty properties
    /// let (auth_packet, consumed) = mqtt::packet::v5_0::Auth::parse(data)?;
    /// assert_eq!(consumed, 2);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;

        // reason_code
        let reason_code_buf = if cursor < data.len() {
            let rc = data[cursor];
            let _ = AuthReasonCode::try_from(rc).map_err(|_| MqttError::MalformedPacket)?;
            cursor += 1;
            Some([rc])
        } else {
            None
        };

        // properties
        let (property_length, props) = if reason_code_buf.is_some() && cursor < data.len() {
            let (props, consumed) = Properties::parse(&data[cursor..])?;
            cursor += consumed;
            let prop_len = VariableByteInteger::from_u32(props.size() as u32).unwrap();

            (Some(prop_len), Some(props))
        } else {
            (None, None)
        };

        // Validate the combination of reason code and properties
        let reason_code = reason_code_buf
            .as_ref()
            .and_then(|buf| AuthReasonCode::try_from(buf[0]).ok());
        validate_auth_packet(reason_code, &props)?;

        let remaining_size = reason_code_buf.as_ref().map_or(0, |_| 1)
            + property_length.as_ref().map_or(0, |pl| pl.size())
            + props.as_ref().map_or(0, |ps| ps.size());

        let auth = Auth {
            fixed_header: [FixedHeader::Auth.as_u8()],
            remaining_length: VariableByteInteger::from_u32(remaining_size as u32).unwrap(),
            reason_code_buf,
            property_length,
            props,
        };

        Ok((auth, cursor))
    }
}

/// Builder implementation for AUTH packets
///
/// The `AuthBuilder` provides a fluent interface for constructing AUTH packets
/// with proper validation of MQTT v5.0 protocol requirements.
impl AuthBuilder {
    /// Validate the current builder state against MQTT protocol rules
    ///
    /// Performs comprehensive validation of the AUTH packet configuration to ensure
    /// it complies with MQTT v5.0 specification requirements.
    ///
    /// # Validation Rules
    ///
    /// - Properties cannot be present without a reason code
    /// - If reason code is not Success, Authentication Method property is required
    /// - Authentication Data property requires Authentication Method property
    /// - Properties must not contain invalid duplicates
    /// - Only valid properties are allowed in AUTH packets
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration is valid
    /// * `Err(MqttError::MalformedPacket)` - If the packet structure is invalid
    /// * `Err(MqttError::ProtocolError)` - If the packet violates protocol rules
    fn validate(&self) -> Result<(), MqttError> {
        if self.reason_code_buf.is_none() && self.props.is_some() {
            return Err(MqttError::MalformedPacket);
        }
        if self.props.is_some() {
            // Properties validation is done in validate_auth_packet
        }

        // Validate the combination of reason code and properties
        let reason_code = self
            .reason_code_buf
            .as_ref()
            .and_then(|opt| opt.as_ref())
            .and_then(|buf| AuthReasonCode::try_from(buf[0]).ok());
        let props = self.props.as_ref().and_then(|p| p.as_ref()).cloned();
        validate_auth_packet(reason_code, &props)?;

        Ok(())
    }

    /// Set the reason code for the AUTH packet
    ///
    /// Sets the reason code that indicates the purpose or result of the authentication
    /// operation. When a reason code is set, empty properties are automatically added
    /// if no properties were previously configured.
    ///
    /// # Parameters
    ///
    /// * `rc` - The authentication reason code to set
    ///
    /// # Returns
    ///
    /// * `Self` - The builder instance for method chaining
    ///
    /// # Valid Reason Codes
    ///
    /// - `Success` (0x00): Authentication successful
    /// - `ContinueAuthentication` (0x18): Continue the authentication process
    /// - `ReAuthenticate` (0x19): Initiate re-authentication
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
    ///
    /// let auth = mqtt::packet::v5_0::Auth::builder()
    ///     .reason_code(AuthReasonCode::ContinueAuthentication)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn reason_code(mut self, rc: AuthReasonCode) -> Self {
        self.reason_code_buf = Some(Some([rc as u8]));
        // For AUTH packets, if reason_code is set and props is not already set, set empty props
        if self.props.is_none() {
            self.props = Some(Some(Properties::new()));
        }
        self
    }

    /// Build the AUTH packet from the current configuration
    ///
    /// Validates the current builder state and constructs the final AUTH packet.
    /// This method performs comprehensive validation to ensure the packet complies
    /// with MQTT v5.0 protocol requirements.
    ///
    /// # Returns
    ///
    /// * `Ok(Auth)` - Successfully constructed AUTH packet
    /// * `Err(MqttError::MalformedPacket)` - If the packet structure is invalid
    /// * `Err(MqttError::ProtocolError)` - If the packet violates protocol rules
    ///
    /// # Validation
    ///
    /// The build process validates:
    /// - Properties cannot exist without a reason code
    /// - Authentication property relationships are correct
    /// - No invalid property duplicates exist
    /// - Reason code and property combinations are valid
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    /// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
    /// use mqtt_protocol_core::mqtt::packet::Properties;
    /// use mqtt_protocol_core::mqtt::packet::Property;
    ///
    /// let auth = mqtt::packet::v5_0::Auth::builder()
    ///     .reason_code(AuthReasonCode::ContinueAuthentication)
    ///     .props(Properties::from_vec(vec![
    ///         Property::AuthenticationMethod("SCRAM-SHA-256".into()),
    ///         Property::AuthenticationData(b"challenge".to_vec()),
    ///     ]))
    ///     .build()?;
    /// ```
    pub fn build(self) -> Result<Auth, MqttError> {
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

        Ok(Auth {
            fixed_header: [FixedHeader::Auth.as_u8()],
            remaining_length,
            reason_code_buf,
            property_length,
            props,
        })
    }
}

/// Serialize implementation for AUTH packets
///
/// Provides JSON serialization support for AUTH packets, enabling conversion to
/// structured data formats for logging, debugging, and inter-process communication.
///
/// The serialized format includes:
/// - `type`: Always "AUTH" for packet type identification
/// - `reason_code`: The authentication reason code (if present)
/// - `props`: The MQTT v5.0 properties (if present)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
/// use serde_json;
///
/// let auth = mqtt::packet::v5_0::Auth::builder()
///     .reason_code(AuthReasonCode::Success)
///     .build()
///     .unwrap();
///
/// let json = serde_json::to_string(&auth).unwrap();
/// // Results in: {"type":"AUTH","reason_code":"Success","props":{}}
/// ```
impl Serialize for Auth {
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

        let mut state = serializer.serialize_struct("auth", field_count)?;
        state.serialize_field("type", PacketType::Auth.as_str())?;

        if self.reason_code_buf.is_some() {
            state.serialize_field("reason_code", &self.reason_code())?;
        }

        if let Some(props) = &self.props {
            state.serialize_field("props", props)?;
        }

        state.end()
    }
}

/// Display implementation for AUTH packets
///
/// Provides human-readable string representation of AUTH packets using JSON format.
/// This implementation leverages the `Serialize` trait to create consistent,
/// structured output suitable for logging and debugging.
///
/// If serialization fails, an error message is returned instead of panicking.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
///
/// let auth = mqtt::packet::v5_0::Auth::builder()
///     .reason_code(AuthReasonCode::ContinueAuthentication)
///     .build()
///     .unwrap();
///
/// println!("{}", auth);
/// // Outputs: {"type":"AUTH","reason_code":"ContinueAuthentication","props":{}}
/// ```
impl fmt::Display for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{json}"),
            Err(e) => write!(f, "{{\"error\": \"{e}\"}}"),
        }
    }
}

/// Debug implementation for AUTH packets
///
/// Provides debug output for AUTH packets by delegating to the `Display` implementation.
/// This ensures consistent formatting between debug and display representations,
/// which is useful for logging and debugging scenarios.
///
/// The debug output is identical to the display output, showing the packet
/// in JSON format with all relevant fields.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
///
/// let auth = mqtt::packet::v5_0::Auth::builder()
///     .reason_code(AuthReasonCode::Success)
///     .build()
///     .unwrap();
///
/// println!("{:?}", auth);
/// // Outputs: {"type":"AUTH","reason_code":"Success","props":{}}
/// ```
impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// GenericPacketTrait implementation for AUTH packets
///
/// Implements the common packet interface that allows AUTH packets to be used
/// polymorphically with other MQTT packet types. This trait provides standardized
/// methods for packet size calculation and buffer generation.
impl GenericPacketTrait for Auth {
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

/// GenericPacketDisplay implementation for AUTH packets
///
/// Implements the generic packet display interface that provides standardized
/// formatting capabilities for AUTH packets. This trait enables consistent
/// display and debug output across different packet types.
impl GenericPacketDisplay for Auth {
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }

    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Display::fmt(self, f)
    }
}

/// Validate AUTH packet properties and reason code combinations
///
/// Performs comprehensive validation of AUTH packet contents to ensure compliance
/// with MQTT v5.0 protocol specifications. This function validates both the
/// individual properties and their relationships with the reason code.
///
/// # Parameters
///
/// * `reason_code` - Optional authentication reason code
/// * `props` - Optional properties to validate
///
/// # Returns
///
/// * `Ok(())` - If the packet configuration is valid
/// * `Err(MqttError::ProtocolError)` - If the packet violates MQTT protocol rules
///
/// # Validation Rules
///
/// ## Property Validation
/// - Authentication Method: Maximum one occurrence allowed
/// - Authentication Data: Maximum one occurrence allowed
/// - Reason String: Maximum one occurrence allowed
/// - User Property: Multiple occurrences allowed
/// - No other properties are permitted in AUTH packets
///
/// ## Property Relationships
/// - Authentication Data requires Authentication Method to be present
/// - If reason code is not Success, Authentication Method is required
/// - If no properties are present, reason code must be Success (or absent)
///
/// ## Protocol Compliance
/// - Ensures proper authentication flow state management
/// - Validates required property combinations for extended authentication
/// - Prevents invalid property combinations that could cause protocol errors
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
/// use mqtt_protocol_core::mqtt::result_code::AuthReasonCode;
/// use mqtt_protocol_core::mqtt::packet::Properties;
/// use mqtt_protocol_core::mqtt::packet::Property;
///
/// // Valid: Success with no special properties required
/// validate_auth_packet(Some(AuthReasonCode::Success), &None)?;
///
/// // Valid: Continue authentication with method
/// let props = Properties::from_vec(vec![
///     Property::AuthenticationMethod("SCRAM-SHA-256".into())
/// ]);
/// validate_auth_packet(Some(AuthReasonCode::ContinueAuthentication), &Some(props))?;
///
/// // Invalid: Continue authentication without method
/// validate_auth_packet(Some(AuthReasonCode::ContinueAuthentication), &None); // Error
/// ```
fn validate_auth_packet(
    reason_code: Option<AuthReasonCode>,
    props: &Option<Properties>,
) -> Result<(), MqttError> {
    // Validate properties if present
    if let Some(properties) = props {
        let mut count_authentication_method = 0;
        let mut count_authentication_data = 0;
        let mut count_reason_string = 0;

        for prop in properties {
            match prop {
                Property::AuthenticationMethod(_) => count_authentication_method += 1,
                Property::AuthenticationData(_) => count_authentication_data += 1,
                Property::ReasonString(_) => count_reason_string += 1,
                Property::UserProperty(_) => {}
                _ => return Err(MqttError::ProtocolError),
            }
        }

        // Check for duplicates
        if count_authentication_method > 1 {
            return Err(MqttError::ProtocolError);
        }
        if count_authentication_data > 1 {
            return Err(MqttError::ProtocolError);
        }
        if count_reason_string > 1 {
            return Err(MqttError::ProtocolError);
        }

        // AuthenticationData requires AuthenticationMethod
        if count_authentication_data > 0 && count_authentication_method == 0 {
            return Err(MqttError::ProtocolError);
        }

        // If reason code is not Success, Authentication Method is required
        if let Some(rc) = reason_code {
            if rc != AuthReasonCode::Success && count_authentication_method == 0 {
                return Err(MqttError::ProtocolError);
            }
        }
    } else {
        // No properties case
        if let Some(rc) = reason_code {
            if rc != AuthReasonCode::Success {
                // Non-Success reason code without properties is invalid
                return Err(MqttError::ProtocolError);
            }
        }
    }

    Ok(())
}
