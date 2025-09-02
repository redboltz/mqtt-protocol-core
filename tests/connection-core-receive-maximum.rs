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
fn get_receive_maximum_vacancy_for_send_client() {
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
fn get_receive_maximum_vacancy_for_send_server() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Initially no limit is set
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), None);

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .props(vec![mqtt::packet::ReceiveMaximum::new(5).unwrap().into()])
        .build()
        .expect("Failed to build Connect packet");
    let bytes = packet.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Receive CONNACK with ReceiveMaximum property set to 5
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();
    let _events = connection.send(connack.into());

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
fn receive_maximum_exceeded_send() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Initially no limit is set
    assert_eq!(connection.get_receive_maximum_vacancy_for_send(), None);

    // Receive CONNACK with ReceiveMaximum property set to 1
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()])
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

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

    let packet_id_b = connection.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let events = connection.send(publish_b.into());

    // Check that events contains exactly 2 events in the correct order
    assert_eq!(events.len(), 2, "Should have exactly 2 events");

    // Check first event: NotifyError(MqttError::ReceiveMaximumExceeded)
    if let mqtt::connection::Event::NotifyError(error) = &events[0] {
        assert_eq!(
            *error,
            mqtt::result_code::MqttError::ReceiveMaximumExceeded,
            "First event should be NotifyError(ReceiveMaximumExceeded)"
        );
    } else {
        panic!("First event should be NotifyError but got: {:?}", events[0]);
    }

    // Check second event: NotifyPacketIdReleased(packet_id_b)
    if let mqtt::connection::Event::NotifyPacketIdReleased(packet_id) = &events[1] {
        assert_eq!(
            *packet_id, packet_id_b,
            "Second event should be NotifyPacketIdReleased({})",
            packet_id_b
        );
    } else {
        panic!(
            "Second event should be NotifyPacketIdReleased but got: {:?}",
            events[1]
        );
    }
}

#[test]
fn receive_maximum_exceeded_recv() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .props(vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()])
        .build()
        .expect("Failed to build Connect packet");
    let _events = con.checked_send(packet.clone());

    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet_id_a = con.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet_id_b = con.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let bytes = publish_b.to_continuous_buffer();
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    assert_eq!(events.len(), 3, "Should have exactly 3 events");

    // Test first event: RequestSendPacket with Disconnect packet
    if let mqtt::connection::GenericEvent::RequestSendPacket {
        packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        if let mqtt::packet::Packet::V5_0Disconnect(disconnect) = packet {
            assert_eq!(
                disconnect.reason_code(),
                Some(mqtt::result_code::DisconnectReasonCode::ReceiveMaximumExceeded)
            );
        } else {
            panic!("Expected V5_0Disconnect packet, but got: {:?}", packet);
        }
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        panic!("Expected RequestSendPacket event, but got: {:?}", events[0]);
    }

    // Test second event: RequestClose
    if let mqtt::connection::GenericEvent::RequestClose = &events[1] {
        // Expected RequestClose event
    } else {
        panic!("Expected RequestClose event, but got: {:?}", events[1]);
    }

    // Test third event: NotifyError(MqttError::ReceiveMaximumExceeded)
    if let mqtt::connection::GenericEvent::NotifyError(error) = &events[2] {
        assert_eq!(
            *error,
            mqtt::result_code::MqttError::ReceiveMaximumExceeded,
            "Third event should be NotifyError(ReceiveMaximumExceeded)"
        );
    } else {
        panic!(
            "Expected NotifyError(ReceiveMaximumExceeded) event, but got: {:?}",
            events[2]
        );
    }
}

#[test]
fn receive_maximum_exceeded_recv_server() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connect packet");
    let bytes = packet.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()])
        .build()
        .unwrap();
    let _events = con.checked_send(packet.clone());

    let packet_id_a = con.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_a)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let _events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet_id_b = con.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_b)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let bytes = publish_b.to_continuous_buffer();
    let events = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    assert_eq!(events.len(), 3, "Should have exactly 3 events");

    // Test first event: RequestSendPacket with Disconnect packet
    if let mqtt::connection::GenericEvent::RequestSendPacket {
        packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        if let mqtt::packet::Packet::V5_0Disconnect(disconnect) = packet {
            assert_eq!(
                disconnect.reason_code(),
                Some(mqtt::result_code::DisconnectReasonCode::ReceiveMaximumExceeded)
            );
        } else {
            panic!("Expected V5_0Disconnect packet, but got: {:?}", packet);
        }
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        panic!("Expected RequestSendPacket event, but got: {:?}", events[0]);
    }

    // Test second event: RequestClose
    if let mqtt::connection::GenericEvent::RequestClose = &events[1] {
        // Expected RequestClose event
    } else {
        panic!("Expected RequestClose event, but got: {:?}", events[1]);
    }

    // Test third event: NotifyError(MqttError::ReceiveMaximumExceeded)
    if let mqtt::connection::GenericEvent::NotifyError(error) = &events[2] {
        assert_eq!(
            *error,
            mqtt::result_code::MqttError::ReceiveMaximumExceeded,
            "Third event should be NotifyError(ReceiveMaximumExceeded)"
        );
    } else {
        panic!(
            "Expected NotifyError(ReceiveMaximumExceeded) event, but got: {:?}",
            events[2]
        );
    }
}
