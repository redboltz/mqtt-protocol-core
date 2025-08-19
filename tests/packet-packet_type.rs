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
mod common;

// PacketType basic functionality tests

#[test]
fn packet_type_as_u8() {
    common::init_tracing();
    assert_eq!(mqtt::packet::PacketType::Connect.as_u8(), 1);
    assert_eq!(mqtt::packet::PacketType::Connack.as_u8(), 2);
    assert_eq!(mqtt::packet::PacketType::Publish.as_u8(), 3);
    assert_eq!(mqtt::packet::PacketType::Puback.as_u8(), 4);
    assert_eq!(mqtt::packet::PacketType::Pubrec.as_u8(), 5);
    assert_eq!(mqtt::packet::PacketType::Pubrel.as_u8(), 6);
    assert_eq!(mqtt::packet::PacketType::Pubcomp.as_u8(), 7);
    assert_eq!(mqtt::packet::PacketType::Subscribe.as_u8(), 8);
    assert_eq!(mqtt::packet::PacketType::Suback.as_u8(), 9);
    assert_eq!(mqtt::packet::PacketType::Unsubscribe.as_u8(), 10);
    assert_eq!(mqtt::packet::PacketType::Unsuback.as_u8(), 11);
    assert_eq!(mqtt::packet::PacketType::Pingreq.as_u8(), 12);
    assert_eq!(mqtt::packet::PacketType::Pingresp.as_u8(), 13);
    assert_eq!(mqtt::packet::PacketType::Disconnect.as_u8(), 14);
    assert_eq!(mqtt::packet::PacketType::Auth.as_u8(), 15);
}

#[test]
fn packet_type_as_str() {
    common::init_tracing();
    assert_eq!(mqtt::packet::PacketType::Connect.as_str(), "connect");
    assert_eq!(mqtt::packet::PacketType::Connack.as_str(), "connack");
    assert_eq!(mqtt::packet::PacketType::Publish.as_str(), "publish");
    assert_eq!(mqtt::packet::PacketType::Puback.as_str(), "puback");
    assert_eq!(mqtt::packet::PacketType::Pubrec.as_str(), "pubrec");
    assert_eq!(mqtt::packet::PacketType::Pubrel.as_str(), "pubrel");
    assert_eq!(mqtt::packet::PacketType::Pubcomp.as_str(), "pubcomp");
    assert_eq!(mqtt::packet::PacketType::Subscribe.as_str(), "subscribe");
    assert_eq!(mqtt::packet::PacketType::Suback.as_str(), "suback");
    assert_eq!(
        mqtt::packet::PacketType::Unsubscribe.as_str(),
        "unsubscribe"
    );
    assert_eq!(mqtt::packet::PacketType::Unsuback.as_str(), "unsuback");
    assert_eq!(mqtt::packet::PacketType::Pingreq.as_str(), "pingreq");
    assert_eq!(mqtt::packet::PacketType::Pingresp.as_str(), "pingresp");
    assert_eq!(mqtt::packet::PacketType::Disconnect.as_str(), "disconnect");
    assert_eq!(mqtt::packet::PacketType::Auth.as_str(), "auth");
}

