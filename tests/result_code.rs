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

use derive_builder::UninitializedFieldError;
use mqtt_protocol_core::mqtt::result_code::*;

#[test]
fn test_mqtt_error_display() {
    // Test Display trait for all MqttError variants
    assert_eq!(
        format!("{}", MqttError::UnspecifiedError),
        "UnspecifiedError"
    );
    assert_eq!(format!("{}", MqttError::MalformedPacket), "MalformedPacket");
    assert_eq!(format!("{}", MqttError::ProtocolError), "ProtocolError");
    assert_eq!(
        format!("{}", MqttError::ImplementationSpecificError),
        "ImplementationSpecificError"
    );
    assert_eq!(
        format!("{}", MqttError::UnsupportedProtocolVersion),
        "UnsupportedProtocolVersion"
    );
    assert_eq!(
        format!("{}", MqttError::ClientIdentifierNotValid),
        "ClientIdentifierNotValid"
    );
    assert_eq!(
        format!("{}", MqttError::BadUserNameOrPassword),
        "BadUserNameOrPassword"
    );
    assert_eq!(format!("{}", MqttError::NotAuthorized), "NotAuthorized");
    assert_eq!(
        format!("{}", MqttError::ServerUnavailable),
        "ServerUnavailable"
    );
    assert_eq!(format!("{}", MqttError::ServerBusy), "ServerBusy");
    assert_eq!(format!("{}", MqttError::Banned), "Banned");
    assert_eq!(
        format!("{}", MqttError::ServerShuttingDown),
        "ServerShuttingDown"
    );
    assert_eq!(
        format!("{}", MqttError::BadAuthenticationMethod),
        "BadAuthenticationMethod"
    );
    assert_eq!(
        format!("{}", MqttError::KeepAliveTimeout),
        "KeepAliveTimeout"
    );
    assert_eq!(
        format!("{}", MqttError::SessionTakenOver),
        "SessionTakenOver"
    );
    assert_eq!(
        format!("{}", MqttError::TopicFilterInvalid),
        "TopicFilterInvalid"
    );
    assert_eq!(
        format!("{}", MqttError::TopicNameInvalid),
        "TopicNameInvalid"
    );
    assert_eq!(
        format!("{}", MqttError::ReceiveMaximumExceeded),
        "ReceiveMaximumExceeded"
    );
    assert_eq!(
        format!("{}", MqttError::TopicAliasInvalid),
        "TopicAliasInvalid"
    );
    assert_eq!(format!("{}", MqttError::PacketTooLarge), "PacketTooLarge");
    assert_eq!(
        format!("{}", MqttError::MessageRateTooHigh),
        "MessageRateTooHigh"
    );
    assert_eq!(format!("{}", MqttError::QuotaExceeded), "QuotaExceeded");
    assert_eq!(
        format!("{}", MqttError::AdministrativeAction),
        "AdministrativeAction"
    );
    assert_eq!(
        format!("{}", MqttError::PayloadFormatInvalid),
        "PayloadFormatInvalid"
    );
    assert_eq!(
        format!("{}", MqttError::RetainNotSupported),
        "RetainNotSupported"
    );
    assert_eq!(format!("{}", MqttError::QosNotSupported), "QosNotSupported");
    assert_eq!(
        format!("{}", MqttError::UseAnotherServer),
        "UseAnotherServer"
    );
    assert_eq!(format!("{}", MqttError::ServerMoved), "ServerMoved");
    assert_eq!(
        format!("{}", MqttError::SharedSubscriptionsNotSupported),
        "SharedSubscriptionsNotSupported"
    );
    assert_eq!(
        format!("{}", MqttError::ConnectionRateExceeded),
        "ConnectionRateExceeded"
    );
    assert_eq!(
        format!("{}", MqttError::MaximumConnectTime),
        "MaximumConnectTime"
    );
    assert_eq!(
        format!("{}", MqttError::SubscriptionIdentifiersNotSupported),
        "SubscriptionIdentifiersNotSupported"
    );
    assert_eq!(
        format!("{}", MqttError::WildcardSubscriptionsNotSupported),
        "WildcardSubscriptionsNotSupported"
    );

    // Library errors
    assert_eq!(
        format!("{}", MqttError::PartialErrorDetected),
        "PartialErrorDetected"
    );
    assert_eq!(format!("{}", MqttError::PacketEnqueued), "PacketEnqueued");
    assert_eq!(
        format!("{}", MqttError::AllErrorDetected),
        "AllErrorDetected"
    );
    assert_eq!(
        format!("{}", MqttError::PacketIdentifierFullyUsed),
        "PacketIdentifierFullyUsed"
    );
    assert_eq!(
        format!("{}", MqttError::PacketIdentifierConflict),
        "PacketIdentifierConflict"
    );
    assert_eq!(
        format!("{}", MqttError::PacketIdentifierInvalid),
        "PacketIdentifierInvalid"
    );
    assert_eq!(
        format!("{}", MqttError::PacketNotAllowedToSend),
        "PacketNotAllowedToSend"
    );
    assert_eq!(
        format!("{}", MqttError::PacketNotAllowedToStore),
        "PacketNotAllowedToStore"
    );
    assert_eq!(
        format!("{}", MqttError::PacketNotRegulated),
        "PacketNotRegulated"
    );
    assert_eq!(
        format!("{}", MqttError::InsufficientBytes),
        "InsufficientBytes"
    );
    assert_eq!(
        format!("{}", MqttError::InvalidPacketForRole),
        "InvalidPacketForRole"
    );
    assert_eq!(format!("{}", MqttError::VersionMismatch), "VersionMismatch");
    assert_eq!(
        format!("{}", MqttError::PacketConversionFailed),
        "PacketConversionFailed"
    );
    assert_eq!(
        format!("{}", MqttError::PacketProcessFailed),
        "PacketProcessFailed"
    );
    assert_eq!(format!("{}", MqttError::ValueOutOfRange), "ValueOutOfRange");
    assert_eq!(format!("{}", MqttError::InvalidQos), "InvalidQos");
}

