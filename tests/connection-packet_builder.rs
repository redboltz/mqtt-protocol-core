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

mod common;
use std::sync::Arc;

#[test]
fn test_packet_data_methods() {
    common::init_tracing();
    // Test Normal PacketData
    let normal_data = mqtt::connection::PacketData::Normal(vec![1, 2, 3, 4]);
    assert_eq!(normal_data.as_slice(), &[1, 2, 3, 4]);
    assert_eq!(normal_data.len(), 4);
    assert!(!normal_data.is_empty());

    // Test Publish PacketData
    let publish_data = mqtt::connection::PacketData::Publish(Arc::from([5, 6, 7, 8].as_slice()));
    assert_eq!(publish_data.as_slice(), &[5, 6, 7, 8]);
    assert_eq!(publish_data.len(), 4);
    assert!(!publish_data.is_empty());

    // Test empty data
    let empty_data = mqtt::connection::PacketData::Normal(Vec::new());
    assert!(empty_data.is_empty());
    assert_eq!(empty_data.len(), 0);
}

#[test]
fn test_raw_packet_methods() {
    common::init_tracing();
    // CONNECT packet (packet type = 1)
    let connect_bytes = [
        0x10, 0x0A, // Fixed header + Remaining Length
        0x00, 0x04, b'M', b'Q', b'T', b'T', // Protocol name
        0x05, 0x00, 0x00, 0x3C, // Version, flags, Keep Alive
    ];

    let mut cursor = mqtt::common::Cursor::new(&connect_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    let connect_packet = match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => packet,
        _ => panic!("Expected Complete result"),
    };

    assert_eq!(connect_packet.packet_type(), 1);
    assert_eq!(connect_packet.flags(), 0);
    assert!(!connect_packet.is_publish());
    assert_eq!(connect_packet.remaining_length(), 10);

    // PUBLISH packet (packet type = 3)
    let publish_bytes = [
        0x31, 0x0D, // Fixed header + Remaining Length (with DUP flag)
        0x00, 0x04, b't', b'e', b's', b't', // Topic name
        b'p', b'a', b'y', b'l', b'o', b'a', b'd', // Payload
    ];

    let mut cursor = mqtt::common::Cursor::new(&publish_bytes[..]);
    builder.reset(); // Reset previous state

    let publish_packet = match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => packet,
        _ => panic!("Expected Complete result"),
    };

    assert_eq!(publish_packet.packet_type(), 3);
    assert_eq!(publish_packet.flags(), 1); // DUP flag is set
    assert!(publish_packet.is_publish());
    assert_eq!(publish_packet.remaining_length(), 13);

    // Test data access
    assert_eq!(
        publish_packet.data_as_slice(),
        &[0x00, 0x04, b't', b'e', b's', b't', b'p', b'a', b'y', b'l', b'o', b'a', b'd']
    );
}

#[test]
fn test_build_empty_payload_packet() {
    common::init_tracing();
    // PINGREQ packet (type = 12, no payload)
    let pingreq_bytes = [0xC0, 0x00]; // Fixed header = 0xC0, Remaining Length = 0
    let mut cursor = mqtt::common::Cursor::new(&pingreq_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 12);
            assert_eq!(packet.flags(), 0);
            assert_eq!(packet.remaining_length(), 0);
            assert!(!packet.is_publish());
        }
        _ => panic!("Expected Complete result"),
    }
}

#[test]
fn test_build_normal_packet() {
    common::init_tracing();
    // CONNECT packet
    let connect_bytes = [
        0x10, 0x0A, // Fixed header + Remaining Length
        0x00, 0x04, b'M', b'Q', b'T', b'T', // Protocol name
        0x05, // Protocol version (MQTT 5.0)
        0x00, // Connection flags
        0x00, 0x3C, // Keep Alive (60 seconds)
    ];

    let mut cursor = mqtt::common::Cursor::new(&connect_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 1);
            assert_eq!(packet.flags(), 0);
            assert_eq!(packet.remaining_length(), 10);

            let expected_data = [0x00, 0x04, b'M', b'Q', b'T', b'T', 0x05, 0x00, 0x00, 0x3C];
            assert_eq!(packet.data_as_slice(), &expected_data);

            // Verify that Normal variant is used for regular packets
            match packet.data {
                mqtt::connection::PacketData::Normal(_) => (),
                mqtt::connection::PacketData::Publish(_) => panic!("Expected Normal packet"),
            }
        }
        _ => panic!("Expected Complete result"),
    }
}

#[test]
fn test_build_publish_packet() {
    common::init_tracing();
    // PUBLISH packet
    let publish_bytes = [
        0x30, 0x0D, // Fixed header + Remaining Length
        0x00, 0x04, b't', b'e', b's', b't', // Topic name
        b'p', b'a', b'y', b'l', b'o', b'a', b'd', // Payload
    ];

    let mut cursor = mqtt::common::Cursor::new(&publish_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 3);
            assert_eq!(packet.flags(), 0);
            assert_eq!(packet.remaining_length(), 13);

            // Verify that Publish variant is used for PUBLISH packets
            match &packet.data {
                mqtt::connection::PacketData::Normal(_) => panic!("Expected Publish packet"),
                mqtt::connection::PacketData::Publish(arc) => {
                    let expected_data = [
                        0x00, 0x04, b't', b'e', b's', b't', b'p', b'a', b'y', b'l', b'o', b'a',
                        b'd',
                    ];
                    assert_eq!(arc.as_ref(), &expected_data);
                }
            }
        }
        _ => panic!("Expected Complete result"),
    }
}

