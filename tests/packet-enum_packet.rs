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
use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;

#[test]
fn test_generic_packet_v3_1_1_connect_size() {
    common::init_tracing();
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .clean_start(true)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Connect(connect.clone());

    assert_eq!(packet.size(), connect.size());
    assert!(packet.size() > 0);
}

#[test]
#[cfg(feature = "std")]
fn test_generic_packet_v3_1_1_connect_to_buffers() {
    common::init_tracing();
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .clean_start(true)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Connect(connect.clone());

    let packet_buffers = packet.to_buffers();
    let connect_buffers = connect.to_buffers();

    assert_eq!(packet_buffers.len(), connect_buffers.len());
    // IoSlice doesn't implement PartialEq, so we compare the lengths and actual data
    for (packet_buf, connect_buf) in packet_buffers.iter().zip(connect_buffers.iter()) {
        assert_eq!(packet_buf.len(), connect_buf.len());
        assert_eq!(**packet_buf, **connect_buf);
    }
}

#[test]
fn test_generic_packet_v3_1_1_connect_debug() {
    common::init_tracing();
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test-client")
        .unwrap()
        .clean_start(true)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Connect(connect);

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("test-client"));
    assert!(!debug_str.is_empty());
}

#[test]
fn test_generic_packet_v3_1_1_connect_display() {
    common::init_tracing();
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test-client")
        .unwrap()
        .clean_start(true)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Connect(connect);

    let display_str = format!("{packet}");
    assert!(display_str.contains("test-client"));
    assert!(!display_str.is_empty());
}

#[test]
fn test_generic_packet_v3_1_1_connack() {
    common::init_tracing();
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Connack(connack.clone());

    assert_eq!(packet.size(), connack.size());
    assert!(packet.size() > 0);

    #[cfg(feature = "std")]
    {
        let packet_buffers = packet.to_buffers();
        let connack_buffers = connack.to_buffers();
        assert_eq!(packet_buffers.len(), connack_buffers.len());
    }

    let debug_str = format!("{packet:?}");
    assert!(!debug_str.is_empty());

    let display_str = format!("{packet}");
    assert!(!display_str.is_empty());
}

#[test]
fn test_generic_packet_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(vec![1, 2, 3])
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Publish(publish.clone());

    assert_eq!(packet.size(), publish.size());
    assert!(packet.size() > 0);

    #[cfg(feature = "std")]
    {
        let packet_buffers = packet.to_buffers();
        let publish_buffers = publish.to_buffers();
        assert_eq!(packet_buffers.len(), publish_buffers.len());
    }

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("test/topic"));

    let display_str = format!("{packet}");
    assert!(display_str.contains("test/topic"));
}

#[test]
fn test_generic_packet_v3_1_1_puback() {
    common::init_tracing();
    let puback = mqtt::packet::v3_1_1::Puback::builder()
        .packet_id(123)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Puback(puback.clone());

    assert_eq!(packet.size(), puback.size());
    assert!(packet.size() > 0);

    let debug_str = format!("{packet:?}");
    assert!(!debug_str.is_empty());

    let display_str = format!("{packet}");
    assert!(!display_str.is_empty());
}

#[test]
fn test_generic_packet_v3_1_1_disconnect() {
    common::init_tracing();
    let disconnect = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Disconnect(disconnect.clone());

    assert_eq!(packet.size(), disconnect.size());
    assert!(packet.size() > 0);

    let debug_str = format!("{packet:?}");
    assert!(!debug_str.is_empty());

    let display_str = format!("{packet}");
    assert!(!display_str.is_empty());
}

#[test]
fn test_generic_packet_v3_1_1_pingreq() {
    common::init_tracing();
    let pingreq = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Pingreq(pingreq.clone());

    assert_eq!(packet.size(), pingreq.size());
    assert!(packet.size() > 0);

    let debug_str = format!("{packet:?}");
    assert!(!debug_str.is_empty());

    let display_str = format!("{packet}");
    assert!(!display_str.is_empty());
}

#[test]
fn test_generic_packet_v3_1_1_pingresp() {
    common::init_tracing();
    let pingresp = mqtt::packet::v3_1_1::Pingresp::builder().build().unwrap();
    let packet = mqtt::packet::Packet::V3_1_1Pingresp(pingresp.clone());

    assert_eq!(packet.size(), pingresp.size());
    assert!(packet.size() > 0);

    let debug_str = format!("{packet:?}");
    assert!(!debug_str.is_empty());

    let display_str = format!("{packet}");
    assert!(!display_str.is_empty());
}

#[test]
fn test_generic_packet_v5_0_connect() {
    common::init_tracing();
    let connect = mqtt::packet::v5_0::Connect::builder()
        .clean_start(true)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V5_0Connect(connect.clone());

    assert_eq!(packet.size(), connect.size());
    assert!(packet.size() > 0);

    let debug_str = format!("{packet:?}");
    assert!(!debug_str.is_empty());

    let display_str = format!("{packet}");
    assert!(!display_str.is_empty());
}

