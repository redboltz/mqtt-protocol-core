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
fn v5_0_send_stored_success() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    // store QoS1 publish with no MaximumPacketSize

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .props(vec![mqtt::packet::SessionExpiryInterval::new(0xffffffff)
            .unwrap()
            .into()])
        .build()
        .expect("Failed to build Connect packet");
    let _ = con.checked_send(packet);

    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .expect("Failed to build Connack packet");
    let flattened: Vec<u8> = packet.to_continuous_buffer();
    let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
    let _ = con.recv(&mut cursor);

    let pid_q1_a = con.acquire_packet_id().unwrap();
    let pub_q1_a = mqtt::packet::v5_0::Publish::builder()
        .packet_id(pid_q1_a)
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .topic_name("t")
        .unwrap()
        .payload("payload_a")
        .build()
        .expect("Failed to build Publish packet");
    let _ = con.checked_send(pub_q1_a);

    let pid_q2_b = con.acquire_packet_id().unwrap();
    let pub_q2_b = mqtt::packet::v5_0::Publish::builder()
        .packet_id(pid_q2_b)
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .topic_name("t")
        .unwrap()
        .payload("payload_b")
        .build()
        .expect("Failed to build Publish packet");
    let _ = con.checked_send(pub_q2_b);

    let pid_q2_c = con.acquire_packet_id().unwrap();
    let pub_q2_c = mqtt::packet::v5_0::Publish::builder()
        .packet_id(pid_q2_c)
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .topic_name("t")
        .unwrap()
        .payload("payload_c")
        .build()
        .expect("Failed to build Publish packet");
    let _ = con.checked_send(pub_q2_c);

    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(pid_q2_b)
        .build()
        .expect("Failed to build Pubrec packet");
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let rel_b = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(pid_q2_b)
        .build()
        .expect("Failed to build Pubrel packet");
    let _ = con.checked_send(rel_b);

    con.notify_closed();

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_start(false)
        .build()
        .expect("Failed to build Connect packet");
    let _ = con.checked_send(packet);

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .expect("Failed to build Connack packet");
    let flattened: Vec<u8> = connack.to_continuous_buffer();
    let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
    let events = con.recv(&mut cursor);
    assert_eq!(events.len(), 4);

    // Check RequestSendPacket for pub_q1_a
    if let mqtt::connection::Event::RequestSendPacket { packet, .. } = &events[0] {
        if let mqtt::packet::GenericPacket::V5_0Publish(publish) = packet {
            assert_eq!(publish.packet_id(), Some(pid_q1_a));
            assert_eq!(publish.qos(), mqtt::packet::Qos::AtLeastOnce);
            assert_eq!(publish.topic_name(), "t");
            assert_eq!(publish.payload().as_slice(), b"payload_a");
        } else {
            panic!("Expected V5_0Publish packet, got: {packet:?}");
        }
    } else {
        panic!("Expected RequestSendPacket event, got: {:?}", events[0]);
    }

    // Check RequestSendPacket for pub_q2_c
    if let mqtt::connection::Event::RequestSendPacket { packet, .. } = &events[1] {
        if let mqtt::packet::GenericPacket::V5_0Publish(publish) = packet {
            assert_eq!(publish.packet_id(), Some(pid_q2_c));
            assert_eq!(publish.qos(), mqtt::packet::Qos::ExactlyOnce);
            assert_eq!(publish.topic_name(), "t");
            assert_eq!(publish.payload().as_slice(), b"payload_c");
        } else {
            panic!("Expected V5_0Publish packet, got: {packet:?}");
        }
    } else {
        panic!("Expected RequestSendPacket event, got: {:?}", events[1]);
    }

    // Check RequestSendPacket for rel_b
    if let mqtt::connection::Event::RequestSendPacket { packet, .. } = &events[2] {
        if let mqtt::packet::GenericPacket::V5_0Pubrel(pubrel) = packet {
            assert_eq!(pubrel.packet_id(), pid_q2_b);
        } else {
            panic!("Expected V5_0Pubrel packet, got: {packet:?}");
        }
    } else {
        panic!("Expected RequestSendPacket event, got: {:?}", events[2]);
    }

    // Check NotifyPacketReceived for connack
    if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[3] {
        if let mqtt::packet::GenericPacket::V5_0Connack(connack_received) = packet {
            assert_eq!(connack_received.session_present(), true);
            assert_eq!(
                connack_received.reason_code(),
                mqtt::result_code::ConnectReasonCode::Success
            );
        } else {
            panic!("Expected V5_0Connack packet, got: {packet:?}");
        }
    } else {
        panic!("Expected NotifyPacketReceived event, got: {:?}", events[3]);
    }
}

