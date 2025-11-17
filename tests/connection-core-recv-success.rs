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
use common::*;

#[test]
fn client_recv_publish_qos0_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    // Create and receive QoS1 PUBLISH A
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload A")
        .build()
        .unwrap();
    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, publish_a.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn client_recv_pingresp_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    connection.set_pingresp_recv_timeout(3000);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let packet = mqtt::packet::v3_1_1::Pingreq::new();
    let _events = connection.checked_send(packet);

    let packet = mqtt::packet::v3_1_1::Pingresp::new();
    let bytes = packet.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // Check RequestTimerCancel event
    if let mqtt::connection::Event::RequestTimerCancel(kind) = &events[0] {
        assert_eq!(*kind, mqtt::connection::TimerKind::PingrespRecv);
    } else {
        panic!("Expected RequestTimerCancel event, got: {:?}", events[0]);
    }

    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(evt_packet) => {
            assert_eq!(*evt_packet, packet.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn server_recv_disconnect_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_establish_connection(&mut connection, true, false);

    let packet = mqtt::packet::v3_1_1::Disconnect::new();
    let bytes = packet.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(evt_packet) => {
            assert_eq!(*evt_packet, packet.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn client_recv_publish_qos0_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    // Create and receive QoS1 PUBLISH A
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload A")
        .build()
        .unwrap();
    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, publish_a.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn client_recv_pubrel_success_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connection.set_auto_pub_response(true);
    v5_0_client_establish_connection(&mut connection);

    let packet = mqtt::packet::v5_0::Publish::builder()
        .packet_id(Some(1))
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .payload(b"payload A")
        .build()
        .unwrap();
    let bytes = packet.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1)
        .build()
        .unwrap();
    let _events = connection.checked_send(packet);

    let packet = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1)
        .build()
        .unwrap();
    let bytes = packet.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    assert_eq!(events.len(), 2);
    if let mqtt::connection::GenericEvent::RequestSendPacket {
        packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        if let mqtt::packet::Packet::V5_0Pubcomp(pubcomp) = packet {
            assert!(pubcomp.reason_code().is_none());
        } else {
            panic!("Expected V5_0Pubcomp packet, but got: {:?}", packet);
        }
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        panic!("Expected RequestSendPacket event, but got: {:?}", events[0]);
    }

    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(evt_packet) => {
            assert_eq!(*evt_packet, packet.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn client_recv_pubrel_pid_not_found_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connection.set_auto_pub_response(true);
    v5_0_client_establish_connection(&mut connection);

    let packet = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1)
        .build()
        .unwrap();
    let bytes = packet.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    assert_eq!(events.len(), 2);
    if let mqtt::connection::GenericEvent::RequestSendPacket {
        packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        if let mqtt::packet::Packet::V5_0Pubcomp(pubcomp) = packet {
            assert_eq!(
                pubcomp.reason_code().unwrap(),
                mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound
            );
        } else {
            panic!("Expected V5_0Pubcomp packet, but got: {:?}", packet);
        }
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        panic!("Expected RequestSendPacket event, but got: {:?}", events[0]);
    }

    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(evt_packet) => {
            assert_eq!(*evt_packet, packet.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn client_recv_pingresp_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connection.set_pingresp_recv_timeout(3000);
    v5_0_client_establish_connection(&mut connection);

    let packet = mqtt::packet::v5_0::Pingreq::new();
    let _events = connection.checked_send(packet);

    let packet = mqtt::packet::v5_0::Pingresp::new();
    let bytes = packet.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // Check RequestTimerCancel event
    if let mqtt::connection::Event::RequestTimerCancel(kind) = &events[0] {
        assert_eq!(*kind, mqtt::connection::TimerKind::PingrespRecv);
    } else {
        panic!("Expected RequestTimerCancel event, got: {:?}", events[0]);
    }

    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(evt_packet) => {
            assert_eq!(*evt_packet, packet.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn server_recv_disconnect_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    v5_0_server_establish_connection(&mut connection);

    let packet = mqtt::packet::v5_0::Disconnect::builder().build().unwrap();
    let bytes = packet.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(evt_packet) => {
            assert_eq!(*evt_packet, packet.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn server_recv_auth_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    v5_0_server_establish_connection(&mut connection);

    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();
    let bytes = packet.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(evt_packet) => {
            assert_eq!(*evt_packet, packet.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}