#[test]
fn packet_type_to_fixed_header() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::PacketType::Connect.to_fixed_header(),
        mqtt::packet::FixedHeader::Connect
    );
    assert_eq!(
        mqtt::packet::PacketType::Connack.to_fixed_header(),
        mqtt::packet::FixedHeader::Connack
    );
    assert_eq!(
        mqtt::packet::PacketType::Publish.to_fixed_header(),
        mqtt::packet::FixedHeader::Publish
    );
    assert_eq!(
        mqtt::packet::PacketType::Puback.to_fixed_header(),
        mqtt::packet::FixedHeader::Puback
    );
    assert_eq!(
        mqtt::packet::PacketType::Pubrec.to_fixed_header(),
        mqtt::packet::FixedHeader::Pubrec
    );
    assert_eq!(
        mqtt::packet::PacketType::Pubrel.to_fixed_header(),
        mqtt::packet::FixedHeader::Pubrel
    );
    assert_eq!(
        mqtt::packet::PacketType::Pubcomp.to_fixed_header(),
        mqtt::packet::FixedHeader::Pubcomp
    );
    assert_eq!(
        mqtt::packet::PacketType::Subscribe.to_fixed_header(),
        mqtt::packet::FixedHeader::Subscribe
    );
    assert_eq!(
        mqtt::packet::PacketType::Suback.to_fixed_header(),
        mqtt::packet::FixedHeader::Suback
    );
    assert_eq!(
        mqtt::packet::PacketType::Unsubscribe.to_fixed_header(),
        mqtt::packet::FixedHeader::Unsubscribe
    );
    assert_eq!(
        mqtt::packet::PacketType::Unsuback.to_fixed_header(),
        mqtt::packet::FixedHeader::Unsuback
    );
    assert_eq!(
        mqtt::packet::PacketType::Pingreq.to_fixed_header(),
        mqtt::packet::FixedHeader::Pingreq
    );
    assert_eq!(
        mqtt::packet::PacketType::Pingresp.to_fixed_header(),
        mqtt::packet::FixedHeader::Pingresp
    );
    assert_eq!(
        mqtt::packet::PacketType::Disconnect.to_fixed_header(),
        mqtt::packet::FixedHeader::Disconnect
    );
    assert_eq!(
        mqtt::packet::PacketType::Auth.to_fixed_header(),
        mqtt::packet::FixedHeader::Auth
    );
}

// FixedHeader basic functionality tests

#[test]
fn fixed_header_as_u8() {
    common::init_tracing();
    assert_eq!(mqtt::packet::FixedHeader::Connect.as_u8(), 0x10);
    assert_eq!(mqtt::packet::FixedHeader::Connack.as_u8(), 0x20);
    assert_eq!(mqtt::packet::FixedHeader::Publish.as_u8(), 0x30);
    assert_eq!(mqtt::packet::FixedHeader::Puback.as_u8(), 0x40);
    assert_eq!(mqtt::packet::FixedHeader::Pubrec.as_u8(), 0x50);
    assert_eq!(mqtt::packet::FixedHeader::Pubrel.as_u8(), 0x62);
    assert_eq!(mqtt::packet::FixedHeader::Pubcomp.as_u8(), 0x70);
    assert_eq!(mqtt::packet::FixedHeader::Subscribe.as_u8(), 0x82);
    assert_eq!(mqtt::packet::FixedHeader::Suback.as_u8(), 0x90);
    assert_eq!(mqtt::packet::FixedHeader::Unsubscribe.as_u8(), 0xa2);
    assert_eq!(mqtt::packet::FixedHeader::Unsuback.as_u8(), 0xb0);
    assert_eq!(mqtt::packet::FixedHeader::Pingreq.as_u8(), 0xc0);
    assert_eq!(mqtt::packet::FixedHeader::Pingresp.as_u8(), 0xd0);
    assert_eq!(mqtt::packet::FixedHeader::Disconnect.as_u8(), 0xe0);
    assert_eq!(mqtt::packet::FixedHeader::Auth.as_u8(), 0xf0);
}

#[test]
fn fixed_header_packet_type() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::FixedHeader::Connect.packet_type(),
        mqtt::packet::PacketType::Connect
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Connack.packet_type(),
        mqtt::packet::PacketType::Connack
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Publish.packet_type(),
        mqtt::packet::PacketType::Publish
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Puback.packet_type(),
        mqtt::packet::PacketType::Puback
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Pubrec.packet_type(),
        mqtt::packet::PacketType::Pubrec
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Pubrel.packet_type(),
        mqtt::packet::PacketType::Pubrel
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Pubcomp.packet_type(),
        mqtt::packet::PacketType::Pubcomp
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Subscribe.packet_type(),
        mqtt::packet::PacketType::Subscribe
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Suback.packet_type(),
        mqtt::packet::PacketType::Suback
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Unsubscribe.packet_type(),
        mqtt::packet::PacketType::Unsubscribe
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Unsuback.packet_type(),
        mqtt::packet::PacketType::Unsuback
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Pingreq.packet_type(),
        mqtt::packet::PacketType::Pingreq
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Pingresp.packet_type(),
        mqtt::packet::PacketType::Pingresp
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Disconnect.packet_type(),
        mqtt::packet::PacketType::Disconnect
    );
    assert_eq!(
        mqtt::packet::FixedHeader::Auth.packet_type(),
        mqtt::packet::PacketType::Auth
    );
}