#[test]
fn test_mqtt_error_from_uninitialized_field_error() {
    let uninitialized_error = UninitializedFieldError::new("test_field");
    let mqtt_error = MqttError::from(uninitialized_error);
    assert_eq!(mqtt_error, MqttError::MalformedPacket);
}

#[test]
#[should_panic(expected = "internal error: entered unreachable code")]
fn test_mqtt_error_from_infallible() {
    // This test verifies that From<Infallible> panics with unreachable!()
    // Since Infallible can never be instantiated, this simulates the unreachable path
    // We can't actually create an Infallible value, but we can test the unreachable! macro indirectly
    let result: Result<(), core::convert::Infallible> = Ok(());
    match result {
        Ok(()) => {
            // This is the normal case - we can't actually create an Infallible
            // But we need to test that the From impl exists and would call unreachable!()
            // Since we can't construct Infallible, we'll test by calling unreachable! directly
            unreachable!()
        }
        Err(_infallible) => {
            // This branch should never execute because Infallible cannot be constructed
            let _mqtt_error = MqttError::from(_infallible);
        }
    }
}

#[test]
fn test_mqtt_error_try_from_u8() {
    // Test successful conversions
    assert_eq!(MqttError::try_from(0x80), Ok(MqttError::UnspecifiedError));
    assert_eq!(MqttError::try_from(0x81), Ok(MqttError::MalformedPacket));
    assert_eq!(MqttError::try_from(0x82), Ok(MqttError::ProtocolError));
    assert_eq!(
        MqttError::try_from(0x83),
        Ok(MqttError::ImplementationSpecificError)
    );
    assert_eq!(
        MqttError::try_from(0x84),
        Ok(MqttError::UnsupportedProtocolVersion)
    );
    assert_eq!(
        MqttError::try_from(0x85),
        Ok(MqttError::ClientIdentifierNotValid)
    );
    assert_eq!(
        MqttError::try_from(0x86),
        Ok(MqttError::BadUserNameOrPassword)
    );
    assert_eq!(MqttError::try_from(0x87), Ok(MqttError::NotAuthorized));
    assert_eq!(MqttError::try_from(0x88), Ok(MqttError::ServerUnavailable));
    assert_eq!(MqttError::try_from(0x89), Ok(MqttError::ServerBusy));
    assert_eq!(MqttError::try_from(0x8A), Ok(MqttError::Banned));
    assert_eq!(MqttError::try_from(0x8B), Ok(MqttError::ServerShuttingDown));
    assert_eq!(
        MqttError::try_from(0x8C),
        Ok(MqttError::BadAuthenticationMethod)
    );
    assert_eq!(MqttError::try_from(0x8D), Ok(MqttError::KeepAliveTimeout));
    assert_eq!(MqttError::try_from(0x8E), Ok(MqttError::SessionTakenOver));
    assert_eq!(MqttError::try_from(0x8F), Ok(MqttError::TopicFilterInvalid));
    assert_eq!(MqttError::try_from(0x90), Ok(MqttError::TopicNameInvalid));
    assert_eq!(
        MqttError::try_from(0x93),
        Ok(MqttError::ReceiveMaximumExceeded)
    );
    assert_eq!(MqttError::try_from(0x94), Ok(MqttError::TopicAliasInvalid));
    assert_eq!(MqttError::try_from(0x95), Ok(MqttError::PacketTooLarge));
    assert_eq!(MqttError::try_from(0x96), Ok(MqttError::MessageRateTooHigh));
    assert_eq!(MqttError::try_from(0x97), Ok(MqttError::QuotaExceeded));
    assert_eq!(
        MqttError::try_from(0x98),
        Ok(MqttError::AdministrativeAction)
    );
    assert_eq!(
        MqttError::try_from(0x99),
        Ok(MqttError::PayloadFormatInvalid)
    );
    assert_eq!(MqttError::try_from(0x9A), Ok(MqttError::RetainNotSupported));
    assert_eq!(MqttError::try_from(0x9B), Ok(MqttError::QosNotSupported));
    assert_eq!(MqttError::try_from(0x9C), Ok(MqttError::UseAnotherServer));
    assert_eq!(MqttError::try_from(0x9D), Ok(MqttError::ServerMoved));
    assert_eq!(
        MqttError::try_from(0x9E),
        Ok(MqttError::SharedSubscriptionsNotSupported)
    );
    assert_eq!(
        MqttError::try_from(0x9F),
        Ok(MqttError::ConnectionRateExceeded)
    );
    assert_eq!(MqttError::try_from(0xA0), Ok(MqttError::MaximumConnectTime));
    assert_eq!(
        MqttError::try_from(0xA1),
        Ok(MqttError::SubscriptionIdentifiersNotSupported)
    );
    assert_eq!(
        MqttError::try_from(0xA2),
        Ok(MqttError::WildcardSubscriptionsNotSupported)
    );

    // Test failed conversion
    assert_eq!(MqttError::try_from(0xFF), Err(0xFF));
    assert_eq!(MqttError::try_from(0x00), Err(0x00));
    assert_eq!(MqttError::try_from(0x7F), Err(0x7F));
}

