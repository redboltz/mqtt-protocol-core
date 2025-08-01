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
use std::fmt::Write;
use std::sync::Arc;

// Build fail tests

#[test]
fn build_fail_no_topic() {
    let err = mqtt::packet::v5_0::Publish::builder().build().unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_empty_topic_no_alias() {
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_topic_too_long() {
    let long_topic = "a".repeat(65536);
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name(long_topic.as_str())
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_topic_with_wildcard() {
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("bad+topic")
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_qos1_no_packet_id() {
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_qos0_with_packet_id() {
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_duplicate_property() {
    let mut props = mqtt::packet::Properties::new();
    props.push(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::Binary)
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String)
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_qos0_with_packet_id_validation() {
    // This tests line 306: QoS 0 with packet ID validation in validate() method
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_qos1_packet_id_zero() {
    // This tests line 315: packet ID zero validation
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(0u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_payload_too_large() {
    // This tests line 327: payload size limit validation (268435455 bytes)
    let large_payload = vec![0u8; 268435456]; // 1 byte over limit
    let err = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .payload(&large_payload)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Build success tests

#[test]
fn build_success_empty_topic_with_alias() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());

    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.topic_name(), "");
}

#[test]
fn build_success_qos2() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    assert_eq!(packet.qos(), mqtt::packet::Qos::ExactlyOnce);
    assert_eq!(packet.packet_id(), Some(1234));
}

#[test]
fn build_success_empty_payload() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .payload(())
        .build()
        .unwrap();

    assert_eq!(packet.payload().as_slice(), &[] as &[u8]);
}

// Display tests

#[test]
fn display_qos0() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert!(output.contains(r#""type":"publish""#));
    assert!(output.contains(r#""topic_name":"test/topic""#));
    assert!(output.contains(r#""qos":"AtMostOnce""#));
    assert!(output.contains(r#""payload":"hello""#));
}

#[test]
fn display_qos1_with_packet_id() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert!(output.contains(r#""type":"publish""#));
    assert!(output.contains(r#""topic_name":"test/topic""#));
    assert!(output.contains(r#""qos":"AtLeastOnce""#));
    assert!(output.contains(r#""packet_id":1234"#));
    assert!(output.contains(r#""payload":"hello""#));
}

#[test]
fn display_with_flags() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .retain(false)
        .dup(false)
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert!(output.contains(r#""retain":false"#));
    assert!(output.contains(r#""dup":false"#));
}

#[test]
fn display_binary_payload() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload(&[0x00, b'a', b'b'])
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert!(output.contains(r#""payload":"\u0000ab""#));
}

#[test]
fn display_binary_payload_array() {
    // Test payload with invalid UTF-8 bytes that should be serialized as array
    // This tests line 404: None => state.serialize_field("payload", &payload_data)?
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload(&[0x80, 0x81, 0x82, 0x83]) // Invalid UTF-8 continuation bytes
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert!(output.contains(r#""payload":[128,129,130,131]"#));
}

#[test]
fn display_with_props() {
    // This tests lines 388, 400: props field serialization
    let props = vec![
        mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into(),
        mqtt::packet::CorrelationData::new(b"12345").unwrap().into(),
        mqtt::packet::MessageExpiryInterval::new(300)
            .unwrap()
            .into(),
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String)
            .unwrap()
            .into(),
        mqtt::packet::ResponseTopic::new("response/topic")
            .unwrap()
            .into(),
        mqtt::packet::SubscriptionIdentifier::new(12345)
            .unwrap()
            .into(),
        mqtt::packet::TopicAlias::new(1).unwrap().into(),
        mqtt::packet::UserProperty::new("key", "value")
            .unwrap()
            .into(),
    ];

    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .props(props)
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert!(output.contains(r#""type":"publish""#));
    assert!(output.contains(r#""props":["#));
}

// Debug tests

#[test]
fn debug_qos0() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{:?}", packet).unwrap();
    assert!(output.contains(r#""type":"publish""#));
    assert!(output.contains(r#""topic_name":"test/topic""#));
    assert!(output.contains(r#""qos":"AtMostOnce""#));
    assert!(output.contains(r#""payload":"hello""#));
}

// Getter tests

#[test]
fn getter_qos0() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    assert_eq!(packet.topic_name(), "test/topic");
    assert_eq!(packet.qos(), mqtt::packet::Qos::AtMostOnce);
    assert_eq!(packet.packet_id(), None);
    assert!(!packet.retain());
    assert!(!packet.dup());
    assert_eq!(packet.payload().as_slice(), b"hello");
    assert!(packet.props().is_none());
}

#[test]
fn getter_qos1_with_packet_id() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    assert_eq!(packet.topic_name(), "test/topic");
    assert_eq!(packet.qos(), mqtt::packet::Qos::AtLeastOnce);
    assert_eq!(packet.packet_id(), Some(1234));
    assert!(!packet.retain());
    assert!(!packet.dup());
    assert_eq!(packet.payload().as_slice(), b"hello");
}

#[test]
fn getter_with_flags() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .retain(true)
        .dup(true)
        .payload("hello")
        .build()
        .unwrap();

    assert!(packet.retain());
    assert!(packet.dup());
}

#[test]
fn getter_with_props() {
    let mut props = mqtt::packet::Properties::new();
    props.push(
        mqtt::packet::MessageExpiryInterval::new(300)
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .props(props.clone())
        .payload("hello")
        .build()
        .unwrap();

    assert!(packet.props().is_some());
    assert_eq!(packet.props().as_ref().unwrap().len(), 1);
    match &packet.props().as_ref().unwrap()[0] {
        mqtt::packet::Property::MessageExpiryInterval(m) => {
            assert_eq!(m.val(), 300);
        }
        _ => panic!("unexpected property variant"),
    }
}

// to_buffers() tests

#[test]
fn to_buffers_qos0() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers[0].as_ref(), &[0x30]); // fixed header (QoS 0)
    assert_eq!(buffers[1].as_ref(), &[0x0C]); // remaining length (12)

    // topic length (2) + topic (4) + property length (1) + payload (5) = 12
    assert_eq!(packet.size(), 1 + 1 + 12); // fixed header + remaining length + payload
}

#[test]
fn to_buffers_qos1() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers[0].as_ref(), &[0x32]); // fixed header (QoS 1)
    assert_eq!(buffers[1].as_ref(), &[0x0E]); // remaining length (14)

    // topic length (2) + topic (4) + packet_id (2) + property length (1) + payload (5) = 14
    assert_eq!(packet.size(), 1 + 1 + 14); // fixed header + remaining length + payload
}

#[test]
fn to_buffers_with_flags() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .retain(true)
        .dup(true)
        .payload("hello")
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers[0].as_ref(), &[0x39]); // fixed header with DUP and RETAIN flags
}

#[test]
fn to_buffers_with_props() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::MessageExpiryInterval(
        mqtt::packet::MessageExpiryInterval::new(300).unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .props(props)
        .payload("hello")
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    // Property length should be 5 bytes (1 byte property ID + 4 bytes value)
    // Total: topic length (2) + topic (4) + property length (1) + property (5) + payload (5) = 17
    assert_eq!(buffers[1].as_ref(), &[0x11]); // remaining length (17)
}

// Parse tests

#[test]
fn parse_empty() {
    let empty_arc: Arc<[u8]> = Arc::from(Vec::new().into_boxed_slice());
    let err = mqtt::packet::v5_0::Publish::parse(0, empty_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_topic_incomplete() {
    let raw = vec![0x00]; // incomplete topic length
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let err = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_qos() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(0u16).to_be_bytes()); // empty topic
    raw.push(0x00); // property length 0
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());

    let err = mqtt::packet::v5_0::Publish::parse(0b0000_11 << 1, data_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_qos0() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x00); // property length 0
    raw.extend_from_slice(b"hello"); // payload

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap();

    assert_eq!(consumed, 12); // 2 + 4 + 1 + 5
    assert_eq!(packet.topic_name(), "test");
    assert_eq!(packet.qos(), mqtt::packet::Qos::AtMostOnce);
    assert_eq!(packet.packet_id(), None);
    assert_eq!(packet.payload().as_slice(), b"hello");
}

#[test]
fn parse_qos1() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.extend_from_slice(&1234u16.to_be_bytes()); // packet id
    raw.push(0x00); // property length 0
    raw.extend_from_slice(b"hello"); // payload

    let flags = (mqtt::packet::Qos::AtLeastOnce as u8) << 1;
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v5_0::Publish::parse(flags, data_arc).unwrap();

    assert_eq!(consumed, 14); // 2 + 4 + 2 + 1 + 5
    assert_eq!(packet.topic_name(), "test");
    assert_eq!(packet.qos(), mqtt::packet::Qos::AtLeastOnce);
    assert_eq!(packet.packet_id(), Some(1234));
    assert_eq!(packet.payload().as_slice(), b"hello");
}

#[test]
fn parse_qos2() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.extend_from_slice(&1234u16.to_be_bytes()); // packet id
    raw.push(0x00); // property length 0
    raw.extend_from_slice(b"hello"); // payload

    let flags = (mqtt::packet::Qos::ExactlyOnce as u8) << 1;
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v5_0::Publish::parse(flags, data_arc).unwrap();

    assert_eq!(consumed, 14); // 2 + 4 + 2 + 1 + 5
    assert_eq!(packet.topic_name(), "test");
    assert_eq!(packet.qos(), mqtt::packet::Qos::ExactlyOnce);
    assert_eq!(packet.packet_id(), Some(1234));
    assert_eq!(packet.payload().as_slice(), b"hello");
}

#[test]
fn parse_qos1_packet_id_incomplete() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x01); // incomplete packet id (only 1 byte instead of 2)

    let flags = (mqtt::packet::Qos::AtLeastOnce as u8) << 1;
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let err = mqtt::packet::v5_0::Publish::parse(flags, data_arc).unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_empty_payload() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x00); // property length 0
    // no payload

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap();

    assert_eq!(consumed, 7); // 2 + 4 + 1
    assert_eq!(packet.topic_name(), "test");
    assert_eq!(packet.payload().as_slice(), &[] as &[u8]);
}

#[test]
fn parse_no_properties() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    // no property length - parsing ends at cursor position

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap();

    assert_eq!(consumed, 6); // 2 + 4
    assert_eq!(packet.topic_name(), "test");
    assert!(packet.props().is_none());
    assert_eq!(packet.payload().as_slice(), &[] as &[u8]);
}

#[test]
fn parse_with_flags() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x00); // property length 0
    raw.extend_from_slice(b"hello"); // payload

    let flags = 0x09; // DUP + RETAIN
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, _consumed) = mqtt::packet::v5_0::Publish::parse(flags, data_arc).unwrap();

    assert!(packet.dup());
    assert!(packet.retain());
}

#[test]
fn parse_with_props() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x05); // property length
    raw.push(0x02); // MessageExpiryInterval property ID
    raw.extend_from_slice(&300u32.to_be_bytes()); // property value
    raw.extend_from_slice(b"hello"); // payload

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap();

    assert_eq!(consumed, 17); // 2 + 4 + 1 + 5 + 5
    assert!(packet.props().is_some());
    assert_eq!(packet.props().as_ref().unwrap().len(), 1);
    match &packet.props().as_ref().unwrap()[0] {
        mqtt::packet::Property::MessageExpiryInterval(m) => {
            assert_eq!(m.val(), 300);
        }
        _ => panic!("unexpected property variant"),
    }
}

#[test]
fn parse_prop_length_over() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x10); // property length 16, but no props follow

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let err = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_property() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x02); // property length
    raw.push(0x99); // invalid property ID
    raw.push(0x00); // dummy byte

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let err = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_property_validation() {
    // This tests lines 465-467: property validation in validate_publish_properties
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.push(0x02); // property length
    raw.push(0x01); // PayloadFormatIndicator property ID
    raw.push(0x01); // property value

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, _consumed) = mqtt::packet::v5_0::Publish::parse(0, data_arc).unwrap();

    // Verify that properties are parsed correctly
    assert!(packet.props().is_some());
    assert_eq!(packet.props().as_ref().unwrap().len(), 1);
    match &packet.props().as_ref().unwrap()[0] {
        mqtt::packet::Property::PayloadFormatIndicator(p) => {
            assert_eq!(p.val(), 1u8); // PayloadFormat::String = 1
        }
        _ => panic!("unexpected property variant"),
    }
}

// Size tests

#[test]
fn size_qos0() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    // 1 byte fixed header + 1 byte remaining length + 12 bytes variable header/payload
    assert_eq!(packet.size(), 14);
}

#[test]
fn size_qos1() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    // 1 byte fixed header + 1 byte remaining length + 14 bytes variable header/payload
    assert_eq!(packet.size(), 16);
}

// Topic alias manipulation tests

#[test]
fn test_remove_topic_alias_add_topic_basic() {
    // Create a publish packet with empty topic name and TopicAlias property
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());
    props.push(
        mqtt::packet::UserProperty::new("key", "value")
            .unwrap()
            .into(),
    );

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload(())
        .build()
        .unwrap();

    let original_size = publish.size();

    // Remove topic alias and add topic
    let result = publish.remove_topic_alias_add_topic("test/topic".to_string());
    assert!(result.is_ok());
    let publish = result.unwrap();

    // Verify topic name was set
    assert_eq!(publish.topic_name(), "test/topic");

    // Verify TopicAlias property was removed
    if let Some(props) = publish.props() {
        let has_topic_alias = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)));
        assert!(!has_topic_alias, "TopicAlias property should be removed");

        // Verify other properties remain
        let has_user_property = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::UserProperty(_)));
        assert!(has_user_property, "UserProperty should remain");
    } else {
        panic!("Properties should not be None");
    }

    // Verify size changed (topic added, TopicAlias removed)
    let new_size = publish.size();
    assert_ne!(original_size, new_size);
}

#[test]
fn test_remove_topic_alias_add_topic_with_topic_alias() {
    // Create a publish packet with empty topic name and TopicAlias property
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload(())
        .build()
        .unwrap();

    // Remove topic alias and add topic (should work with TopicAlias properties)
    let result = publish.remove_topic_alias_add_topic("new/topic".to_string());
    assert!(result.is_ok());
    let publish = result.unwrap();

    // Verify topic name was set
    assert_eq!(publish.topic_name(), "new/topic");

    // Verify TopicAlias property was removed
    if let Some(props) = publish.props() {
        let has_topic_alias = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)));
        assert!(!has_topic_alias, "TopicAlias property should be removed");
    }
}

#[test]
fn test_remove_topic_alias_add_topic_invalid_topic() {
    // Create a publish packet with TopicAlias to allow empty topic name
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload(())
        .build()
        .unwrap();

    // Try to set invalid topic name (with wildcard)
    let result = publish.remove_topic_alias_add_topic("test/+/topic".to_string());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

#[test]
fn test_remove_topic_alias_add_topic_non_empty_topic_error() {
    // Create a publish packet with existing topic name
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("existing/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload("test payload")
        .build()
        .unwrap();

    // Should fail because topic name is not empty
    let result = publish.remove_topic_alias_add_topic("new/topic".to_string());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::TopicNameInvalid
    );
}

#[test]
fn test_remove_topic_alias_basic() {
    // Create a publish packet with TopicAlias property
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(5).unwrap().into());
    props.push(
        mqtt::packet::MessageExpiryInterval::new(256)
            .unwrap()
            .into(),
    );

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("existing/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload(())
        .build()
        .unwrap();

    let original_size = publish.size();

    // Remove topic alias
    let publish = publish.remove_topic_alias();

    // Verify topic name remains unchanged
    assert_eq!(publish.topic_name(), "existing/topic");

    // Verify TopicAlias property was removed
    if let Some(props) = publish.props() {
        let has_topic_alias = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)));
        assert!(!has_topic_alias, "TopicAlias property should be removed");

        // Verify other properties remain
        let has_expiry = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::MessageExpiryInterval(_)));
        assert!(has_expiry, "MessageExpiryInterval should remain");
    } else {
        panic!("Properties should not be None");
    }

    // Verify size changed (TopicAlias removed)
    let new_size = publish.size();
    assert!(
        new_size < original_size,
        "Size should decrease after removing TopicAlias"
    );
}

