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

// Build success tests
#[test]
fn build_success() {
    common::init_tracing();
    let _packet = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    // Disconnect packet has no fields to check
    assert!(true); // Just ensure it builds successfully
}

// Display tests
#[test]
fn display_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("disconnect"));
}

// Debug tests
#[test]
fn debug_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("disconnect"));
}

// to_buffers() tests
#[test]
#[cfg(feature = "std")]
fn to_buffers_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    let buffers = packet.to_buffers();
    assert!(!buffers.is_empty());

    // Collect all bytes
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Check fixed header
    assert_eq!(all_bytes[0], 0xE0); // DISCONNECT packet type

    // Should be minimal size (just fixed header + remaining length = 0)
    assert_eq!(all_bytes.len(), 2);
}

#[test]
#[cfg(not(feature = "std"))]
fn to_continuous_buffer_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    let all_bytes = packet.to_continuous_buffer();
    assert!(!all_bytes.is_empty());

    // Check fixed header
    assert_eq!(all_bytes[0], 0xE0); // DISCONNECT packet type

    // Should be minimal size (just fixed header + remaining length = 0)
    assert_eq!(all_bytes.len(), 2);
}

// Parse tests
#[test]
fn parse_minimal() {
    common::init_tracing();
    let original = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let continuous = original.to_continuous_buffer();
    let data = &continuous[2..]; // Skip fixed header and remaining length

    let (_parsed, consumed) = mqtt::packet::v3_1_1::Disconnect::parse(data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(consumed, 0); // No payload for DISCONNECT
}

// Size tests
#[test]
fn size_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    let size = packet.size();
    assert!(size > 0);

    // Verify size matches actual buffer size using to_continuous_buffer
    let continuous = packet.to_continuous_buffer();
    assert_eq!(size, continuous.len());

    #[cfg(feature = "std")]
    {
        // Also verify to_buffers() produces same result when std is available
        let buffers = packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);

        // Verify to_buffers() and to_continuous_buffer() produce same result
        let mut from_buffers = Vec::new();
        for buf in buffers {
            from_buffers.extend_from_slice(&buf);
        }
        assert_eq!(continuous, from_buffers);
    }
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    common::init_tracing();
    let original = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let continuous = original.to_continuous_buffer();
    let data = &continuous[2..]; // Skip fixed header and remaining length

    let (parsed, consumed) = mqtt::packet::v3_1_1::Disconnect::parse(data).unwrap();
    assert_eq!(consumed, 0);
    assert_eq!(parsed, original);
}

#[test]
fn test_packet_type() {
    common::init_tracing();
    let packet_type = mqtt::packet::v3_1_1::Disconnect::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Disconnect);
}
