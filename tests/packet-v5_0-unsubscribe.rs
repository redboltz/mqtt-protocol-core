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

// Build fail tests
#[test]
fn build_fail_empty_entries() {
    let err = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_no_packet_id() {
    let err = mqtt::packet::v5_0::Unsubscribe::builder()
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_property() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::PayloadFormatIndicator(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::Binary).unwrap(),
    ));

    let err = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Build success tests
#[test]
fn build_success_minimal() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
    assert_eq!(packet.entries().len(), 1);
    assert!(packet.props().is_empty());
}

#[test]
fn build_success_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(42u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 42u16);
    assert_eq!(packet.entries().len(), 1);
    assert_eq!(packet.props().len(), 1);
}

#[test]
fn build_success_multiple_entries() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(100u16)
        .entries(vec!["topic1", "topic2", "topic3"])
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 100u16);
    assert_eq!(packet.entries().len(), 3);
    assert_eq!(packet.entries()[0].as_str(), "topic1");
    assert_eq!(packet.entries()[1].as_str(), "topic2");
    assert_eq!(packet.entries()[2].as_str(), "topic3");
}

// Display tests
#[test]
fn display_minimal() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"type\":\"unsubscribe\""));
    assert!(display_str.contains("\"packet_id\":1"));
    assert!(display_str.contains("\"entries\""));
}

#[test]
fn display_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(42u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"type\":\"unsubscribe\""));
    assert!(display_str.contains("\"packet_id\":42"));
    assert!(display_str.contains("\"props\""));
    assert!(display_str.contains("\"entries\""));
}

// Debug tests
#[test]
fn debug_minimal() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test"])
        .unwrap()
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"type\":\"unsubscribe\""));
    assert!(debug_str.contains("\"packet_id\":1"));
}

#[test]
fn debug_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("test", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(99u16)
        .entries(vec!["test"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"props\""));
}

// Getter tests
#[test]
fn getter_packet_id() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(12345u16)
        .entries(vec!["test"])
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

#[test]
fn getter_entries() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["topic1", "topic2"])
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.entries().len(), 2);
    assert_eq!(packet.entries()[0].as_str(), "topic1");
    assert_eq!(packet.entries()[1].as_str(), "topic2");
}

#[test]
fn getter_props_empty() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test"])
        .unwrap()
        .build()
        .unwrap();

    assert!(packet.props().is_empty());
}

#[test]
fn getter_props_with_values() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.props().len(), 1);
}

// to_buffers() tests
#[test]
fn to_buffers_minimal() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert!(!buffers.is_empty());

    // Collect all bytes
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(buf.as_ref());
    }

    // Check fixed header
    assert_eq!(all_bytes[0], 0xa2); // UNSUBSCRIBE packet type

    // Should contain packet ID, property length, and topic filter
    assert!(all_bytes.len() > 5);
}

#[test]
fn to_buffers_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(42u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(buf.as_ref());
    }

    // Should be larger than minimal case due to properties
    assert!(all_bytes.len() > 15);
    assert_eq!(all_bytes[0], 0xa2);
}

#[test]
fn to_buffers_multiple_entries() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(100u16)
        .entries(vec!["topic1", "topic2", "topic3"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(buf.as_ref());
    }

    // Should contain all three topic filters
    assert!(all_bytes.len() > 25);
}

// Parse tests
#[test]
fn parse_minimal() {
    let original = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        // Skip fixed header and remaining length
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].as_str(), "test");
    assert!(parsed.props().is_empty());
}

#[test]
fn parse_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let original = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(42u16)
        .entries(vec!["topic/test"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 42u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].as_str(), "topic/test");
    assert_eq!(parsed.props().len(), 1);
}

#[test]
fn parse_multiple_entries() {
    let original = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(200u16)
        .entries(vec!["topic1", "topic2", "topic3"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 200u16);
    assert_eq!(parsed.entries().len(), 3);
    assert_eq!(parsed.entries()[0].as_str(), "topic1");
    assert_eq!(parsed.entries()[1].as_str(), "topic2");
    assert_eq!(parsed.entries()[2].as_str(), "topic3");
}

#[test]
fn parse_invalid_too_short() {
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_entries() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x00); // property length = 0

    let err = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_property() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x02); // property length = 2
    data.push(0x01); // PayloadFormatIndicator (invalid for UNSUBSCRIBE)
    data.push(0x00); // property value

    let err = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Size tests
#[test]
fn size_minimal() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test"])
        .unwrap()
        .build()
        .unwrap();

    let size = packet.size();
    assert!(size > 0);

    // Verify size matches actual buffer size
    let buffers = packet.to_buffers();
    let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(size, actual_size);
}

#[test]
fn size_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(42u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let size = packet.size();
    let buffers = packet.to_buffers();
    let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(size, actual_size);
}

#[test]
fn size_multiple_entries() {
    let packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(100u16)
        .entries(vec!["topic1", "topic2", "topic3"])
        .unwrap()
        .build()
        .unwrap();

    let size = packet.size();
    let buffers = packet.to_buffers();
    let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(size, actual_size);
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    let original = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());
    assert_eq!(original.props().len(), parsed.props().len());
}

#[test]
fn roundtrip_with_all_valid_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("client", "test").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("version", "1.0").unwrap(),
    ));

    let original = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(65535u16)
        .entries(vec!["sensor/+/temperature", "control/#"])
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v5_0::Unsubscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());
    assert_eq!(original.props().len(), parsed.props().len());

    // Check specific values
    assert_eq!(parsed.entries()[0].as_str(), "sensor/+/temperature");
    assert_eq!(parsed.entries()[1].as_str(), "control/#");
}
