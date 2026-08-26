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
use mqtt::connection::{ConnectionOptions, Event, MQTT_PACKET_SIZE_NO_LIMIT};
use mqtt::result_code::MqttError;
use mqtt_protocol_core::mqtt;

mod common;

fn has_error(events: &[Event], err: MqttError) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::NotifyError(x) if *x == err))
}

fn has_close(events: &[Event]) -> bool {
    events.iter().any(|e| matches!(e, Event::RequestClose))
}

fn sent_packet(events: &[Event]) -> Option<&mqtt::packet::Packet> {
    events.iter().find_map(|e| match e {
        Event::RequestSendPacket { packet, .. } => Some(packet),
        _ => None,
    })
}

fn max_packet_size_prop(props: &mqtt::packet::Properties) -> Option<u32> {
    props.iter().find_map(|p| match p {
        mqtt::packet::Property::MaximumPacketSize(v) => Some(v.val()),
        _ => None,
    })
}

// A CONNECT fixed header + remaining length claiming a huge body.
// Only these bytes are fed; the body never arrives.
// 0xFF 0xFF 0xFF 0x7F = 268,435,455 (maximum remaining length)
const HUGE_CONNECT_HEADER: [u8; 5] = [0x10, 0xFF, 0xFF, 0xFF, 0x7F];

///////////////////////////////////////////////////////////////////////////////
// ConnectionOptions

#[test]
fn options_default() {
    let opts = ConnectionOptions::default();
    assert_eq!(opts.maximum_packet_size_recv, MQTT_PACKET_SIZE_NO_LIMIT);
    assert_eq!(ConnectionOptions::new(), opts);
}

#[test]
fn options_maximum_packet_size_recv_clamp() {
    assert_eq!(
        ConnectionOptions::new()
            .maximum_packet_size_recv(0)
            .maximum_packet_size_recv,
        MQTT_PACKET_SIZE_NO_LIMIT
    );
    assert_eq!(
        ConnectionOptions::new()
            .maximum_packet_size_recv(u32::MAX)
            .maximum_packet_size_recv,
        MQTT_PACKET_SIZE_NO_LIMIT
    );
    assert_eq!(
        ConnectionOptions::new()
            .maximum_packet_size_recv(100)
            .maximum_packet_size_recv,
        100
    );
}

///////////////////////////////////////////////////////////////////////////////
// Limit is enforced before CONNECT/CONNACK (server side), on header only

#[test]
fn v5_0_server_rejects_huge_connect_before_connack() {
    common::init_tracing();
    let opts = ConnectionOptions::new().maximum_packet_size_recv(1024);
    let mut con = mqtt::Connection::<mqtt::role::Server>::with_options(mqtt::Version::V5_0, opts);

    let mut cursor = mqtt::common::Cursor::new(&HUGE_CONNECT_HEADER[..]);
    let events = con.recv(&mut cursor);

    // Rejected as soon as the remaining length is decoded, without the body
    assert!(has_close(&events), "{events:?}");
    assert!(has_error(&events, MqttError::PacketTooLarge), "{events:?}");
    // No DISCONNECT may be sent before CONNACK
    assert!(sent_packet(&events).is_none(), "{events:?}");
    // All header bytes consumed, nothing more requested
    assert_eq!(cursor.position() as usize, HUGE_CONNECT_HEADER.len());
}

#[test]
fn v3_1_1_server_rejects_huge_connect() {
    common::init_tracing();
    let opts = ConnectionOptions::new().maximum_packet_size_recv(1024);
    let mut con = mqtt::Connection::<mqtt::role::Server>::with_options(mqtt::Version::V3_1_1, opts);

    let mut cursor = mqtt::common::Cursor::new(&HUGE_CONNECT_HEADER[..]);
    let events = con.recv(&mut cursor);

    assert!(has_close(&events), "{events:?}");
    assert!(has_error(&events, MqttError::PacketTooLarge), "{events:?}");
    assert!(sent_packet(&events).is_none(), "{events:?}");
}