#[test]
fn test_multi_byte_remaining_length() {
    common::init_tracing();
    // 2-byte Remaining Length (example: 321 = 0xC1, 0x02)
    let packet_bytes = vec![
        0x20, 0xC1, 0x02, // CONNACK packet, Remaining Length = 321
    ];
    let payload = vec![0; 321]; // 321-byte dummy payload
    let mut full_packet = packet_bytes.clone();
    full_packet.extend_from_slice(&payload);

    // Fix: Convert vector to slice reference
    let mut cursor = mqtt::common::Cursor::new(&full_packet[..]); // Convert &Vec<u8> → &[u8]
    let mut builder = mqtt::connection::PacketBuilder::new();

    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 2); // CONNACK
            assert_eq!(packet.remaining_length(), 321);
        }
        _ => panic!("Expected Complete result"),
    }
}

#[test]
fn test_incomplete_packet() {
    common::init_tracing();
    // Packet that ends with just the header (no remaining bytes)
    let incomplete_bytes = [0x10, 0x0A]; // CONNECT, needs 10 bytes of payload but missing

    let mut cursor = mqtt::common::Cursor::new(&incomplete_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result"),
    }
}

#[test]
fn test_malformed_remaining_length() {
    common::init_tracing();
    // Invalid Remaining Length (5 bytes or more)
    let malformed_bytes = [0x10, 0x80, 0x80, 0x80, 0x80];

    let mut cursor = mqtt::common::Cursor::new(&malformed_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Error(
            mqtt::result_code::MqttError::MalformedPacket,
        ) => (),
        _ => panic!("Expected MalformedPacket error"),
    }
}

#[test]
fn test_fragmented_packet_feed() {
    common::init_tracing();
    // Feed CONNECT packet in multiple parts
    let connect_bytes = [
        0x10, 0x0A, // Fixed header + Remaining Length
        0x00, 0x04, b'M', b'Q', b'T', b'T', // Protocol name
        0x05, // Protocol version
        0x00, // Connection flags
        0x00, 0x3C, // Keep Alive
    ];

    let mut builder = mqtt::connection::PacketBuilder::new();

    // Part 1: Fixed header only
    let mut cursor1 = mqtt::common::Cursor::new(&connect_bytes[0..1]);
    match builder.feed(&mut cursor1) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after first feed"),
    }

    // Part 2: Remaining Length
    let mut cursor2 = mqtt::common::Cursor::new(&connect_bytes[1..2]);
    match builder.feed(&mut cursor2) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after second feed"),
    }

    // Part 3: Part of payload
    let mut cursor3 = mqtt::common::Cursor::new(&connect_bytes[2..8]);
    match builder.feed(&mut cursor3) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after third feed"),
    }

    // Part 4: Remaining payload
    let mut cursor4 = mqtt::common::Cursor::new(&connect_bytes[8..]);
    match builder.feed(&mut cursor4) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 1);
            assert_eq!(packet.remaining_length(), 10);
        }
        _ => panic!("Expected Complete result after final feed"),
    }
}

#[test]
fn test_reset_builder() {
    common::init_tracing();
    let mut builder = mqtt::connection::PacketBuilder::new();

    // Process partial packet
    let partial_bytes = [0x10, 0x0A]; // CONNECT header only
    let mut cursor = mqtt::common::Cursor::new(&partial_bytes[..]);
    assert!(matches!(
        builder.feed(&mut cursor),
        mqtt::connection::PacketBuildResult::Incomplete
    ));

    // Remove internal state verification
    // assert!(builder.raw_buf.is_none()); // ← Removed
    // assert!(!builder.header_buf.is_empty()); // ← Removed

    // Reset
    builder.reset();

    // Remove internal state verification
    // assert!(builder.raw_buf.is_none()); // ← Removed
    // assert!(builder.header_buf.is_empty()); // ← Removed

    // Instead, verify functionally: should be able to process new packets after reset
    let complete_bytes = [0xC0, 0x00]; // PINGREQ
    let mut cursor2 = mqtt::common::Cursor::new(&complete_bytes[..]);
    match builder.feed(&mut cursor2) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 12); // PINGREQ
            assert_eq!(packet.flags(), 0);
            assert_eq!(packet.remaining_length(), 0);
        }
        _ => panic!("Expected Complete result after reset"),
    }
}

#[test]
fn test_empty_data_access() {
    common::init_tracing();
    // Empty DISCONNECT packet
    let disconnect_bytes = [0xE0, 0x00]; // DISCONNECT, Remaining Length = 0
    let mut cursor = mqtt::common::Cursor::new(&disconnect_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    let packet = match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => packet,
        _ => panic!("Expected Complete result"),
    };

    assert_eq!(packet.packet_type(), 14); // DISCONNECT
    assert_eq!(packet.data_as_slice().len(), 0);
    assert_eq!(packet.remaining_length(), 0);
}

