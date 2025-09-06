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
fn connack_error_server() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .keep_alive(10u16)
        .build()
        .expect("Failed to build Connect packet");
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::NotAuthorized)
        .props(vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()])
        .build()
        .unwrap();
    let events = con.checked_send(packet.clone());

    assert_eq!(events.len(), 3);

    match &events[0] {
        mqtt::connection::Event::RequestSendPacket {
            packet: sent_packet,
            release_packet_id_if_send_error,
        } => {
            assert_eq!(*sent_packet, packet.into());
            assert_eq!(*release_packet_id_if_send_error, None);
        }
        _ => panic!("Expected RequestSendPacket event, got {:?}", events[1]),
    }
    match &events[1] {
        mqtt::connection::Event::RequestTimerCancel(timer_kind) => {
            assert_eq!(*timer_kind, mqtt::connection::TimerKind::PingreqRecv);
        }
        _ => panic!("Expected RequestTimerCancel event, got {:?}", events[0]),
    }
    match &events[2] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }
}

#[test]
fn offline_publish_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Enable offline publishing
    connection.set_offline_publish(true);

    // Send QoS1 PUBLISH A while offline
    let packet_id_a = connection.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish_a.into());

    // Send QoS2 PUBLISH B while offline
    let packet_id_b = connection.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish_b.into());

    // Send CONNECT with clean_session false
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_session(false)
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK with session_present true
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find RequestSendPacket events for PUBLISH A and B
    let mut publish_a_found = false;
    let mut publish_b_found = false;
    let mut publish_a_index = None;
    let mut publish_b_index = None;

    for (index, event) in events.iter().enumerate() {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Publish(p),
            ..
        } = event
        {
            if p.topic_name() == "topic/a" && p.payload().as_slice() == b"payload A" {
                publish_a_found = true;
                publish_a_index = Some(index);
            } else if p.topic_name() == "topic/b" && p.payload().as_slice() == b"payload B" {
                publish_b_found = true;
                publish_b_index = Some(index);
            }
        }
    }

    // Verify both PUBLISH packets are found and in correct order (A before B)
    assert!(publish_a_found, "PUBLISH A should be found in events");
    assert!(publish_b_found, "PUBLISH B should be found in events");
    assert!(
        publish_a_index.unwrap() < publish_b_index.unwrap(),
        "PUBLISH A should come before PUBLISH B"
    );
}

#[test]
fn offline_publish_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Enable offline publishing
    connection.set_offline_publish(true);

    // Send QoS1 PUBLISH A while offline
    let packet_id_a = connection.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish_a.into());

    // Send QoS2 PUBLISH B while offline
    let packet_id_b = connection.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish_b.into());

    // Send CONNECT with clean_start false
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_start(false)
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK with session_present true
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find RequestSendPacket events for PUBLISH A and B
    let mut publish_a_found = false;
    let mut publish_b_found = false;
    let mut publish_a_index = None;
    let mut publish_b_index = None;

    for (index, event) in events.iter().enumerate() {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Publish(p),
            ..
        } = event
        {
            if p.topic_name() == "topic/a" && p.payload().as_slice() == b"payload A" {
                publish_a_found = true;
                publish_a_index = Some(index);
            } else if p.topic_name() == "topic/b" && p.payload().as_slice() == b"payload B" {
                publish_b_found = true;
                publish_b_index = Some(index);
            }
        }
    }

    // Verify both PUBLISH packets are found and in correct order (A before B)
    assert!(publish_a_found, "PUBLISH A should be found in events");
    assert!(publish_b_found, "PUBLISH B should be found in events");
    assert!(
        publish_a_index.unwrap() < publish_b_index.unwrap(),
        "PUBLISH A should come before PUBLISH B"
    );
}

#[test]
fn puback_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    // Create and receive QoS1 PUBLISH A
    let packet_id_a = connection.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();
    let _events = connection.send(publish_a.into());

    let puback_a = mqtt::packet::v3_1_1::Puback::builder()
        .packet_id(packet_id_a)
        .build()
        .unwrap();
    let bytes = puback_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: NotifyPacketIdReleased
    match &events[0] {
        mqtt::connection::Event::NotifyPacketIdReleased(packet_id) => {
            assert_eq!(*packet_id, packet_id_a);
        }
        _ => panic!("Expected NotifyPacketIdReleased event, got {:?}", events[0]),
    }

    // Second event: NotifyPacketReceived
    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, puback_a.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn puback_no_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let puback_a = mqtt::packet::v3_1_1::Puback::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = puback_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: RequestClose
    match &events[0] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }

    // Second event: NotifyError(MqttError::ProtocolError)
    match &events[1] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[1]),
    }
}

