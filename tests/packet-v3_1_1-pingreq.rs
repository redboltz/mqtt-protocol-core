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

// Build success tests
#[test]
fn build_success() {
    let _packet = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();

    // Pingreq packet has no fields to check
    assert!(true); // Just ensure it builds successfully
}

// Display tests
#[test]
fn display_minimal() {
    let packet = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();

    let display_str = format!("{}", packet);
    assert!(display_str.contains("pingreq"));
}

// Debug tests
#[test]
fn debug_minimal() {
    let packet = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();

    let debug_str = format!("{:?}", packet);
    assert!(debug_str.contains("pingreq"));
}

// to_buffers() tests
#[test]
fn to_buffers_minimal() {
    let packet = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();

    let buffers = packet.to_buffers();
    assert!(!buffers.is_empty());

    // Collect all bytes
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Check fixed header
    assert_eq!(all_bytes[0], 0xC0); // PINGREQ packet type

    // Should be minimal size (just fixed header + remaining length = 0)
    assert_eq!(all_bytes.len(), 2);
}

// Parse tests
#[test]
fn parse_minimal() {
    let original = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        // Skip fixed header and remaining length
        data.extend_from_slice(buf);
    }

    let (_parsed, consumed) = mqtt::packet::v3_1_1::Pingreq::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(consumed, 0); // No payload for PINGREQ
}

// Size tests
#[test]
fn size_minimal() {
    let packet = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();

    let size = packet.size();
    assert!(size > 0);

    // Verify size matches actual buffer size
    let buffers = packet.to_buffers();
    let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(size, actual_size);
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    let original = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (_parsed, _) = mqtt::packet::v3_1_1::Pingreq::parse(&data).unwrap();

    // For pingreq, we just need to ensure parse succeeds
    // since there are no fields to compare
    assert!(true);
}
