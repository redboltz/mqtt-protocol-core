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
// tests/connection-pingreq-interval.rs
// Tests for set_pingreq_send_interval functionality
use mqtt_protocol_core::mqtt;
mod common;

use crate::common::v5_0_client_establish_connection;

#[test]
fn test_set_pingreq_send_interval_enable_disconnected() {
    common::init_tracing();
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);

    // Enable PINGREQ timer with 30 second interval
    let events = connection.set_pingreq_send_interval(Some(30000));

    assert!(events.is_empty());
}

#[test]
fn test_set_pingreq_send_interval_enable_connected() {
    common::init_tracing();
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    // Enable PINGREQ timer with 30 second interval
    let events = connection.set_pingreq_send_interval(Some(30000));

    // Should return one TimerReset event
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::GenericEvent::RequestTimerReset { kind, duration_ms } => {
            assert_eq!(kind, &mqtt::connection::TimerKind::PingreqSend);
            assert_eq!(duration_ms, &30000);
        }
        _ => panic!("Expected TimerReset event"),
    }
}

#[test]
fn test_set_pingreq_send_interval_disable_disconnected() {
    common::init_tracing();
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);

    // First enable the timer
    connection.set_pingreq_send_interval(Some(15000));

    // Then disable it with 0 duration
    let events = connection.set_pingreq_send_interval(Some(0));

    assert!(events.is_empty());
}

#[test]
fn test_set_pingreq_send_interval_disable_connected() {
    common::init_tracing();
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    // First enable the timer
    connection.set_pingreq_send_interval(Some(15000));

    // Then disable it with 0 duration
    let events = connection.set_pingreq_send_interval(Some(0));

    // Should return one TimerCancel event
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::GenericEvent::RequestTimerCancel(kind) => {
            assert_eq!(kind, &mqtt::connection::TimerKind::PingreqSend);
        }
        _ => panic!("Expected TimerCancel event"),
    }
}

#[test]
fn test_set_pingreq_send_interval_update_disconnected() {
    common::init_tracing();
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);

    // Set initial interval
    let events1 = connection.set_pingreq_send_interval(Some(10000));
    assert!(events1.is_empty());

    // Update to different interval
    let events2 = connection.set_pingreq_send_interval(Some(20000));
    assert!(events2.is_empty());
}

#[test]
fn test_set_pingreq_send_interval_update_connected() {
    common::init_tracing();
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    // Set initial interval
    let events1 = connection.set_pingreq_send_interval(Some(10000));
    assert_eq!(events1.len(), 1);

    // Update to different interval
    let events2 = connection.set_pingreq_send_interval(Some(20000));
    assert_eq!(events2.len(), 1);
    match &events2[0] {
        mqtt::connection::GenericEvent::RequestTimerReset { kind, duration_ms } => {
            assert_eq!(kind, &mqtt::connection::TimerKind::PingreqSend);
            assert_eq!(duration_ms, &20000);
        }
        _ => panic!("Expected TimerReset event"),
    }
}

#[test]
fn test_set_pingreq_send_interval_server_keep_alive() {
    common::init_tracing();
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_start(true)
        .build()
        .unwrap();
    let _events = connection.checked_send(connect.clone());

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::ServerKeepAlive::new(1).unwrap().into()])
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // Check RequestTimerReset event
    if let mqtt::connection::Event::RequestTimerReset { kind, duration_ms } = &events[0] {
        assert_eq!(*kind, mqtt::connection::TimerKind::PingreqSend);
        assert_eq!(*duration_ms, 1000);
    } else {
        panic!("Expected RequestTimerReset event, got: {:?}", events[0]);
    }

    // Check NotifyPacketReceived event for connack
    if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[1] {
        if let mqtt::packet::GenericPacket::V5_0Connack(connack_received) = packet {
            assert_eq!(*connack_received, connack);
        } else {
            panic!("Expected V5_0Connack packet, got: {packet:?}");
        }
    } else {
        panic!("Expected NotifyPacketReceived event, got: {:?}", events[1]);
    }
}
