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
use mqtt_protocol_core::mqtt::prelude::*;
mod common;
use common::*;

///////////////////////////////////////////////////////////////////////////////

// disconnected

#[test]
fn v3_1_1_client_send_connect() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let send_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_session(true)
        .build()
        .expect("Failed to build Connect packet")
        .into();
    let events = con.send(send_packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::RequestSendPacket {
        packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*packet, send_packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }
}

// connected

#[test]
fn v3_1_1_server_send_suback() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_establish_connection(&mut con, true, false);

    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con.send(packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v5_0_client_send_suback() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut con);

    let qh = con.get_qos2_publish_handled();
    assert!(qh.is_empty());

    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Publish::builder()
        .packet_id(packet_id)
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .payload("payload")
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let qh = con.get_qos2_publish_handled();
    assert_eq!(qh.len(), 1);

    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(packet_id)
        .reason_code(mqtt::result_code::PubrecReasonCode::NotAuthorized)
        .build()
        .expect("Failed to build Pubrec packet")
        .into();
    let _events = con.send(packet.clone());

    let qh = con.get_qos2_publish_handled();
    assert_eq!(qh.len(), 0);
}

#[test]
fn v5_0_server_send_suback() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    v5_0_server_establish_connection(&mut con);

    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con.send(packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_send_unsuback() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_establish_connection(&mut con, true, false);

    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build unsubscribe packet")
        .into();
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build unsuback packet")
        .into();
    let events = con.send(packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v5_0_server_send_unsuback() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    v5_0_server_establish_connection(&mut con);

    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build unsubscribe packet")
        .into();
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con.send(packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_send_disconnect() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut con, true, false);

    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Disconnect::builder()
        .build()
        .expect("Failed to build disconnect packet")
        .into();
    let events = con.send(packet.clone());
    assert_eq!(events.len(), 2);

    if let mqtt::connection::Event::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }

    if let mqtt::connection::Event::RequestClose = &events[1] {
        // Expected RequestClose event
    } else {
        assert!(
            false,
            "Expected RequestClose event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v5_0_client_send_pingreq() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    con.set_pingresp_recv_timeout(Some(5000));
    v5_0_client_establish_connection(&mut con);

    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pingreq::new().into();
    let events = con.send(packet.clone());
    assert_eq!(events.len(), 2);

    if let mqtt::connection::Event::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }

    if let mqtt::connection::GenericEvent::RequestTimerReset {
        kind: mqtt::connection::TimerKind::PingrespRecv,
        duration_ms,
    } = &events[1]
    {
        assert_eq!(*duration_ms, 5000);
    } else {
        assert!(
            false,
            "Expected RequestTimerReset event with PingrespRecv, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v5_0_client_send_auth() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    con.set_pingresp_recv_timeout(Some(5000));
    v5_0_client_establish_connection(&mut con);

    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .expect("Failed to build auth packet")
        .into();
    let events = con.send(packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v5_0_client_send_connect_keep_alive() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .keep_alive(10)
        .build()
        .expect("Failed to build Connect packet");
    let events = con.checked_send(packet.clone());
    assert_eq!(events.len(), 2);

    // Test first event: RequestSendPacket
    if let mqtt::connection::GenericEvent::RequestSendPacket {
        packet: event_packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*event_packet, packet.into());
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }

    // Test second event: RequestTimerReset for PingreqSend
    if let mqtt::connection::GenericEvent::RequestTimerReset {
        kind: mqtt::connection::TimerKind::PingreqSend,
        duration_ms: timeout_ms,
    } = &events[1]
    {
        assert_eq!(*timeout_ms, 10000); // 10 seconds * 1000ms
    } else {
        assert!(
            false,
            "Expected RequestTimerReset event with PingreqSend, but got: {:?}",
            events[1]
        );
    }
}
