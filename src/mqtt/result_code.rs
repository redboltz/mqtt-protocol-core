use core::fmt;
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
use derive_builder::UninitializedFieldError;
use num_enum::TryFromPrimitive;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum MqttError {
    // MQTT protocol based error
    UnspecifiedError = 0x0080,
    MalformedPacket = 0x0081,
    ProtocolError = 0x0082,
    ImplementationSpecificError = 0x0083,
    UnsupportedProtocolVersion = 0x0084,
    ClientIdentifierNotValid = 0x0085,
    BadUserNameOrPassword = 0x0086,
    NotAuthorized = 0x0087,
    ServerUnavailable = 0x0088,
    ServerBusy = 0x0089,
    Banned = 0x008A,
    ServerShuttingDown = 0x008B,
    BadAuthenticationMethod = 0x008C,
    KeepAliveTimeout = 0x008D,
    SessionTakenOver = 0x008E,
    TopicFilterInvalid = 0x008F,
    TopicNameInvalid = 0x0090,
    ReceiveMaximumExceeded = 0x0093,
    TopicAliasInvalid = 0x0094,
    PacketTooLarge = 0x0095,
    MessageRateTooHigh = 0x0096,
    QuotaExceeded = 0x0097,
    AdministrativeAction = 0x0098,
    PayloadFormatInvalid = 0x0099,
    RetainNotSupported = 0x009A,
    QosNotSupported = 0x009B,
    UseAnotherServer = 0x009C,
    ServerMoved = 0x009D,
    SharedSubscriptionsNotSupported = 0x009E,
    ConnectionRateExceeded = 0x009F,
    MaximumConnectTime = 0x00A0,
    SubscriptionIdentifiersNotSupported = 0x00A1,
    WildcardSubscriptionsNotSupported = 0x00A2,

    // Library error
    PartialErrorDetected = 0x0101,
    PacketEnqueued = 0x0102,
    AllErrorDetected = 0x0180,
    PacketIdentifierFullyUsed = 0x0181,
    PacketIdentifierConflict = 0x0182,
    PacketIdentifierInvalid = 0x0183,
    PacketNotAllowedToSend = 0x0184,
    PacketNotAllowedToStore = 0x0185,
    PacketNotRegulated = 0x0186,
    InsufficientBytes = 0x0187,
    InvalidPacketForRole = 0x0188,
    VersionMismatch = 0x0189,
    PacketConversionFailed = 0x018A,
    PacketProcessFailed = 0x018B,
    ValueOutOfRange = 0x018C,
    InvalidQos = 0x018D,
}

// Implement mapping from UninitializedFieldError to MqttError
impl From<UninitializedFieldError> for MqttError {
    fn from(_: UninitializedFieldError) -> Self {
        // Map all uninitialized field errors to MalformedPacket
        MqttError::MalformedPacket
    }
}

// Implement mapping from Infallible to MqttError
impl From<std::convert::Infallible> for MqttError {
    fn from(_: std::convert::Infallible) -> Self {
        // Infallible is an error that never occurs, so this is never actually called
        unreachable!()
    }
}

