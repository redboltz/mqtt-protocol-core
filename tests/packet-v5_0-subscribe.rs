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
    let err = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_no_packet_id() {
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let err = mqtt::packet::v5_0::Subscribe::builder()
        .entries(vec![entry])
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

    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let err = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Build success tests
#[test]
fn build_success_minimal() {
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
    assert_eq!(packet.entries().len(), 1);
    assert!(packet.props().is_empty());
}

#[test]
fn build_success_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(123).unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(42u16)
        .entries(vec![entry])
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 42u16);
    assert_eq!(packet.entries().len(), 1);
    assert_eq!(packet.props().len(), 2);
}

#[test]
fn build_success_multiple_entries() {
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("topic3", mqtt::packet::SubOpts::default()).unwrap();

    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(100u16)
        .entries(vec![entry1, entry2, entry3])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 100u16);
    assert_eq!(packet.entries().len(), 3);
    assert_eq!(packet.entries()[0].topic_filter(), "topic1");
    assert_eq!(packet.entries()[1].topic_filter(), "topic2");
    assert_eq!(packet.entries()[2].topic_filter(), "topic3");
}

// Display tests
#[test]
fn display_minimal() {
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"type\":\"subscribe\""));
    assert!(display_str.contains("\"packet_id\":1"));
    assert!(display_str.contains("\"entries\""));
}

#[test]
fn display_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(123).unwrap(),
    ));

    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(42u16)
        .entries(vec![entry])
        .props(props)
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"type\":\"subscribe\""));
    assert!(display_str.contains("\"packet_id\":42"));
    assert!(display_str.contains("\"props\""));
    assert!(display_str.contains("\"entries\""));
}

// Debug tests
#[test]
fn debug_minimal() {
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"type\":\"subscribe\""));
    assert!(debug_str.contains("\"packet_id\":1"));
}

#[test]
fn debug_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("test", "value").unwrap(),
    ));

    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(99u16)
        .entries(vec![entry])
        .props(props)
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"props\""));
}

// Getter tests
#[test]
fn getter_packet_id() {
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(12345u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

#[test]
fn getter_entries() {
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();

    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry1, entry2])
        .build()
        .unwrap();

    assert_eq!(packet.entries().len(), 2);
    assert_eq!(packet.entries()[0].topic_filter(), "topic1");
    assert_eq!(packet.entries()[1].topic_filter(), "topic2");
}

#[test]
fn getter_props_empty() {
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    assert!(packet.props().is_empty());
}

#[test]
fn getter_props_with_values() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(456).unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.props().len(), 2);
}

// to_buffers() tests
#[test]
fn to_buffers_minimal() {
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert!(!buffers.is_empty());

    // Collect all bytes
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Check fixed header
    assert_eq!(all_bytes[0], 0x82); // SUBSCRIBE packet type

    // Should contain packet ID, property length, and topic filter
    assert!(all_bytes.len() > 5);
}

#[test]
fn to_buffers_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(123).unwrap(),
    ));

    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(42u16)
        .entries(vec![entry])
        .props(props)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Should be larger than minimal case due to properties
    assert!(all_bytes.len() > 10);
    assert_eq!(all_bytes[0], 0x82);
}

#[test]
fn to_buffers_multiple_entries() {
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("topic3", mqtt::packet::SubOpts::default()).unwrap();

    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(100u16)
        .entries(vec![entry1, entry2, entry3])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Should contain all three topic filters
    assert!(all_bytes.len() > 20);
}

// Parse tests
#[test]
fn parse_minimal() {
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        // Skip fixed header and remaining length
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].topic_filter(), "test");
    assert!(parsed.props().is_empty());
}

#[test]
fn parse_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(789).unwrap(),
    ));

    let entry =
        mqtt::packet::SubEntry::new("topic/test", mqtt::packet::SubOpts::default()).unwrap();
    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(42u16)
        .entries(vec![entry])
        .props(props)
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 42u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].topic_filter(), "topic/test");
    assert_eq!(parsed.props().len(), 1);
}

#[test]
fn parse_multiple_entries() {
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("topic3", mqtt::packet::SubOpts::default()).unwrap();

    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(200u16)
        .entries(vec![entry1, entry2, entry3])
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 200u16);
    assert_eq!(parsed.entries().len(), 3);
    assert_eq!(parsed.entries()[0].topic_filter(), "topic1");
    assert_eq!(parsed.entries()[1].topic_filter(), "topic2");
    assert_eq!(parsed.entries()[2].topic_filter(), "topic3");
}

#[test]
fn parse_invalid_too_short() {
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_entries() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x00); // property length = 0

    let err = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_property() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x02); // property length = 2
    data.push(0x01); // PayloadFormatIndicator (invalid for SUBSCRIBE)
    data.push(0x00); // property value

    let err = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Size tests
#[test]
fn size_minimal() {
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
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
    props.push(mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(123).unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(42u16)
        .entries(vec![entry])
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
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("topic3", mqtt::packet::SubOpts::default()).unwrap();

    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(100u16)
        .entries(vec![entry1, entry2, entry3])
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
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());
    assert_eq!(original.props().len(), parsed.props().len());
}

#[test]
fn roundtrip_with_all_valid_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(12345).unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("client", "test").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("version", "1.0").unwrap(),
    ));

    let entry1 =
        mqtt::packet::SubEntry::new("sensor/+/temperature", mqtt::packet::SubOpts::default())
            .unwrap();
    let entry2 =
        mqtt::packet::SubEntry::new("control/#", mqtt::packet::SubOpts::default()).unwrap();

    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(65535u16)
        .entries(vec![entry1, entry2])
        .props(props)
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());
    assert_eq!(original.props().len(), parsed.props().len());

    // Check specific values
    assert_eq!(parsed.entries()[0].topic_filter(), "sensor/+/temperature");
    assert_eq!(parsed.entries()[1].topic_filter(), "control/#");
}

#[test]
fn test_packet_type() {
    let packet_type = mqtt::packet::v5_0::Subscribe::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Subscribe);
}