#[test]
fn test_remove_topic_alias_no_topic_alias() {
    // Create a publish packet without TopicAlias property
    let mut props = mqtt::packet::Properties::new();
    props.push(
        mqtt::packet::MessageExpiryInterval::new(256)
            .unwrap()
            .into(),
    );

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload(())
        .build()
        .unwrap();

    let original_size = publish.size();

    // Remove topic alias (should be no-op)
    let publish = publish.remove_topic_alias();

    // Verify topic name remains unchanged
    assert_eq!(publish.topic_name(), "test/topic");

    // Verify properties remain unchanged
    if let Some(props) = publish.props() {
        let has_expiry = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::MessageExpiryInterval(_)));
        assert!(has_expiry, "MessageExpiryInterval should remain");
    }

    // Verify size remains the same
    let new_size = publish.size();
    assert_eq!(
        original_size, new_size,
        "Size should remain the same when no TopicAlias to remove"
    );
}

#[test]
fn test_remove_topic_alias_no_properties() {
    // Create a publish packet without properties
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(())
        .build()
        .unwrap();

    let original_size = publish.size();

    // Remove topic alias (should be no-op)
    let publish = publish.remove_topic_alias();

    // Verify topic name remains unchanged
    assert_eq!(publish.topic_name(), "test/topic");

    // Verify properties remain None
    assert!(publish.props().is_none() || publish.props().as_ref().unwrap().is_empty());

    // Verify size remains the same
    let new_size = publish.size();
    assert_eq!(
        original_size, new_size,
        "Size should remain the same when no properties"
    );
}