impl std::fmt::Display for MqttError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::UnspecifiedError => "UnspecifiedError",
            Self::MalformedPacket => "MalformedPacket",
            Self::ProtocolError => "ProtocolError",
            Self::ImplementationSpecificError => "ImplementationSpecificError",
            Self::UnsupportedProtocolVersion => "UnsupportedProtocolVersion",
            Self::ClientIdentifierNotValid => "ClientIdentifierNotValid",
            Self::BadUserNameOrPassword => "BadUserNameOrPassword",
            Self::NotAuthorized => "NotAuthorized",
            Self::ServerUnavailable => "ServerUnavailable",
            Self::ServerBusy => "ServerBusy",
            Self::Banned => "Banned",
            Self::ServerShuttingDown => "ServerShuttingDown",
            Self::BadAuthenticationMethod => "BadAuthenticationMethod",
            Self::KeepAliveTimeout => "KeepAliveTimeout",
            Self::SessionTakenOver => "SessionTakenOver",
            Self::TopicFilterInvalid => "TopicFilterInvalid",
            Self::TopicNameInvalid => "TopicNameInvalid",
            Self::ReceiveMaximumExceeded => "ReceiveMaximumExceeded",
            Self::TopicAliasInvalid => "TopicAliasInvalid",
            Self::PacketTooLarge => "PacketTooLarge",
            Self::MessageRateTooHigh => "MessageRateTooHigh",
            Self::QuotaExceeded => "QuotaExceeded",
            Self::AdministrativeAction => "AdministrativeAction",
            Self::PayloadFormatInvalid => "PayloadFormatInvalid",
            Self::RetainNotSupported => "RetainNotSupported",
            Self::QosNotSupported => "QosNotSupported",
            Self::UseAnotherServer => "UseAnotherServer",
            Self::ServerMoved => "ServerMoved",
            Self::SharedSubscriptionsNotSupported => "SharedSubscriptionsNotSupported",
            Self::ConnectionRateExceeded => "ConnectionRateExceeded",
            Self::MaximumConnectTime => "MaximumConnectTime",
            Self::SubscriptionIdentifiersNotSupported => "SubscriptionIdentifiersNotSupported",
            Self::WildcardSubscriptionsNotSupported => "WildcardSubscriptionsNotSupported",

            Self::PartialErrorDetected => "PartialErrorDetected",
            Self::PacketEnqueued => "PacketEnqueued",
            Self::AllErrorDetected => "AllErrorDetected",
            Self::PacketIdentifierFullyUsed => "PacketIdentifierFullyUsed",
            Self::PacketIdentifierConflict => "PacketIdentifierConflict",
            Self::PacketIdentifierInvalid => "PacketIdentifierInvalid",
            Self::PacketNotAllowedToSend => "PacketNotAllowedToSend",
            Self::PacketNotAllowedToStore => "PacketNotAllowedToStore",
            Self::PacketNotRegulated => "PacketNotRegulated",
            Self::InsufficientBytes => "InsufficientBytes",
            Self::InvalidPacketForRole => "InvalidPacketForRole",
            Self::VersionMismatch => "VersionMismatch",
            Self::PacketConversionFailed => "PacketConversionFailed",
            Self::PacketProcessFailed => "PacketProcessFailed",
            Self::ValueOutOfRange => "ValueOutOfRange",
            Self::InvalidQos => "InvalidQos",
        };
        write!(f, "{s}")
    }
}

impl Serialize for MqttError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl core::convert::TryFrom<u8> for MqttError {
    type Error = u8;

    fn try_from(code: u8) -> Result<Self, Self::Error> {
        match code {
            0x80 => Ok(Self::UnspecifiedError),
            0x81 => Ok(Self::MalformedPacket),
            0x82 => Ok(Self::ProtocolError),
            0x83 => Ok(Self::ImplementationSpecificError),
            0x84 => Ok(Self::UnsupportedProtocolVersion),
            0x85 => Ok(Self::ClientIdentifierNotValid),
            0x86 => Ok(Self::BadUserNameOrPassword),
            0x87 => Ok(Self::NotAuthorized),
            0x88 => Ok(Self::ServerUnavailable),
            0x89 => Ok(Self::ServerBusy),
            0x8A => Ok(Self::Banned),
            0x8B => Ok(Self::ServerShuttingDown),
            0x8C => Ok(Self::BadAuthenticationMethod),
            0x8D => Ok(Self::KeepAliveTimeout),
            0x8E => Ok(Self::SessionTakenOver),
            0x8F => Ok(Self::TopicFilterInvalid),
            0x90 => Ok(Self::TopicNameInvalid),
            0x93 => Ok(Self::ReceiveMaximumExceeded),
            0x94 => Ok(Self::TopicAliasInvalid),
            0x95 => Ok(Self::PacketTooLarge),
            0x96 => Ok(Self::MessageRateTooHigh),
            0x97 => Ok(Self::QuotaExceeded),
            0x98 => Ok(Self::AdministrativeAction),
            0x99 => Ok(Self::PayloadFormatInvalid),
            0x9A => Ok(Self::RetainNotSupported),
            0x9B => Ok(Self::QosNotSupported),
            0x9C => Ok(Self::UseAnotherServer),
            0x9D => Ok(Self::ServerMoved),
            0x9E => Ok(Self::SharedSubscriptionsNotSupported),
            0x9F => Ok(Self::ConnectionRateExceeded),
            0xA0 => Ok(Self::MaximumConnectTime),
            0xA1 => Ok(Self::SubscriptionIdentifiersNotSupported),
            0xA2 => Ok(Self::WildcardSubscriptionsNotSupported),
            other => Err(other),
        }
    }
}

