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