#[test]
fn pubrec_no_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let pubrec_a = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = pubrec_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: RequestClose
    match &events[0] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }

    // Second event: NotifyError(MqttError::ProtocolError)
    match &events[1] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[1]),
    }
}

#[test]
fn pubcomp_no_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let pubcomp_a = mqtt::packet::v3_1_1::Pubcomp::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = pubcomp_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: RequestClose
    match &events[0] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }

    // Second event: NotifyError(MqttError::ProtocolError)
    match &events[1] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[1]),
    }
}

#[test]
fn suback_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let packet_id = connection.acquire_packet_id().unwrap();
    let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(packet_id)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
        .build()
        .unwrap();
    let _events = connection.checked_send(subscribe.clone());

    let suback = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(packet_id)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();
    let bytes = suback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: NotifyPacketIdReleased
    match &events[0] {
        mqtt::connection::Event::NotifyPacketIdReleased(released_packet_id) => {
            assert_eq!(*released_packet_id, packet_id);
        }
        _ => panic!("Expected NotifyPacketIdReleased event, got {:?}", events[0]),
    }

    // Second event: NotifyPacketReceived
    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, suback.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn suback_no_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let suback = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();
    let bytes = suback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: RequestClose
    match &events[0] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }

    // Second event: NotifyError(MqttError::ProtocolError)
    match &events[1] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[1]),
    }
}

#[test]
fn unsuback_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let packet_id = connection.acquire_packet_id().unwrap();
    let unsubscribe = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(packet_id)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();
    let _events = connection.checked_send(unsubscribe.clone());

    let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = unsuback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: NotifyPacketIdReleased
    match &events[0] {
        mqtt::connection::Event::NotifyPacketIdReleased(released_packet_id) => {
            assert_eq!(*released_packet_id, packet_id);
        }
        _ => panic!("Expected NotifyPacketIdReleased event, got {:?}", events[0]),
    }

    // Second event: NotifyPacketReceived
    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, unsuback.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn unsuback_no_match_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = unsuback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: RequestClose
    match &events[0] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[0]),
    }

    // Second event: NotifyError(MqttError::ProtocolError)
    match &events[1] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[1]),
    }
}

#[test]
fn puback_no_match_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    let puback_a = mqtt::packet::v5_0::Puback::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = puback_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 3);

    // First event: RequestSendPacket with V5.0 Disconnect packet
    match &events[0] {
        mqtt::connection::Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } => {
            if let mqtt::packet::Packet::V5_0Disconnect(disconnect) = packet {
                assert_eq!(
                    disconnect.reason_code(),
                    Some(mqtt::result_code::DisconnectReasonCode::ProtocolError)
                );
            } else {
                panic!("Expected V5_0Disconnect packet, got {:?}", packet);
            }
            assert_eq!(*release_packet_id_if_send_error, None);
        }
        _ => panic!("Expected RequestSendPacket event, got {:?}", events[0]),
    }

    // Second event: RequestClose
    match &events[1] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[1]),
    }

    // Third event: NotifyError(MqttError::ProtocolError)
    match &events[2] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[2]),
    }
}

#[test]
fn pubrec_no_match_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    let pubrec_a = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = pubrec_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 3);

    // First event: RequestSendPacket with V5.0 Disconnect packet
    match &events[0] {
        mqtt::connection::Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } => {
            if let mqtt::packet::Packet::V5_0Disconnect(disconnect) = packet {
                assert_eq!(
                    disconnect.reason_code(),
                    Some(mqtt::result_code::DisconnectReasonCode::ProtocolError)
                );
            } else {
                panic!("Expected V5_0Disconnect packet, got {:?}", packet);
            }
            assert_eq!(*release_packet_id_if_send_error, None);
        }
        _ => panic!("Expected RequestSendPacket event, got {:?}", events[0]),
    }

    // Second event: RequestClose
    match &events[1] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[1]),
    }

    // Third event: NotifyError(MqttError::ProtocolError)
    match &events[2] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[2]),
    }
}

