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
mqtt_protocol_core::make_default_aliases!();
mod common;

///////////////////////////////////////////////////////////////////////////////
// Runtime check for packet not allowed to send

// v3.1.1 client invalid packet type

#[test]
fn v3_1_1_client_not_allowed_to_send_v3_1_1_connack() {
    common::init_tracing();
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
    common::init_tracing();
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
    common::init_tracing();
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
    common::init_tracing();
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

// v3.1.1 client version mismatch (representative packet)

#[test]
fn v3_1_1_client_not_allowed_to_send_v5_0_connect() {
    common::init_tracing();
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

///////////////////////////////////////////////////////////////////////////////

// v3.1.1 server invalid packet type

#[test]
fn v3_1_1_server_not_allowed_to_send_v3_1_1_connect() {
    common::init_tracing();
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
    common::init_tracing();
    let mut con_v3_1_1 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);
    let packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(1)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
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
    common::init_tracing();
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
    common::init_tracing();
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
    common::init_tracing();
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

// v3.1.1 server version mismatch (representative packet)

#[test]
fn v3_1_1_server_not_allowed_to_send_v5_0_connack() {
    common::init_tracing();
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

///////////////////////////////////////////////////////////////////////////////

// v5.0 client invalid packet type

#[test]
fn v5_0_client_not_allowed_to_send_v5_0_connack() {
    common::init_tracing();
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
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
    common::init_tracing();
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
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
    common::init_tracing();
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
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
    common::init_tracing();
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pingresp::builder()
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

///////////////////////////////////////////////////////////////////////////////

// v5.0 client version mismatch (representative packet)

#[test]
fn v5_0_client_not_allowed_to_send_v3_1_1_connect() {
    common::init_tracing();
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

///////////////////////////////////////////////////////////////////////////////

// v5.0 server invalid packet type

#[test]
fn v5_0_server_not_allowed_to_send_v5_0_connect() {
    common::init_tracing();
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
    common::init_tracing();
    let mut con_v5_0 = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);
    let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(1)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
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
    common::init_tracing();
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
    common::init_tracing();
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

///////////////////////////////////////////////////////////////////////////////

// v5.0 client version mismatch (representative packet)

#[test]
fn v5_0_server_not_allowed_to_send_v3_1_1_connack() {
    common::init_tracing();
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
