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

// Build fail tests
#[test]
fn build_fail_empty_entries() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_no_packet_id() {
    common::init_tracing();
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
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::PayloadFormatIndicator(
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
    common::init_tracing();
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
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(123).unwrap(),
    ));
    props.push(mqtt::packet::GenericProperty::UserProperty(
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
    common::init_tracing();
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
    common::init_tracing();
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
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::SubscriptionIdentifier(
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
    common::init_tracing();
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
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::UserProperty(
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
    common::init_tracing();
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
    common::init_tracing();
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
    common::init_tracing();
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
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(456).unwrap(),
    ));
    props.push(mqtt::packet::GenericProperty::UserProperty(
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
    common::init_tracing();
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous[0], 0x82); // SUBSCRIBE packet type

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    // Should contain packet ID, property length, and topic filter
    assert!(continuous.len() > 5);
    assert_eq!(packet.size(), continuous.len());
}

#[test]
fn to_buffers_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::SubscriptionIdentifier(
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

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous[0], 0x82);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    // Should be larger than minimal case due to properties
    assert!(continuous.len() > 10);
    assert_eq!(packet.size(), continuous.len());
}

#[test]
fn to_buffers_multiple_entries() {
    common::init_tracing();
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("topic3", mqtt::packet::SubOpts::default()).unwrap();

    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(100u16)
        .entries(vec![entry1, entry2, entry3])
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous[0], 0x82);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    // Should contain all three topic filters
    assert!(continuous.len() > 20);
    assert_eq!(packet.size(), continuous.len());
}

// Parse tests
#[test]
fn parse_minimal() {
    common::init_tracing();
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, consumed) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].topic_filter(), "test");
    assert!(parsed.props().is_empty());
}

#[test]
fn parse_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::SubscriptionIdentifier(
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

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, consumed) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 42u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].topic_filter(), "topic/test");
    assert_eq!(parsed.props().len(), 1);
}

#[test]
fn parse_multiple_entries() {
    common::init_tracing();
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("topic3", mqtt::packet::SubOpts::default()).unwrap();

    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(200u16)
        .entries(vec![entry1, entry2, entry3])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
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
    common::init_tracing();
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_entries() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x00); // property length = 0

    let err = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_property() {
    common::init_tracing();
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
    common::init_tracing();
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let size = packet.size();
    assert!(size > 0);

    // Verify size matches actual buffer size
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let buffers_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, buffers_size);
    }
}

#[test]
fn size_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(123).unwrap(),
    ));
    props.push(mqtt::packet::GenericProperty::UserProperty(
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
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let buffers_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, buffers_size);
    }
}

#[test]
fn size_multiple_entries() {
    common::init_tracing();
    let entry1 = mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("topic2", mqtt::packet::SubOpts::default()).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("topic3", mqtt::packet::SubOpts::default()).unwrap();

    let packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(100u16)
        .entries(vec![entry1, entry2, entry3])
        .build()
        .unwrap();

    let size = packet.size();
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let buffers_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, buffers_size);
    }
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    common::init_tracing();
    let entry = mqtt::packet::SubEntry::new("test", mqtt::packet::SubOpts::default()).unwrap();
    let original = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, _) = mqtt::packet::v5_0::Subscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());
    assert_eq!(original.props().len(), parsed.props().len());
}

#[test]
fn roundtrip_with_all_valid_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::GenericProperty::SubscriptionIdentifier(
        mqtt::packet::SubscriptionIdentifier::new(12345).unwrap(),
    ));
    props.push(mqtt::packet::GenericProperty::UserProperty(
        mqtt::packet::UserProperty::new("client", "test").unwrap(),
    ));
    props.push(mqtt::packet::GenericProperty::UserProperty(
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

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
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
    common::init_tracing();
    let packet_type = mqtt::packet::v5_0::Subscribe::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Subscribe);
}