#[test]
fn test_length_recalculation_with_qos1() {
    // Create a QoS1 publish packet with properties
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(10).unwrap().into());
    props.push(
        mqtt::packet::MessageExpiryInterval::new(968)
            .unwrap()
            .into(),
    );

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(123u16)
        .props(props)
        .payload("test payload")
        .build()
        .unwrap();

    // Remove topic alias and add topic
    let result = publish.remove_topic_alias_add_topic("long/topic/name/for/testing".to_string());
    assert!(result.is_ok());
    let publish = result.unwrap();

    // Verify the packet can be serialized correctly (implies correct length calculation)
    let buffers = publish.to_buffers();
    assert!(!buffers.is_empty());

    // Verify packet ID is preserved for QoS1
    assert_eq!(publish.packet_id(), Some(123u16));
    assert_eq!(publish.qos(), mqtt::packet::Qos::AtLeastOnce);
}

#[test]
fn test_length_recalculation_property_length_changes() {
    // Create packet with multiple properties including TopicAlias
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());
    props.push(
        mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::UserProperty::new("key1", "value1")
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::UserProperty::new("key2", "value2")
            .unwrap()
            .into(),
    );

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("short")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload("test payload content")
        .build()
        .unwrap();

    let original_property_count = publish.props().as_ref().map(|p| p.len()).unwrap_or(0);

    // Remove topic alias
    let publish = publish.remove_topic_alias();

    // Verify property count decreased
    let new_property_count = publish.props().as_ref().map(|p| p.len()).unwrap_or(0);
    assert_eq!(
        new_property_count,
        original_property_count - 1,
        "Property count should decrease by 1"
    );

    // Verify other properties are preserved
    if let Some(props) = publish.props() {
        let has_content_type = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::ContentType(_)));
        assert!(has_content_type, "ContentType should be preserved");

        let user_prop_count = props
            .iter()
            .filter(|prop| matches!(prop, mqtt::packet::Property::UserProperty(_)))
            .count();
        assert_eq!(
            user_prop_count, 2,
            "Both UserProperty entries should be preserved"
        );
    }
}

