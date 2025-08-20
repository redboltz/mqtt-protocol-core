#![cfg(feature = "std")]

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
use mqtt_protocol_core::default_alias;
use mqtt_protocol_core::mqtt;
mod common;
use common::*;

///////////////////////////////////////////////////////////////////////////////

// Test notify_timer_fired method - PingreqSend timer

#[test]
fn notify_timer_fired_pingreq_send_v3_1_1_connected() {
    common::init_tracing();
    // Test PingreqSend timer when connected with v3.1.1
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Establish connection
    v3_1_1_client_establish_connection(&mut con, true, false);

    // Fire PingreqSend timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingreqSend);

    // Should send PINGREQ packet
    assert!(events.len() >= 1);

    // Should contain RequestSendPacket event
    let has_send_packet = events
        .iter()
        .any(|e| matches!(e, mqtt::connection::Event::RequestSendPacket { .. }));
    assert!(has_send_packet, "Expected RequestSendPacket event");
}

#[test]
fn notify_timer_fired_pingreq_send_v5_0_connected() {
    common::init_tracing();
    // Test PingreqSend timer when connected with v5.0
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Establish connection
    v5_0_client_establish_connection(&mut con);

    // Fire PingreqSend timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingreqSend);

    // Should send PINGREQ packet
    assert!(events.len() >= 1);

    // Should contain RequestSendPacket event
    let has_send_packet = events
        .iter()
        .any(|e| matches!(e, mqtt::connection::Event::RequestSendPacket { .. }));
    assert!(has_send_packet, "Expected RequestSendPacket event");
}

#[test]
fn notify_timer_fired_pingreq_send_disconnected() {
    common::init_tracing();
    // Test PingreqSend timer when not connected
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Don't establish connection (remain disconnected)

    // Fire PingreqSend timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingreqSend);

    // Should not send PINGREQ when disconnected
    assert_eq!(events.len(), 0);
}

///////////////////////////////////////////////////////////////////////////////

// Test notify_timer_fired method - PingreqRecv timer

#[test]
fn notify_timer_fired_pingreq_recv_v3_1_1() {
    common::init_tracing();
    // Test PingreqRecv timer with v3.1.1 (should close connection)
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);

    // Fire PingreqRecv timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingreqRecv);

    // Should request connection close for v3.1.1
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::RequestClose => {}
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }
}

#[test]
fn notify_timer_fired_pingreq_recv_v5_0_connected() {
    common::init_tracing();
    // Test PingreqRecv timer with v5.0 when connected (should send DISCONNECT)
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Establish connection
    v5_0_server_establish_connection(&mut con);

    // Fire PingreqRecv timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingreqRecv);

    // Should send DISCONNECT packet with KeepAliveTimeout
    assert!(events.len() >= 1);

    // Should contain RequestSendPacket event
    let has_send_packet = events
        .iter()
        .any(|e| matches!(e, mqtt::connection::Event::RequestSendPacket { .. }));
    assert!(has_send_packet, "Expected RequestSendPacket event");
}

#[test]
fn notify_timer_fired_pingreq_recv_v5_0_disconnected() {
    common::init_tracing();
    // Test PingreqRecv timer with v5.0 when not connected
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Don't establish connection (remain disconnected)

    // Fire PingreqRecv timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingreqRecv);

    // Should not send DISCONNECT when disconnected
    assert_eq!(events.len(), 0);
}

///////////////////////////////////////////////////////////////////////////////

// Test notify_timer_fired method - PingrespRecv timer

#[test]
fn notify_timer_fired_pingresp_recv_v3_1_1() {
    common::init_tracing();
    // Test PingrespRecv timer with v3.1.1 (should close connection)
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Fire PingrespRecv timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingrespRecv);

    // Should request connection close for v3.1.1
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::RequestClose => {}
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }
}

#[test]
fn notify_timer_fired_pingresp_recv_v5_0_connected() {
    common::init_tracing();
    // Test PingrespRecv timer with v5.0 when connected (should send DISCONNECT)
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Establish connection
    v5_0_client_establish_connection(&mut con);

    // Fire PingrespRecv timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingrespRecv);

    // Should send DISCONNECT packet with KeepAliveTimeout
    assert!(events.len() >= 1);

    // Should contain RequestSendPacket event
    let has_send_packet = events
        .iter()
        .any(|e| matches!(e, mqtt::connection::Event::RequestSendPacket { .. }));
    assert!(has_send_packet, "Expected RequestSendPacket event");
}

#[test]
fn notify_timer_fired_pingresp_recv_v5_0_disconnected() {
    common::init_tracing();
    // Test PingrespRecv timer with v5.0 when not connected
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Don't establish connection (remain disconnected)

    // Fire PingrespRecv timer
    let events = con.notify_timer_fired(mqtt::connection::TimerKind::PingrespRecv);

    // Should not send DISCONNECT when disconnected
    assert_eq!(events.len(), 0);
}