#[test]
fn test_empty_publish_packet() {
    common::init_tracing();
    // Empty PUBLISH packet
    let empty_publish_bytes = [0x30, 0x00]; // PUBLISH, Remaining Length = 0
    let mut cursor = mqtt::common::Cursor::new(&empty_publish_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 3); // PUBLISH
            assert_eq!(packet.remaining_length(), 0);
            assert!(packet.is_publish());

            // Verify that Publish variant is used for empty PUBLISH and contains empty Arc
            match &packet.data {
                mqtt::connection::PacketData::Normal(_) => panic!("Expected Publish packet"),
                mqtt::connection::PacketData::Publish(arc) => {
                    assert_eq!(arc.len(), 0);
                }
            }
        }
        _ => panic!("Expected Complete result"),
    }
}

// Add after existing use declarations

#[test]
fn test_feed_with_empty_data() {
    common::init_tracing();
    // Test with empty buffer
    let empty_bytes: [u8; 0] = [];
    let mut cursor = mqtt::common::Cursor::new(&empty_bytes[..]);
    let mut builder = mqtt::connection::PacketBuilder::new();

    // Passing empty data returns Incomplete (line 162)
    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result for empty data"),
    }
}

#[test]
fn test_two_phase_packet_building() {
    common::init_tracing();
    // CONNECT packet (packet type = 1)
    let connect_bytes = [
        0x10, 0x0A, // Fixed header + Remaining Length
        0x00, 0x04, b'M', b'Q', b'T', b'T', // Protocol name
        0x05, 0x00, 0x00, 0x3C, // Version, flags, Keep Alive
    ];

    let mut builder = mqtt::connection::PacketBuilder::new();

    // First send only header and length field
    let mut cursor1 = mqtt::common::Cursor::new(&connect_bytes[0..2]);
    match builder.feed(&mut cursor1) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after partial feed"),
    }

    // Send remaining data
    let mut cursor2 = mqtt::common::Cursor::new(&connect_bytes[2..]);
    match builder.feed(&mut cursor2) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 1);
            assert_eq!(packet.remaining_length(), 10);
        }
        _ => panic!("Expected Complete result after second feed"),
    }
}

#[test]
fn test_incomplete_remaining_length() {
    common::init_tracing();
    // Packet with 3-byte Remaining Length
    // Interrupted with incomplete length field
    let partial_bytes = [
        0x20, 0x80, 0x80, // CONNACK packet + incomplete Remaining Length
    ];

    let mut builder = mqtt::connection::PacketBuilder::new();

    let mut cursor = mqtt::common::Cursor::new(&partial_bytes[..]);
    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result for partial remaining length"),
    }

    // Next part (final byte without continuation bit)
    let remaining_bytes = [0x01]; // Final byte (total 128*128 + 1 bytes)
    let mut cursor2 = mqtt::common::Cursor::new(&remaining_bytes[..]);
    match builder.feed(&mut cursor2) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after setting full remaining length"),
    }
}

#[test]
fn test_payload_zero_available_bytes() {
    common::init_tracing();
    // Send only part of packet to enter Payload state
    let connect_bytes = [
        0x10, 0x0A, // Fixed header + Remaining Length
        0x00, 0x04, b'M', b'Q', // Part of protocol name (not complete)
    ];

    let mut builder = mqtt::connection::PacketBuilder::new();

    // First header and length field
    let mut cursor1 = mqtt::common::Cursor::new(&connect_bytes[0..2]);
    match builder.feed(&mut cursor1) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after header"),
    }

    // Next part of payload
    let mut cursor2 = mqtt::common::Cursor::new(&connect_bytes[2..]);
    match builder.feed(&mut cursor2) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after partial payload"),
    }

    // Send empty feed (available = 0 case)
    let empty_bytes: [u8; 0] = [];
    let mut cursor3 = mqtt::common::Cursor::new(&empty_bytes[..]);
    match builder.feed(&mut cursor3) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result with zero available bytes"),
    }
}

#[test]
fn test_multi_packet_in_one_feed() {
    common::init_tracing();
    // Test case with two consecutive packets
    let two_packets = [
        // 1st: PINGREQ
        0xC0, 0x00, // 2nd: PINGRESP
        0xD0, 0x00,
    ];

    let mut builder = mqtt::connection::PacketBuilder::new();
    let mut cursor = mqtt::common::Cursor::new(&two_packets[..]);

    // Read first packet
    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 12); // PINGREQ
        }
        _ => panic!("Expected Complete result for first packet"),
    }

    // Cursor position should have advanced automatically
    // Should be able to read second packet too
    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(packet) => {
            assert_eq!(packet.packet_type(), 13); // PINGRESP
        }
        _ => panic!("Expected Complete result for second packet"),
    }

    // No more data to read
    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Incomplete => (),
        _ => panic!("Expected Incomplete result after reading all packets"),
    }
}