#[test]
fn test_generic_packet_v5_0_auth() {
    common::init_tracing();
    let auth = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();
    let packet = mqtt::packet::Packet::V5_0Auth(auth.clone());

    assert_eq!(packet.size(), auth.size());
    assert!(packet.size() > 0);

    let debug_str = format!("{packet:?}");
    assert!(!debug_str.is_empty());

    let display_str = format!("{packet}");
    assert!(!display_str.is_empty());
}

// GenericPacket packet_type() tests
#[test]
fn test_generic_packet_packet_type_v3_1_1() {
    common::init_tracing();
    // V3.1.1 Connect
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .clean_start(true)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Connect(connect);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Connect);

    // V3.1.1 Connack
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Connack(connack);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Connack);

    // V3.1.1 Publish
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Publish(publish);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Publish);

    // V3.1.1 Puback
    let puback = mqtt::packet::v3_1_1::Puback::builder()
        .packet_id(123)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Puback(puback);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Puback);

    // V3.1.1 Pubrec
    let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(123)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Pubrec(pubrec);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pubrec);

    // V3.1.1 Pubrel
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(123)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Pubrel(pubrel);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pubrel);

    // V3.1.1 Pubcomp
    let pubcomp = mqtt::packet::v3_1_1::Pubcomp::builder()
        .packet_id(123)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Pubcomp(pubcomp);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pubcomp);

    // V3.1.1 Subscribe
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let subscribe = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(123)
        .entries(vec![entry])
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Subscribe(subscribe);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Subscribe);

    // V3.1.1 Suback
    let suback = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(123)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Suback(suback);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Suback);

    // V3.1.1 Unsubscribe
    let unsubscribe = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(123)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Unsubscribe(unsubscribe);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Unsubscribe);

    // V3.1.1 Unsuback
    let unsuback = mqtt::packet::v3_1_1::Unsuback::builder()
        .packet_id(123)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Unsuback(unsuback);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Unsuback);

    // V3.1.1 Pingreq
    let pingreq = mqtt::packet::v3_1_1::Pingreq::builder().build().unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Pingreq(pingreq);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pingreq);

    // V3.1.1 Pingresp
    let pingresp = mqtt::packet::v3_1_1::Pingresp::builder().build().unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Pingresp(pingresp);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pingresp);

    // V3.1.1 Disconnect
    let disconnect = mqtt::packet::v3_1_1::Disconnect::builder().build().unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V3_1_1Disconnect(disconnect);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Disconnect);
}

#[test]
fn test_generic_packet_packet_type_v5_0() {
    common::init_tracing();
    // V5.0 Connect
    let connect = mqtt::packet::v5_0::Connect::builder()
        .clean_start(true)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Connect(connect);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Connect);

    // V5.0 Connack
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Connack(connack);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Connack);

    // V5.0 Publish
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Publish(publish);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Publish);

    // V5.0 Puback
    let puback = mqtt::packet::v5_0::Puback::builder()
        .packet_id(123)
        .reason_code(mqtt::result_code::PubackReasonCode::Success)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> = mqtt::packet::GenericPacket::V5_0Puback(puback);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Puback);

    // V5.0 Pubrec
    let pubrec = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(123)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> = mqtt::packet::GenericPacket::V5_0Pubrec(pubrec);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pubrec);

    // V5.0 Pubrel
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(123)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> = mqtt::packet::GenericPacket::V5_0Pubrel(pubrel);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pubrel);

    // V5.0 Pubcomp
    let pubcomp = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(123)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Pubcomp(pubcomp);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pubcomp);

    // V5.0 Subscribe
    let entry =
        mqtt::packet::SubEntry::new("test/topic", mqtt::packet::SubOpts::default()).unwrap();
    let subscribe = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(123)
        .entries(vec![entry])
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Subscribe(subscribe);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Subscribe);

    // V5.0 Suback
    let suback = mqtt::packet::v5_0::Suback::builder()
        .packet_id(123)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> = mqtt::packet::GenericPacket::V5_0Suback(suback);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Suback);

    // V5.0 Unsubscribe
    let unsubscribe = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(123)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Unsubscribe(unsubscribe);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Unsubscribe);

    // V5.0 Unsuback
    let unsuback = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(123)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Unsuback(unsuback);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Unsuback);

    // V5.0 Pingreq
    let pingreq = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Pingreq(pingreq);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pingreq);

    // V5.0 Pingresp
    let pingresp = mqtt::packet::v5_0::Pingresp::builder().build().unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Pingresp(pingresp);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Pingresp);

    // V5.0 Disconnect
    let disconnect = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> =
        mqtt::packet::GenericPacket::V5_0Disconnect(disconnect);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Disconnect);

    // V5.0 Auth
    let auth = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();
    let packet: mqtt::packet::GenericPacket<u16> = mqtt::packet::GenericPacket::V5_0Auth(auth);
    assert_eq!(packet.packet_type(), mqtt::packet::PacketType::Auth);
}