#[test]
fn test_empty_properties_after_topic_alias_removal() {
    // Create packet with only TopicAlias property
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload(())
        .build()
        .unwrap();

    // Remove topic alias
    let publish = publish.remove_topic_alias();

    // Verify properties become empty but not None
    if let Some(props) = publish.props() {
        assert!(
            props.is_empty(),
            "Properties should be empty after removing the only TopicAlias"
        );
    }

    // Verify packet can still be serialized
    let buffers = publish.to_buffers();
    assert!(!buffers.is_empty());
}

#[test]
fn test_roundtrip_serialization_after_modification() {
    // Create a packet, modify it, then verify it can be serialized and parsed correctly
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(20).unwrap().into());
    props.push(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String)
            .unwrap()
            .into(),
    );

    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(456u16)
        .props(props)
        .payload("modified packet payload")
        .build()
        .unwrap();

    // Modify the packet
    let result = publish.remove_topic_alias_add_topic("modified/topic/name".to_string());
    assert!(result.is_ok());
    let publish = result.unwrap();

    // Verify all fields are correct
    assert_eq!(publish.topic_name(), "modified/topic/name");
    assert_eq!(publish.packet_id(), Some(456u16));
    assert_eq!(publish.qos(), mqtt::packet::Qos::AtLeastOnce);
    assert_eq!(publish.payload().as_slice(), b"modified packet payload");

    // Verify properties are correct
    if let Some(props) = publish.props() {
        let has_topic_alias = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)));
        assert!(!has_topic_alias, "TopicAlias should be removed");

        let has_payload_format = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::PayloadFormatIndicator(_)));
        assert!(
            has_payload_format,
            "PayloadFormatIndicator should be preserved"
        );
    }

    // Verify packet can be serialized
    let buffers = publish.to_buffers();
    assert!(!buffers.is_empty());

    // Calculate total size from buffers
    let total_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(
        total_size,
        publish.size(),
        "Serialized size should match calculated size"
    );
}

