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
fn test_auto_map_topic_alias_send_v5_0() {
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
                if let Some(props) = p.props() {
                    for prop in props.iter() {
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
                if let Some(props) = p.props() {
                    for prop in props.iter() {
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
    }
    assert!(
        publish_b_mapped,
        "PUBLISH B should have empty topic name and TopicAlias=2"
    );
}

#[test]
fn test_auto_replace_topic_alias_send_v5_0() {
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
                let has_topic_alias = if let Some(props) = p.props() {
                    props
                        .iter()
                        .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_)))
                } else {
                    false
                };
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
                if let Some(props) = p.props() {
                    for prop in props.iter() {
                        if let mqtt::packet::Property::TopicAlias(ta) = prop {
                            if ta.val() == 1 {
                                publish_c_replaced = true;
                                break;
                            }
                        }
                    }
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
fn test_regulate_for_store_topic_alias_v5_0() {
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
    if let Some(props) = regulated.props() {
        assert!(!props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_))));
    }

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
    if let Some(props) = regulated.props() {
        assert!(!props
            .iter()
            .any(|prop| matches!(prop, mqtt::packet::Property::TopicAlias(_))));
    }

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