#[test]
fn pubcomp_no_match_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    let pubcomp_a = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1u16)
        .build()
        .unwrap();
    let bytes = pubcomp_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 3);

    // First event: RequestSendPacket with V5.0 Disconnect packet
    match &events[0] {
        mqtt::connection::Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } => {
            if let mqtt::packet::Packet::V5_0Disconnect(disconnect) = packet {
                assert_eq!(
                    disconnect.reason_code(),
                    Some(mqtt::result_code::DisconnectReasonCode::ProtocolError)
                );
            } else {
                panic!("Expected V5_0Disconnect packet, got {:?}", packet);
            }
            assert_eq!(*release_packet_id_if_send_error, None);
        }
        _ => panic!("Expected RequestSendPacket event, got {:?}", events[0]),
    }

    // Second event: RequestClose
    match &events[1] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[1]),
    }

    // Third event: NotifyError(MqttError::ProtocolError)
    match &events[2] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[2]),
    }
}

#[test]
fn suback_match_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    let packet_id = connection.acquire_packet_id().unwrap();
    let subscribe = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(packet_id)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
        .build()
        .unwrap();
    let _events = connection.checked_send(subscribe.clone());

    let suback = mqtt::packet::v5_0::Suback::builder()
        .packet_id(packet_id)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();
    let bytes = suback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: NotifyPacketIdReleased
    match &events[0] {
        mqtt::connection::Event::NotifyPacketIdReleased(released_packet_id) => {
            assert_eq!(*released_packet_id, packet_id);
        }
        _ => panic!("Expected NotifyPacketIdReleased event, got {:?}", events[0]),
    }

    // Second event: NotifyPacketReceived
    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, suback.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn suback_no_match_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    let suback = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();
    let bytes = suback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 3);

    // First event: RequestSendPacket with V5.0 Disconnect packet
    match &events[0] {
        mqtt::connection::Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } => {
            if let mqtt::packet::Packet::V5_0Disconnect(disconnect) = packet {
                assert_eq!(
                    disconnect.reason_code(),
                    Some(mqtt::result_code::DisconnectReasonCode::ProtocolError)
                );
            } else {
                panic!("Expected V5_0Disconnect packet, got {:?}", packet);
            }
            assert_eq!(*release_packet_id_if_send_error, None);
        }
        _ => panic!("Expected RequestSendPacket event, got {:?}", events[0]),
    }

    // Second event: RequestClose
    match &events[1] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[1]),
    }

    // Third event: NotifyError(MqttError::ProtocolError)
    match &events[2] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[2]),
    }
}

#[test]
fn unsuback_match_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    let packet_id = connection.acquire_packet_id().unwrap();
    let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(packet_id)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();
    let _events = connection.checked_send(unsubscribe.clone());

    let unsuback = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(packet_id)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();
    let bytes = unsuback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // First event: NotifyPacketIdReleased
    match &events[0] {
        mqtt::connection::Event::NotifyPacketIdReleased(released_packet_id) => {
            assert_eq!(*released_packet_id, packet_id);
        }
        _ => panic!("Expected NotifyPacketIdReleased event, got {:?}", events[0]),
    }

    // Second event: NotifyPacketReceived
    match &events[1] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, unsuback.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn unsuback_no_match_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    let unsuback = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();
    let bytes = unsuback.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 3);

    // First event: RequestSendPacket with V5.0 Disconnect packet
    match &events[0] {
        mqtt::connection::Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } => {
            if let mqtt::packet::Packet::V5_0Disconnect(disconnect) = packet {
                assert_eq!(
                    disconnect.reason_code(),
                    Some(mqtt::result_code::DisconnectReasonCode::ProtocolError)
                );
            } else {
                panic!("Expected V5_0Disconnect packet, got {:?}", packet);
            }
            assert_eq!(*release_packet_id_if_send_error, None);
        }
        _ => panic!("Expected RequestSendPacket event, got {:?}", events[0]),
    }

    // Second event: RequestClose
    match &events[1] {
        mqtt::connection::Event::RequestClose => {
            // Expected RequestClose event
        }
        _ => panic!("Expected RequestClose event, got {:?}", events[1]),
    }

    // Third event: NotifyError(MqttError::ProtocolError)
    match &events[2] {
        mqtt::connection::Event::NotifyError(error) => {
            assert_eq!(*error, mqtt::result_code::MqttError::ProtocolError);
        }
        _ => panic!("Expected NotifyError event, got {:?}", events[2]),
    }
}

