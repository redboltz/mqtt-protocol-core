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
use mqtt_protocol_core::mqtt;

mod common;
use mqtt_protocol_core::mqtt::prelude::*;

#[test]
fn test_all_properties() {
    common::init_tracing();
    let test_cases: Vec<(mqtt::packet::PropertyId, mqtt::packet::GenericProperty)> = vec![
        (
            mqtt::packet::PropertyId::PayloadFormatIndicator,
            mqtt::packet::Property::PayloadFormatIndicator(
                mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String)
                    .expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::MessageExpiryInterval,
            mqtt::packet::Property::MessageExpiryInterval(
                mqtt::packet::MessageExpiryInterval::new(60).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ContentType,
            mqtt::packet::Property::GenericContentType(
                mqtt::packet::ContentType::new("text/plain").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ResponseTopic,
            mqtt::packet::Property::GenericResponseTopic(
                mqtt::packet::ResponseTopic::new("reply/topic").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::CorrelationData,
            mqtt::packet::Property::GenericCorrelationData(
                mqtt::packet::CorrelationData::new("binary").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::SubscriptionIdentifier,
            mqtt::packet::Property::SubscriptionIdentifier(
                mqtt::packet::SubscriptionIdentifier::new(456).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::SessionExpiryInterval,
            mqtt::packet::Property::SessionExpiryInterval(
                mqtt::packet::SessionExpiryInterval::new(300).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::AssignedClientIdentifier,
            mqtt::packet::Property::GenericAssignedClientIdentifier(
                mqtt::packet::AssignedClientIdentifier::new("client-id").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ServerKeepAlive,
            mqtt::packet::Property::ServerKeepAlive(
                mqtt::packet::ServerKeepAlive::new(120).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::AuthenticationMethod,
            mqtt::packet::Property::GenericAuthenticationMethod(
                mqtt::packet::AuthenticationMethod::new("token").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::AuthenticationData,
            mqtt::packet::Property::GenericAuthenticationData(
                mqtt::packet::AuthenticationData::new(vec![1, 2, 3]).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::RequestProblemInformation,
            mqtt::packet::Property::RequestProblemInformation(
                mqtt::packet::RequestProblemInformation::new(1).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::WillDelayInterval,
            mqtt::packet::Property::WillDelayInterval(
                mqtt::packet::WillDelayInterval::new(10).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::RequestResponseInformation,
            mqtt::packet::Property::RequestResponseInformation(
                mqtt::packet::RequestResponseInformation::new(1).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ResponseInformation,
            mqtt::packet::Property::GenericResponseInformation(
                mqtt::packet::ResponseInformation::new("info").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ServerReference,
            mqtt::packet::Property::GenericServerReference(
                mqtt::packet::ServerReference::new("server").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ReasonString,
            mqtt::packet::Property::GenericReasonString(
                mqtt::packet::ReasonString::new("ok").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ReceiveMaximum,
            mqtt::packet::Property::ReceiveMaximum(
                mqtt::packet::ReceiveMaximum::new(10).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::TopicAliasMaximum,
            mqtt::packet::Property::TopicAliasMaximum(
                mqtt::packet::TopicAliasMaximum::new(20).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::TopicAlias,
            mqtt::packet::Property::TopicAlias(
                mqtt::packet::TopicAlias::new(1).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::MaximumQos,
            mqtt::packet::Property::MaximumQos(
                mqtt::packet::MaximumQos::new(1).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::RetainAvailable,
            mqtt::packet::Property::RetainAvailable(
                mqtt::packet::RetainAvailable::new(1).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::UserProperty,
            mqtt::packet::Property::GenericUserProperty(
                mqtt::packet::UserProperty::new("k", "v").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::MaximumPacketSize,
            mqtt::packet::Property::MaximumPacketSize(
                mqtt::packet::MaximumPacketSize::new(1024).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::WildcardSubscriptionAvailable,
            mqtt::packet::Property::WildcardSubscriptionAvailable(
                mqtt::packet::WildcardSubscriptionAvailable::new(1).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::SubscriptionIdentifierAvailable,
            mqtt::packet::Property::SubscriptionIdentifierAvailable(
                mqtt::packet::SubscriptionIdentifierAvailable::new(0).expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::SharedSubscriptionAvailable,
            mqtt::packet::Property::SharedSubscriptionAvailable(
                mqtt::packet::SharedSubscriptionAvailable::new(1).expect("valid value"),
            ),
        ),
    ];

    for (id, prop) in test_cases {
        assert_eq!(prop.id(), id);

        // Test type-specific values using PropertyValueAccess trait
        match &prop {
            mqtt::packet::Property::PayloadFormatIndicator(_) => {
                assert_eq!(prop.as_u8(), Some(1));
            }
            mqtt::packet::Property::MessageExpiryInterval(_) => {
                assert_eq!(prop.as_u32(), Some(60));
            }
            mqtt::packet::Property::GenericContentType(_) => {
                assert_eq!(prop.as_str(), Some("text/plain"));
            }
            mqtt::packet::Property::GenericResponseTopic(_) => {
                assert_eq!(prop.as_str(), Some("reply/topic"));
            }
            mqtt::packet::Property::GenericCorrelationData(_) => {
                assert_eq!(prop.as_bytes(), Some("binary".as_bytes()));
            }
            mqtt::packet::Property::SubscriptionIdentifier(_) => {
                assert_eq!(prop.as_u32(), Some(456));
            }
            mqtt::packet::Property::SessionExpiryInterval(_) => {
                assert_eq!(prop.as_u32(), Some(300));
            }
            mqtt::packet::Property::GenericAssignedClientIdentifier(_) => {
                assert_eq!(prop.as_str(), Some("client-id"));
            }
            mqtt::packet::Property::ServerKeepAlive(_) => {
                assert_eq!(prop.as_u16(), Some(120));
            }
            mqtt::packet::Property::GenericAuthenticationMethod(_) => {
                assert_eq!(prop.as_str(), Some("token"));
            }
            mqtt::packet::Property::GenericAuthenticationData(_) => {
                assert_eq!(prop.as_bytes(), Some(&[1, 2, 3][..]));
            }
            mqtt::packet::Property::RequestProblemInformation(_) => {
                assert_eq!(prop.as_u8(), Some(1));
            }
            mqtt::packet::Property::WillDelayInterval(_) => {
                assert_eq!(prop.as_u32(), Some(10));
            }
            mqtt::packet::Property::RequestResponseInformation(_) => {
                assert_eq!(prop.as_u8(), Some(1));
            }
            mqtt::packet::Property::GenericResponseInformation(_) => {
                assert_eq!(prop.as_str(), Some("info"));
            }
            mqtt::packet::Property::GenericServerReference(_) => {
                assert_eq!(prop.as_str(), Some("server"));
            }
            mqtt::packet::Property::GenericReasonString(_) => {
                assert_eq!(prop.as_str(), Some("ok"));
            }
            mqtt::packet::Property::ReceiveMaximum(_) => {
                assert_eq!(prop.as_u16(), Some(10));
            }
            mqtt::packet::Property::TopicAliasMaximum(_) => {
                assert_eq!(prop.as_u16(), Some(20));
            }
            mqtt::packet::Property::TopicAlias(_) => {
                assert_eq!(prop.as_u16(), Some(1));
            }
            mqtt::packet::Property::MaximumQos(_) => {
                assert_eq!(prop.as_u8(), Some(1));
            }
            mqtt::packet::Property::RetainAvailable(_) => {
                assert_eq!(prop.as_u8(), Some(1));
            }
            mqtt::packet::Property::GenericUserProperty(_) => {
                assert_eq!(prop.as_key_value(), Some(("k", "v")));
            }
            mqtt::packet::Property::MaximumPacketSize(_) => {
                assert_eq!(prop.as_u32(), Some(1024));
            }
            mqtt::packet::Property::WildcardSubscriptionAvailable(_) => {
                assert_eq!(prop.as_u8(), Some(1));
            }
            mqtt::packet::Property::SubscriptionIdentifierAvailable(_) => {
                assert_eq!(prop.as_u8(), Some(0));
            }
            mqtt::packet::Property::SharedSubscriptionAvailable(_) => {
                assert_eq!(prop.as_u8(), Some(1));
            }
        }

        // Also test direct access to specific property instances
        match &prop {
            mqtt::packet::Property::TopicAlias(p) => {
                assert_eq!(p.val(), 1);
            }
            mqtt::packet::Property::GenericContentType(p) => {
                assert_eq!(p.val(), "text/plain");
            }
            mqtt::packet::Property::GenericUserProperty(p) => {
                assert_eq!(p.key(), "k");
                assert_eq!(p.val(), "v");
            }
            // Others omitted...
            _ => {}
        }

        // Test serialization and parsing
        let continuous = prop.to_continuous_buffer();

        #[cfg(feature = "std")]
        {
            let buffers = prop.to_buffers();
            let concatenated: Vec<u8> = buffers.iter().flat_map(|s| s.as_ref().to_vec()).collect();
            assert_eq!(continuous, concatenated);
        }
        let (parsed_prop, parsed_len) = mqtt::packet::Property::parse(&continuous).unwrap();
        assert_eq!(parsed_len, continuous.len());
        assert_eq!(parsed_prop, prop);
    }
}

// Test PropertyId as_str method
#[test]
fn test_property_id_as_str() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::PropertyId::PayloadFormatIndicator.as_str(),
        "payload_format_indicator"
    );
    assert_eq!(
        mqtt::packet::PropertyId::MessageExpiryInterval.as_str(),
        "message_expiry_interval"
    );
    assert_eq!(
        mqtt::packet::PropertyId::ContentType.as_str(),
        "content_type"
    );
    assert_eq!(
        mqtt::packet::PropertyId::ResponseTopic.as_str(),
        "response_topic"
    );
    assert_eq!(
        mqtt::packet::PropertyId::CorrelationData.as_str(),
        "correlation_data"
    );
    assert_eq!(
        mqtt::packet::PropertyId::SubscriptionIdentifier.as_str(),
        "subscription_identifier"
    );
    assert_eq!(
        mqtt::packet::PropertyId::SessionExpiryInterval.as_str(),
        "session_expiry_interval"
    );
    assert_eq!(
        mqtt::packet::PropertyId::AssignedClientIdentifier.as_str(),
        "assigned_client_identifier"
    );
    assert_eq!(
        mqtt::packet::PropertyId::ServerKeepAlive.as_str(),
        "server_keep_alive"
    );
    assert_eq!(
        mqtt::packet::PropertyId::AuthenticationMethod.as_str(),
        "authentication_method"
    );
    assert_eq!(
        mqtt::packet::PropertyId::AuthenticationData.as_str(),
        "authentication_data"
    );
    assert_eq!(
        mqtt::packet::PropertyId::RequestProblemInformation.as_str(),
        "request_problem_information"
    );
    assert_eq!(
        mqtt::packet::PropertyId::WillDelayInterval.as_str(),
        "will_delay_interval"
    );
    assert_eq!(
        mqtt::packet::PropertyId::RequestResponseInformation.as_str(),
        "request_response_information"
    );
    assert_eq!(
        mqtt::packet::PropertyId::ResponseInformation.as_str(),
        "response_information"
    );
    assert_eq!(
        mqtt::packet::PropertyId::ServerReference.as_str(),
        "server_reference"
    );
    assert_eq!(
        mqtt::packet::PropertyId::ReasonString.as_str(),
        "reason_string"
    );
    assert_eq!(
        mqtt::packet::PropertyId::ReceiveMaximum.as_str(),
        "receive_maximum"
    );
    assert_eq!(
        mqtt::packet::PropertyId::TopicAliasMaximum.as_str(),
        "topic_alias_maximum"
    );
    assert_eq!(mqtt::packet::PropertyId::TopicAlias.as_str(), "topic_alias");
    assert_eq!(mqtt::packet::PropertyId::MaximumQos.as_str(), "maximum_qos");
    assert_eq!(
        mqtt::packet::PropertyId::RetainAvailable.as_str(),
        "retain_available"
    );
    assert_eq!(
        mqtt::packet::PropertyId::UserProperty.as_str(),
        "user_property"
    );
    assert_eq!(
        mqtt::packet::PropertyId::MaximumPacketSize.as_str(),
        "maximum_packet_size"
    );
    assert_eq!(
        mqtt::packet::PropertyId::WildcardSubscriptionAvailable.as_str(),
        "wildcard_subscription_available"
    );
    assert_eq!(
        mqtt::packet::PropertyId::SubscriptionIdentifierAvailable.as_str(),
        "subscription_identifier_available"
    );
    assert_eq!(
        mqtt::packet::PropertyId::SharedSubscriptionAvailable.as_str(),
        "shared_subscription_available"
    );
}

// Test PropertyId serialization, display, and debug
#[test]
fn test_property_id_traits() {
    common::init_tracing();
    let prop_id = mqtt::packet::PropertyId::ContentType;

    // Test Serialize trait
    let serialized = serde_json::to_string(&prop_id).unwrap();
    assert_eq!(serialized, "\"content_type\"");

    // Test Display trait
    let display_str = format!("{prop_id}");
    assert_eq!(display_str, "\"content_type\"");

    // Test Debug trait (should use Display)
    let debug_str = format!("{prop_id:?}");
    assert_eq!(debug_str, "\"content_type\"");
}

// Test PayloadFormat Display trait
#[test]
fn test_payload_format_display() {
    common::init_tracing();
    let binary_format = mqtt::packet::PayloadFormat::Binary;
    let string_format = mqtt::packet::PayloadFormat::String;

    assert_eq!(format!("{binary_format}"), "binary");
    assert_eq!(format!("{string_format}"), "string");
}

// Test new trait methods
#[test]
fn test_property_type_access() {
    common::init_tracing();
    // Test u8 values
    let max_qos: mqtt::packet::Property =
        mqtt::packet::Property::MaximumQos(mqtt::packet::MaximumQos::new(1).expect("valid value"));
    assert_eq!(max_qos.as_u8(), Some(1));
    assert_eq!(max_qos.as_u16(), None); // Inappropriate type access returns None

    // Test u16 values
    let topic_alias: mqtt::packet::Property =
        mqtt::packet::Property::TopicAlias(mqtt::packet::TopicAlias::new(5).expect("valid value"));
    assert_eq!(topic_alias.as_u16(), Some(5));
    assert_eq!(topic_alias.as_u8(), None);
    assert_eq!(topic_alias.as_u32(), None);

    // Test string values
    let content_type: mqtt::packet::Property = mqtt::packet::Property::GenericContentType(
        mqtt::packet::ContentType::new("application/json").expect("valid value"),
    );
    assert_eq!(content_type.as_str(), Some("application/json"));
    assert_eq!(content_type.as_u8(), None);

    // Test UserProperty
    let user_prop: mqtt::packet::Property = mqtt::packet::Property::GenericUserProperty(
        mqtt::packet::UserProperty::new("name", "value").expect("valid value"),
    );
    assert_eq!(user_prop.as_key_value(), Some(("name", "value")));
    assert_eq!(user_prop.as_str(), None);

    // Test additional property value access methods
    let correlation_data: mqtt::packet::Property = mqtt::packet::Property::GenericCorrelationData(
        mqtt::packet::CorrelationData::new("test_data").expect("valid value"),
    );
    assert_eq!(correlation_data.as_bytes(), Some(b"test_data" as &[u8]));
    assert_eq!(correlation_data.as_str(), None);

    let session_expiry: mqtt::packet::Property = mqtt::packet::Property::SessionExpiryInterval(
        mqtt::packet::SessionExpiryInterval::new(7200).expect("valid value"),
    );
    assert_eq!(session_expiry.as_u32(), Some(7200));
    assert_eq!(session_expiry.as_u16(), None);
    assert_eq!(session_expiry.as_u8(), None);
}

// Test PropertySize trait implementations
#[test]
fn test_property_size_trait() {
    common::init_tracing();
    use mqtt_protocol_core::mqtt::packet::VariableByteInteger;
    use mqtt_protocol_core::mqtt::prelude::PropertySize;

    // Test u8 size (line 257)
    let u8_val: u8 = 42;
    assert_eq!(u8_val.size(), 1);

    // Test u16 size (line 264)
    let u16_val: u16 = 1234;
    assert_eq!(u16_val.size(), 2);

    // Test u32 size (line 271)
    let u32_val: u32 = 123456;
    assert_eq!(u32_val.size(), 4);

    // Test String size (line 277)
    let string_val = String::from("hello");
    assert_eq!(string_val.size(), 7); // 2 bytes length + 5 bytes data

    let empty_string = String::new();
    assert_eq!(empty_string.size(), 2); // 2 bytes length + 0 bytes data

    // Test Vec<u8> size (line 284)
    let vec_val = vec![1, 2, 3, 4, 5];
    assert_eq!(vec_val.size(), 7); // 2 bytes length + 5 bytes data

    let empty_vec: Vec<u8> = Vec::new();
    assert_eq!(empty_vec.size(), 2); // 2 bytes length + 0 bytes data

    // Test VariableByteInteger size (lines 293-296)
    let vbi_1 = VariableByteInteger::from_u32(0x7F).unwrap(); // 1 byte
    assert_eq!(vbi_1.size(), 1);

    let vbi_2 = VariableByteInteger::from_u32(0x80).unwrap(); // 2 bytes
    assert_eq!(vbi_2.size(), 2);

    let vbi_3 = VariableByteInteger::from_u32(0x4000).unwrap(); // 3 bytes
    assert_eq!(vbi_3.size(), 3);

    let vbi_4 = VariableByteInteger::from_u32(0x200000).unwrap(); // 4 bytes
    assert_eq!(vbi_4.size(), 4);
}

// Test Property Display trait implementation (lines 1088-1116)
#[test]
fn test_property_display() {
    common::init_tracing();
    // Test PayloadFormatIndicator display (line 1090)
    let payload_format: mqtt::packet::Property = mqtt::packet::Property::PayloadFormatIndicator(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String).unwrap(),
    );
    let display_str = format!("{payload_format}");
    assert!(display_str.contains("payload_format_indicator"));

    // Test MessageExpiryInterval display (line 1091)
    let message_expiry: mqtt::packet::Property = mqtt::packet::Property::MessageExpiryInterval(
        mqtt::packet::MessageExpiryInterval::new(3600).unwrap(),
    );
    let display_str = format!("{message_expiry}");
    assert!(display_str.contains("message_expiry_interval"));

    // Test ContentType display (line 1092)
    let content_type: mqtt::packet::Property = mqtt::packet::Property::GenericContentType(
        mqtt::packet::ContentType::new("application/json").unwrap(),
    );
    let display_str = format!("{content_type}");
    assert!(display_str.contains("content_type"));

    // Test ResponseTopic display (line 1093)
    let response_topic: mqtt::packet::Property = mqtt::packet::Property::GenericResponseTopic(
        mqtt::packet::ResponseTopic::new("response/topic").unwrap(),
    );
    let display_str = format!("{response_topic}");
    assert!(display_str.contains("response_topic"));

    // Test CorrelationData display (line 1094)
    let correlation_data: mqtt::packet::Property = mqtt::packet::Property::GenericCorrelationData(
        mqtt::packet::CorrelationData::new("correlation").unwrap(),
    );
    let display_str = format!("{correlation_data}");
    assert!(display_str.contains("correlation_data"));

    // Test SubscriptionIdentifier display (line 1095)
    let subscription_id: mqtt::packet::Property = mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(123).unwrap(),
    );
    let display_str = format!("{subscription_id}");
    assert!(display_str.contains("subscription_identifier"));

    // Test SessionExpiryInterval display (line 1096)
    let session_expiry: mqtt::packet::Property = mqtt::packet::Property::SessionExpiryInterval(
        mqtt::packet::SessionExpiryInterval::new(7200).unwrap(),
    );
    let display_str = format!("{session_expiry}");
    assert!(display_str.contains("session_expiry_interval"));

    // Test AssignedClientIdentifier display (line 1097)
    let assigned_client_id: mqtt::packet::Property =
        mqtt::packet::Property::GenericAssignedClientIdentifier(
            mqtt::packet::AssignedClientIdentifier::new("assigned_client").unwrap(),
        );
    let display_str = format!("{assigned_client_id}");
    assert!(display_str.contains("assigned_client_identifier"));

    // Test ServerKeepAlive display (line 1098)
    let server_keep_alive: mqtt::packet::Property =
        mqtt::packet::Property::ServerKeepAlive(mqtt::packet::ServerKeepAlive::new(60).unwrap());
    let display_str = format!("{server_keep_alive}");
    assert!(display_str.contains("server_keep_alive"));

    // Test AuthenticationMethod display (line 1099)
    let auth_method: mqtt::packet::Property = mqtt::packet::Property::GenericAuthenticationMethod(
        mqtt::packet::AuthenticationMethod::new("SCRAM-SHA-1").unwrap(),
    );
    let display_str = format!("{auth_method}");
    assert!(display_str.contains("authentication_method"));

    // Test AuthenticationData display (line 1100)
    let auth_data: mqtt::packet::Property = mqtt::packet::Property::GenericAuthenticationData(
        mqtt::packet::AuthenticationData::new(vec![1, 2, 3]).unwrap(),
    );
    let display_str = format!("{auth_data}");
    assert!(display_str.contains("authentication_data"));

    // Test RequestProblemInformation display (line 1101)
    let request_problem_info: mqtt::packet::Property =
        mqtt::packet::Property::RequestProblemInformation(
            mqtt::packet::RequestProblemInformation::new(1).unwrap(),
        );
    let display_str = format!("{request_problem_info}");
    assert!(display_str.contains("request_problem_information"));

    // Test WillDelayInterval display (line 1102)
    let will_delay: mqtt::packet::Property = mqtt::packet::Property::WillDelayInterval(
        mqtt::packet::WillDelayInterval::new(30).unwrap(),
    );
    let display_str = format!("{will_delay}");
    assert!(display_str.contains("will_delay_interval"));

    // Test RequestResponseInformation display (line 1103)
    let request_response_info: mqtt::packet::Property =
        mqtt::packet::Property::RequestResponseInformation(
            mqtt::packet::RequestResponseInformation::new(1).unwrap(),
        );
    let display_str = format!("{request_response_info}");
    assert!(display_str.contains("request_response_information"));

    // Test ResponseInformation display (line 1104)
    let response_info: mqtt::packet::Property = mqtt::packet::Property::GenericResponseInformation(
        mqtt::packet::ResponseInformation::new("response_info").unwrap(),
    );
    let display_str = format!("{response_info}");
    assert!(display_str.contains("response_information"));

    // Test ServerReference display (line 1105)
    let server_ref: mqtt::packet::Property = mqtt::packet::Property::GenericServerReference(
        mqtt::packet::ServerReference::new("server_reference").unwrap(),
    );
    let display_str = format!("{server_ref}");
    assert!(display_str.contains("server_reference"));

    // Test ReasonString display (line 1106)
    let reason_string: mqtt::packet::Property = mqtt::packet::Property::GenericReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    );
    let display_str = format!("{reason_string}");
    assert!(display_str.contains("reason_string"));

    // Test ReceiveMaximum display (line 1107)
    let receive_max: mqtt::packet::Property =
        mqtt::packet::Property::ReceiveMaximum(mqtt::packet::ReceiveMaximum::new(100).unwrap());
    let display_str = format!("{receive_max}");
    assert!(display_str.contains("receive_maximum"));

    // Test TopicAliasMaximum display (line 1108)
    let topic_alias_max: mqtt::packet::Property = mqtt::packet::Property::TopicAliasMaximum(
        mqtt::packet::TopicAliasMaximum::new(50).unwrap(),
    );
    let display_str = format!("{topic_alias_max}");
    assert!(display_str.contains("topic_alias_maximum"));

    // Test TopicAlias display (line 1109)
    let topic_alias: mqtt::packet::Property =
        mqtt::packet::Property::TopicAlias(mqtt::packet::TopicAlias::new(10).unwrap());
    let display_str = format!("{topic_alias}");
    assert!(display_str.contains("topic_alias"));

    // Test MaximumQos display (line 1110)
    let max_qos: mqtt::packet::Property =
        mqtt::packet::Property::MaximumQos(mqtt::packet::MaximumQos::new(1).unwrap());
    let display_str = format!("{max_qos}");
    assert!(display_str.contains("maximum_qos"));

    // Test RetainAvailable display (line 1111)
    let retain_available: mqtt::packet::Property =
        mqtt::packet::Property::RetainAvailable(mqtt::packet::RetainAvailable::new(1).unwrap());
    let display_str = format!("{retain_available}");
    assert!(display_str.contains("retain_available"));

    // Test UserProperty display (line 1112)
    let user_property: mqtt::packet::Property = mqtt::packet::Property::GenericUserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    );
    let display_str = format!("{user_property}");
    assert!(display_str.contains("user_property"));

    // Test MaximumPacketSize display (line 1113)
    let max_packet_size: mqtt::packet::Property = mqtt::packet::Property::MaximumPacketSize(
        mqtt::packet::MaximumPacketSize::new(8192).unwrap(),
    );
    let display_str = format!("{max_packet_size}");
    assert!(display_str.contains("maximum_packet_size"));

    // Test WildcardSubscriptionAvailable display (line 1114)
    let wildcard_sub_available: mqtt::packet::Property =
        mqtt::packet::Property::WildcardSubscriptionAvailable(
            mqtt::packet::WildcardSubscriptionAvailable::new(1).unwrap(),
        );
    let display_str = format!("{wildcard_sub_available}");
    assert!(display_str.contains("wildcard_subscription_available"));

    // Test SubscriptionIdentifierAvailable display (line 1115)
    let sub_id_available: mqtt::packet::Property =
        mqtt::packet::Property::SubscriptionIdentifierAvailable(
            mqtt::packet::SubscriptionIdentifierAvailable::new(0).unwrap(),
        );
    let display_str = format!("{sub_id_available}");
    assert!(display_str.contains("subscription_identifier_available"));

    // Test SharedSubscriptionAvailable display (line 1116)
    let shared_sub_available: mqtt::packet::Property =
        mqtt::packet::Property::SharedSubscriptionAvailable(
            mqtt::packet::SharedSubscriptionAvailable::new(1).unwrap(),
        );
    let display_str = format!("{shared_sub_available}");
    assert!(display_str.contains("shared_subscription_available"));
}

// Test property error handling and validation (lines 812, 813, 835, 854, etc.)
#[test]
fn test_property_error_handling() {
    common::init_tracing();
    // Test invalid MaximumQos value (should fail validation)
    let invalid_max_qos = mqtt::packet::MaximumQos::new(3);
    assert!(invalid_max_qos.is_err());
    assert_eq!(
        invalid_max_qos.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );

    // Test invalid ReceiveMaximum value (0 is invalid)
    let invalid_receive_max = mqtt::packet::ReceiveMaximum::new(0);
    assert!(invalid_receive_max.is_err());
    assert_eq!(
        invalid_receive_max.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );

    // TopicAliasMaximum allows 0 (no validator), so test a valid case instead
    let valid_topic_alias_max = mqtt::packet::TopicAliasMaximum::new(0);
    assert!(valid_topic_alias_max.is_ok());
    assert_eq!(valid_topic_alias_max.unwrap().val(), 0);

    // Test invalid TopicAlias value (0 is invalid)
    let invalid_topic_alias = mqtt::packet::TopicAlias::new(0);
    assert!(invalid_topic_alias.is_err());
    assert_eq!(
        invalid_topic_alias.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );

    // Test invalid MaximumPacketSize value (0 is invalid)
    let invalid_max_packet_size = mqtt::packet::MaximumPacketSize::new(0);
    assert!(invalid_max_packet_size.is_err());
    assert_eq!(
        invalid_max_packet_size.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );
}

// Test property vbi parsing and validation (lines 812, 813)
#[test]
fn test_subscription_identifier_vbi_handling() {
    common::init_tracing();
    // Test valid SubscriptionIdentifier
    let valid_sub_id = mqtt::packet::SubscriptionIdentifier::new(123);
    assert!(valid_sub_id.is_ok());
    let sub_id = valid_sub_id.unwrap();
    assert_eq!(sub_id.val(), 123);

    // Test edge case - maximum valid value
    let max_sub_id = mqtt::packet::SubscriptionIdentifier::new(268435455); // VBI max
    assert!(max_sub_id.is_ok());

    // Test invalid SubscriptionIdentifier (0 is invalid)
    let invalid_sub_id = mqtt::packet::SubscriptionIdentifier::new(0);
    assert!(invalid_sub_id.is_err());
    assert_eq!(
        invalid_sub_id.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );
}

// Test property string validation and edge cases
#[test]
fn test_property_string_validation() {
    common::init_tracing();
    // Test empty string properties (should be valid)
    let empty_content_type = mqtt::packet::ContentType::new("");
    assert!(empty_content_type.is_ok());
    assert_eq!(empty_content_type.unwrap().val(), "");

    let empty_response_topic: Result<mqtt::packet::ResponseTopic, _> =
        mqtt::packet::ResponseTopic::new("");
    assert!(empty_response_topic.is_ok());
    assert_eq!(empty_response_topic.unwrap().val(), "");

    // Test UserProperty with empty key/value
    let empty_key_user_prop: Result<mqtt::packet::UserProperty, _> =
        mqtt::packet::UserProperty::new("", "value");
    assert!(empty_key_user_prop.is_ok());
    let prop = empty_key_user_prop.unwrap();
    assert_eq!(prop.key(), "");
    assert_eq!(prop.val(), "value");

    let empty_value_user_prop = mqtt::packet::UserProperty::new("key", "");
    assert!(empty_value_user_prop.is_ok());
    let prop = empty_value_user_prop.unwrap();
    assert_eq!(prop.key(), "key");
    assert_eq!(prop.val(), "");
}

// Test property binary data validation
#[test]
fn test_property_binary_validation() {
    common::init_tracing();
    // Test empty binary data (should be valid)
    let empty_correlation_data = mqtt::packet::CorrelationData::new(&[]);
    assert!(empty_correlation_data.is_ok());
    assert_eq!(empty_correlation_data.unwrap().val(), &[] as &[u8]);

    let empty_auth_data = mqtt::packet::AuthenticationData::new(Vec::<u8>::new());
    assert!(empty_auth_data.is_ok());
    assert_eq!(empty_auth_data.unwrap().val(), &[] as &[u8]);

    // Test binary data with various sizes
    let small_binary = vec![1, 2, 3];
    let correlation_data = mqtt::packet::CorrelationData::new(&small_binary);
    assert!(correlation_data.is_ok());
    assert_eq!(correlation_data.unwrap().val(), &[1, 2, 3]);

    let large_binary = vec![0xAB; 1000];
    let auth_data = mqtt::packet::AuthenticationData::new(large_binary.clone());
    assert!(auth_data.is_ok());
    assert_eq!(auth_data.unwrap().val(), large_binary.as_slice());
}

// Test property size calculations for all types
#[test]
fn test_property_sizes() {
    common::init_tracing();
    // Test u8 properties
    let payload_format =
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String).unwrap();
    assert_eq!(payload_format.size(), 2); // 1 byte ID + 1 byte value

    // Test u16 properties
    let server_keep_alive = mqtt::packet::ServerKeepAlive::new(60).unwrap();
    assert_eq!(server_keep_alive.size(), 3); // 1 byte ID + 2 bytes value

    // Test u32 properties
    let message_expiry = mqtt::packet::MessageExpiryInterval::new(3600).unwrap();
    assert_eq!(message_expiry.size(), 5); // 1 byte ID + 4 bytes value

    // Test string properties
    let content_type = mqtt::packet::ContentType::new("application/json").unwrap();
    assert_eq!(content_type.size(), 1 + 2 + 16); // 1 byte ID + 2 bytes length + 16 bytes string

    // Test binary properties
    let correlation_data = mqtt::packet::CorrelationData::new(&[1, 2, 3, 4, 5]).unwrap();
    assert_eq!(correlation_data.size(), 1 + 2 + 5); // 1 byte ID + 2 bytes length + 5 bytes data

    // Test VBI properties
    let subscription_id = mqtt::packet::SubscriptionIdentifier::new(127).unwrap(); // 1-byte VBI
    assert_eq!(subscription_id.size(), 2); // 1 byte ID + 1 byte VBI

    let large_subscription_id = mqtt::packet::SubscriptionIdentifier::new(16384).unwrap(); // 3-byte VBI
    assert_eq!(large_subscription_id.size(), 4); // 1 byte ID + 3 bytes VBI

    // Test pair properties
    let user_property = mqtt::packet::UserProperty::new("key", "value").unwrap();
    let expected_size = 1 + 2 + 3 + 2 + 5; // 1 byte ID + key MqttString + value MqttString
    assert_eq!(user_property.size(), expected_size);
}

// Test VariableByteInteger size calculation edge cases (lines 293-296)
#[test]
fn test_vbi_size_edge_cases() {
    common::init_tracing();
    use mqtt_protocol_core::mqtt::packet::VariableByteInteger;

    // Test boundary values for each size category

    // 1 byte: 0..=0x7F (lines 293)
    let vbi_max_1byte = VariableByteInteger::from_u32(0x7F).unwrap(); // 127
    assert_eq!(vbi_max_1byte.size(), 1);

    // 2 bytes: 0x80..=0x3FFF (line 294)
    let vbi_min_2byte = VariableByteInteger::from_u32(0x80).unwrap(); // 128
    assert_eq!(vbi_min_2byte.size(), 2);
    let vbi_max_2byte = VariableByteInteger::from_u32(0x3FFF).unwrap(); // 16383
    assert_eq!(vbi_max_2byte.size(), 2);

    // 3 bytes: 0x4000..=0x1F_FFFF (line 295)
    let vbi_min_3byte = VariableByteInteger::from_u32(0x4000).unwrap(); // 16384
    assert_eq!(vbi_min_3byte.size(), 3);
    let vbi_max_3byte = VariableByteInteger::from_u32(0x1F_FFFF).unwrap(); // 2097151
    assert_eq!(vbi_max_3byte.size(), 3);

    // 4 bytes: everything else (line 296)
    let vbi_min_4byte = VariableByteInteger::from_u32(0x200000).unwrap(); // 2097152
    assert_eq!(vbi_min_4byte.size(), 4);
    let vbi_max_4byte = VariableByteInteger::from_u32(0x0FFF_FFFF).unwrap(); // max VBI
    assert_eq!(vbi_max_4byte.size(), 4);
}

// Test property parsing error handling (lines 659, 729, 812, 813)
#[test]
fn test_property_parsing_errors() {
    common::init_tracing();
    // Test u16 property parsing with insufficient data (line 659)
    let insufficient_u16_data = [0x21]; // Only 1 byte for ReceiveMaximum property
    let result = mqtt::packet::ReceiveMaximum::parse(&insufficient_u16_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );

    // Test u32 property parsing with insufficient data (line 729)
    let insufficient_u32_data = [0x02, 0x00, 0x01]; // Only 3 bytes for MessageExpiryInterval
    let result = mqtt::packet::MessageExpiryInterval::parse(&insufficient_u32_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );

    // Test VBI property parsing errors (lines 812, 813)
    // Create malformed VBI data that will trigger DecodeResult::Incomplete
    let incomplete_vbi_data = [0x80]; // Incomplete VBI (continuation bit set but no next byte)
    let result = mqtt::packet::SubscriptionIdentifier::parse(&incomplete_vbi_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::InsufficientBytes
    );

    // Test VBI parsing with invalid data that triggers DecodeResult::Err
    let invalid_vbi_data = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF]; // 5 bytes, too many for VBI
    let result = mqtt::packet::SubscriptionIdentifier::parse(&invalid_vbi_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::InsufficientBytes
    );
}

// Test property size methods (lines 909, 925, 977, 1000, 1011, 1022, 1324, 1330, 1340, 1341, 1342, 1430)
#[test]
fn test_property_enum_size_methods() {
    common::init_tracing();
    // Test Property enum size() method calls individual property size() methods

    // Line 1324: ServerKeepAlive size
    let server_keep_alive: mqtt::packet::Property =
        mqtt::packet::Property::ServerKeepAlive(mqtt::packet::ServerKeepAlive::new(3600).unwrap());
    assert_eq!(server_keep_alive.size(), 3); // 1 byte ID + 2 bytes value

    // Line 1330: RequestResponseInformation size
    let request_response: mqtt::packet::Property =
        mqtt::packet::Property::RequestResponseInformation(
            mqtt::packet::RequestResponseInformation::new(1).unwrap(),
        );
    assert_eq!(request_response.size(), 2); // 1 byte ID + 1 byte value

    // Line 1340: MaximumQos size
    let max_qos: mqtt::packet::Property =
        mqtt::packet::Property::MaximumQos(mqtt::packet::MaximumQos::new(1).unwrap());
    assert_eq!(max_qos.size(), 2); // 1 byte ID + 1 byte value

    // Line 1341: RetainAvailable size
    let retain_available: mqtt::packet::Property =
        mqtt::packet::Property::RetainAvailable(mqtt::packet::RetainAvailable::new(0).unwrap());
    assert_eq!(retain_available.size(), 2); // 1 byte ID + 1 byte value

    // Line 1342: UserProperty size
    let user_property: mqtt::packet::Property = mqtt::packet::Property::GenericUserProperty(
        mqtt::packet::UserProperty::new("test_key", "test_val").unwrap(),
    );
    let expected_size = 1 + 2 + 8 + 2 + 8; // 1 byte ID + key MqttString + val MqttString
    assert_eq!(user_property.size(), expected_size);

    // Line 1430: Test Property enum to_buffers method
    let content_type: mqtt::packet::Property = mqtt::packet::Property::GenericContentType(
        mqtt::packet::ContentType::new("application/json").unwrap(),
    );
    let continuous = content_type.to_continuous_buffer();
    assert_eq!(continuous.len(), content_type.size());

    #[cfg(feature = "std")]
    {
        let buffers = content_type.to_buffers();
        assert!(!buffers.is_empty());
        let total_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(total_size, content_type.size());
    }
}

// Test specific property parsing and validation error cases (lines 909, 925, 977, 1000, 1011, 1022)
#[test]
fn test_property_parsing_validation_errors() {
    common::init_tracing();
    // These lines are in property parsing where validators are called

    // Test ReceiveMaximum parsing with invalid value (0) - should trigger validator error
    let invalid_receive_max_data = [0x00, 0x00]; // Value 0 is invalid
    let result = mqtt::packet::ReceiveMaximum::parse(&invalid_receive_max_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );

    // Test TopicAlias parsing with invalid value (0) - should trigger validator error
    let invalid_topic_alias_data = [0x00, 0x00]; // Value 0 is invalid
    let result = mqtt::packet::TopicAlias::parse(&invalid_topic_alias_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );

    // Test MaximumQos parsing with invalid value (>1) - should trigger validator error
    let invalid_max_qos_data = [0x03]; // Value 3 is invalid (max is 1)
    let result = mqtt::packet::MaximumQos::parse(&invalid_max_qos_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );

    // Test MaximumPacketSize parsing with invalid value (0) - should trigger validator error
    let invalid_max_packet_data = [0x00, 0x00, 0x00, 0x00]; // Value 0 is invalid
    let result = mqtt::packet::MaximumPacketSize::parse(&invalid_max_packet_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );

    // Test SubscriptionIdentifier parsing with invalid value (0) - should trigger validator error
    let invalid_sub_id_data = [0x00]; // Value 0 is invalid for VBI
    let result = mqtt::packet::SubscriptionIdentifier::parse(&invalid_sub_id_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::ProtocolError
    );
}

// Test Property enum parse method for specific property types (lines 1651, 1656)
#[test]
fn test_property_enum_parsing() {
    common::init_tracing();
    // Test Property::parse for properties that may not be covered by other tests

    // Test parsing WildcardSubscriptionAvailable (line 1651)
    let wildcard_sub_data = [
        mqtt::packet::PropertyId::WildcardSubscriptionAvailable as u8,
        0x01, // value 1
    ];
    let (property, consumed) = mqtt::packet::Property::parse(&wildcard_sub_data).unwrap();
    assert_eq!(consumed, 2);
    match property {
        mqtt::packet::Property::WildcardSubscriptionAvailable(p) => {
            assert_eq!(p.val(), 1);
        }
        _ => panic!("Expected WildcardSubscriptionAvailable"),
    }

    // Test parsing SharedSubscriptionAvailable (line 1656)
    let shared_sub_data = [
        mqtt::packet::PropertyId::SharedSubscriptionAvailable as u8,
        0x00, // value 0
    ];
    let (property, consumed) = mqtt::packet::Property::parse(&shared_sub_data).unwrap();
    assert_eq!(consumed, 2);
    match property {
        mqtt::packet::Property::SharedSubscriptionAvailable(p) => {
            assert_eq!(p.val(), 0);
        }
        _ => panic!("Expected SharedSubscriptionAvailable"),
    }
}