#[test]
fn v5_0_server_limit_exact_boundary() {
    common::init_tracing();
    // CONNECT with client id "cid1": build it and use its size as the limit
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .unwrap();
    let bytes = connect.to_continuous_buffer();
    let size = bytes.len() as u32;

    // limit == size: accepted
    let opts = ConnectionOptions::new().maximum_packet_size_recv(size);
    let mut con = mqtt::Connection::<mqtt::role::Server>::with_options(mqtt::Version::V5_0, opts);
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes[..]));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotifyPacketReceived(_))),
        "{events:?}"
    );

    // limit == size - 1: rejected
    let opts = ConnectionOptions::new().maximum_packet_size_recv(size - 1);
    let mut con = mqtt::Connection::<mqtt::role::Server>::with_options(mqtt::Version::V5_0, opts);
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes[..]));
    assert!(has_error(&events, MqttError::PacketTooLarge), "{events:?}");
    assert!(has_close(&events), "{events:?}");
}

#[test]
fn default_new_has_no_limit_before_connack() {
    common::init_tracing();
    // Backward compatibility: Connection::new() accepts the header and waits for the body
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let mut cursor = mqtt::common::Cursor::new(&HUGE_CONNECT_HEADER[..]);
    let events = con.recv(&mut cursor);
    assert!(events.is_empty(), "{events:?}");
}

///////////////////////////////////////////////////////////////////////////////
// CONNECT: automatic MaximumPacketSize property / explicit property rules

#[test]
fn v5_0_client_connect_auto_adds_maximum_packet_size() {
    common::init_tracing();
    let opts = ConnectionOptions::new().maximum_packet_size_recv(2048);
    let mut con = mqtt::Connection::<mqtt::role::Client>::with_options(mqtt::Version::V5_0, opts);

    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .unwrap();
    let events = con.checked_send(connect);

    let sent = sent_packet(&events).expect("RequestSendPacket");
    let mqtt::packet::Packet::V5_0Connect(c) = sent else {
        panic!("expected CONNECT, got {sent:?}");
    };
    assert_eq!(max_packet_size_prop(c.props()), Some(2048));

    // The sent packet must be self-consistent: re-parse the encoded bytes
    let bytes = c.to_continuous_buffer();
    let (parsed, consumed) = mqtt::packet::v5_0::Connect::parse(&bytes[2..]).unwrap();
    assert_eq!(consumed, bytes.len() - 2);
    assert_eq!(max_packet_size_prop(parsed.props()), Some(2048));
    assert_eq!(parsed.client_id(), "cid1");
}

#[test]
fn v5_0_client_connect_explicit_smaller_is_kept_and_enforced() {
    common::init_tracing();
    let opts = ConnectionOptions::new().maximum_packet_size_recv(2048);
    let mut con = mqtt::Connection::<mqtt::role::Client>::with_options(mqtt::Version::V5_0, opts);

    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .props(vec![mqtt::packet::MaximumPacketSize::new(30)
            .unwrap()
            .into()])
        .build()
        .unwrap();
    let events = con.checked_send(connect);
    let sent = sent_packet(&events).expect("RequestSendPacket");
    let mqtt::packet::Packet::V5_0Connect(c) = sent else {
        panic!("expected CONNECT, got {sent:?}");
    };
    assert_eq!(max_packet_size_prop(c.props()), Some(30));
    assert_eq!(
        c.props()
            .iter()
            .filter(|p| matches!(p, mqtt::packet::Property::MaximumPacketSize(_)))
            .count(),
        1
    );

    // CONNACK (5 bytes) is fine
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();
    let bytes = connack.to_continuous_buffer();
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes[..]));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::NotifyPacketReceived(_))),
        "{events:?}"
    );

    // A 40-byte PUBLISH exceeds the explicit 30 (though under the 2048 limit)
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("t")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(vec![0u8; 34])
        .build()
        .unwrap();
    let bytes = publish.to_continuous_buffer();
    assert!(bytes.len() > 30 && bytes.len() < 2048);
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes[..]));

    // Connected on v5.0: DISCONNECT(PacketTooLarge), RequestClose, NotifyError
    assert_eq!(events.len(), 3, "{events:?}");
    let sent = sent_packet(&events).expect("DISCONNECT");
    let mqtt::packet::Packet::V5_0Disconnect(d) = sent else {
        panic!("expected DISCONNECT, got {sent:?}");
    };
    assert_eq!(
        d.reason_code(),
        Some(mqtt::result_code::DisconnectReasonCode::PacketTooLarge)
    );
    assert!(has_close(&events));
    assert!(has_error(&events, MqttError::PacketTooLarge));
}