/// MQTT v3.1.1 Connect Return Code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum ConnectReturnCode {
    Accepted = 0,                    // Connection accepted (not an error)
    UnacceptableProtocolVersion = 1, // The Server does not support the level of the MQTT protocol requested by the Client
    IdentifierRejected = 2, // The Client identifier is correct UTF-8 but not allowed by the Server
    ServerUnavailable = 3, // The Network Connection has been made but the MQTT service is unavailable
    BadUserNameOrPassword = 4, // The data in the user name or password is malformed
    NotAuthorized = 5,     // The Client is not authorized to connect
}

impl ConnectReturnCode {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Accepted)
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl fmt::Display for ConnectReturnCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Accepted => "Accepted",
            Self::UnacceptableProtocolVersion => "UnacceptableProtocolVersion",
            Self::IdentifierRejected => "IdentifierRejected",
            Self::ServerUnavailable => "ServerUnavailable",
            Self::BadUserNameOrPassword => "BadUserNameOrPassword",
            Self::NotAuthorized => "NotAuthorized",
        };
        write!(f, "{s}")
    }
}

impl Serialize for ConnectReturnCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// MQTT v3.1.1 SUBACK Return Code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum SubackReturnCode {
    SuccessMaximumQos0 = 0x00, // Success with QoS0 (not an error)
    SuccessMaximumQos1 = 0x01, // Success with QoS1 (not an error)
    SuccessMaximumQos2 = 0x02, // Success with QoS2 (not an error)
    Failure = 0x80,            // Failure
}

impl SubackReturnCode {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::SuccessMaximumQos0 | Self::SuccessMaximumQos1 | Self::SuccessMaximumQos2
        )
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl fmt::Display for SubackReturnCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::SuccessMaximumQos0 => "SuccessMaximumQos0",
            Self::SuccessMaximumQos1 => "SuccessMaximumQos1",
            Self::SuccessMaximumQos2 => "SuccessMaximumQos2",
            Self::Failure => "Failure",
        };
        write!(f, "{s}")
    }
}

impl Serialize for SubackReturnCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// MQTT v5.0 Connect Reason Code (used in CONNACK)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum ConnectReasonCode {
    Success = 0x00,                     // Success (not an error)
    UnspecifiedError = 0x80,            // Unspecified error
    MalformedPacket = 0x81,             // Malformed Packet
    ProtocolError = 0x82,               // Protocol Error
    ImplementationSpecificError = 0x83, // Implementation specific error
    UnsupportedProtocolVersion = 0x84,  // Unsupported Protocol Version
    ClientIdentifierNotValid = 0x85,    // Client Identifier not valid
    BadUserNameOrPassword = 0x86,       // Bad User Name or Password
    NotAuthorized = 0x87,               // Not authorized
    ServerUnavailable = 0x88,           // Server unavailable
    ServerBusy = 0x89,                  // Server busy
    Banned = 0x8a,                      // Banned
    BadAuthenticationMethod = 0x8c,     // Bad authentication method
    TopicNameInvalid = 0x90,            // Topic Name invalid
    PacketTooLarge = 0x95,              // Packet too large
    QuotaExceeded = 0x97,               // Quota exceeded
    PayloadFormatInvalid = 0x99,        // Payload format invalid
    RetainNotSupported = 0x9a,          // Retain not supported
    QosNotSupported = 0x9b,             // QoS not supported
    UseAnotherServer = 0x9c,            // Use another server
    ServerMoved = 0x9d,                 // Server moved
    ConnectionRateExceeded = 0x9f,      // Connection rate exceeded
}

impl fmt::Display for ConnectReasonCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Success => "Success",
            Self::UnspecifiedError => "UnspecifiedError",
            Self::MalformedPacket => "MalformedPacket",
            Self::ProtocolError => "ProtocolError",
            Self::ImplementationSpecificError => "ImplementationSpecificError",
            Self::UnsupportedProtocolVersion => "UnsupportedProtocolVersion",
            Self::ClientIdentifierNotValid => "ClientIdentifierNotValid",
            Self::BadUserNameOrPassword => "BadUserNameOrPassword",
            Self::NotAuthorized => "NotAuthorized",
            Self::ServerUnavailable => "ServerUnavailable",
            Self::ServerBusy => "ServerBusy",
            Self::Banned => "Banned",
            Self::BadAuthenticationMethod => "BadAuthenticationMethod",
            Self::TopicNameInvalid => "TopicNameInvalid",
            Self::PacketTooLarge => "PacketTooLarge",
            Self::QuotaExceeded => "QuotaExceeded",
            Self::PayloadFormatInvalid => "PayloadFormatInvalid",
            Self::RetainNotSupported => "RetainNotSupported",
            Self::QosNotSupported => "QosNotSupported",
            Self::UseAnotherServer => "UseAnotherServer",
            Self::ServerMoved => "ServerMoved",
            Self::ConnectionRateExceeded => "ConnectionRateExceeded",
        };
        write!(f, "{s}")
    }
}

