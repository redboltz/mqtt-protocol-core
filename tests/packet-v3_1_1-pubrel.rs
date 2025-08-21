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
fn build_fail_no_packet_id() {
    common::init_tracing();
    let err = mqtt::packet::v3_1_1::Pubrel::builder().build().unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_pid0() {
    common::init_tracing();
    let err = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(0)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Build success tests
#[test]
fn build_success_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
}

#[test]
fn build_success_various_packet_ids() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(65535u16)
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 65535u16);
}

// Display tests
#[test]
fn display_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":1"));
}

// Debug tests
#[test]
fn debug_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(42u16)
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"packet_id\":42"));
}

// Getter tests
#[test]
fn getter_packet_id() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(12345u16)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

// to_buffers() tests
#[test]
#[cfg(feature = "std")]
fn to_buffers_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
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
    assert_eq!(all_bytes[0], 0x62); // PUBREL packet type
}

// Parse tests
#[test]
fn parse_minimal() {
    common::init_tracing();
    let original = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
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
    let (parsed, consumed) = mqtt::packet::v3_1_1::Pubrel::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
}

#[test]
fn parse_invalid_too_short() {
    common::init_tracing();
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v3_1_1::Pubrel::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Size tests
#[test]
fn size_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
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
    let original = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
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
    let (parsed, _) = mqtt::packet::v3_1_1::Pubrel::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
}

#[test]
fn test_packet_type() {
    common::init_tracing();
    let packet_type = mqtt::packet::v3_1_1::Pubrel::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Pubrel);
}
