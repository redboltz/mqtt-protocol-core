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
mod common;
use common::mqtt;

// Build fail tests
#[test]
fn build_fail_empty_entries() {
    common::init_tracing();
    let err = mqtt::packet::v3_1_1::Subscribe::builder()
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
    let err = mqtt::packet::v3_1_1::Subscribe::builder()
        .entries(vec![entry])
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Build success tests
#[test]
fn build_success_minimal() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
    assert_eq!(packet.entries().len(), 1);
}

#[test]
fn build_success_multiple_entries() {
    common::init_tracing();
    let qos1_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    let qos2_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::ExactlyOnce);

    let entries = vec![
        mqtt::packet::SubEntry::new("test/topic1", mqtt::packet::SubOpts::default()).unwrap(),
        mqtt::packet::SubEntry::new("test/topic2", qos1_opts).unwrap(),
        mqtt::packet::SubEntry::new("test/topic3", qos2_opts).unwrap(),
    ];
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(42u16)
        .entries(entries)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 42u16);
    assert_eq!(packet.entries().len(), 3);
    assert_eq!(
        packet.entries()[0].topic_filter().to_string(),
        "test/topic1"
    );
    assert_eq!(
        packet.entries()[1].topic_filter().to_string(),
        "test/topic2"
    );
    assert_eq!(
        packet.entries()[2].topic_filter().to_string(),
        "test/topic3"
    );
}

// Display tests
#[test]
fn display_minimal() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":1"));
    assert!(display_str.contains("\"entries\""));
}

#[test]
fn display_multiple_entries() {
    common::init_tracing();
    let qos1_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);

    let entries = vec![
        mqtt::packet::SubEntry::new("test/topic1", mqtt::packet::SubOpts::default()).unwrap(),
        mqtt::packet::SubEntry::new("test/topic2", qos1_opts).unwrap(),
    ];
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(42u16)
        .entries(entries)
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":42"));
    assert!(display_str.contains("\"entries\""));
}

// Debug tests
#[test]
fn debug_minimal() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"packet_id\":1"));
}

// Getter tests
#[test]
fn getter_packet_id() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(12345u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

#[test]
fn getter_entries() {
    common::init_tracing();
    let qos1_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);

    let entries = vec![
        mqtt::packet::SubEntry::new("test/topic1", mqtt::packet::SubOpts::default()).unwrap(),
        mqtt::packet::SubEntry::new("test/topic2", qos1_opts).unwrap(),
    ];
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1u16)
        .entries(entries)
        .build()
        .unwrap();

    assert_eq!(packet.entries().len(), 2);
    assert_eq!(
        packet.entries()[0].topic_filter().to_string(),
        "test/topic1"
    );
    assert_eq!(
        packet.entries()[1].topic_filter().to_string(),
        "test/topic2"
    );
}

// to_buffers() tests
#[test]
#[cfg(feature = "std")]
fn to_buffers_minimal() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert!(!buffers.is_empty());

    // Collect all bytes
    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Check fixed header
    assert_eq!(all_bytes[0], 0x82); // SUBSCRIBE packet type

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    assert_eq!(all_bytes, continuous);
}

#[test]
#[cfg(feature = "std")]
fn to_buffers_multiple_entries() {
    common::init_tracing();
    let qos1_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);

    let entries = vec![
        mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap(),
        mqtt::packet::SubEntry::new("topic2", qos1_opts).unwrap(),
    ];
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(100u16)
        .entries(entries)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(&buf);
    }

    assert_eq!(all_bytes[0], 0x82);
    // Should be larger due to multiple entries
    assert!(all_bytes.len() > 10);

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    assert_eq!(all_bytes, continuous);
}

// Parse tests
#[test]
fn parse_minimal() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let original = mqtt::packet::v3_1_1::Subscribe::builder()
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
            // Skip fixed header and remaining length
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Subscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].topic_filter().to_string(), "test/topic");
}

#[test]
fn parse_multiple_entries() {
    common::init_tracing();
    let qos1_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);

    let qos2_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::ExactlyOnce);

    let entries = vec![
        mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap(),
        mqtt::packet::SubEntry::new("topic2", qos1_opts).unwrap(),
        mqtt::packet::SubEntry::new("topic3", qos2_opts).unwrap(),
    ];
    let original = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(200u16)
        .entries(entries)
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

    let data = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Subscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 200u16);
    assert_eq!(parsed.entries().len(), 3);
    assert_eq!(parsed.entries()[0].topic_filter().to_string(), "topic1");
    assert_eq!(parsed.entries()[1].topic_filter().to_string(), "topic2");
    assert_eq!(parsed.entries()[2].topic_filter().to_string(), "topic3");
}

#[test]
fn parse_invalid_too_short() {
    common::init_tracing();
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v3_1_1::Subscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_entries() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID only

    let err = mqtt::packet::v3_1_1::Subscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Size tests
#[test]
fn size_minimal() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1u16)
        .entries(vec![entry])
        .build()
        .unwrap();

    let size = packet.size();
    assert!(size > 0);
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        // Verify size matches actual buffer size
        let buffers = packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

#[test]
fn size_multiple_entries() {
    common::init_tracing();
    let qos1_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);

    let qos2_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::ExactlyOnce);

    let entries = vec![
        mqtt::packet::SubEntry::new("topic1", mqtt::packet::SubOpts::default()).unwrap(),
        mqtt::packet::SubEntry::new("topic2", qos1_opts).unwrap(),
        mqtt::packet::SubEntry::new("topic3", qos2_opts).unwrap(),
    ];
    let packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(100u16)
        .entries(entries)
        .build()
        .unwrap();

    let size = packet.size();
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    common::init_tracing();
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let original = mqtt::packet::v3_1_1::Subscribe::builder()
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

    let data = &continuous[2..];
    let (parsed, _) = mqtt::packet::v3_1_1::Subscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());
    assert_eq!(
        original.entries()[0].topic_filter().to_string(),
        parsed.entries()[0].topic_filter().to_string()
    );
}

#[test]
fn roundtrip_multiple_entries_with_qos() {
    common::init_tracing();
    let qos1_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);

    let qos2_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::ExactlyOnce);

    let entries = vec![
        mqtt::packet::SubEntry::new("test/topic1", mqtt::packet::SubOpts::default()).unwrap(),
        mqtt::packet::SubEntry::new("test/topic2", qos1_opts).unwrap(),
        mqtt::packet::SubEntry::new("test/topic3", qos2_opts).unwrap(),
    ];
    let original = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(65535u16)
        .entries(entries)
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

    let data = &continuous[2..];
    let (parsed, _) = mqtt::packet::v3_1_1::Subscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());

    for (orig, parsed) in original.entries().iter().zip(parsed.entries().iter()) {
        assert_eq!(
            orig.topic_filter().to_string(),
            parsed.topic_filter().to_string()
        );
        assert_eq!(orig.sub_opts().qos(), parsed.sub_opts().qos());
    }
}

#[test]
fn test_packet_type() {
    common::init_tracing();
    let packet_type = mqtt::packet::v3_1_1::Subscribe::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Subscribe);
}