impl Serialize for ConnectReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<ConnectReasonCode> for MqttError {
    fn from(code: ConnectReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum DisconnectReasonCode {
    NormalDisconnection = 0x00,
    DisconnectWithWillMessage = 0x04,
    UnspecifiedError = 0x80,
    MalformedPacket = 0x81,
    ProtocolError = 0x82,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    ServerBusy = 0x89,
    ServerShuttingDown = 0x8b,
    KeepAliveTimeout = 0x8d,
    SessionTakenOver = 0x8e,
    TopicFilterInvalid = 0x8f,
    TopicNameInvalid = 0x90,
    ReceiveMaximumExceeded = 0x93,
    TopicAliasInvalid = 0x94,
    PacketTooLarge = 0x95,
    MessageRateTooHigh = 0x96,
    QuotaExceeded = 0x97,
    AdministrativeAction = 0x98,
    PayloadFormatInvalid = 0x99,
    RetainNotSupported = 0x9a,
    QosNotSupported = 0x9b,
    UseAnotherServer = 0x9c,
    ServerMoved = 0x9d,
    SharedSubscriptionsNotSupported = 0x9e,
    ConnectionRateExceeded = 0x9f,
    MaximumConnectTime = 0xa0,
    SubscriptionIdentifiersNotSupported = 0xa1,
    WildcardSubscriptionsNotSupported = 0xa2,
}

impl std::fmt::Display for DisconnectReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::NormalDisconnection => "NormalDisconnection",
            Self::DisconnectWithWillMessage => "DisconnectWithWillMessage",
            Self::UnspecifiedError => "UnspecifiedError",
            Self::MalformedPacket => "MalformedPacket",
            Self::ProtocolError => "ProtocolError",
            Self::ImplementationSpecificError => "ImplementationSpecificError",
            Self::NotAuthorized => "NotAuthorized",
            Self::ServerBusy => "ServerBusy",
            Self::ServerShuttingDown => "ServerShuttingDown",
            Self::KeepAliveTimeout => "KeepAliveTimeout",
            Self::SessionTakenOver => "SessionTakenOver",
            Self::TopicFilterInvalid => "TopicFilterInvalid",
            Self::TopicNameInvalid => "TopicNameInvalid",
            Self::ReceiveMaximumExceeded => "ReceiveMaximumExceeded",
            Self::TopicAliasInvalid => "TopicAliasInvalid",
            Self::PacketTooLarge => "PacketTooLarge",
            Self::MessageRateTooHigh => "MessageRateTooHigh",
            Self::QuotaExceeded => "QuotaExceeded",
            Self::AdministrativeAction => "AdministrativeAction",
            Self::PayloadFormatInvalid => "PayloadFormatInvalid",
            Self::RetainNotSupported => "RetainNotSupported",
            Self::QosNotSupported => "QosNotSupported",
            Self::UseAnotherServer => "UseAnotherServer",
            Self::ServerMoved => "ServerMoved",
            Self::SharedSubscriptionsNotSupported => "SharedSubscriptionsNotSupported",
            Self::ConnectionRateExceeded => "ConnectionRateExceeded",
            Self::MaximumConnectTime => "MaximumConnectTime",
            Self::SubscriptionIdentifiersNotSupported => "SubscriptionIdentifiersNotSupported",
            Self::WildcardSubscriptionsNotSupported => "WildcardSubscriptionsNotSupported",
        };
        write!(f, "{s}")
    }
}

