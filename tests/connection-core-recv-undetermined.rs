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

#[test]
fn undetermined_server_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::Undetermined);

    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    // Receive CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let bytes = connect.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(connection.get_protocol_version(), mqtt::Version::V3_1_1);

    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => match packet {
            mqtt::packet::GenericPacket::V3_1_1Connect(packet) => {
                assert_eq!(*packet, connect);
            }
            _ => panic!("Expected V3_1_1Connect packet, got {:?}", packet),
        },
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[0]),
    }
}

#[test]
fn undetermined_server_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::Undetermined);

    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    // Receive CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let bytes = connect.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(connection.get_protocol_version(), mqtt::Version::V5_0);

    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => match packet {
            mqtt::packet::GenericPacket::V5_0Connect(packet) => {
                assert_eq!(*packet, connect);
            }
            _ => panic!("Expected V5_0Connect packet, got {:?}", packet),
        },
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[0]),
    }
}

#[test]
fn undetermined_server_error_short() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::Undetermined);

    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    // Receive CONNECT
    let data = [
        0x10, // CONNECT packet type
        0x06, // Remaining length: 6 bytes
        0x00, 0x04, // Protocol name length
        b'M', b'Q', b'T', b'T', // Protocol name "MQTT"
    ];
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());
    let events = connection.recv(&mut cursor);
    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::MalformedPacket);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[0]),
    }
}

#[test]
fn undetermined_server_error_version() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::Undetermined);

    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    // Receive CONNECT
    let data = [
        0x10, // CONNECT packet type
        0x11, // Remaining length: 17 bytes
        0x00, 0x04, // Protocol name length
        b'M', b'Q', b'T', b'T', // Protocol name "MQTT"
        0x03, // Protocol version (v3.0 - unsupported)
        0x02, // Connect flags (clean session)
        0x00, 0x3C, // Keep alive (60 seconds)
        0x00, 0x05, // Client ID length: 5 bytes
        b't', b'e', b's', b't', b'1', // Client ID "test1"
    ];
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());
    let events = connection.recv(&mut cursor);
    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(
                *error,
                mqtt::result_code::MqttError::UnsupportedProtocolVersion
            );
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[0]),
    }
}

#[test]
fn undetermined_server_error_type() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::Undetermined);

    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    // Receive CONNECT
    let data = [
        0xF0, // Invalid packet type
        0x11, // Remaining length: 17 bytes
        0x00, 0x04, // Protocol name length
        b'M', b'Q', b'T', b'T', // Protocol name "MQTT"
        0x03, // Protocol version (v3.0 - unsupported)
        0x02, // Connect flags (clean session)
        0x00, 0x3C, // Keep alive (60 seconds)
        0x00, 0x05, // Client ID length: 5 bytes
        b't', b'e', b's', b't', b'1', // Client ID "test1"
    ];
    let mut cursor = mqtt::common::Cursor::new(data.as_slice());
    let events = connection.recv(&mut cursor);
    assert_eq!(
        connection.get_protocol_version(),
        mqtt::Version::Undetermined
    );

    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::MalformedPacket);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[0]),
    }
}
