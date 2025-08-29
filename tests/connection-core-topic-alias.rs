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
fn auto_map_topic_alias_send() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Enable automatic topic alias mapping for send
    connection.set_auto_map_topic_alias_send(true);

    // Send CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK with TopicAliasMaximum set to 65535
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::TopicAliasMaximum::new(65535)
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Send QoS0 PUBLISH A
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let events_a = connection.send(publish_a.into());

    // Find RequestSendPacket event for PUBLISH A and verify topic alias mapping
    let mut publish_a_mapped = false;
    for event in &events_a {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Publish(p),
            ..
        } = event
        {
            // Verify topic name is empty and topic alias is 1
            if p.topic_name().is_empty() {
                // Check for TopicAlias property with value 1
                for prop in p.props().iter() {
                    if let mqtt::packet::Property::TopicAlias(ta) = prop {
                        if ta.val() == 1 {
                            publish_a_mapped = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    assert!(
        publish_a_mapped,
        "PUBLISH A should have empty topic name and TopicAlias=1"
    );

    // Send QoS0 PUBLISH B
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let events_b = connection.send(publish_b.into());

    // Find RequestSendPacket event for PUBLISH B and verify topic alias mapping
    let mut publish_b_mapped = false;
    for event in &events_b {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Publish(p),
            ..
        } = event
        {
            // Verify topic name is empty and topic alias is 2
            if p.topic_name().is_empty() {
                // Check for TopicAlias property with value 2
                for prop in p.props().iter() {
                    if let mqtt::packet::Property::TopicAlias(ta) = prop {
                        if ta.val() == 2 {
                            publish_b_mapped = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    assert!(
        publish_b_mapped,
        "PUBLISH B should have empty topic name and TopicAlias=2"
    );

    {
        // Send QoS0 PUBLISH B again
        let publish_b = mqtt::packet::v5_0::Publish::builder()
            .topic_name("topic/b")
            .unwrap()
            .qos(mqtt::packet::Qos::AtMostOnce)
            .payload(b"payload B".to_vec())
            .build()
            .unwrap();

        let events_b = connection.send(publish_b.into());

        // Find RequestSendPacket event for PUBLISH B and verify topic alias mapping
        let mut publish_b_mapped = false;
        for event in &events_b {
            if let mqtt::connection::Event::RequestSendPacket {
                packet: mqtt::packet::Packet::V5_0Publish(p),
                ..
            } = event
            {
                // Verify topic name is empty and topic alias is 2
                if p.topic_name().is_empty() {
                    // Check for TopicAlias property with value 2
                    for prop in p.props().iter() {
                        if let mqtt::packet::Property::TopicAlias(ta) = prop {
                            if ta.val() == 2 {
                                publish_b_mapped = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        assert!(
            publish_b_mapped,
            "PUBLISH B should have empty topic name and TopicAlias=2"
        );
    }
}

#[test]
fn auto_replace_topic_alias_send() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Enable automatic topic alias replacement for send
    connection.set_auto_replace_topic_alias_send(true);

    // Receive CONNECT with TopicAliasMaximum set to 65535
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .props(vec![mqtt::packet::TopicAliasMaximum::new(65535)
            .unwrap()
            .into()])
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

    // Send QoS0 PUBLISH A (first time - should maintain topic name and no properties)
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let events_a = connection.send(publish_a.into());

    // Find RequestSendPacket event for PUBLISH A and verify no topic alias mapping
    let mut publish_a_unchanged = false;
    for event in &events_a {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Publish(p),
            ..
        } = event
        {
            // Verify topic name is maintained and no properties
            if p.topic_name() == "topic/test" {
                // Check that there are no properties or no TopicAlias property
                let has_topic_alias = p
                    .props()
                    .iter()
                    .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)));
                if !has_topic_alias {
                    publish_a_unchanged = true;
                    break;
                }
            }
        }
    }
    assert!(
        publish_a_unchanged,
        "PUBLISH A should maintain topic name and have no TopicAlias property"
    );

    // Send QoS0 PUBLISH B with TopicAlias property set to 1
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload B".to_vec())
        .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
        .build()
        .unwrap();

    let _events_b = connection.send(publish_b.into());

    // Send QoS0 PUBLISH C with same topic name and no properties
    let publish_c = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/test")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload C".to_vec())
        .build()
        .unwrap();

    let events_c = connection.send(publish_c.into());

    // Find RequestSendPacket event for PUBLISH C and verify topic alias replacement
    let mut publish_c_replaced = false;
    for event in &events_c {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Publish(p),
            ..
        } = event
        {
            // Verify topic name is empty and topic alias is 1
            if p.topic_name().is_empty() {
                // Check for TopicAlias property with value 1
                let has_topic_alias = p
                    .props()
                    .iter()
                    .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)));
                if has_topic_alias {
                    publish_c_replaced = true;
                    break;
                }
            }
        }
    }
    assert!(
        publish_c_replaced,
        "PUBLISH C should have empty topic name and TopicAlias=1"
    );
}

#[test]
fn manual_topic_alias() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .props(vec![mqtt::packet::SessionExpiryInterval::new(0xffffffff)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::TopicAliasMaximum::new(65535)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Send QoS0 PUBLISH A
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .topic_name("topic/a")
            .unwrap()
            .qos(mqtt::packet::Qos::AtMostOnce)
            .payload(b"payload A".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
            .build()
            .unwrap();

        let events_a = connection.send(publish_a.into());

        // Find RequestSendPacket event for PUBLISH A and verify topic alias mapping
        let mut publish_a_registered = false;
        for event in &events_a {
            if let mqtt::connection::Event::RequestSendPacket {
                packet: mqtt::packet::Packet::V5_0Publish(p),
                ..
            } = event
            {
                assert_eq!(p.topic_name(), "topic/a");
                // Check for TopicAlias property with value 1
                for prop in p.props().iter() {
                    if let mqtt::packet::Property::TopicAlias(ta) = prop {
                        if ta.val() == 1 {
                            publish_a_registered = true;
                            break;
                        }
                    }
                }
            }
        }
        assert!(
            publish_a_registered,
            "PUBLISH A should have a topic name and TopicAlias=1"
        );
    }
    {
        // Send QoS0 PUBLISH B
        let publish_b = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(connection.acquire_packet_id().unwrap())
            .payload(b"payload B".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
            .build()
            .unwrap();

        let events_b = connection.send(publish_b.into());

        // Find RequestSendPacket event for PUBLISH B and verify topic alias mapping
        let mut publish_b_mapped = false;
        for event in &events_b {
            if let mqtt::connection::Event::RequestSendPacket {
                packet: mqtt::packet::Packet::V5_0Publish(p),
                ..
            } = event
            {
                // Verify topic name is empty and topic alias is 1
                if p.topic_name().is_empty() {
                    // Check for TopicAlias property with value 1
                    for prop in p.props().iter() {
                        if let mqtt::packet::Property::TopicAlias(ta) = prop {
                            if ta.val() == 1 {
                                publish_b_mapped = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        assert!(
            publish_b_mapped,
            "PUBLISH B should have empty topic name and TopicAlias=1"
        );
    }
    connection.notify_closed();

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .clean_start(false)
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(true)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::TopicAliasMaximum::new(65535)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
        assert_eq!(events.len(), 2);
        if let mqtt::connection::Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } = &events[0]
        {
            let publish_extracted: mqtt::packet::Packet = mqtt::packet::v5_0::Publish::builder()
                .qos(mqtt::packet::Qos::AtLeastOnce)
                .packet_id(1)
                .topic_name("topic/a")
                .unwrap()
                .payload(b"payload B".to_vec())
                .build()
                .unwrap()
                .set_dup(true)
                .into();

            assert_eq!(packet, &publish_extracted);
            assert!(release_packet_id_if_send_error.is_none());
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
}

#[test]
fn manual_topic_alias_store_not_registered() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .props(vec![mqtt::packet::SessionExpiryInterval::new(0xffffffff)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::TopicAliasMaximum::new(65535)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Send QoS0 PUBLISH B with unregistered topic alias
        let pid = connection.acquire_packet_id().unwrap();
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(pid)
            .payload(b"payload A".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
            .build()
            .unwrap();

        let events_a = connection.send(publish_a.into());

        // Test that events_a contains exactly 2 events:
        // 1. GenericEvent::NotifyError(MqttError::PacketNotAllowedToSend)
        // 2. GenericEvent::NotifyPacketIdReleased(pid)
        assert_eq!(events_a.len(), 2);

        // First event should be NotifyError
        if let mqtt::connection::Event::NotifyError(error) = &events_a[0] {
            assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
        } else {
            panic!(
                "Expected NotifyError(PacketNotAllowedToSend), got: {:?}",
                events_a[0]
            );
        }

        // Second event should be NotifyPacketIdReleased
        if let mqtt::connection::Event::NotifyPacketIdReleased(released_pid) = &events_a[1] {
            assert_eq!(released_pid, &pid);
        } else {
            panic!(
                "Expected NotifyPacketIdReleased({}), got: {:?}",
                pid, events_a[1]
            );
        }
    }
}

#[test]
fn manual_topic_alias_not_registered() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::TopicAliasMaximum::new(65535)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Send QoS0 PUBLISH B with unregistered topic alias
        let pid = connection.acquire_packet_id().unwrap();
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(pid)
            .payload(b"payload A".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
            .build()
            .unwrap();

        let events_a = connection.send(publish_a.into());

        // Test that events_a contains exactly 2 events:
        // 1. GenericEvent::NotifyError(MqttError::PacketNotAllowedToSend)
        // 2. GenericEvent::NotifyPacketIdReleased(pid)
        assert_eq!(events_a.len(), 2);

        // First event should be NotifyError
        if let mqtt::connection::Event::NotifyError(error) = &events_a[0] {
            assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
        } else {
            panic!(
                "Expected NotifyError(PacketNotAllowedToSend), got: {:?}",
                events_a[0]
            );
        }

        // Second event should be NotifyPacketIdReleased
        if let mqtt::connection::Event::NotifyPacketIdReleased(released_pid) = &events_a[1] {
            assert_eq!(released_pid, &pid);
        } else {
            panic!(
                "Expected NotifyPacketIdReleased({}), got: {:?}",
                pid, events_a[1]
            );
        }
    }
}

#[test]
fn manual_topic_alias_oor() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::TopicAliasMaximum::new(3)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Send QoS0 PUBLISH B with unregistered topic alias
        let pid = connection.acquire_packet_id().unwrap();
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(pid)
            .topic_name("topic/a")
            .unwrap()
            .payload(b"payload B".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(4).unwrap().into()])
            .build()
            .unwrap();

        let events_a = connection.send(publish_a.into());

        // Test that events_a contains exactly 2 events:
        // 1. GenericEvent::NotifyError(MqttError::PacketNotAllowedToSend)
        // 2. GenericEvent::NotifyPacketIdReleased(pid)
        assert_eq!(events_a.len(), 2);

        // First event should be NotifyError
        if let mqtt::connection::Event::NotifyError(error) = &events_a[0] {
            assert_eq!(error, &mqtt::result_code::MqttError::PacketNotAllowedToSend);
        } else {
            panic!(
                "Expected NotifyError(PacketNotAllowedToSend), got: {:?}",
                events_a[0]
            );
        }

        // Second event should be NotifyPacketIdReleased
        if let mqtt::connection::Event::NotifyPacketIdReleased(released_pid) = &events_a[1] {
            assert_eq!(released_pid, &pid);
        } else {
            panic!(
                "Expected NotifyPacketIdReleased({}), got: {:?}",
                pid, events_a[1]
            );
        }
    }
}

#[test]
fn manual_topic_alias_register_oor_recv() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .props(vec![mqtt::packet::TopicAliasMaximum::new(3)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Recv QoS0 PUBLISH A with unregistered topic alias
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(1)
            .topic_name("topic/a")
            .unwrap()
            .payload(b"payload A".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(4).unwrap().into()])
            .build()
            .unwrap();

        let bytes = publish_a.to_continuous_buffer();
        let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

        assert_eq!(events.len(), 3);

        // First event: RequestSendPacket with Disconnect packet
        if let mqtt::connection::Event::RequestSendPacket {
            packet: event_packet,
            release_packet_id_if_send_error,
        } = &events[0]
        {
            let expected_disconnect: mqtt::packet::Packet =
                mqtt::packet::v5_0::Disconnect::builder()
                    .reason_code(mqtt::result_code::DisconnectReasonCode::TopicAliasInvalid)
                    .build()
                    .unwrap()
                    .into();
            assert_eq!(*event_packet, expected_disconnect);
            assert!(release_packet_id_if_send_error.is_none());
        } else {
            assert!(
                false,
                "Expected RequestSendPacket event, but got: {:?}",
                events[0]
            );
        }

        // Second event: RequestClose
        if let mqtt::connection::Event::RequestClose = &events[1] {
            // Expected RequestClose event
        } else {
            assert!(
                false,
                "Expected RequestClose event, but got: {:?}",
                events[1]
            );
        }

        // Third event: NotifyError with TopicAliasInvalid
        if let mqtt::connection::Event::NotifyError(error) = &events[2] {
            assert_eq!(*error, mqtt::result_code::MqttError::TopicAliasInvalid);
        } else {
            assert!(
                false,
                "Expected NotifyError(TopicAliasInvalid) event, but got: {:?}",
                events[2]
            );
        }
    }
}

#[test]
fn manual_topic_alias_extract_recv() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .props(vec![mqtt::packet::TopicAliasMaximum::new(3)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Recv QoS0 PUBLISH A with topic alias register
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(1)
            .topic_name("topic/a")
            .unwrap()
            .payload(b"payload A".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
            .build()
            .unwrap();

        let bytes = publish_a.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }
    {
        // Recv QoS0 PUBLISH B with using topic alias
        let publish_b = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(1)
            .payload(b"payload B".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
            .build()
            .unwrap();

        let bytes = publish_b.to_continuous_buffer();
        let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

        assert_eq!(events.len(), 1);
        let expected = publish_b.add_extracted_topic_name("topic/a").unwrap();
        if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[0] {
            if let mqtt::packet::GenericPacket::V5_0Publish(publish) = packet {
                assert_eq!(*publish, expected);
            } else {
                panic!("Expected V5_0Publish packet, got: {:?}", packet);
            }
        } else {
            panic!("Expected NotifyPacketReceived event, got: {:?}", events[0]);
        }
    }
}

#[test]
fn manual_topic_alias_use_oor_recv() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .props(vec![mqtt::packet::TopicAliasMaximum::new(3)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Recv QoS0 PUBLISH A with unregistered topic alias
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(1)
            .payload(b"payload A".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(4).unwrap().into()])
            .build()
            .unwrap();

        let bytes = publish_a.to_continuous_buffer();
        let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

        assert_eq!(events.len(), 3);

        // First event: RequestSendPacket with Disconnect packet
        if let mqtt::connection::Event::RequestSendPacket {
            packet: event_packet,
            release_packet_id_if_send_error,
        } = &events[0]
        {
            let expected_disconnect: mqtt::packet::Packet =
                mqtt::packet::v5_0::Disconnect::builder()
                    .reason_code(mqtt::result_code::DisconnectReasonCode::TopicAliasInvalid)
                    .build()
                    .unwrap()
                    .into();
            assert_eq!(*event_packet, expected_disconnect);
            assert!(release_packet_id_if_send_error.is_none());
        } else {
            assert!(
                false,
                "Expected RequestSendPacket event, but got: {:?}",
                events[0]
            );
        }

        // Second event: RequestClose
        if let mqtt::connection::Event::RequestClose = &events[1] {
            // Expected RequestClose event
        } else {
            assert!(
                false,
                "Expected RequestClose event, but got: {:?}",
                events[1]
            );
        }

        // Third event: NotifyError with TopicAliasInvalid
        if let mqtt::connection::Event::NotifyError(error) = &events[2] {
            assert_eq!(*error, mqtt::result_code::MqttError::TopicAliasInvalid);
        } else {
            assert!(
                false,
                "Expected NotifyError(TopicAliasInvalid) event, but got: {:?}",
                events[2]
            );
        }
    }
}

#[test]
fn manual_topic_alias_use_unreg_recv() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .props(vec![mqtt::packet::TopicAliasMaximum::new(3)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Recv QoS0 PUBLISH A with unregistered topic alias
        let publish_a = mqtt::packet::v5_0::Publish::builder()
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .packet_id(1)
            .payload(b"payload A".to_vec())
            .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
            .build()
            .unwrap();

        let bytes = publish_a.to_continuous_buffer();
        let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

        assert_eq!(events.len(), 3);

        // First event: RequestSendPacket with Disconnect packet
        if let mqtt::connection::Event::RequestSendPacket {
            packet: event_packet,
            release_packet_id_if_send_error,
        } = &events[0]
        {
            let expected_disconnect: mqtt::packet::Packet =
                mqtt::packet::v5_0::Disconnect::builder()
                    .reason_code(mqtt::result_code::DisconnectReasonCode::TopicAliasInvalid)
                    .build()
                    .unwrap()
                    .into();
            assert_eq!(*event_packet, expected_disconnect);
            assert!(release_packet_id_if_send_error.is_none());
        } else {
            assert!(
                false,
                "Expected RequestSendPacket event, but got: {:?}",
                events[0]
            );
        }

        // Second event: RequestClose
        if let mqtt::connection::Event::RequestClose = &events[1] {
            // Expected RequestClose event
        } else {
            assert!(
                false,
                "Expected RequestClose event, but got: {:?}",
                events[1]
            );
        }

        // Third event: NotifyError with TopicAliasInvalid
        if let mqtt::connection::Event::NotifyError(error) = &events[2] {
            assert_eq!(*error, mqtt::result_code::MqttError::TopicAliasInvalid);
        } else {
            assert!(
                false,
                "Expected NotifyError(TopicAliasInvalid) event, but got: {:?}",
                events[2]
            );
        }
    }
}

#[test]
fn manual_topic_alias_no_prop_recv() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    {
        // Send CONNECT
        let connect = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .props(vec![mqtt::packet::TopicAliasMaximum::new(3)
                .unwrap()
                .into()])
            .build()
            .unwrap();

        let _events = connection.send(connect.into());

        // Receive CONNACK with TopicAliasMaximum set to 65535
        let connack = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .unwrap();

        let bytes = connack.to_continuous_buffer();
        let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    }

    {
        // Recv QoS1 PUBLISH A has empty topic and no topic alias
        // Manually create MQTT v5.0 PUBLISH packet bytes:
        // Fixed Header: 0x32 (PUBLISH, QoS=1, DUP=0, RETAIN=0), remaining length
        // Variable Header: topic_name (0 bytes), packet_id (2 bytes), properties length (1 byte = 0)
        // Payload: "payload A"
        let bytes = vec![
            0x32, // Fixed header: PUBLISH packet type (3 << 4) | QoS=1 (0x02)
            0x0E, // Remaining length: 14 bytes
            // Variable header:
            0x00, 0x00, // Topic name length = 0 (empty topic)
            0x00, 0x01, // Packet ID = 1
            0x00, // Properties length = 0
            // Payload:
            b'p', b'a', b'y', b'l', b'o', b'a', b'd', b' ', b'A',
        ];
        let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

        assert_eq!(events.len(), 3);

        // First event: RequestSendPacket with Disconnect packet
        if let mqtt::connection::Event::RequestSendPacket {
            packet: event_packet,
            release_packet_id_if_send_error,
        } = &events[0]
        {
            let expected_disconnect: mqtt::packet::Packet =
                mqtt::packet::v5_0::Disconnect::builder()
                    .reason_code(mqtt::result_code::DisconnectReasonCode::TopicAliasInvalid)
                    .build()
                    .unwrap()
                    .into();
            assert_eq!(*event_packet, expected_disconnect);
            assert!(release_packet_id_if_send_error.is_none());
        } else {
            assert!(
                false,
                "Expected RequestSendPacket event, but got: {:?}",
                events[0]
            );
        }

        // Second event: RequestClose
        if let mqtt::connection::Event::RequestClose = &events[1] {
            // Expected RequestClose event
        } else {
            assert!(
                false,
                "Expected RequestClose event, but got: {:?}",
                events[1]
            );
        }

        // Third event: NotifyError with TopicAliasInvalid
        if let mqtt::connection::Event::NotifyError(error) = &events[2] {
            assert_eq!(*error, mqtt::result_code::MqttError::TopicAliasInvalid);
        } else {
            assert!(
                false,
                "Expected NotifyError(TopicAliasInvalid) event, but got: {:?}",
                events[2]
            );
        }
    }
}
#[test]
fn regulate_for_store_topic_alias() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::TopicAliasMaximum::new(65535)
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    let packet_id_1 = connection.acquire_packet_id().unwrap();
    let publish_original = mqtt::packet::v5_0::Publish::builder()
        .topic_name("original/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_1)
        .payload(b"original payload".to_vec())
        .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
        .build()
        .unwrap();

    let _events = connection.send(publish_original.into());

    let packet_id_2 = connection.acquire_packet_id().unwrap();
    let publish_empty_topic = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_2)
        .payload(b"test payload 1".to_vec())
        .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
        .build()
        .unwrap();

    let result_1 = connection.regulate_for_store(publish_empty_topic);
    assert!(result_1.is_ok());
    let regulated = result_1.unwrap();
    assert_eq!(regulated.topic_name(), "original/topic");
    assert!(!regulated
        .props()
        .iter()
        .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_))));

    let packet_id_3 = connection.acquire_packet_id().unwrap();
    let publish_specified_topic = mqtt::packet::v5_0::Publish::builder()
        .topic_name("new/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_3)
        .payload(b"test payload 2".to_vec())
        .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
        .build()
        .unwrap();

    let result_2 = connection.regulate_for_store(publish_specified_topic);
    assert!(result_2.is_ok());
    let regulated = result_2.unwrap();
    assert_eq!(regulated.topic_name(), "new/topic");
    assert!(!regulated
        .props()
        .iter()
        .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_))));

    let packet_id_6 = connection.acquire_packet_id().unwrap();
    let publish_unregistered_alias = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(packet_id_6)
        .payload(b"test payload 5".to_vec())
        .props(vec![mqtt::packet::TopicAlias::new(2).unwrap().into()])
        .build()
        .unwrap();

    let result_5 = connection.regulate_for_store(publish_unregistered_alias);
    assert!(result_5.is_err());
    if let Err(e) = result_5 {
        assert!(matches!(
            e,
            mqtt::result_code::MqttError::PacketNotRegulated
        ));
    }
}

// fn client_set_topic_alias_maximum_recv_out_of_range() {
//     common::init_tracing();
//     let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

//     // Send CONNECT
//     let connect = mqtt::packet::v5_0::Connect::builder()
//         .client_id("test_client")
//         .unwrap()
//         .props(vec![mqtt::packet::TopicAliasMaximum::new(3)
//             .unwrap()
//             .into()])
//         .build()
//         .unwrap();

//     let _events = connection.send(connect.into());

//     // Receive CONNACK with TopicAliasMaximum set to 65535
//     let connack = mqtt::packet::v5_0::Connack::builder()
//         .session_present(false)
//         .reason_code(mqtt::result_code::ConnectReasonCode::Success)
//         .build()
//         .unwrap();

// }
