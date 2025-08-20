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
fn v5_0_any_maximum_packet_size_test() {
    common::init_tracing();
    // Create MQTT v5.0 Any connection
    let mut con = mqtt::Connection::<mqtt::role::Any>::new(mqtt::Version::V5_0);

    // Send CONNECT packet
    let connect_packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test-client")
        .unwrap()
        .clean_start(true)
        .build()
        .expect("Failed to build Connect packet");
    let events = con.send(connect_packet.into());
    assert_eq!(events.len(), 1);

    // Create CONNACK with small MaximumPacketSize property
    let maximum_packet_size = 50u32; // Small but realistic value
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::MaximumPacketSize::new(maximum_packet_size)
            .expect("Failed to create MaximumPacketSize property")
            .into(),
    );

    let connack_packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(props)
        .build()
        .expect("Failed to build Connack packet");

    // Convert CONNACK to raw bytes and receive it
    let flattened = connack_packet.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Also verify to_buffers() produces same result when std is available
        let buffers = connack_packet.to_buffers();
        let from_buffers: Vec<u8> = buffers
            .iter()
            .flat_map(|slice| slice.iter().copied())
            .collect();
        assert_eq!(flattened, from_buffers);
    }

    let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
    let events = con.recv(&mut cursor);

    // Verify connection is established
    assert!(events
        .iter()
        .any(|event| matches!(event, mqtt::connection::Event::NotifyPacketReceived(_))));

    // Test PUBLISH packet that exceeds maximum packet size
    let large_payload = vec![0u8; (maximum_packet_size + 10) as usize]; // Exceeds limit
    let publish_packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(large_payload)
        .build()
        .expect("Failed to build Publish packet");

    let events = con.send(publish_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for PUBLISH packet"
    );

    // Test PUBACK packet that exceeds maximum packet size
    let mut large_props = mqtt::packet::GenericProperties::new();
    for i in 0..20 {
        large_props.push(
            mqtt::packet::UserProperty::new(&format!("key{}", i), &format!("value{}", i))
                .expect("Failed to create UserProperty")
                .into(),
        );
    }

    let puback_packet = mqtt::packet::v5_0::Puback::builder()
        .packet_id(1u16)
        .reason_code(mqtt::result_code::PubackReasonCode::Success)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Puback packet");

    let events = con.send(puback_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for PUBACK packet"
    );

    // Test PUBREC packet that exceeds maximum packet size
    let pubrec_packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(2u16)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Pubrec packet");

    let events = con.send(pubrec_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for PUBREC packet"
    );

    // Test PUBREL packet that exceeds maximum packet size
    let pubrel_packet = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(3u16)
        .reason_code(mqtt::result_code::PubrelReasonCode::Success)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Pubrel packet");

    let events = con.send(pubrel_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for PUBREL packet"
    );

    // Test PUBCOMP packet that exceeds maximum packet size
    let pubcomp_packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(4u16)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Pubcomp packet");

    let events = con.send(pubcomp_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for PUBCOMP packet"
    );

    // Test SUBSCRIBE packet that exceeds maximum packet size
    let mut large_entries = Vec::new();
    for i in 0..50 {
        large_entries.push(
            mqtt::packet::SubEntry::new(
                &format!("very/long/topic/name/that/will/make/packet/large/{}", i),
                mqtt::packet::SubOpts::default(),
            )
            .expect("Failed to create SubEntry"),
        );
    }

    let subscribe_packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(5u16)
        .entries(large_entries)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Subscribe packet");

    let events = con.send(subscribe_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for SUBSCRIBE packet"
    );

    // Test SUBACK packet that exceeds maximum packet size
    let large_reason_codes = vec![mqtt::result_code::SubackReasonCode::GrantedQos0; 100];
    let suback_packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(6u16)
        .reason_codes(large_reason_codes)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Suback packet");

    let events = con.send(suback_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for SUBACK packet"
    );

    // Test UNSUBSCRIBE packet that exceeds maximum packet size
    let mut large_topic_filters = Vec::new();
    for i in 0..50 {
        large_topic_filters.push(format!(
            "very/long/topic/filter/that/will/make/packet/large/{}",
            i
        ));
    }

    let unsubscribe_packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(7u16)
        .entries(large_topic_filters)
        .unwrap()
        .props(large_props.clone())
        .build()
        .expect("Failed to build Unsubscribe packet");

    let events = con.send(unsubscribe_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for UNSUBSCRIBE packet"
    );

    // Test UNSUBACK packet that exceeds maximum packet size
    let large_unsuback_reason_codes = vec![mqtt::result_code::UnsubackReasonCode::Success; 100];
    let unsuback_packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(8u16)
        .reason_codes(large_unsuback_reason_codes)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Unsuback packet");

    let events = con.send(unsuback_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for UNSUBACK packet"
    );

    // Test AUTH packet that exceeds maximum packet size
    let auth_packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .props(large_props.clone())
        .build()
        .expect("Failed to build Auth packet");

    let events = con.send(auth_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for AUTH packet"
    );

    // Test DISCONNECT packet that exceeds maximum packet size
    let disconnect_packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(large_props)
        .build()
        .expect("Failed to build Disconnect packet");

    let events = con.send(disconnect_packet.into());
    assert!(
        events.iter().any(|event| matches!(
            event,
            mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketTooLarge)
        )),
        "Expected PacketTooLarge error for DISCONNECT packet"
    );
}
