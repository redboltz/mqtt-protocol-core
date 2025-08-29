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

///////////////////////////////////////////////////////////////////////////////

// Test recv method Incomplete pattern

#[test]
fn recv_incomplete_partial_fixed_header() {
    common::init_tracing();
    // Test with partial fixed header data
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Only provide first byte of fixed header (incomplete)
    let data = [0x10]; // CONNECT packet type but no remaining length
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());

    let events = con.recv(&mut cursor);

    // Should return empty events for incomplete packet
    assert_eq!(events.len(), 0);
}

#[test]
fn recv_incomplete_partial_remaining_length() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Provide fixed header with incomplete remaining length
    let data = [0x10, 0x80]; // CONNECT packet with incomplete multi-byte remaining length
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());

    let events = con.recv(&mut cursor);

    // Should return empty events for incomplete packet
    assert_eq!(events.len(), 0);
}

#[test]
fn recv_incomplete_partial_variable_header() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Provide fixed header with complete remaining length but incomplete variable header
    let data = [0x10, 0x10, 0x00]; // CONNECT packet with remaining length 16 but only partial variable header
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());

    let events = con.recv(&mut cursor);

    // Should return empty events for incomplete packet
    assert_eq!(events.len(), 0);
}

///////////////////////////////////////////////////////////////////////////////

// Test recv method Error pattern

#[test]
fn recv_error_malformed_packet_invalid_remaining_length() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Create malformed packet with invalid remaining length encoding
    let data = [0x10, 0xFF, 0xFF, 0xFF, 0xFF]; // Invalid remaining length (too many continuation bytes)
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());

    let events = con.recv(&mut cursor);

    // Should return error events
    assert_eq!(events.len(), 2);

    // First event should be RequestClose
    match &events[0] {
        mqtt::connection::Event::RequestClose => {}
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }

    // Second event should be NotifyError with MalformedPacket
    match &events[1] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::MalformedPacket);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[1]),
    }
}

#[test]
fn recv_error_malformed_packet_v3_1_1_connect_cid_not_valid() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);

    // Create malformed CONNECT packet with ClientId length specified but insufficient data
    // MQTT v3.1.1 CONNECT packet structure:
    // Fixed Header: 0x10 (CONNECT packet type), remaining length
    // Variable Header: Protocol Name (6 bytes: 0x00,0x04,"MQTT"), Protocol Version (0x04), Connect Flags (0x02), Keep Alive (2 bytes)
    // Payload: Client Identifier length (2 bytes) + Client Identifier data (but we provide insufficient data)
    let data = [
        0x10, // CONNECT packet type
        0x0C, // Remaining length: 12 bytes
        0x00, 0x04, // Protocol name length
        b'M', b'Q', b'T', b'T', // Protocol name "MQTT"
        0x04, // Protocol version (v3.1.1)
        0x02, // Connect flags (clean session)
        0x00, 0x3C, // Keep alive (60 seconds)
        0x00,
        0x05, // Client ID length: 5 bytes (but we don't provide the 5 bytes of data)
              // Missing Client ID data - this makes the packet malformed
    ];
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());

    let events = con.recv(&mut cursor);

    // Should return error events
    assert_eq!(events.len(), 2);

    // First event should be RequestSendPacket with CONNACK containing IdentifierRejected
    match &events[0] {
        mqtt::connection::Event::RequestSendPacket { packet, .. } => {
            if let mqtt::packet::Packet::V3_1_1Connack(connack) = packet {
                assert_eq!(
                    connack.return_code(),
                    mqtt::result_code::ConnectReturnCode::IdentifierRejected
                );
            } else {
                panic!("Expected CONNACK packet, got {:?}", packet);
            }
        }
        _ => panic!("Expected RequestSendPacket event, got {:?}", events[0]),
    }

    // Second event should be NotifyError with ClientIdentifierNotValid
    match &events[1] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(
                *error,
                mqtt::result_code::MqttError::ClientIdentifierNotValid
            );
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[1]),
    }
}