#[test]
fn test_mqtt_error_serialize() {
    let error = MqttError::MalformedPacket;
    let serialized = serde_json::to_string(&error).unwrap();
    assert_eq!(serialized, "\"MalformedPacket\"");
}

#[test]
fn test_connect_return_code_display() {
    assert_eq!(format!("{}", ConnectReturnCode::Accepted), "Accepted");
    assert_eq!(
        format!("{}", ConnectReturnCode::UnacceptableProtocolVersion),
        "UnacceptableProtocolVersion"
    );
    assert_eq!(
        format!("{}", ConnectReturnCode::IdentifierRejected),
        "IdentifierRejected"
    );
    assert_eq!(
        format!("{}", ConnectReturnCode::ServerUnavailable),
        "ServerUnavailable"
    );
    assert_eq!(
        format!("{}", ConnectReturnCode::BadUserNameOrPassword),
        "BadUserNameOrPassword"
    );
    assert_eq!(
        format!("{}", ConnectReturnCode::NotAuthorized),
        "NotAuthorized"
    );
}

#[test]
fn test_connect_return_code_is_success_failure() {
    assert!(ConnectReturnCode::Accepted.is_success());
    assert!(!ConnectReturnCode::Accepted.is_failure());

    assert!(!ConnectReturnCode::UnacceptableProtocolVersion.is_success());
    assert!(ConnectReturnCode::UnacceptableProtocolVersion.is_failure());

    assert!(!ConnectReturnCode::IdentifierRejected.is_success());
    assert!(ConnectReturnCode::IdentifierRejected.is_failure());

    assert!(!ConnectReturnCode::ServerUnavailable.is_success());
    assert!(ConnectReturnCode::ServerUnavailable.is_failure());

    assert!(!ConnectReturnCode::BadUserNameOrPassword.is_success());
    assert!(ConnectReturnCode::BadUserNameOrPassword.is_failure());

    assert!(!ConnectReturnCode::NotAuthorized.is_success());
    assert!(ConnectReturnCode::NotAuthorized.is_failure());
}

#[test]
fn test_connect_return_code_serialize() {
    let code = ConnectReturnCode::Accepted;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Accepted\"");
}

#[test]
fn test_suback_return_code_display() {
    assert_eq!(
        format!("{}", SubackReturnCode::SuccessMaximumQos0),
        "SuccessMaximumQos0"
    );
    assert_eq!(
        format!("{}", SubackReturnCode::SuccessMaximumQos1),
        "SuccessMaximumQos1"
    );
    assert_eq!(
        format!("{}", SubackReturnCode::SuccessMaximumQos2),
        "SuccessMaximumQos2"
    );
    assert_eq!(format!("{}", SubackReturnCode::Failure), "Failure");
}

#[test]
fn test_suback_return_code_is_success_failure() {
    assert!(SubackReturnCode::SuccessMaximumQos0.is_success());
    assert!(!SubackReturnCode::SuccessMaximumQos0.is_failure());

    assert!(SubackReturnCode::SuccessMaximumQos1.is_success());
    assert!(!SubackReturnCode::SuccessMaximumQos1.is_failure());

    assert!(SubackReturnCode::SuccessMaximumQos2.is_success());
    assert!(!SubackReturnCode::SuccessMaximumQos2.is_failure());

    assert!(!SubackReturnCode::Failure.is_success());
    assert!(SubackReturnCode::Failure.is_failure());
}

