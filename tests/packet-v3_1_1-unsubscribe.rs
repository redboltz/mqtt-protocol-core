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
    let err = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_no_packet_id() {
    let err = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Build success tests
#[test]
fn build_success_minimal() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
    assert_eq!(packet.entries().len(), 1);
}

#[test]
fn build_success_multiple_entries() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(42u16)
        .entries(vec!["test/topic1", "test/topic2", "test/topic3"])
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 42u16);
    assert_eq!(packet.entries().len(), 3);
    assert_eq!(packet.entries()[0].as_str(), "test/topic1");
    assert_eq!(packet.entries()[1].as_str(), "test/topic2");
    assert_eq!(packet.entries()[2].as_str(), "test/topic3");
}

// Display tests
#[test]
fn display_minimal() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":1"));
    assert!(display_str.contains("\"entries\""));
}

#[test]
fn display_multiple_entries() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(42u16)
        .entries(vec!["test/topic1", "test/topic2"])
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":42"));
    assert!(display_str.contains("\"entries\""));
}

// Debug tests
#[test]
fn debug_minimal() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"packet_id\":1"));
}

// Getter tests
#[test]
fn getter_packet_id() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(12345u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

#[test]
fn getter_entries() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic1", "test/topic2"])
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.entries().len(), 2);
    assert_eq!(packet.entries()[0].as_str(), "test/topic1");
    assert_eq!(packet.entries()[1].as_str(), "test/topic2");
}

// to_buffers() tests
#[test]
fn to_buffers_minimal() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
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
    assert_eq!(all_bytes[0], 0xA2); // UNSUBSCRIBE packet type
}

#[test]
fn to_buffers_multiple_entries() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(100u16)
        .entries(vec!["topic1", "topic2"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(buf.as_ref());
    }

    assert_eq!(all_bytes[0], 0xA2);
    // Should be larger due to multiple entries
    assert!(all_bytes.len() > 10);
}

// Parse tests
#[test]
fn parse_minimal() {
    let original = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        // Skip fixed header and remaining length
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v3_1_1::Unsubscribe::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.entries().len(), 1);
    assert_eq!(parsed.entries()[0].as_str(), "test/topic");
}

#[test]
fn parse_multiple_entries() {
    let original = mqtt::packet::v3_1_1::Unsubscribe::builder()
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

    let (parsed, consumed) = mqtt::packet::v3_1_1::Unsubscribe::parse(&data).unwrap();
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
    let err = mqtt::packet::v3_1_1::Unsubscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_entries() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID only

    let err = mqtt::packet::v3_1_1::Unsubscribe::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Size tests
#[test]
fn size_minimal() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
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
fn size_multiple_entries() {
    let packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
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
    let original = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1u16)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v3_1_1::Unsubscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());
    assert_eq!(original.entries()[0].as_str(), parsed.entries()[0].as_str());
}

#[test]
fn roundtrip_multiple_entries() {
    let original = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(65535u16)
        .entries(vec!["test/topic1", "test/topic2", "test/topic3"])
        .unwrap()
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v3_1_1::Unsubscribe::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.entries().len(), parsed.entries().len());

    for (orig, parsed) in original.entries().iter().zip(parsed.entries().iter()) {
        assert_eq!(orig.as_str(), parsed.as_str());
    }
}