#[test]
fn fixed_header_as_str() {
    common::init_tracing();
    assert_eq!(mqtt::packet::FixedHeader::Connect.as_str(), "connect");
    assert_eq!(mqtt::packet::FixedHeader::Connack.as_str(), "connack");
    assert_eq!(mqtt::packet::FixedHeader::Publish.as_str(), "publish");
    assert_eq!(mqtt::packet::FixedHeader::Puback.as_str(), "puback");
    assert_eq!(mqtt::packet::FixedHeader::Pubrec.as_str(), "pubrec");
    assert_eq!(mqtt::packet::FixedHeader::Pubrel.as_str(), "pubrel");
    assert_eq!(mqtt::packet::FixedHeader::Pubcomp.as_str(), "pubcomp");
    assert_eq!(mqtt::packet::FixedHeader::Subscribe.as_str(), "subscribe");
    assert_eq!(mqtt::packet::FixedHeader::Suback.as_str(), "suback");
    assert_eq!(
        mqtt::packet::FixedHeader::Unsubscribe.as_str(),
        "unsubscribe"
    );
    assert_eq!(mqtt::packet::FixedHeader::Unsuback.as_str(), "unsuback");
    assert_eq!(mqtt::packet::FixedHeader::Pingreq.as_str(), "pingreq");
    assert_eq!(mqtt::packet::FixedHeader::Pingresp.as_str(), "pingresp");
    assert_eq!(mqtt::packet::FixedHeader::Disconnect.as_str(), "disconnect");
    assert_eq!(mqtt::packet::FixedHeader::Auth.as_str(), "auth");
}

// TryFromPrimitive tests

#[test]
fn packet_type_try_from_primitive_valid() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::PacketType::try_from(1u8).unwrap(),
        mqtt::packet::PacketType::Connect
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(2u8).unwrap(),
        mqtt::packet::PacketType::Connack
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(3u8).unwrap(),
        mqtt::packet::PacketType::Publish
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(4u8).unwrap(),
        mqtt::packet::PacketType::Puback
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(5u8).unwrap(),
        mqtt::packet::PacketType::Pubrec
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(6u8).unwrap(),
        mqtt::packet::PacketType::Pubrel
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(7u8).unwrap(),
        mqtt::packet::PacketType::Pubcomp
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(8u8).unwrap(),
        mqtt::packet::PacketType::Subscribe
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(9u8).unwrap(),
        mqtt::packet::PacketType::Suback
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(10u8).unwrap(),
        mqtt::packet::PacketType::Unsubscribe
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(11u8).unwrap(),
        mqtt::packet::PacketType::Unsuback
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(12u8).unwrap(),
        mqtt::packet::PacketType::Pingreq
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(13u8).unwrap(),
        mqtt::packet::PacketType::Pingresp
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(14u8).unwrap(),
        mqtt::packet::PacketType::Disconnect
    );
    assert_eq!(
        mqtt::packet::PacketType::try_from(15u8).unwrap(),
        mqtt::packet::PacketType::Auth
    );
}

#[test]
fn packet_type_try_from_primitive_invalid() {
    common::init_tracing();
    assert!(mqtt::packet::PacketType::try_from(0u8).is_err());
    assert!(mqtt::packet::PacketType::try_from(16u8).is_err());
    assert!(mqtt::packet::PacketType::try_from(255u8).is_err());
}

