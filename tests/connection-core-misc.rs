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

#[test]
fn test_get_receive_maximum_vacancy_for_send() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Initially no limit is set
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), None);

    // Receive CONNACK with ReceiveMaximum property set to 5
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::ReceiveMaximum::new(5).unwrap().into()])
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Should now return the initial value of 5
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(5));

    // Acquire packet IDs and send QoS1 PUBLISH A - should reduce vacancy by 1 to 4
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
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(4));

    // Acquire packet ID and send QoS2 PUBLISH B - should reduce vacancy by 1 to 3
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
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(3));

    // Acquire packet ID and send QoS2 PUBLISH C - should reduce vacancy by 1 to 2
    let packet_id_c = connection.acquire_packet_id().unwrap();
    let publish_c = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/c")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_c)
        .payload(b"payload C".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish_c.into());
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(2));

    // Receive PUBACK A - should increase vacancy by 1 to 3
    let puback_a = mqtt::packet::v5_0::Puback::builder()
        .packet_id(packet_id_a)
        .reason_code(mqtt::result_code::PubackReasonCode::Success)
        .build()
        .unwrap();

    let bytes = puback_a.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(3));

    // Receive PUBREC B success - vacancy should remain same (3)
    let pubrec_b = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(packet_id_b)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let bytes = pubrec_b.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(3));

    // Send PUBREL B and receive PUBCOMP B - should reduce vacancy by 1 to 2
    let pubrel_b = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(packet_id_b)
        .reason_code(mqtt::result_code::PubrelReasonCode::Success)
        .build()
        .unwrap();

    let _events = connection.send(pubrel_b.into());

    let pubcomp_b = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(packet_id_b)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .build()
        .unwrap();

    let bytes = pubcomp_b.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(4));

    // Receive PUBREC C error - should increase vacancy by 1 to 5 (error releases packet ID)
    let pubrec_c = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(packet_id_c)
        .reason_code(mqtt::result_code::PubrecReasonCode::UnspecifiedError)
        .build()
        .unwrap();

    let bytes = pubrec_c.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), Some(5));
}

#[test]
fn test_offline_publish_v3_1_1() {
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
fn test_offline_publish_v5_0() {
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
fn test_auto_pub_response_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

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

    // Create and receive QoS1 PUBLISH A
    let packet_id_a = 1u16;
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBACK send request event with same packet_id
    let mut puback_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Puback(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_a {
                puback_found = true;
                break;
            }
        }
    }
    assert!(
        puback_found,
        "PUBACK with packet_id {} should be found in events",
        packet_id_a
    );

    // Create and receive QoS2 PUBLISH B
    let packet_id_b = 2u16;
    let publish_b = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let bytes = publish_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREC send request event with same packet_id
    let mut pubrec_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pubrec(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubrec_found = true;
                break;
            }
        }
    }
    assert!(
        pubrec_found,
        "PUBREC with packet_id {} should be found in events",
        packet_id_b
    );

    // Send PUBREC B
    let pubrec_b = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(packet_id_b)
        .build()
        .unwrap();

    let _events = connection.send(pubrec_b.into());

    // Receive PUBREL B
    let pubrel_b = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(packet_id_b)
        .build()
        .unwrap();

    let bytes = pubrel_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBCOMP send request event with same packet_id
    let mut pubcomp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pubcomp(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubcomp_found = true;
                break;
            }
        }
    }
    assert!(
        pubcomp_found,
        "PUBCOMP with packet_id {} should be found in events",
        packet_id_b
    );
}