// set_dup tests

#[test]
fn test_set_dup_true() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .dup(false)
        .payload("test payload")
        .build()
        .unwrap();

    // Initially dup should be false
    assert!(!packet.dup());

    // Set dup to true
    let packet_with_dup = packet.set_dup(true);
    assert!(packet_with_dup.dup());

    // Verify other properties remain unchanged
    assert_eq!(packet_with_dup.topic_name(), "test/topic");
    assert_eq!(packet_with_dup.qos(), mqtt::packet::Qos::AtMostOnce);
    assert!(!packet_with_dup.retain());
}

#[test]
fn test_set_dup_false() {
    let mut props = mqtt::packet::Properties::new();
    props.push(
        mqtt::packet::UserProperty::new("test", "value")
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(123u16)
        .dup(true)
        .retain(true)
        .props(props)
        .payload("test payload")
        .build()
        .unwrap();

    // Initially dup should be true
    assert!(packet.dup());

    // Set dup to false
    let packet_without_dup = packet.set_dup(false);
    assert!(!packet_without_dup.dup());

    // Verify other properties remain unchanged
    assert_eq!(packet_without_dup.topic_name(), "test/topic");
    assert_eq!(packet_without_dup.qos(), mqtt::packet::Qos::AtLeastOnce);
    assert_eq!(packet_without_dup.packet_id(), Some(123u16));
    assert!(packet_without_dup.retain());

    // Verify properties remain unchanged
    if let Some(props) = packet_without_dup.props() {
        let has_user_property = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::UserProperty(_)));
        assert!(has_user_property, "UserProperty should remain");
    }
}