#[test]
fn v5_0_send_stored_oversize() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    // store QoS1 publish with no MaximumPacketSize
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("cid1")
            .unwrap()
            .props(vec![mqtt::packet::SessionExpiryInterval::new(0xffffffff)
                .unwrap()
                .into()])
            .build()
            .expect("Failed to build Connect packet");
        let _ = con.checked_send(packet);
    }
    {
        let packet = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .expect("Failed to build Connack packet");
        let flattened: Vec<u8> = packet.to_continuous_buffer();
        let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
        let _ = con.recv(&mut cursor);
    }
    let pid = con.acquire_packet_id().unwrap();
    {
        let packet = mqtt::packet::v5_0::Publish::builder()
            .packet_id(pid)
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .topic_name("t")
            .unwrap()
            .payload("0123456789abcdef")
            .build()
            .expect("Failed to build Publish packet");
        let _ = con.checked_send(packet);
    }
    con.notify_closed();

    // send_store QoS1 publish but MaximumPacketSize is too small
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("cid1")
            .unwrap()
            .clean_start(false)
            .build()
            .expect("Failed to build Connect packet");
        let _ = con.checked_send(packet);
    }
    {
        let packet = mqtt::packet::v5_0::Connack::builder()
            .session_present(true)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::MaximumPacketSize::new(15)
                .unwrap()
                .into()])
            .build()
            .expect("Failed to build Connack packet");
        let flattened: Vec<u8> = packet.to_continuous_buffer();
        let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
        let events = con.recv(&mut cursor);
        assert_eq!(events.len(), 2);
        if let mqtt::connection::Event::NotifyPacketIdReleased(packet_id) = &events[0] {
            assert_eq!(*packet_id, pid);
        } else {
            panic!(
                "Expected NotifyPacketIdReleased event, got: {:?}",
                events[0]
            );
        }
        if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[1] {
            if let mqtt::packet::GenericPacket::V5_0Connack(connack) = packet {
                assert_eq!(connack.session_present(), true);
                assert_eq!(
                    connack.reason_code(),
                    mqtt::result_code::ConnectReasonCode::Success
                );
            } else {
                panic!("Expected V5_0Connack packet, got: {:?}", packet);
            }
        } else {
            panic!("Expected NotifyPacketReceived event, got: {:?}", events[1]);
        }
    }
    con.notify_closed();

    // Not send_store QoS1 publish even if MaximumPacketSize is not set
    // because the publish packet has already been erased
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("cid1")
            .unwrap()
            .clean_start(false)
            .build()
            .expect("Failed to build Connect packet");
        let _ = con.checked_send(packet);
    }
    {
        let packet = mqtt::packet::v5_0::Connack::builder()
            .session_present(true)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::MaximumPacketSize::new(15)
                .unwrap()
                .into()])
            .build()
            .expect("Failed to build Connack packet");
        let flattened: Vec<u8> = packet.to_continuous_buffer();
        let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
        let events = con.recv(&mut cursor);
        assert_eq!(events.len(), 1);
        if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[0] {
            if let mqtt::packet::GenericPacket::V5_0Connack(connack) = packet {
                assert_eq!(connack.session_present(), true);
                assert_eq!(
                    connack.reason_code(),
                    mqtt::result_code::ConnectReasonCode::Success
                );
            } else {
                panic!("Expected V5_0Connack packet, got: {:?}", packet);
            }
        } else {
            panic!("Expected NotifyPacketReceived event, got: {:?}", events[1]);
        }
    }
}