#[test]
fn packet_id_management_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    connection.register_packet_id(1).unwrap();
    let acquired_id = connection.acquire_packet_id().unwrap();
    assert_eq!(acquired_id, 2);

    let events = connection.release_packet_id(1);
    let mut packet_id_released_found = false;
    for event in &events {
        if let mqtt::connection::Event::NotifyPacketIdReleased(packet_id) = event {
            if *packet_id == 1 {
                packet_id_released_found = true;
                break;
            }
        }
    }
    assert!(
        packet_id_released_found,
        "NotifyPacketIdReleased(1) event should be found"
    );

    let new_acquired_id = connection.acquire_packet_id().unwrap();
    assert_eq!(new_acquired_id, 1);
}

#[test]
fn qos2_publish_handled_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Send CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Check initial state - should be empty
    let handled_packet_ids = connection.get_qos2_publish_handled();
    assert!(
        handled_packet_ids.is_empty(),
        "QoS2 publish handled should be empty initially"
    );

    // Receive QoS2 PUBLISH A with packet_id 1
    let packet_id_a = 1u16;
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Check state after receiving QoS2 PUBLISH - should contain packet_id 1
    let handled_packet_ids = connection.get_qos2_publish_handled();
    assert_eq!(
        handled_packet_ids.len(),
        1,
        "QoS2 publish handled should contain 1 element"
    );
    assert!(
        handled_packet_ids.contains(&packet_id_a),
        "QoS2 publish handled should contain packet_id 1"
    );

    // Send PUBREC
    let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(packet_id_a)
        .build()
        .unwrap();

    let _events = connection.send(pubrec.into());

    // Receive PUBREL
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(packet_id_a)
        .build()
        .unwrap();

    let bytes = pubrel.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Check final state after receiving PUBREL - should be empty again
    let handled_packet_ids = connection.get_qos2_publish_handled();
    assert!(
        handled_packet_ids.is_empty(),
        "QoS2 publish handled should be empty after PUBREL"
    );
}

#[test]
fn v3_1_1_client_qos2_publish_processing_state() {
    common::init_tracing();
    // Test v3.1.1 Client QoS2 publish processing state through the complete flow
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Send CONNECT packet
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .keep_alive(60)
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Acquire packet ID for QoS2 publish
    let packet_id = connection.acquire_packet_id().unwrap();
    assert_eq!(packet_id, 1);

    // Send QoS2 PUBLISH (id=1)
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id)
        .payload(b"test_payload")
        .build()
        .unwrap();

    let _events = connection.send(publish.into());

    // Verify is_publish_processing(1) is true after sending PUBLISH
    assert!(
        connection.is_publish_processing(packet_id),
        "Should be processing publish after sending PUBLISH"
    );

    // Receive PUBREC (id=1)
    let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(packet_id)
        .build()
        .unwrap();

    let bytes = pubrec.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Verify is_publish_processing(1) is still true after receiving PUBREC
    assert!(
        connection.is_publish_processing(packet_id),
        "Should still be processing publish after receiving PUBREC"
    );

    // Send PUBREL (id=1) - this should be automatically generated
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(packet_id)
        .build()
        .unwrap();

    let _events = connection.send(pubrel.into());

    // Verify is_publish_processing(1) is still true after sending PUBREL
    assert!(
        connection.is_publish_processing(packet_id),
        "Should still be processing publish after sending PUBREL"
    );

    // Receive PUBCOMP (id=1)
    let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
        .packet_id(packet_id)
        .build()
        .unwrap();

    let bytes = pubcomp.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Verify is_publish_processing(1) is false after receiving PUBCOMP
    assert!(
        !connection.is_publish_processing(packet_id),
        "Should not be processing publish after receiving PUBCOMP"
    );
}

#[test]
fn pingresp_recv_timer_reset_v3_1_1() {
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Set pingresp recv timeout to 10000ms
    connection.set_pingresp_recv_timeout(10000);

    // Send CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Send PINGREQ
    let pingreq = mqtt::packet::v3_1_1::Pingreq::new();
    let events = connection.send(pingreq.into());

    // Find pingresp_recv timer reset event
    let mut timer_reset_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestTimerReset { kind, duration_ms } = event {
            if matches!(kind, mqtt::connection::TimerKind::PingrespRecv) && *duration_ms == 10000 {
                timer_reset_found = true;
                break;
            }
        }
    }
    assert!(
        timer_reset_found,
        "pingresp_recv timer reset with 10000ms should be found in events"
    );
}