impl Serialize for DisconnectReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<DisconnectReasonCode> for MqttError {
    fn from(code: DisconnectReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

impl From<MqttError> for DisconnectReasonCode {
    fn from(error: MqttError) -> Self {
        match error {
            MqttError::UnspecifiedError => DisconnectReasonCode::UnspecifiedError,
            MqttError::MalformedPacket => DisconnectReasonCode::MalformedPacket,
            MqttError::ProtocolError => DisconnectReasonCode::ProtocolError,
            MqttError::ImplementationSpecificError => {
                DisconnectReasonCode::ImplementationSpecificError
            }
            MqttError::NotAuthorized => DisconnectReasonCode::NotAuthorized,
            MqttError::ServerBusy => DisconnectReasonCode::ServerBusy,
            MqttError::ServerShuttingDown => DisconnectReasonCode::ServerShuttingDown,
            MqttError::KeepAliveTimeout => DisconnectReasonCode::KeepAliveTimeout,
            MqttError::SessionTakenOver => DisconnectReasonCode::SessionTakenOver,
            MqttError::TopicFilterInvalid => DisconnectReasonCode::TopicFilterInvalid,
            MqttError::TopicNameInvalid => DisconnectReasonCode::TopicNameInvalid,
            MqttError::ReceiveMaximumExceeded => DisconnectReasonCode::ReceiveMaximumExceeded,
            MqttError::TopicAliasInvalid => DisconnectReasonCode::TopicAliasInvalid,
            MqttError::PacketTooLarge => DisconnectReasonCode::PacketTooLarge,
            MqttError::MessageRateTooHigh => DisconnectReasonCode::MessageRateTooHigh,
            MqttError::QuotaExceeded => DisconnectReasonCode::QuotaExceeded,
            MqttError::AdministrativeAction => DisconnectReasonCode::AdministrativeAction,
            MqttError::PayloadFormatInvalid => DisconnectReasonCode::PayloadFormatInvalid,
            MqttError::RetainNotSupported => DisconnectReasonCode::RetainNotSupported,
            MqttError::QosNotSupported => DisconnectReasonCode::QosNotSupported,
            MqttError::UseAnotherServer => DisconnectReasonCode::UseAnotherServer,
            MqttError::ServerMoved => DisconnectReasonCode::ServerMoved,
            MqttError::SharedSubscriptionsNotSupported => {
                DisconnectReasonCode::SharedSubscriptionsNotSupported
            }
            MqttError::ConnectionRateExceeded => DisconnectReasonCode::ConnectionRateExceeded,
            MqttError::MaximumConnectTime => DisconnectReasonCode::MaximumConnectTime,
            MqttError::SubscriptionIdentifiersNotSupported => {
                DisconnectReasonCode::SubscriptionIdentifiersNotSupported
            }
            MqttError::WildcardSubscriptionsNotSupported => {
                DisconnectReasonCode::WildcardSubscriptionsNotSupported
            }
            // All other MqttError variants map to UnspecifiedError
            _ => DisconnectReasonCode::UnspecifiedError,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum SubackReasonCode {
    GrantedQos0 = 0x00,
    GrantedQos1 = 0x01,
    GrantedQos2 = 0x02,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8f,
    PacketIdentifierInUse = 0x91,
    QuotaExceeded = 0x97,
    SharedSubscriptionsNotSupported = 0x9e,
    SubscriptionIdentifiersNotSupported = 0xa1,
    WildcardSubscriptionsNotSupported = 0xa2,
}

impl SubackReasonCode {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::GrantedQos0 | Self::GrantedQos1 | Self::GrantedQos2
        )
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl std::fmt::Display for SubackReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::GrantedQos0 => "GrantedQos0",
            Self::GrantedQos1 => "GrantedQos1",
            Self::GrantedQos2 => "GrantedQos2",
            Self::UnspecifiedError => "UnspecifiedError",
            Self::ImplementationSpecificError => "ImplementationSpecificError",
            Self::NotAuthorized => "NotAuthorized",
            Self::TopicFilterInvalid => "TopicFilterInvalid",
            Self::PacketIdentifierInUse => "PacketIdentifierInUse",
            Self::QuotaExceeded => "QuotaExceeded",
            Self::SharedSubscriptionsNotSupported => "SharedSubscriptionsNotSupported",
            Self::SubscriptionIdentifiersNotSupported => "SubscriptionIdentifiersNotSupported",
            Self::WildcardSubscriptionsNotSupported => "WildcardSubscriptionsNotSupported",
        };
        write!(f, "{s}")
    }
}

impl Serialize for SubackReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<SubackReasonCode> for MqttError {
    fn from(code: SubackReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum UnsubackReasonCode {
    Success = 0x00,
    NoSubscriptionExisted = 0x11,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicFilterInvalid = 0x8f,
    PacketIdentifierInUse = 0x91,
}

impl UnsubackReasonCode {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::NoSubscriptionExisted)
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl std::fmt::Display for UnsubackReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Success => "Success",
            Self::NoSubscriptionExisted => "NoSubscriptionExisted",
            Self::UnspecifiedError => "UnspecifiedError",
            Self::ImplementationSpecificError => "ImplementationSpecificError",
            Self::NotAuthorized => "NotAuthorized",
            Self::TopicFilterInvalid => "TopicFilterInvalid",
            Self::PacketIdentifierInUse => "PacketIdentifierInUse",
        };
        write!(f, "{s}")
    }
}

impl Serialize for UnsubackReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<UnsubackReasonCode> for MqttError {
    fn from(code: UnsubackReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum PubackReasonCode {
    Success = 0x00,
    NoMatchingSubscribers = 0x10,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicNameInvalid = 0x90,
    PacketIdentifierInUse = 0x91,
    QuotaExceeded = 0x97,
    PayloadFormatInvalid = 0x99,
}

impl PubackReasonCode {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::NoMatchingSubscribers)
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl std::fmt::Display for PubackReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Success => "Success",
            Self::NoMatchingSubscribers => "NoMatchingSubscribers",
            Self::UnspecifiedError => "UnspecifiedError",
            Self::ImplementationSpecificError => "ImplementationSpecificError",
            Self::NotAuthorized => "NotAuthorized",
            Self::TopicNameInvalid => "TopicNameInvalid",
            Self::PacketIdentifierInUse => "PacketIdentifierInUse",
            Self::QuotaExceeded => "QuotaExceeded",
            Self::PayloadFormatInvalid => "PayloadFormatInvalid",
        };
        write!(f, "{s}")
    }
}