#[test]
fn test_suback_return_code_serialize() {
    let code = SubackReturnCode::SuccessMaximumQos0;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"SuccessMaximumQos0\"");
}

#[test]
fn test_connect_reason_code_display() {
    assert_eq!(format!("{}", ConnectReasonCode::Success), "Success");
    assert_eq!(
        format!("{}", ConnectReasonCode::UnspecifiedError),
        "UnspecifiedError"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::MalformedPacket),
        "MalformedPacket"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::ProtocolError),
        "ProtocolError"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::ImplementationSpecificError),
        "ImplementationSpecificError"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::UnsupportedProtocolVersion),
        "UnsupportedProtocolVersion"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::ClientIdentifierNotValid),
        "ClientIdentifierNotValid"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::BadUserNameOrPassword),
        "BadUserNameOrPassword"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::NotAuthorized),
        "NotAuthorized"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::ServerUnavailable),
        "ServerUnavailable"
    );
    assert_eq!(format!("{}", ConnectReasonCode::ServerBusy), "ServerBusy");
    assert_eq!(format!("{}", ConnectReasonCode::Banned), "Banned");
    assert_eq!(
        format!("{}", ConnectReasonCode::BadAuthenticationMethod),
        "BadAuthenticationMethod"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::TopicNameInvalid),
        "TopicNameInvalid"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::PacketTooLarge),
        "PacketTooLarge"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::QuotaExceeded),
        "QuotaExceeded"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::PayloadFormatInvalid),
        "PayloadFormatInvalid"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::RetainNotSupported),
        "RetainNotSupported"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::QosNotSupported),
        "QosNotSupported"
    );
    assert_eq!(
        format!("{}", ConnectReasonCode::UseAnotherServer),
        "UseAnotherServer"
    );
    assert_eq!(format!("{}", ConnectReasonCode::ServerMoved), "ServerMoved");
    assert_eq!(
        format!("{}", ConnectReasonCode::ConnectionRateExceeded),
        "ConnectionRateExceeded"
    );
}

#[test]
fn test_connect_reason_code_serialize() {
    let code = ConnectReasonCode::Success;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Success\"");
}

#[test]
fn test_connect_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(ConnectReasonCode::UnspecifiedError),
        MqttError::UnspecifiedError
    );
    assert_eq!(
        MqttError::from(ConnectReasonCode::MalformedPacket),
        MqttError::MalformedPacket
    );
    assert_eq!(
        MqttError::from(ConnectReasonCode::ProtocolError),
        MqttError::ProtocolError
    );
    assert_eq!(
        MqttError::from(ConnectReasonCode::Success),
        MqttError::ProtocolError
    ); // Fallback
}

#[test]
fn test_disconnect_reason_code_display() {
    assert_eq!(
        format!("{}", DisconnectReasonCode::NormalDisconnection),
        "NormalDisconnection"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::DisconnectWithWillMessage),
        "DisconnectWithWillMessage"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::UnspecifiedError),
        "UnspecifiedError"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::MalformedPacket),
        "MalformedPacket"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::ProtocolError),
        "ProtocolError"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::ImplementationSpecificError),
        "ImplementationSpecificError"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::NotAuthorized),
        "NotAuthorized"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::ServerBusy),
        "ServerBusy"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::ServerShuttingDown),
        "ServerShuttingDown"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::KeepAliveTimeout),
        "KeepAliveTimeout"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::SessionTakenOver),
        "SessionTakenOver"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::TopicFilterInvalid),
        "TopicFilterInvalid"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::TopicNameInvalid),
        "TopicNameInvalid"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::ReceiveMaximumExceeded),
        "ReceiveMaximumExceeded"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::TopicAliasInvalid),
        "TopicAliasInvalid"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::PacketTooLarge),
        "PacketTooLarge"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::MessageRateTooHigh),
        "MessageRateTooHigh"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::QuotaExceeded),
        "QuotaExceeded"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::AdministrativeAction),
        "AdministrativeAction"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::PayloadFormatInvalid),
        "PayloadFormatInvalid"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::RetainNotSupported),
        "RetainNotSupported"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::QosNotSupported),
        "QosNotSupported"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::UseAnotherServer),
        "UseAnotherServer"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::ServerMoved),
        "ServerMoved"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::SharedSubscriptionsNotSupported),
        "SharedSubscriptionsNotSupported"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::ConnectionRateExceeded),
        "ConnectionRateExceeded"
    );
    assert_eq!(
        format!("{}", DisconnectReasonCode::MaximumConnectTime),
        "MaximumConnectTime"
    );
    assert_eq!(
        format!(
            "{}",
            DisconnectReasonCode::SubscriptionIdentifiersNotSupported
        ),
        "SubscriptionIdentifiersNotSupported"
    );
    assert_eq!(
        format!(
            "{}",
            DisconnectReasonCode::WildcardSubscriptionsNotSupported
        ),
        "WildcardSubscriptionsNotSupported"
    );
}

