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
    let err = mqtt::packet::v3_1_1::Publish::builder()
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_empty_topic() {
    let err = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("")
        .unwrap()
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_topic_too_long() {
    let long_topic = "a".repeat(65536);
    let err = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name(long_topic.as_str())
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_topic_with_wildcard() {
    let err = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("bad+topic")
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_qos1_no_packet_id() {
    let err = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_qos0_with_packet_id() {
    let err = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_qos0_with_packet_id_validation() {
    // This tests QoS 0 with packet ID validation in validate() method
    let err = mqtt::packet::v3_1_1::Publish::builder()
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
    // This tests packet ID zero validation
    let err = mqtt::packet::v3_1_1::Publish::builder()
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
    // This tests payload size limit validation (268435455 bytes)
    let large_payload = vec![0u8; 268435456]; // 1 byte over limit
    let err = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .payload(&large_payload)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Build success tests

#[test]
fn build_success_qos2() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
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
    let packet = mqtt::packet::v3_1_1::Publish::builder()
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
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert!(output.contains(r#""type":"publish""#));
    assert!(output.contains(r#""topic_name":"test/topic""#));
    assert!(output.contains(r#""qos":"AtMostOnce""#));
    assert!(output.contains(r#""payload":"hello""#));
}

#[test]
fn display_qos1_with_packet_id() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert!(output.contains(r#""type":"publish""#));
    assert!(output.contains(r#""topic_name":"test/topic""#));
    assert!(output.contains(r#""qos":"AtLeastOnce""#));
    assert!(output.contains(r#""packet_id":1234"#));
    assert!(output.contains(r#""payload":"hello""#));
}

#[test]
fn display_with_flags() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .retain(false)
        .dup(false)
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert!(output.contains(r#""retain":false"#));
    assert!(output.contains(r#""dup":false"#));
}

#[test]
fn display_binary_payload() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload(&[0x00, b'a', b'b'])
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert!(output.contains(r#""payload":"\u0000ab""#));
}

#[test]
fn display_binary_payload_array() {
    // Test payload with invalid UTF-8 bytes that should be serialized as array
    // This tests line 333: None => state.serialize_field("payload", &payload_data)?
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload(&[0x80, 0x81, 0x82, 0x83]) // Invalid UTF-8 continuation bytes
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert!(output.contains(r#""payload":[128,129,130,131]"#));
}

// Debug tests

#[test]
fn debug_qos0() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert!(output.contains(r#""type":"publish""#));
    assert!(output.contains(r#""topic_name":"test/topic""#));
    assert!(output.contains(r#""qos":"AtMostOnce""#));
    assert!(output.contains(r#""payload":"hello""#));
}

// Getter tests

#[test]
fn getter_qos0() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
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
}

#[test]
fn getter_qos1_with_packet_id() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
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
    let packet = mqtt::packet::v3_1_1::Publish::builder()
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

// to_buffers() tests

#[test]
fn to_buffers_qos0() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous[0], 0x30); // fixed header (QoS 0)
    assert_eq!(continuous[1], 0x0B); // remaining length (11)

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }
    // topic length (2) + topic (4) + payload (5) = 11
    assert_eq!(packet.size(), 1 + 1 + 11); // fixed header + remaining length + payload
    assert_eq!(packet.size(), continuous.len());
}

#[test]
fn to_buffers_qos1() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous[0], 0x32); // fixed header (QoS 1)
    assert_eq!(continuous[1], 0x0D); // remaining length (13)

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    // topic length (2) + topic (4) + packet_id (2) + payload (5) = 13
    assert_eq!(packet.size(), 1 + 1 + 13); // fixed header + remaining length + payload
    assert_eq!(packet.size(), continuous.len());
}

#[test]
fn to_buffers_with_flags() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .retain(true)
        .dup(true)
        .payload("hello")
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous[0], 0x39); // fixed header with DUP and RETAIN flags

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    assert_eq!(packet.size(), continuous.len());
}

// Parse tests

#[test]
fn parse_empty() {
    let empty_arc: Arc<[u8]> = Arc::from(Vec::new().into_boxed_slice());
    let err = mqtt::packet::v3_1_1::Publish::parse(0, empty_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_topic_incomplete() {
    let raw = vec![0x00]; // incomplete topic length
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let err = mqtt::packet::v3_1_1::Publish::parse(0, data_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_qos() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(0u16).to_be_bytes()); // empty topic
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());

    let err = mqtt::packet::v3_1_1::Publish::parse(0b0000_11 << 1, data_arc).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_qos0() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.extend_from_slice(b"hello"); // payload

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v3_1_1::Publish::parse(0, data_arc).unwrap();

    assert_eq!(consumed, 11); // 2 + 4 + 5
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
    raw.extend_from_slice(b"hello"); // payload

    let flags = (mqtt::packet::Qos::AtLeastOnce as u8) << 1;
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v3_1_1::Publish::parse(flags, data_arc).unwrap();

    assert_eq!(consumed, 13); // 2 + 4 + 2 + 5
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
    raw.extend_from_slice(b"hello"); // payload

    let flags = (mqtt::packet::Qos::ExactlyOnce as u8) << 1;
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v3_1_1::Publish::parse(flags, data_arc).unwrap();

    assert_eq!(consumed, 13); // 2 + 4 + 2 + 5
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
    let err = mqtt::packet::v3_1_1::Publish::parse(flags, data_arc).unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_empty_payload() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
                                    // no payload

    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, consumed) = mqtt::packet::v3_1_1::Publish::parse(0, data_arc).unwrap();

    assert_eq!(consumed, 6); // 2 + 4
    assert_eq!(packet.topic_name(), "test");
    assert_eq!(packet.payload().as_slice(), &[] as &[u8]);
}

#[test]
fn parse_with_flags() {
    let mut raw = Vec::new();
    raw.extend_from_slice(&(4u16).to_be_bytes()); // topic length
    raw.extend_from_slice(b"test"); // topic
    raw.extend_from_slice(b"hello"); // payload

    let flags = 0x09; // DUP + RETAIN
    let data_arc: Arc<[u8]> = Arc::from(raw.into_boxed_slice());
    let (packet, _consumed) = mqtt::packet::v3_1_1::Publish::parse(flags, data_arc).unwrap();

    assert!(packet.dup());
    assert!(packet.retain());
}

// Size tests

#[test]
fn size_qos0() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .payload("hello")
        .build()
        .unwrap();

    // 1 byte fixed header + 1 byte remaining length + 11 bytes variable header/payload
    assert_eq!(packet.size(), 13);
}

#[test]
fn size_qos1() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1234u16)
        .payload("hello")
        .build()
        .unwrap();

    // 1 byte fixed header + 1 byte remaining length + 13 bytes variable header/payload
    assert_eq!(packet.size(), 15);
}

// set_dup tests

#[test]
fn test_set_dup_true() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
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
    let packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(123u16)
        .dup(true)
        .retain(true)
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
}

#[test]
fn test_set_dup_chaining() {
    let packet = mqtt::packet::v3_1_1::Publish::builder()
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

#[test]
fn test_packet_type() {
    let packet_type = mqtt::packet::v3_1_1::Publish::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Publish);
}