#[test]
fn v5_0_client_connect_explicit_larger_is_rejected() {
    common::init_tracing();
    let opts = ConnectionOptions::new().maximum_packet_size_recv(2048);
    let mut con = mqtt::Connection::<mqtt::role::Client>::with_options(mqtt::Version::V5_0, opts);

    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .props(vec![mqtt::packet::MaximumPacketSize::new(4096)
            .unwrap()
            .into()])
        .build()
        .unwrap();
    let events = con.checked_send(connect);
    assert_eq!(events.len(), 1, "{events:?}");
    assert!(has_error(&events, MqttError::ProtocolError));
    assert!(sent_packet(&events).is_none());

    // Connection is still usable: a valid CONNECT can be sent afterwards
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .unwrap();
    let events = con.checked_send(connect);
    assert!(sent_packet(&events).is_some(), "{events:?}");
}

#[test]
fn v5_0_client_connect_no_limit_adds_nothing() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .unwrap();
    let events = con.checked_send(connect);
    let sent = sent_packet(&events).expect("RequestSendPacket");
    let mqtt::packet::Packet::V5_0Connect(c) = sent else {
        panic!("expected CONNECT, got {sent:?}");
    };
    assert!(c.props().is_empty());
}

///////////////////////////////////////////////////////////////////////////////
// CONNACK: automatic MaximumPacketSize property / explicit property rules

fn v5_0_server_connecting_with_limit(limit: u32) -> mqtt::Connection<mqtt::role::Server> {
    let opts = ConnectionOptions::new().maximum_packet_size_recv(limit);
    let mut con = mqtt::Connection::<mqtt::role::Server>::with_options(mqtt::Version::V5_0, opts);
    common::v5_0_server_connecting(&mut con);
    con
}

#[test]
fn v5_0_server_connack_auto_adds_maximum_packet_size() {
    common::init_tracing();
    let mut con = v5_0_server_connecting_with_limit(4096);

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();
    let events = con.checked_send(connack);
    let sent = sent_packet(&events).expect("RequestSendPacket");
    let mqtt::packet::Packet::V5_0Connack(c) = sent else {
        panic!("expected CONNACK, got {sent:?}");
    };
    assert_eq!(max_packet_size_prop(c.props()), Some(4096));

    let bytes = c.to_continuous_buffer();
    let (parsed, consumed) = mqtt::packet::v5_0::Connack::parse(&bytes[2..]).unwrap();
    assert_eq!(consumed, bytes.len() - 2);
    assert_eq!(max_packet_size_prop(parsed.props()), Some(4096));
}

#[test]
fn v5_0_server_connack_error_reason_code_adds_nothing() {
    common::init_tracing();
    let mut con = v5_0_server_connecting_with_limit(4096);

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::NotAuthorized)
        .build()
        .unwrap();
    let events = con.checked_send(connack);
    let sent = sent_packet(&events).expect("RequestSendPacket");
    let mqtt::packet::Packet::V5_0Connack(c) = sent else {
        panic!("expected CONNACK, got {sent:?}");
    };
    assert!(c.props().is_empty());
}