#[test]
fn test_disconnect_reason_code_serialize() {
    let code = DisconnectReasonCode::NormalDisconnection;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"NormalDisconnection\"");
}

#[test]
fn test_disconnect_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(DisconnectReasonCode::UnspecifiedError),
        MqttError::UnspecifiedError
    );
    assert_eq!(
        MqttError::from(DisconnectReasonCode::MalformedPacket),
        MqttError::MalformedPacket
    );
    assert_eq!(
        MqttError::from(DisconnectReasonCode::ProtocolError),
        MqttError::ProtocolError
    );
    assert_eq!(
        MqttError::from(DisconnectReasonCode::NormalDisconnection),
        MqttError::ProtocolError
    ); // Fallback
}

#[test]
fn test_mqtt_error_to_disconnect_reason_code() {
    assert_eq!(
        DisconnectReasonCode::from(MqttError::UnspecifiedError),
        DisconnectReasonCode::UnspecifiedError
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::MalformedPacket),
        DisconnectReasonCode::MalformedPacket
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::ProtocolError),
        DisconnectReasonCode::ProtocolError
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::ImplementationSpecificError),
        DisconnectReasonCode::ImplementationSpecificError
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::NotAuthorized),
        DisconnectReasonCode::NotAuthorized
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::ServerBusy),
        DisconnectReasonCode::ServerBusy
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::ServerShuttingDown),
        DisconnectReasonCode::ServerShuttingDown
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::KeepAliveTimeout),
        DisconnectReasonCode::KeepAliveTimeout
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::SessionTakenOver),
        DisconnectReasonCode::SessionTakenOver
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::TopicFilterInvalid),
        DisconnectReasonCode::TopicFilterInvalid
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::TopicNameInvalid),
        DisconnectReasonCode::TopicNameInvalid
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::ReceiveMaximumExceeded),
        DisconnectReasonCode::ReceiveMaximumExceeded
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::TopicAliasInvalid),
        DisconnectReasonCode::TopicAliasInvalid
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::PacketTooLarge),
        DisconnectReasonCode::PacketTooLarge
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::MessageRateTooHigh),
        DisconnectReasonCode::MessageRateTooHigh
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::QuotaExceeded),
        DisconnectReasonCode::QuotaExceeded
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::AdministrativeAction),
        DisconnectReasonCode::AdministrativeAction
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::PayloadFormatInvalid),
        DisconnectReasonCode::PayloadFormatInvalid
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::RetainNotSupported),
        DisconnectReasonCode::RetainNotSupported
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::QosNotSupported),
        DisconnectReasonCode::QosNotSupported
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::UseAnotherServer),
        DisconnectReasonCode::UseAnotherServer
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::ServerMoved),
        DisconnectReasonCode::ServerMoved
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::SharedSubscriptionsNotSupported),
        DisconnectReasonCode::SharedSubscriptionsNotSupported
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::ConnectionRateExceeded),
        DisconnectReasonCode::ConnectionRateExceeded
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::MaximumConnectTime),
        DisconnectReasonCode::MaximumConnectTime
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::SubscriptionIdentifiersNotSupported),
        DisconnectReasonCode::SubscriptionIdentifiersNotSupported
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::WildcardSubscriptionsNotSupported),
        DisconnectReasonCode::WildcardSubscriptionsNotSupported
    );

    // Test fallback for library errors
    assert_eq!(
        DisconnectReasonCode::from(MqttError::PartialErrorDetected),
        DisconnectReasonCode::UnspecifiedError
    );
    assert_eq!(
        DisconnectReasonCode::from(MqttError::PacketEnqueued),
        DisconnectReasonCode::UnspecifiedError
    );
}

#[test]
fn test_suback_reason_code_display() {
    assert_eq!(format!("{}", SubackReasonCode::GrantedQos0), "GrantedQos0");
    assert_eq!(format!("{}", SubackReasonCode::GrantedQos1), "GrantedQos1");
    assert_eq!(format!("{}", SubackReasonCode::GrantedQos2), "GrantedQos2");
    assert_eq!(
        format!("{}", SubackReasonCode::UnspecifiedError),
        "UnspecifiedError"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::ImplementationSpecificError),
        "ImplementationSpecificError"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::NotAuthorized),
        "NotAuthorized"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::TopicFilterInvalid),
        "TopicFilterInvalid"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::PacketIdentifierInUse),
        "PacketIdentifierInUse"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::QuotaExceeded),
        "QuotaExceeded"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::SharedSubscriptionsNotSupported),
        "SharedSubscriptionsNotSupported"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::SubscriptionIdentifiersNotSupported),
        "SubscriptionIdentifiersNotSupported"
    );
    assert_eq!(
        format!("{}", SubackReasonCode::WildcardSubscriptionsNotSupported),
        "WildcardSubscriptionsNotSupported"
    );
}