#[test]
fn fixed_header_try_from_primitive_valid() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x10u8).unwrap(),
        mqtt::packet::FixedHeader::Connect
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x20u8).unwrap(),
        mqtt::packet::FixedHeader::Connack
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x30u8).unwrap(),
        mqtt::packet::FixedHeader::Publish
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x40u8).unwrap(),
        mqtt::packet::FixedHeader::Puback
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x50u8).unwrap(),
        mqtt::packet::FixedHeader::Pubrec
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x62u8).unwrap(),
        mqtt::packet::FixedHeader::Pubrel
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x70u8).unwrap(),
        mqtt::packet::FixedHeader::Pubcomp
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x82u8).unwrap(),
        mqtt::packet::FixedHeader::Subscribe
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0x90u8).unwrap(),
        mqtt::packet::FixedHeader::Suback
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0xa2u8).unwrap(),
        mqtt::packet::FixedHeader::Unsubscribe
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0xb0u8).unwrap(),
        mqtt::packet::FixedHeader::Unsuback
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0xc0u8).unwrap(),
        mqtt::packet::FixedHeader::Pingreq
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0xd0u8).unwrap(),
        mqtt::packet::FixedHeader::Pingresp
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0xe0u8).unwrap(),
        mqtt::packet::FixedHeader::Disconnect
    );
    assert_eq!(
        mqtt::packet::FixedHeader::try_from(0xf0u8).unwrap(),
        mqtt::packet::FixedHeader::Auth
    );
}

#[test]
fn fixed_header_try_from_primitive_invalid() {
    common::init_tracing();
    assert!(mqtt::packet::FixedHeader::try_from(0x00u8).is_err());
    assert!(mqtt::packet::FixedHeader::try_from(0x11u8).is_err());
    assert!(mqtt::packet::FixedHeader::try_from(0x21u8).is_err());
    assert!(mqtt::packet::FixedHeader::try_from(0x31u8).is_err());
}

// Display and Debug tests

#[test]
fn packet_type_display() {
    common::init_tracing();
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Connect),
        "\"connect\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Connack),
        "\"connack\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Publish),
        "\"publish\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Puback),
        "\"puback\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Pubrec),
        "\"pubrec\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Pubrel),
        "\"pubrel\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Pubcomp),
        "\"pubcomp\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Subscribe),
        "\"subscribe\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Suback),
        "\"suback\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Unsubscribe),
        "\"unsubscribe\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Unsuback),
        "\"unsuback\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Pingreq),
        "\"pingreq\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Pingresp),
        "\"pingresp\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::PacketType::Disconnect),
        "\"disconnect\""
    );
    assert_eq!(format!("{}", mqtt::packet::PacketType::Auth), "\"auth\"");
}

#[test]
fn packet_type_debug() {
    common::init_tracing();
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Connect),
        "\"connect\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Connack),
        "\"connack\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Publish),
        "\"publish\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Puback),
        "\"puback\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Pubrec),
        "\"pubrec\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Pubrel),
        "\"pubrel\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Pubcomp),
        "\"pubcomp\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Subscribe),
        "\"subscribe\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Suback),
        "\"suback\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Unsubscribe),
        "\"unsubscribe\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Unsuback),
        "\"unsuback\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Pingreq),
        "\"pingreq\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Pingresp),
        "\"pingresp\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::PacketType::Disconnect),
        "\"disconnect\""
    );
    assert_eq!(format!("{:?}", mqtt::packet::PacketType::Auth), "\"auth\"");
}

#[test]
fn fixed_header_display() {
    common::init_tracing();
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Connect),
        "\"connect\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Connack),
        "\"connack\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Publish),
        "\"publish\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Puback),
        "\"puback\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Pubrec),
        "\"pubrec\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Pubrel),
        "\"pubrel\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Pubcomp),
        "\"pubcomp\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Subscribe),
        "\"subscribe\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Suback),
        "\"suback\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Unsubscribe),
        "\"unsubscribe\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Unsuback),
        "\"unsuback\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Pingreq),
        "\"pingreq\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Pingresp),
        "\"pingresp\""
    );
    assert_eq!(
        format!("{}", mqtt::packet::FixedHeader::Disconnect),
        "\"disconnect\""
    );
    assert_eq!(format!("{}", mqtt::packet::FixedHeader::Auth), "\"auth\"");
}

