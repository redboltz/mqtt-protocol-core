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

// Runtime check for packet not allowed to send
// v3.1.1 client
#[test]
fn v3_1_1_client_not_allowed_to_send_v3_1_1_connack() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v3_1_1_suback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v3_1_1_unsuback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v3_1_1_pingresp() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingresp::builder()
        .build()
        .expect("Failed to build Pingresp packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_connect() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_connack() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_publish() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(())
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_puback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Puback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Puback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_pubrec() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrec packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_pubrel() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_pubcomp() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubcomp packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_subscribe() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_suback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_unsubscribe() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let entry = mqtt::packet::MqttString::new("test/topic").unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_unsuback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_pingreq() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pingreq::builder()
        .build()
        .expect("Failed to build Pingreq packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_pingresp() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pingresp::builder()
        .build()
        .expect("Failed to build Pingresp packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_disconnect() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Disconnect::builder()
        .build()
        .expect("Failed to build Disconnect packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_client_not_allowed_to_send_v5_0_auth() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Auth::builder()
        .build()
        .expect("Failed to build Auth packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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

// v3.1.1 server

#[test]
fn v3_1_1_server_not_allowed_to_send_v3_1_1_connect() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connect packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v3_1_1_subscribe() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1)
        .entries(vec![
            mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap(),
        ])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v3_1_1_unsubscribe() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec![mqtt::packet::MqttString::new("test/topic").unwrap()])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v3_1_1_pingresq() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingreq::builder()
        .build()
        .expect("Failed to build Pingreq packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v3_1_1_disconnect() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Disconnect::builder()
        .build()
        .expect("Failed to build Disconnect packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_connect() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_connack() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_publish() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(())
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_puback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Puback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Puback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_pubrec() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrec packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_pubrel() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_pubcomp() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubcomp packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_subscribe() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_suback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_unsubscribe() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let entry = mqtt::packet::MqttString::new("test/topic").unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_unsuback() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_pingreq() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pingreq::builder()
        .build()
        .expect("Failed to build Pingreq packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_pingresp() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pingresp::builder()
        .build()
        .expect("Failed to build Pingresp packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_disconnect() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Disconnect::builder()
        .build()
        .expect("Failed to build Disconnect packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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
fn v3_1_1_server_not_allowed_to_send_v5_0_auth() {
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Auth::builder()
        .build()
        .expect("Failed to build Auth packet")
        .into();
    let events = con_v3_1_1.checked_send(packet);
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

// v5.0 client
#[test]
fn v5_0_client_not_allowed_to_send_v5_0_connack() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v5_0_suback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v5_0_unsuback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v5_0_pingresp() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingresp::builder()
        .build()
        .expect("Failed to build Pingresp packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_connect() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_connack() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_publish() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(())
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_puback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Puback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Puback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_pubrec() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrec packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_pubrel() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_pubcomp() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubcomp::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubcomp packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_subscribe() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_suback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_unsubscribe() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let entry = mqtt::packet::MqttString::new("test/topic").unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_unsuback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_pingreq() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingreq::builder()
        .build()
        .expect("Failed to build Pingreq packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_pingresp() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingresp::builder()
        .build()
        .expect("Failed to build Pingresp packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_client_not_allowed_to_send_v3_1_1_disconnect() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Disconnect::builder()
        .build()
        .expect("Failed to build Disconnect packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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

// v5.0 server

#[test]
fn v5_0_server_not_allowed_to_send_v5_0_connect() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connect packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v5_0_subscribe() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1)
        .entries(vec![
            mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap(),
        ])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v5_0_unsubscribe() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec![mqtt::packet::MqttString::new("test/topic").unwrap()])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v5_0_pingresq() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pingreq::builder()
        .build()
        .expect("Failed to build Pingreq packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_connect() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_connack() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .expect("Failed to build Connack packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_publish() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(())
        .build()
        .expect("Failed to build Publish packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_puback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Puback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Puback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_pubrec() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrec packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_pubrel() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_pubcomp() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubcomp::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Pubcomp packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_subscribe() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_suback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .expect("Failed to build Suback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_unsubscribe() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let entry = mqtt::packet::MqttString::new("test/topic").unwrap();
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(1)
        .entries(vec![entry])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_unsuback() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(1)
        .build()
        .expect("Failed to build Unsuback packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_pingreq() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingreq::builder()
        .build()
        .expect("Failed to build Pingreq packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_pingresp() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pingresp::builder()
        .build()
        .expect("Failed to build Pingresp packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
fn v5_0_server_not_allowed_to_send_v3_1_1_disconnect() {
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Disconnect::builder()
        .build()
        .expect("Failed to build Disconnect packet")
        .into();
    let events = con_v5_0.checked_send(packet);
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