#[test]
fn v5_0_server_connack_explicit_larger_is_rejected() {
    common::init_tracing();
    let mut con = v5_0_server_connecting_with_limit(4096);

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::MaximumPacketSize::new(8192)
            .unwrap()
            .into()])
        .build()
        .unwrap();
    let events = con.checked_send(connack);
    assert_eq!(events.len(), 1, "{events:?}");
    assert!(has_error(&events, MqttError::ProtocolError));
}

#[test]
fn v5_0_server_connack_explicit_smaller_is_kept() {
    common::init_tracing();
    let mut con = v5_0_server_connecting_with_limit(4096);

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::MaximumPacketSize::new(100)
            .unwrap()
            .into()])
        .build()
        .unwrap();
    let events = con.checked_send(connack);
    let sent = sent_packet(&events).expect("RequestSendPacket");
    let mqtt::packet::Packet::V5_0Connack(c) = sent else {
        panic!("expected CONNACK, got {sent:?}");
    };
    assert_eq!(max_packet_size_prop(c.props()), Some(100));

    // 101+ bytes PUBLISH is now rejected
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("t")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(vec![0u8; 100])
        .build()
        .unwrap();
    let bytes = publish.to_continuous_buffer();
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes[..]));
    assert!(has_error(&events, MqttError::PacketTooLarge), "{events:?}");
}

///////////////////////////////////////////////////////////////////////////////
// v3.1.1 limit is enforced after connection too, and notify_closed keeps the limit

#[test]
fn v3_1_1_limit_enforced_after_connected() {
    common::init_tracing();
    let opts = ConnectionOptions::new().maximum_packet_size_recv(64);
    let mut con = mqtt::Connection::<mqtt::role::Client>::with_options(mqtt::Version::V3_1_1, opts);
    common::v3_1_1_client_establish_connection(&mut con, true, false);

    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("t")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(vec![0u8; 100])
        .build()
        .unwrap();
    let bytes = publish.to_continuous_buffer();
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes[..]));
    // v3.1.1 cannot send DISCONNECT with a reason: just close
    assert!(sent_packet(&events).is_none(), "{events:?}");
    assert!(has_close(&events), "{events:?}");
    assert!(has_error(&events, MqttError::PacketTooLarge), "{events:?}");
}

#[test]
fn limit_survives_notify_closed() {
    common::init_tracing();
    let opts = ConnectionOptions::new().maximum_packet_size_recv(1024);
    let mut con = mqtt::Connection::<mqtt::role::Server>::with_options(mqtt::Version::V5_0, opts);
    common::v5_0_server_establish_connection(&mut con);
    let _ = con.notify_closed();

    let mut cursor = mqtt::common::Cursor::new(&HUGE_CONNECT_HEADER[..]);
    let events = con.recv(&mut cursor);
    assert!(has_error(&events, MqttError::PacketTooLarge), "{events:?}");
}

///////////////////////////////////////////////////////////////////////////////
// PacketBuilder direct

#[test]
fn packet_builder_rejects_before_body() {
    let mut builder = mqtt::connection::PacketBuilder::with_maximum_packet_size(10);
    assert_eq!(builder.maximum_packet_size(), 10);

    // total = 2 (header) + 9 = 11 > 10
    let data = [0x40u8, 0x09];
    let mut cursor = mqtt::common::Cursor::new(&data[..]);
    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Error(MqttError::PacketTooLarge) => {}
        other => panic!("expected PacketTooLarge, got {other:?}"),
    }

    // builder is reusable after the error
    builder.set_maximum_packet_size(11);
    let data = [0x40u8, 0x09, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut cursor = mqtt::common::Cursor::new(&data[..]);
    match builder.feed(&mut cursor) {
        mqtt::connection::PacketBuildResult::Complete(p) => {
            assert_eq!(p.remaining_length(), 9)
        }
        other => panic!("expected Complete, got {other:?}"),
    }
}