#[test]
fn test_set_dup_chaining() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(456u16)
        .payload("test payload")
        .build()
        .unwrap();

    // Test chaining: false -> true -> false
    let result = packet.set_dup(false).set_dup(true).set_dup(false);

    assert!(!result.dup());
    assert_eq!(result.topic_name(), "test/topic");
    assert_eq!(result.qos(), mqtt::packet::Qos::ExactlyOnce);
    assert_eq!(result.packet_id(), Some(456u16));
}

// add_extracted_topic_name tests

#[test]
fn test_add_extracted_topic_name_success() {
    // Create a packet with empty topic name and TopicAlias property
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());

    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload("test payload")
        .build()
        .unwrap();

    // Initially topic_name_extracted should be false
    assert!(!packet.topic_name_extracted());
    assert_eq!(packet.topic_name(), "");

    // Add extracted topic name
    let result = packet.add_extracted_topic_name("resolved/topic".to_string());
    assert!(result.is_ok());
    let packet_with_topic = result.unwrap();

    // Verify topic name was set and flag is true
    assert!(packet_with_topic.topic_name_extracted());
    assert_eq!(packet_with_topic.topic_name(), "resolved/topic");

    // Verify TopicAlias property remains intact
    if let Some(props) = packet_with_topic.props() {
        let has_topic_alias = props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)));
        assert!(has_topic_alias, "TopicAlias property should remain");
    }

    // Verify other properties remain unchanged
    assert_eq!(packet_with_topic.qos(), mqtt::packet::Qos::AtMostOnce);
    assert_eq!(packet_with_topic.payload().as_slice(), b"test payload");
}