#[test]
fn restore_packets_v3_1_1() {
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

    let publish_c = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/c")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
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
        mqtt::packet::GenericStorePacket::V3_1_1Publish(publish_b.clone()), // skipped
        mqtt::packet::GenericStorePacket::V3_1_1Publish(publish_c.clone()), // ignored
        mqtt::packet::GenericStorePacket::V3_1_1Pubrel(pubrel.clone()),
        mqtt::packet::GenericStorePacket::V3_1_1Pubrel(pubrel.clone()), // skipped
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

    assert_eq!(events.len(), 4); // 3 send + 1 recv(connack)
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
fn restore_packets_v5_0_server() {
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

    let publish_c = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/c")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload C".to_vec())
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
        mqtt::packet::GenericStorePacket::V5_0Publish(publish_b.clone()), // skipped
        mqtt::packet::GenericStorePacket::V5_0Publish(publish_c.clone()), // ignored
        mqtt::packet::GenericStorePacket::V5_0Pubrel(pubrel.clone()),
        mqtt::packet::GenericStorePacket::V5_0Pubrel(pubrel.clone()), // skipped
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

    assert_eq!(events.len(), 4); // 1 (connack send) + 3 (publish QoS1, QoS2, pubrel)
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
fn qos2_publish_handled_restore_v5_0() {
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
    let mut handled_set = mqtt::common::HashSet::default();
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
fn v5_0_send_stored_success_server() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    // store QoS1 publish with no MaximumPacketSize

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_start(true)
        .props(vec![mqtt::packet::SessionExpiryInterval::new(0xffffffff)
            .unwrap()
            .into()])
        .build()
        .expect("Failed to build Connect packet");
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .expect("Failed to build Connack packet");
    let _ = con.checked_send(packet);

    let packet_id_a = con.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();
    let _events = con.checked_send(publish_a);

    let packet_id_b = con.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();
    let _events = con.checked_send(publish_b);

    con.notify_closed();

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_start(false)
        .build()
        .expect("Failed to build Connect packet");
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .expect("Failed to build Connack packet");
    let events = con.checked_send(packet);

    assert_eq!(events.len(), 3);

    // Check RequestSendPacket for connack
    if let mqtt::connection::Event::RequestSendPacket { packet, .. } = &events[0] {
        if let mqtt::packet::GenericPacket::V5_0Connack(connack_packet) = packet {
            assert_eq!(connack_packet.session_present(), true);
            assert_eq!(
                connack_packet.reason_code(),
                mqtt::result_code::ConnectReasonCode::Success
            );
        } else {
            panic!("Expected V5_0Connack packet, got: {packet:?}");
        }
    } else {
        panic!("Expected RequestSendPacket event, got: {:?}", events[0]);
    }

    // Check RequestSendPacket for publish_a
    if let mqtt::connection::Event::RequestSendPacket { packet, .. } = &events[1] {
        if let mqtt::packet::GenericPacket::V5_0Publish(publish) = packet {
            assert_eq!(publish.packet_id(), Some(packet_id_a));
            assert_eq!(publish.qos(), mqtt::packet::Qos::AtLeastOnce);
            assert_eq!(publish.topic_name(), "topic/a");
            assert_eq!(publish.payload().as_slice(), b"payload A");
        } else {
            panic!("Expected V5_0Publish packet, got: {packet:?}");
        }
    } else {
        panic!("Expected RequestSendPacket event, got: {:?}", events[1]);
    }

    // Check RequestSendPacket for publish_b
    if let mqtt::connection::Event::RequestSendPacket { packet, .. } = &events[2] {
        if let mqtt::packet::GenericPacket::V5_0Publish(publish) = packet {
            assert_eq!(publish.packet_id(), Some(packet_id_b));
            assert_eq!(publish.qos(), mqtt::packet::Qos::ExactlyOnce);
            assert_eq!(publish.topic_name(), "topic/b");
            assert_eq!(publish.payload().as_slice(), b"payload B");
        } else {
            panic!("Expected V5_0Publish packet, got: {packet:?}");
        }
    } else {
        panic!("Expected RequestSendPacket event, got: {:?}", events[2]);
    }
}
