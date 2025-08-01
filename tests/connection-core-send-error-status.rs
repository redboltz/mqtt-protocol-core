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
use common::*;

///////////////////////////////////////////////////////////////////////////////

// Runtime check for packet not allowed to send
// v3.1.1 client

// invalid pid

#[test]
fn v3_1_1_client_not_allowed_to_send_invalid_pid_subscribe() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1)
        .entries(vec![
            mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap(),
        ])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_invalid_pid_unsubscribe() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_invalid_pid_publish_qos1() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(1)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_invalid_pid_publish_qos2() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(1)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_invalid_pid_pubrel() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

// connected

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connected_connect() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut con, true, false);
    let send_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_session(true)
        .build()
        .expect("Failed to build Connect packet")
        .into();
    let events = con.send(send_packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

// connecting

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connecting_connect() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_connecting(&mut con, true);
    let send_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_session(true)
        .build()
        .expect("Failed to build Connect packet")
        .into();
    let events = con.send(send_packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connecting_publish_qos0() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_connecting(&mut con, true);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connecting_publish_qos1() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_connecting(&mut con, true);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connecting_publish_qos2() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_connecting(&mut con, true);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connecting_subscribe() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_connecting(&mut con, true);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(packet_id)
        .entries(vec![
            mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap(),
        ])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connecting_unsubscribe() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_connecting(&mut con, true);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(packet_id)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_connecting_pingreq() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_connecting(&mut con, true);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingreq::new().into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

// disconnected

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_disconnected_publish_qos0() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_disconnected_publish_qos1() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_disconnected_publish_qos2() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_disconnected_subscribe() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(packet_id)
        .entries(vec![
            mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap(),
        ])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_disconnected_unsubscribe() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(packet_id)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_client_not_allowed_to_send_on_status_disconnected_pingreq() {
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingreq::new().into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

///////////////////////////////////////////////////////////////////////////////

// Runtime check for packet not allowed to send
// v3.1.1 server

// invalid pid

#[test]
fn v3_1_1_server_not_allowed_to_send_invalid_pid_publish_qos1() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(1)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_invalid_pid_publish_qos2() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(1)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_invalid_pid_pubrel() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_establish_connection(&mut con, true, false);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            error,
            &mqtt::result_code::MqttError::PacketIdentifierInvalid
        );
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

// connected

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_connected_connack() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_establish_connection(&mut con, true, false);
    let send_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connack::builder()
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .session_present(false)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con.send(send_packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

// connecting

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_connecting_publish_qos0() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_connecting(&mut con, true);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_connecting_publish_qos1() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_connecting(&mut con, true);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_connecting_publish_qos2() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_connecting(&mut con, true);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_connecting_suback() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_connecting(&mut con, true);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_connecting_unsuback() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_connecting(&mut con, true);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_connecting_pingresp() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    v3_1_1_server_connecting(&mut con, true);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingresp::new().into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

// disconnected

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_disconnected_connack() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let send_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connack::builder()
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .session_present(false)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con.send(send_packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_disconnected_publish_qos0() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_disconnected_publish_qos1() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_disconnected_publish_qos2() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet_id = con.acquire_packet_id().unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .packet_id(packet_id)
        .topic_name("topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 2);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
    if let mqtt::connection::Event::NotifyPacketIdReleased(pid) = &events[1] {
        assert_eq!(pid, &packet_id);
    } else {
        assert!(
            false,
            "Expected NotifyPacketIdReleased event, but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_disconnected_suback() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_disconnected_unsuback() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

#[test]
fn v3_1_1_server_not_allowed_to_send_on_status_disconnected_pingresp() {
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingresp::new().into();
    let events = con.send(packet);
    assert_eq!(events.len(), 1);
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
    } else {
        assert!(
            false,
            "Expected NotifyError event, but got: {:?}",
            events[0]
        );
    }
}

///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////