#[test]
fn test_auto_pub_response_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

    // Send CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Create and receive QoS1 PUBLISH A
    let packet_id_a = 1u16;
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBACK send request event with same packet_id
    let mut puback_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Puback(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_a {
                puback_found = true;
                break;
            }
        }
    }
    assert!(
        puback_found,
        "PUBACK with packet_id {} should be found in events",
        packet_id_a
    );

    // Create and receive QoS2 PUBLISH B
    let packet_id_b = 2u16;
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let bytes = publish_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREC send request event with same packet_id
    let mut pubrec_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pubrec(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubrec_found = true;
                break;
            }
        }
    }
    assert!(
        pubrec_found,
        "PUBREC with packet_id {} should be found in events",
        packet_id_b
    );

    // Send PUBREC B
    let pubrec_b = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(packet_id_b)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let _events = connection.send(pubrec_b.into());

    // Receive PUBREL B
    let pubrel_b = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(packet_id_b)
        .reason_code(mqtt::result_code::PubrelReasonCode::Success)
        .build()
        .unwrap();

    let bytes = pubrel_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBCOMP send request event with same packet_id
    let mut pubcomp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pubcomp(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubcomp_found = true;
                break;
            }
        }
    }
    assert!(
        pubcomp_found,
        "PUBCOMP with packet_id {} should be found in events",
        packet_id_b
    );
}

#[test]
fn test_qos2_pubrel_send_request_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

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

    // Acquire packet ID and send QoS2 PUBLISH
    let packet_id = connection.acquire_packet_id().unwrap();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id)
        .payload(b"test payload".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish.into());

    // Receive PUBREC
    let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(packet_id)
        .build()
        .unwrap();

    let bytes = pubrec.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREL send request event with same packet_id
    let mut pubrel_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pubrel(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id {
                pubrel_found = true;
                break;
            }
        }
    }
    assert!(
        pubrel_found,
        "PUBREL with packet_id {} should be found in events",
        packet_id
    );
}

#[test]
fn test_qos2_pubrel_send_request_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

    // Send CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Acquire packet ID and send QoS2 PUBLISH
    let packet_id = connection.acquire_packet_id().unwrap();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id)
        .payload(b"test payload".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish.into());

    // Receive PUBREC
    let pubrec = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(packet_id)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let bytes = pubrec.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREL send request event with same packet_id
    let mut pubrel_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pubrel(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id {
                pubrel_found = true;
                break;
            }
        }
    }
    assert!(
        pubrel_found,
        "PUBREL with packet_id {} should be found in events",
        packet_id
    );
}

#[test]
fn test_auto_ping_response_server_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);

    // Enable automatic ping response
    connection.set_auto_ping_response(true);

    // Receive CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let bytes = connect.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Send CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let _events = connection.send(connack.into());

    // Receive PINGREQ
    let pingreq = mqtt::packet::v3_1_1::Pingreq::new();

    let bytes = pingreq.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PINGRESP send request event
    let mut pingresp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pingresp(_),
            ..
        } = event
        {
            pingresp_found = true;
            break;
        }
    }
    assert!(pingresp_found, "PINGRESP should be found in events");
}

#[test]
fn test_auto_ping_response_server_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Enable automatic ping response
    connection.set_auto_ping_response(true);

    // Receive CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let bytes = connect.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Send CONNACK
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let _events = connection.send(connack.into());

    // Receive PINGREQ
    let pingreq = mqtt::packet::v5_0::Pingreq::new();

    let bytes = pingreq.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PINGRESP send request event
    let mut pingresp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pingresp(_),
            ..
        } = event
        {
            pingresp_found = true;
            break;
        }
    }
    assert!(pingresp_found, "PINGRESP should be found in events");
}