#[test]
fn test_add_extracted_topic_name_non_empty_topic_error() {
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("existing/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload("test payload")
        .build()
        .unwrap();

    // Should fail because topic name is not empty
    let result = packet.add_extracted_topic_name("new/topic".to_string());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::TopicNameInvalid
    );
}

#[test]
fn test_add_extracted_topic_name_invalid_topic_wildcard() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::TopicAlias::new(1).unwrap().into());

    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .props(props)
        .payload(())
        .build()
        .unwrap();

    // Should fail with wildcard in topic name
    let result = packet
        .clone()
        .add_extracted_topic_name("test/+/topic".to_string());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );

    let result = packet.add_extracted_topic_name("test/#".to_string());
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

#[test]
fn test_topic_name_extracted_default_false() {
    // Test that topic_name_extracted is false by default for builder
    let packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(())
        .build()
        .unwrap();

    assert!(!packet.topic_name_extracted());
}

#[test]
fn test_topic_name_extracted_parse_default_false() {
    // Test that topic_name_extracted is false by default for parsed packets
    let mut raw = Vec::new();
    raw.extend_from_slice(&(10u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test/topic"); // topic
    raw.push(0); // property length = 0
    raw.extend_from_slice(b"payload"); // payload

    let flags = 0x00; // QoS 0, no DUP, no RETAIN
    let data_arc: std::sync::Arc<[u8]> = std::sync::Arc::from(raw.into_boxed_slice());
    let (packet, _consumed) = mqtt::packet::v5_0::Publish::parse(flags, data_arc).unwrap();

    assert!(!packet.topic_name_extracted());
    assert_eq!(packet.topic_name(), "test/topic");
}