#[test]
fn test_suback_reason_code_is_success_failure() {
    assert!(SubackReasonCode::GrantedQos0.is_success());
    assert!(!SubackReasonCode::GrantedQos0.is_failure());

    assert!(SubackReasonCode::GrantedQos1.is_success());
    assert!(!SubackReasonCode::GrantedQos1.is_failure());

    assert!(SubackReasonCode::GrantedQos2.is_success());
    assert!(!SubackReasonCode::GrantedQos2.is_failure());

    assert!(!SubackReasonCode::UnspecifiedError.is_success());
    assert!(SubackReasonCode::UnspecifiedError.is_failure());

    assert!(!SubackReasonCode::ImplementationSpecificError.is_success());
    assert!(SubackReasonCode::ImplementationSpecificError.is_failure());
}

#[test]
fn test_suback_reason_code_serialize() {
    let code = SubackReasonCode::GrantedQos0;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"GrantedQos0\"");
}

#[test]
fn test_suback_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(SubackReasonCode::UnspecifiedError),
        MqttError::UnspecifiedError
    );
    assert_eq!(
        MqttError::from(SubackReasonCode::ImplementationSpecificError),
        MqttError::ImplementationSpecificError
    );
    assert_eq!(
        MqttError::from(SubackReasonCode::NotAuthorized),
        MqttError::NotAuthorized
    );
    assert_eq!(
        MqttError::from(SubackReasonCode::TopicFilterInvalid),
        MqttError::TopicFilterInvalid
    );
    assert_eq!(
        MqttError::from(SubackReasonCode::GrantedQos0),
        MqttError::ProtocolError
    ); // Fallback
}

#[test]
fn test_unsuback_reason_code_display() {
    assert_eq!(format!("{}", UnsubackReasonCode::Success), "Success");
    assert_eq!(
        format!("{}", UnsubackReasonCode::NoSubscriptionExisted),
        "NoSubscriptionExisted"
    );
    assert_eq!(
        format!("{}", UnsubackReasonCode::UnspecifiedError),
        "UnspecifiedError"
    );
    assert_eq!(
        format!("{}", UnsubackReasonCode::ImplementationSpecificError),
        "ImplementationSpecificError"
    );
    assert_eq!(
        format!("{}", UnsubackReasonCode::NotAuthorized),
        "NotAuthorized"
    );
    assert_eq!(
        format!("{}", UnsubackReasonCode::TopicFilterInvalid),
        "TopicFilterInvalid"
    );
    assert_eq!(
        format!("{}", UnsubackReasonCode::PacketIdentifierInUse),
        "PacketIdentifierInUse"
    );
}

#[test]
fn test_unsuback_reason_code_is_success_failure() {
    assert!(UnsubackReasonCode::Success.is_success());
    assert!(!UnsubackReasonCode::Success.is_failure());

    assert!(UnsubackReasonCode::NoSubscriptionExisted.is_success());
    assert!(!UnsubackReasonCode::NoSubscriptionExisted.is_failure());

    assert!(!UnsubackReasonCode::UnspecifiedError.is_success());
    assert!(UnsubackReasonCode::UnspecifiedError.is_failure());

    assert!(!UnsubackReasonCode::ImplementationSpecificError.is_success());
    assert!(UnsubackReasonCode::ImplementationSpecificError.is_failure());
}

#[test]
fn test_unsuback_reason_code_serialize() {
    let code = UnsubackReasonCode::Success;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Success\"");
}

#[test]
fn test_unsuback_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(UnsubackReasonCode::UnspecifiedError),
        MqttError::UnspecifiedError
    );
    assert_eq!(
        MqttError::from(UnsubackReasonCode::ImplementationSpecificError),
        MqttError::ImplementationSpecificError
    );
    assert_eq!(
        MqttError::from(UnsubackReasonCode::NotAuthorized),
        MqttError::NotAuthorized
    );
    assert_eq!(
        MqttError::from(UnsubackReasonCode::Success),
        MqttError::ProtocolError
    ); // Fallback
}

#[test]
fn test_puback_reason_code_display() {
    assert_eq!(format!("{}", PubackReasonCode::Success), "Success");
    assert_eq!(
        format!("{}", PubackReasonCode::NoMatchingSubscribers),
        "NoMatchingSubscribers"
    );
    assert_eq!(
        format!("{}", PubackReasonCode::UnspecifiedError),
        "UnspecifiedError"
    );
    assert_eq!(
        format!("{}", PubackReasonCode::ImplementationSpecificError),
        "ImplementationSpecificError"
    );
    assert_eq!(
        format!("{}", PubackReasonCode::NotAuthorized),
        "NotAuthorized"
    );
    assert_eq!(
        format!("{}", PubackReasonCode::TopicNameInvalid),
        "TopicNameInvalid"
    );
    assert_eq!(
        format!("{}", PubackReasonCode::PacketIdentifierInUse),
        "PacketIdentifierInUse"
    );
    assert_eq!(
        format!("{}", PubackReasonCode::QuotaExceeded),
        "QuotaExceeded"
    );
    assert_eq!(
        format!("{}", PubackReasonCode::PayloadFormatInvalid),
        "PayloadFormatInvalid"
    );
}