impl Serialize for PubackReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<PubackReasonCode> for MqttError {
    fn from(code: PubackReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum PubrecReasonCode {
    Success = 0x00,
    NoMatchingSubscribers = 0x10,
    UnspecifiedError = 0x80,
    ImplementationSpecificError = 0x83,
    NotAuthorized = 0x87,
    TopicNameInvalid = 0x90,
    PacketIdentifierInUse = 0x91,
    QuotaExceeded = 0x97,
    PayloadFormatInvalid = 0x99,
}

impl PubrecReasonCode {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success | Self::NoMatchingSubscribers)
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}
impl std::fmt::Display for PubrecReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Success => "Success",
            Self::NoMatchingSubscribers => "NoMatchingSubscribers",
            Self::UnspecifiedError => "UnspecifiedError",
            Self::ImplementationSpecificError => "ImplementationSpecificError",
            Self::NotAuthorized => "NotAuthorized",
            Self::TopicNameInvalid => "TopicNameInvalid",
            Self::PacketIdentifierInUse => "PacketIdentifierInUse",
            Self::QuotaExceeded => "QuotaExceeded",
            Self::PayloadFormatInvalid => "PayloadFormatInvalid",
        };
        write!(f, "{s}")
    }
}

impl Serialize for PubrecReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<PubrecReasonCode> for MqttError {
    fn from(code: PubrecReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum PubrelReasonCode {
    Success = 0x00,
    PacketIdentifierNotFound = 0x92,
}

impl PubrelReasonCode {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl std::fmt::Display for PubrelReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Success => "Success",
            Self::PacketIdentifierNotFound => "PacketIdentifierNotFound",
        };
        write!(f, "{s}")
    }
}

impl Serialize for PubrelReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<PubrelReasonCode> for MqttError {
    fn from(code: PubrelReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum PubcompReasonCode {
    Success = 0x00,
    PacketIdentifierNotFound = 0x92,
}

impl PubcompReasonCode {
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl std::fmt::Display for PubcompReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Success => "Success",
            Self::PacketIdentifierNotFound => "PacketIdentifierNotFound",
        };
        write!(f, "{s}")
    }
}

impl Serialize for PubcompReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<PubcompReasonCode> for MqttError {
    fn from(code: PubcompReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromPrimitive)]
#[repr(u8)]
pub enum AuthReasonCode {
    Success = 0x00,
    ContinueAuthentication = 0x18,
    ReAuthenticate = 0x19,
}

impl AuthReasonCode {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::ContinueAuthentication | Self::ReAuthenticate
        )
    }
    pub fn is_failure(&self) -> bool {
        !self.is_success()
    }
}

impl std::fmt::Display for AuthReasonCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Success => "Success",
            Self::ContinueAuthentication => "ContinueAuthentication",
            Self::ReAuthenticate => "ReAuthenticate",
        };
        write!(f, "{s}")
    }
}

impl Serialize for AuthReasonCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl From<AuthReasonCode> for MqttError {
    fn from(code: AuthReasonCode) -> Self {
        // as u8 -> TryFrom<u8> -> unwrap_or fallback
        MqttError::try_from(code as u8).unwrap_or(MqttError::ProtocolError)
    }
}