#[test]
fn fixed_header_debug() {
    common::init_tracing();
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Connect),
        "\"connect\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Connack),
        "\"connack\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Publish),
        "\"publish\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Puback),
        "\"puback\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Pubrec),
        "\"pubrec\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Pubrel),
        "\"pubrel\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Pubcomp),
        "\"pubcomp\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Subscribe),
        "\"subscribe\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Suback),
        "\"suback\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Unsubscribe),
        "\"unsubscribe\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Unsuback),
        "\"unsuback\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Pingreq),
        "\"pingreq\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Pingresp),
        "\"pingresp\""
    );
    assert_eq!(
        format!("{:?}", mqtt::packet::FixedHeader::Disconnect),
        "\"disconnect\""
    );
    assert_eq!(format!("{:?}", mqtt::packet::FixedHeader::Auth), "\"auth\"");
}

// Serialization tests

#[test]
fn packet_type_serialize() {
    common::init_tracing();
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Connect).unwrap(),
        "\"connect\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Connack).unwrap(),
        "\"connack\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Publish).unwrap(),
        "\"publish\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Puback).unwrap(),
        "\"puback\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Pubrec).unwrap(),
        "\"pubrec\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Pubrel).unwrap(),
        "\"pubrel\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Pubcomp).unwrap(),
        "\"pubcomp\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Subscribe).unwrap(),
        "\"subscribe\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Suback).unwrap(),
        "\"suback\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Unsubscribe).unwrap(),
        "\"unsubscribe\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Unsuback).unwrap(),
        "\"unsuback\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Pingreq).unwrap(),
        "\"pingreq\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Pingresp).unwrap(),
        "\"pingresp\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Disconnect).unwrap(),
        "\"disconnect\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::PacketType::Auth).unwrap(),
        "\"auth\""
    );
}

#[test]
fn fixed_header_serialize() {
    common::init_tracing();
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Connect).unwrap(),
        "\"connect\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Connack).unwrap(),
        "\"connack\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Publish).unwrap(),
        "\"publish\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Puback).unwrap(),
        "\"puback\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Pubrec).unwrap(),
        "\"pubrec\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Pubrel).unwrap(),
        "\"pubrel\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Pubcomp).unwrap(),
        "\"pubcomp\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Subscribe).unwrap(),
        "\"subscribe\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Suback).unwrap(),
        "\"suback\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Unsubscribe).unwrap(),
        "\"unsubscribe\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Unsuback).unwrap(),
        "\"unsuback\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Pingreq).unwrap(),
        "\"pingreq\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Pingresp).unwrap(),
        "\"pingresp\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Disconnect).unwrap(),
        "\"disconnect\""
    );
    assert_eq!(
        serde_json::to_string(&mqtt::packet::FixedHeader::Auth).unwrap(),
        "\"auth\""
    );
}

#[test]
fn packet_type_deserialize() {
    common::init_tracing();
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Connect\"").unwrap(),
        mqtt::packet::PacketType::Connect
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Connack\"").unwrap(),
        mqtt::packet::PacketType::Connack
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Publish\"").unwrap(),
        mqtt::packet::PacketType::Publish
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Puback\"").unwrap(),
        mqtt::packet::PacketType::Puback
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Pubrec\"").unwrap(),
        mqtt::packet::PacketType::Pubrec
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Pubrel\"").unwrap(),
        mqtt::packet::PacketType::Pubrel
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Pubcomp\"").unwrap(),
        mqtt::packet::PacketType::Pubcomp
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Subscribe\"").unwrap(),
        mqtt::packet::PacketType::Subscribe
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Suback\"").unwrap(),
        mqtt::packet::PacketType::Suback
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Unsubscribe\"").unwrap(),
        mqtt::packet::PacketType::Unsubscribe
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Unsuback\"").unwrap(),
        mqtt::packet::PacketType::Unsuback
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Pingreq\"").unwrap(),
        mqtt::packet::PacketType::Pingreq
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Pingresp\"").unwrap(),
        mqtt::packet::PacketType::Pingresp
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Disconnect\"").unwrap(),
        mqtt::packet::PacketType::Disconnect
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::PacketType>("\"Auth\"").unwrap(),
        mqtt::packet::PacketType::Auth
    );
}

