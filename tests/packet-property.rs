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
use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::prelude::*;

#[test]
fn test_all_properties() {
    let test_cases: Vec<(mqtt::packet::PropertyId, mqtt::packet::Property)> = vec![
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
            mqtt::packet::Property::ContentType(
                mqtt::packet::ContentType::new("text/plain").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ResponseTopic,
            mqtt::packet::Property::ResponseTopic(
                mqtt::packet::ResponseTopic::new("reply/topic").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::CorrelationData,
            mqtt::packet::Property::CorrelationData(
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
            mqtt::packet::Property::AssignedClientIdentifier(
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
            mqtt::packet::Property::AuthenticationMethod(
                mqtt::packet::AuthenticationMethod::new("token").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::AuthenticationData,
            mqtt::packet::Property::AuthenticationData(
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
            mqtt::packet::Property::ResponseInformation(
                mqtt::packet::ResponseInformation::new("info").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ServerReference,
            mqtt::packet::Property::ServerReference(
                mqtt::packet::ServerReference::new("server").expect("valid value"),
            ),
        ),
        (
            mqtt::packet::PropertyId::ReasonString,
            mqtt::packet::Property::ReasonString(
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
            mqtt::packet::Property::UserProperty(
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
            mqtt::packet::Property::ContentType(_) => {
                assert_eq!(prop.as_str(), Some("text/plain"));
            }
            mqtt::packet::Property::ResponseTopic(_) => {
                assert_eq!(prop.as_str(), Some("reply/topic"));
            }
            mqtt::packet::Property::CorrelationData(_) => {
                assert_eq!(prop.as_bytes(), Some("binary".as_bytes()));
            }
            mqtt::packet::Property::SubscriptionIdentifier(_) => {
                assert_eq!(prop.as_u32(), Some(456));
            }
            mqtt::packet::Property::SessionExpiryInterval(_) => {
                assert_eq!(prop.as_u32(), Some(300));
            }
            mqtt::packet::Property::AssignedClientIdentifier(_) => {
                assert_eq!(prop.as_str(), Some("client-id"));
            }
            mqtt::packet::Property::ServerKeepAlive(_) => {
                assert_eq!(prop.as_u16(), Some(120));
            }
            mqtt::packet::Property::AuthenticationMethod(_) => {
                assert_eq!(prop.as_str(), Some("token"));
            }
            mqtt::packet::Property::AuthenticationData(_) => {
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
            mqtt::packet::Property::ResponseInformation(_) => {
                assert_eq!(prop.as_str(), Some("info"));
            }
            mqtt::packet::Property::ServerReference(_) => {
                assert_eq!(prop.as_str(), Some("server"));
            }
            mqtt::packet::Property::ReasonString(_) => {
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
            mqtt::packet::Property::UserProperty(_) => {
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
            mqtt::packet::Property::ContentType(p) => {
                assert_eq!(p.val(), "text/plain");
            }
            mqtt::packet::Property::UserProperty(p) => {
                assert_eq!(p.key(), "k");
                assert_eq!(p.val(), "v");
            }
            // Others omitted...
            _ => {}
        }

        // Test serialization and parsing
        let buffers = prop.to_buffers();
        let concatenated: Vec<u8> = buffers.iter().flat_map(|s| s.as_ref().to_vec()).collect();
        let (parsed_prop, parsed_len) = mqtt::packet::Property::parse(&concatenated).unwrap();
        assert_eq!(parsed_len, concatenated.len());
        assert_eq!(parsed_prop, prop);
    }
}

// Test new trait methods
#[test]
fn test_property_type_access() {
    // Test u8 values
    let max_qos =
        mqtt::packet::Property::MaximumQos(mqtt::packet::MaximumQos::new(1).expect("valid value"));
    assert_eq!(max_qos.as_u8(), Some(1));
    assert_eq!(max_qos.as_u16(), None); // Inappropriate type access returns None

    // Test u16 values
    let topic_alias =
        mqtt::packet::Property::TopicAlias(mqtt::packet::TopicAlias::new(5).expect("valid value"));
    assert_eq!(topic_alias.as_u16(), Some(5));
    assert_eq!(topic_alias.as_u8(), None);
    assert_eq!(topic_alias.as_u32(), None);

    // Test string values
    let content_type = mqtt::packet::Property::ContentType(
        mqtt::packet::ContentType::new("application/json").expect("valid value"),
    );
    assert_eq!(content_type.as_str(), Some("application/json"));
    assert_eq!(content_type.as_u8(), None);

    // Test UserProperty
    let user_prop = mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("name", "value").expect("valid value"),
    );
    assert_eq!(user_prop.as_key_value(), Some(("name", "value")));
    assert_eq!(user_prop.as_str(), None);
}