#[test]
fn test_packet_id_management_v3_1_1() {
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
fn test_qos2_publish_handled_v3_1_1() {
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
fn test_qos2_publish_handled_restore_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Send CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Restore QoS2 publish handled with packet_id 1
    let mut handled_set = mqtt::common::HashSet::new();
    handled_set.insert(1u16);
    connection.restore_qos2_publish_handled(handled_set);

    // Receive QoS2 PUBLISH A with packet_id 1 (duplicate)
    let packet_id_a = 1u16;
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Verify that NotifyPacketReceived is NOT in the events (duplicate should be ignored)
    let mut notify_packet_received_found = false;
    for event in &events {
        if let mqtt::connection::Event::NotifyPacketReceived { .. } = event {
            notify_packet_received_found = true;
            break;
        }
    }
    assert!(
        !notify_packet_received_found,
        "NotifyPacketReceived should NOT be found for duplicate QoS2 PUBLISH"
    );
}

#[test]
fn test_restore_packets_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Create packets to restore
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let publish_b = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(2)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(3)
        .build()
        .unwrap();

    // Restore packets
    let packets = vec![
        mqtt::packet::GenericStorePacket::V3_1_1Publish(publish_a.clone()),
        mqtt::packet::GenericStorePacket::V3_1_1Publish(publish_b.clone()),
        mqtt::packet::GenericStorePacket::V3_1_1Pubrel(pubrel.clone()),
    ];
    connection.restore_packets(packets);

    // Send CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_session(false)
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find RequestSendPacket events for PUBLISH A, PUBLISH B, and PUBREL
    let mut publish_a_found = false;
    let mut publish_b_found = false;
    let mut pubrel_found = false;
    let mut publish_a_index = None;
    let mut publish_b_index = None;
    let mut pubrel_index = None;

    for (index, event) in events.iter().enumerate() {
        match event {
            mqtt::connection::Event::RequestSendPacket {
                packet: mqtt::packet::Packet::V3_1_1Publish(p),
                ..
            } => {
                if p.topic_name() == "topic/a" && p.packet_id() == Some(1) {
                    publish_a_found = true;
                    publish_a_index = Some(index);
                } else if p.topic_name() == "topic/b" && p.packet_id() == Some(2) {
                    publish_b_found = true;
                    publish_b_index = Some(index);
                }
            }
            mqtt::connection::Event::RequestSendPacket {
                packet: mqtt::packet::Packet::V3_1_1Pubrel(p),
                ..
            } => {
                if p.packet_id() == 3 {
                    pubrel_found = true;
                    pubrel_index = Some(index);
                }
            }
            _ => {}
        }
    }

    // Verify all packets are found and in correct order (A, B, PUBREL)
    assert!(publish_a_found, "PUBLISH A should be found in events");
    assert!(publish_b_found, "PUBLISH B should be found in events");
    assert!(pubrel_found, "PUBREL should be found in events");

    let a_idx = publish_a_index.unwrap();
    let b_idx = publish_b_index.unwrap();
    let rel_idx = pubrel_index.unwrap();

    assert!(
        a_idx < b_idx && b_idx < rel_idx,
        "Packets should be in order: PUBLISH A, PUBLISH B, PUBREL"
    );

    // Verify get_stored_packets returns the same packets
    let stored_packets = connection.get_stored_packets();
    assert_eq!(stored_packets.len(), 3, "Should have 3 stored packets");

    let mut stored_publish_a_found = false;
    let mut stored_publish_b_found = false;
    let mut stored_pubrel_found = false;

    for packet in &stored_packets {
        match packet {
            mqtt::packet::GenericStorePacket::V3_1_1Publish(p) => {
                if p.topic_name() == "topic/a" && p.packet_id() == Some(1) {
                    stored_publish_a_found = true;
                } else if p.topic_name() == "topic/b" && p.packet_id() == Some(2) {
                    stored_publish_b_found = true;
                }
            }
            mqtt::packet::GenericStorePacket::V3_1_1Pubrel(p) => {
                if p.packet_id() == 3 {
                    stored_pubrel_found = true;
                }
            }
            _ => {}
        }
    }

    assert!(
        stored_publish_a_found,
        "PUBLISH A should be in stored packets"
    );
    assert!(
        stored_publish_b_found,
        "PUBLISH B should be in stored packets"
    );
    assert!(stored_pubrel_found, "PUBREL should be in stored packets");
}