#[test]
fn fixed_header_deserialize() {
    common::init_tracing();
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Connect\"").unwrap(),
        mqtt::packet::FixedHeader::Connect
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Connack\"").unwrap(),
        mqtt::packet::FixedHeader::Connack
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Publish\"").unwrap(),
        mqtt::packet::FixedHeader::Publish
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Puback\"").unwrap(),
        mqtt::packet::FixedHeader::Puback
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Pubrec\"").unwrap(),
        mqtt::packet::FixedHeader::Pubrec
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Pubrel\"").unwrap(),
        mqtt::packet::FixedHeader::Pubrel
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Pubcomp\"").unwrap(),
        mqtt::packet::FixedHeader::Pubcomp
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Subscribe\"").unwrap(),
        mqtt::packet::FixedHeader::Subscribe
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Suback\"").unwrap(),
        mqtt::packet::FixedHeader::Suback
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Unsubscribe\"").unwrap(),
        mqtt::packet::FixedHeader::Unsubscribe
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Unsuback\"").unwrap(),
        mqtt::packet::FixedHeader::Unsuback
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Pingreq\"").unwrap(),
        mqtt::packet::FixedHeader::Pingreq
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Pingresp\"").unwrap(),
        mqtt::packet::FixedHeader::Pingresp
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Disconnect\"").unwrap(),
        mqtt::packet::FixedHeader::Disconnect
    );
    assert_eq!(
        serde_json::from_str::<mqtt::packet::FixedHeader>("\"Auth\"").unwrap(),
        mqtt::packet::FixedHeader::Auth
    );
}

// PartialEq, Eq, Copy, Clone tests

#[test]
fn packet_type_equality() {
    common::init_tracing();
    let connect1 = mqtt::packet::PacketType::Connect;
    let connect2 = mqtt::packet::PacketType::Connect;
    let connack = mqtt::packet::PacketType::Connack;

    assert_eq!(connect1, connect2);
    assert_ne!(connect1, connack);
}

#[test]
fn fixed_header_equality() {
    common::init_tracing();
    let connect1 = mqtt::packet::FixedHeader::Connect;
    let connect2 = mqtt::packet::FixedHeader::Connect;
    let connack = mqtt::packet::FixedHeader::Connack;

    assert_eq!(connect1, connect2);
    assert_ne!(connect1, connack);
}

// Edge case tests

#[test]
fn fixed_header_packet_type_extraction_edge_cases() {
    common::init_tracing();
    // Test with type bits only (no additional flags)
    let type_bits = 0x10u8; // Connect
    let header = mqtt::packet::FixedHeader::try_from(type_bits).unwrap();
    assert_eq!(header.packet_type(), mqtt::packet::PacketType::Connect);

    // Test packet_type method's fallback behavior with invalid type
    // This should be tested with manual implementation since try_from would fail
    // The fallback to Connect happens inside packet_type() method
}

#[test]
fn round_trip_conversion() {
    common::init_tracing();
    // Test PacketType -> FixedHeader -> PacketType
    for &packet_type in &[
        mqtt::packet::PacketType::Connect,
        mqtt::packet::PacketType::Connack,
        mqtt::packet::PacketType::Publish,
        mqtt::packet::PacketType::Puback,
        mqtt::packet::PacketType::Pubrec,
        mqtt::packet::PacketType::Pubrel,
        mqtt::packet::PacketType::Pubcomp,
        mqtt::packet::PacketType::Subscribe,
        mqtt::packet::PacketType::Suback,
        mqtt::packet::PacketType::Unsubscribe,
        mqtt::packet::PacketType::Unsuback,
        mqtt::packet::PacketType::Pingreq,
        mqtt::packet::PacketType::Pingresp,
        mqtt::packet::PacketType::Disconnect,
        mqtt::packet::PacketType::Auth,
    ] {
        let fixed_header = packet_type.to_fixed_header();
        let recovered_type = fixed_header.packet_type();
        assert_eq!(packet_type, recovered_type);
    }
}