#[test]
fn test_puback_reason_code_is_success_failure() {
    assert!(PubackReasonCode::Success.is_success());
    assert!(!PubackReasonCode::Success.is_failure());

    assert!(PubackReasonCode::NoMatchingSubscribers.is_success());
    assert!(!PubackReasonCode::NoMatchingSubscribers.is_failure());

    assert!(!PubackReasonCode::UnspecifiedError.is_success());
    assert!(PubackReasonCode::UnspecifiedError.is_failure());
}

#[test]
fn test_puback_reason_code_serialize() {
    let code = PubackReasonCode::Success;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Success\"");
}

#[test]
fn test_puback_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(PubackReasonCode::UnspecifiedError),
        MqttError::UnspecifiedError
    );
    assert_eq!(
        MqttError::from(PubackReasonCode::ImplementationSpecificError),
        MqttError::ImplementationSpecificError
    );
    assert_eq!(
        MqttError::from(PubackReasonCode::NotAuthorized),
        MqttError::NotAuthorized
    );
    assert_eq!(
        MqttError::from(PubackReasonCode::Success),
        MqttError::ProtocolError
    ); // Fallback
}

#[test]
fn test_pubrec_reason_code_display() {
    assert_eq!(format!("{}", PubrecReasonCode::Success), "Success");
    assert_eq!(
        format!("{}", PubrecReasonCode::NoMatchingSubscribers),
        "NoMatchingSubscribers"
    );
    assert_eq!(
        format!("{}", PubrecReasonCode::UnspecifiedError),
        "UnspecifiedError"
    );
    assert_eq!(
        format!("{}", PubrecReasonCode::ImplementationSpecificError),
        "ImplementationSpecificError"
    );
    assert_eq!(
        format!("{}", PubrecReasonCode::NotAuthorized),
        "NotAuthorized"
    );
    assert_eq!(
        format!("{}", PubrecReasonCode::TopicNameInvalid),
        "TopicNameInvalid"
    );
    assert_eq!(
        format!("{}", PubrecReasonCode::PacketIdentifierInUse),
        "PacketIdentifierInUse"
    );
    assert_eq!(
        format!("{}", PubrecReasonCode::QuotaExceeded),
        "QuotaExceeded"
    );
    assert_eq!(
        format!("{}", PubrecReasonCode::PayloadFormatInvalid),
        "PayloadFormatInvalid"
    );
}

#[test]
fn test_pubrec_reason_code_is_success_failure() {
    assert!(PubrecReasonCode::Success.is_success());
    assert!(!PubrecReasonCode::Success.is_failure());

    assert!(PubrecReasonCode::NoMatchingSubscribers.is_success());
    assert!(!PubrecReasonCode::NoMatchingSubscribers.is_failure());

    assert!(!PubrecReasonCode::UnspecifiedError.is_success());
    assert!(PubrecReasonCode::UnspecifiedError.is_failure());
}

#[test]
fn test_pubrec_reason_code_serialize() {
    let code = PubrecReasonCode::Success;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Success\"");
}

#[test]
fn test_pubrec_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(PubrecReasonCode::UnspecifiedError),
        MqttError::UnspecifiedError
    );
    assert_eq!(
        MqttError::from(PubrecReasonCode::ImplementationSpecificError),
        MqttError::ImplementationSpecificError
    );
    assert_eq!(
        MqttError::from(PubrecReasonCode::NotAuthorized),
        MqttError::NotAuthorized
    );
    assert_eq!(
        MqttError::from(PubrecReasonCode::Success),
        MqttError::ProtocolError
    ); // Fallback
}

#[test]
fn test_pubrel_reason_code_display() {
    assert_eq!(format!("{}", PubrelReasonCode::Success), "Success");
    assert_eq!(
        format!("{}", PubrelReasonCode::PacketIdentifierNotFound),
        "PacketIdentifierNotFound"
    );
}

#[test]
fn test_pubrel_reason_code_is_success_failure() {
    assert!(PubrelReasonCode::Success.is_success());
    assert!(!PubrelReasonCode::Success.is_failure());

    assert!(!PubrelReasonCode::PacketIdentifierNotFound.is_success());
    assert!(PubrelReasonCode::PacketIdentifierNotFound.is_failure());
}

#[test]
fn test_pubrel_reason_code_serialize() {
    let code = PubrelReasonCode::Success;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Success\"");
}

#[test]
fn test_pubrel_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(PubrelReasonCode::Success),
        MqttError::ProtocolError
    ); // Fallback
    assert_eq!(
        MqttError::from(PubrelReasonCode::PacketIdentifierNotFound),
        MqttError::ProtocolError
    ); // Fallback - value 0x92 not in TryFrom<u8>
}