#[test]
fn test_restore_packets_v5_0_server() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Create packets to restore
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(2)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(3)
        .build()
        .unwrap();

    // Restore packets
    let packets = vec![
        mqtt::packet::GenericStorePacket::V5_0Publish(publish_a.clone()),
        mqtt::packet::GenericStorePacket::V5_0Publish(publish_b.clone()),
        mqtt::packet::GenericStorePacket::V5_0Pubrel(pubrel.clone()),
    ];
    connection.restore_packets(packets);

    // Receive CONNECT with clean_start false
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_start(false)
        .build()
        .unwrap();

    let bytes = connect.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Send CONNACK with session_present true
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let events = connection.send(connack.into());

    // Find RequestSendPacket events for PUBLISH A, PUBLISH B, and PUBREL
    let mut publish_a_found = false;
    let mut publish_b_found = false;
    let mut pubrel_found = false;
    let mut publish_a_index = None;
    let mut publish_b_index = None;
    let mut pubrel_index = None;

    for (index, event) in events.iter().enumerate() {
        match event {
            mqtt::connection::Event::RequestSendPacket {
                packet: mqtt::packet::Packet::V5_0Publish(p),
                ..
            } => {
                if p.topic_name() == "topic/a" && p.packet_id() == Some(1) {
                    publish_a_found = true;
                    publish_a_index = Some(index);
                } else if p.topic_name() == "topic/b" && p.packet_id() == Some(2) {
                    publish_b_found = true;
                    publish_b_index = Some(index);
                }
            }
            mqtt::connection::Event::RequestSendPacket {
                packet: mqtt::packet::Packet::V5_0Pubrel(p),
                ..
            } => {
                if p.packet_id() == 3 {
                    pubrel_found = true;
                    pubrel_index = Some(index);
                }
            }
            _ => {}
        }
    }

    // Verify all packets are found and in correct order (A, B, PUBREL)
    assert!(publish_a_found, "PUBLISH A should be found in events");
    assert!(publish_b_found, "PUBLISH B should be found in events");
    assert!(pubrel_found, "PUBREL should be found in events");

    let a_idx = publish_a_index.unwrap();
    let b_idx = publish_b_index.unwrap();
    let rel_idx = pubrel_index.unwrap();

    assert!(
        a_idx < b_idx && b_idx < rel_idx,
        "Packets should be in order: PUBLISH A, PUBLISH B, PUBREL"
    );

    // Verify get_stored_packets returns the same packets
    let stored_packets = connection.get_stored_packets();
    assert_eq!(stored_packets.len(), 3, "Should have 3 stored packets");

    let mut stored_publish_a_found = false;
    let mut stored_publish_b_found = false;
    let mut stored_pubrel_found = false;

    for packet in &stored_packets {
        match packet {
            mqtt::packet::GenericStorePacket::V5_0Publish(p) => {
                if p.topic_name() == "topic/a" && p.packet_id() == Some(1) {
                    stored_publish_a_found = true;
                } else if p.topic_name() == "topic/b" && p.packet_id() == Some(2) {
                    stored_publish_b_found = true;
                }
            }
            mqtt::packet::GenericStorePacket::V5_0Pubrel(p) => {
                if p.packet_id() == 3 {
                    stored_pubrel_found = true;
                }
            }
            _ => {}
        }
    }

    assert!(
        stored_publish_a_found,
        "PUBLISH A should be in stored packets"
    );
    assert!(
        stored_publish_b_found,
        "PUBLISH B should be in stored packets"
    );
    assert!(stored_pubrel_found, "PUBREL should be in stored packets");
}

#[test]
fn test_v3_1_1_client_qos2_publish_processing_state() {
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
fn test_pingresp_recv_timer_reset_v3_1_1() {
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Set pingresp recv timeout to 10000ms
    connection.set_pingresp_recv_timeout(Some(10000));

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