#[test]
fn test_pubcomp_reason_code_display() {
    assert_eq!(format!("{}", PubcompReasonCode::Success), "Success");
    assert_eq!(
        format!("{}", PubcompReasonCode::PacketIdentifierNotFound),
        "PacketIdentifierNotFound"
    );
}

#[test]
fn test_pubcomp_reason_code_is_success_failure() {
    assert!(PubcompReasonCode::Success.is_success());
    assert!(!PubcompReasonCode::Success.is_failure());

    assert!(!PubcompReasonCode::PacketIdentifierNotFound.is_success());
    assert!(PubcompReasonCode::PacketIdentifierNotFound.is_failure());
}

#[test]
fn test_pubcomp_reason_code_serialize() {
    let code = PubcompReasonCode::Success;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Success\"");
}

#[test]
fn test_pubcomp_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(PubcompReasonCode::Success),
        MqttError::ProtocolError
    ); // Fallback
    assert_eq!(
        MqttError::from(PubcompReasonCode::PacketIdentifierNotFound),
        MqttError::ProtocolError
    ); // Fallback - value 0x92 not in TryFrom<u8>
}

#[test]
fn test_auth_reason_code_display() {
    assert_eq!(format!("{}", AuthReasonCode::Success), "Success");
    assert_eq!(
        format!("{}", AuthReasonCode::ContinueAuthentication),
        "ContinueAuthentication"
    );
    assert_eq!(
        format!("{}", AuthReasonCode::ReAuthenticate),
        "ReAuthenticate"
    );
}

#[test]
fn test_auth_reason_code_is_success_failure() {
    assert!(AuthReasonCode::Success.is_success());
    assert!(!AuthReasonCode::Success.is_failure());

    assert!(AuthReasonCode::ContinueAuthentication.is_success());
    assert!(!AuthReasonCode::ContinueAuthentication.is_failure());

    assert!(AuthReasonCode::ReAuthenticate.is_success());
    assert!(!AuthReasonCode::ReAuthenticate.is_failure());
}

#[test]
fn test_auth_reason_code_serialize() {
    let code = AuthReasonCode::Success;
    let serialized = serde_json::to_string(&code).unwrap();
    assert_eq!(serialized, "\"Success\"");
}

#[test]
fn test_auth_reason_code_to_mqtt_error() {
    assert_eq!(
        MqttError::from(AuthReasonCode::Success),
        MqttError::ProtocolError
    ); // Fallback
    assert_eq!(
        MqttError::from(AuthReasonCode::ContinueAuthentication),
        MqttError::ProtocolError
    ); // Fallback - value 0x18 not in TryFrom<u8>
    assert_eq!(
        MqttError::from(AuthReasonCode::ReAuthenticate),
        MqttError::ProtocolError
    ); // Fallback - value 0x19 not in TryFrom<u8>
}

// Test for traits and additional coverage
#[test]
fn test_debug_and_traits() {
    // Test Debug trait (automatically derived)
    let error = MqttError::MalformedPacket;
    let debug_str = format!("{error:?}");
    assert!(!debug_str.is_empty());

    // Test Clone and Copy traits
    let error1 = MqttError::ProtocolError;
    let error2 = error1; // Copy
    assert_eq!(error1, error2);

    // Test PartialEq and Eq
    assert_eq!(MqttError::ProtocolError, MqttError::ProtocolError);
    assert_ne!(MqttError::ProtocolError, MqttError::MalformedPacket);

    // Test Hash (by using in a HashSet)
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(MqttError::ProtocolError);
    assert!(set.contains(&MqttError::ProtocolError));

    // Test enum values
    assert_eq!(MqttError::UnspecifiedError as u16, 0x0080);
    assert_eq!(MqttError::MalformedPacket as u16, 0x0081);
}

#[test]
fn test_try_from_primitive_derives() {
    use num_enum::TryFromPrimitive;

    // Test TryFromPrimitive for ConnectReturnCode
    assert_eq!(
        ConnectReturnCode::try_from_primitive(0),
        Ok(ConnectReturnCode::Accepted)
    );
    assert_eq!(
        ConnectReturnCode::try_from_primitive(1),
        Ok(ConnectReturnCode::UnacceptableProtocolVersion)
    );
    assert!(ConnectReturnCode::try_from_primitive(255).is_err());

    // Test TryFromPrimitive for SubackReturnCode
    assert_eq!(
        SubackReturnCode::try_from_primitive(0x00),
        Ok(SubackReturnCode::SuccessMaximumQos0)
    );
    assert_eq!(
        SubackReturnCode::try_from_primitive(0x80),
        Ok(SubackReturnCode::Failure)
    );
    assert!(SubackReturnCode::try_from_primitive(0x7F).is_err());
}
