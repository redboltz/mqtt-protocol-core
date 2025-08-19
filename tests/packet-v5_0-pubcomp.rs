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
use std::fmt::Write;

// Build fail tests

#[test]
fn build_fail_nopid() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Pubcomp::builder().build().unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_pid0() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(0)
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_props_without_rc() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_prop() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .props(vec![mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_mt() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .props(vec![
            mqtt::packet::ReasonString::new("test1").unwrap().into(),
            mqtt::packet::ReasonString::new("test2").unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Display tests

#[test]
fn display_pid() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(output, r#"{"type":"pubcomp","packet_id":1234}"#);
}

#[test]
fn display_pid_rc() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"pubcomp","packet_id":1234,"reason_code":"Success"}"#
    );
}

// Debug tests

#[test]
fn debug_pid() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(output, r#"{"type":"pubcomp","packet_id":1234}"#);
}

#[test]
fn debug_pid_rc() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"pubcomp","packet_id":1234,"reason_code":"Success"}"#
    );
}

#[test]
fn debug_pid_rc_prop0() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"pubcomp","packet_id":1234,"reason_code":"Success","props":[]}"#
    );
}

// Getter tests

#[test]
fn getter_pid() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_none());
    assert!(packet.props().is_none());
}

#[test]
fn getter_pid_rc() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::PubcompReasonCode::Success
    );
    assert!(packet.props().is_none());
}

#[test]
fn getter_pid_rc_prop0() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::PubcompReasonCode::Success
    );
    assert!(packet.props().is_some());
    assert!(packet.props().as_ref().unwrap().is_empty());
}

#[test]
fn getter_pid_rc_prop_mt() {
    common::init_tracing();
    let props = vec![
        mqtt::packet::UserProperty::new("key1", "value1")
            .unwrap()
            .into(),
        mqtt::packet::UserProperty::new("key2", "value2")
            .unwrap()
            .into(),
    ];
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .props(props.clone())
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::PubcompReasonCode::Success
    );
    assert!(packet.props().as_ref().unwrap() == &props);
}

// to_buffers() tests

#[test]
fn to_buffers_pid() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous.len(), 4);
    assert_eq!(continuous[0], 0x70); // fixed header
    assert_eq!(continuous[1], 0x02); // remaining length
    assert_eq!(&continuous[2..4], &1234u16.to_be_bytes()); // packet_id

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }
    assert_eq!(packet.size(), 4); // 1 + 1 + 2
    assert_eq!(packet.size(), continuous.len());
}

#[test]
fn to_buffers_pid_rc() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous.len(), 5);
    assert_eq!(continuous[0], 0x70); // fixed header
    assert_eq!(continuous[1], 0x03); // remaining length
    assert_eq!(&continuous[2..4], &1234u16.to_be_bytes()); // packet_id
    assert_eq!(continuous[4], 0x00); // reason code

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }
    assert_eq!(packet.size(), 5); // 1 + 1 + 2 + 1
    assert_eq!(packet.size(), continuous.len());
}

#[test]
fn to_buffers_pid_rc_prop0() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::Success)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();
    assert_eq!(continuous.len(), 6);
    assert_eq!(continuous[0], 0x70); // fixed header
    assert_eq!(continuous[1], 0x04); // remaining length
    assert_eq!(&continuous[2..4], &1234u16.to_be_bytes()); // packet_id
    assert_eq!(continuous[4], 0x00); // reason code
    assert_eq!(continuous[5], 0x00); // property length

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }
    assert_eq!(packet.size(), 6); // 1 + 1 + 2 + 1 + 1
    assert_eq!(packet.size(), continuous.len());
}

// Parse tests

#[test]
fn parse_empty() {
    common::init_tracing();
    let raw = &[];
    let err = mqtt::packet::v5_0::Pubcomp::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid_incomplete() {
    common::init_tracing();
    let raw = &[0x00];
    let err = mqtt::packet::v5_0::Pubcomp::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid0() {
    common::init_tracing();
    let raw = &[0x00, 0x00];
    let err = mqtt::packet::v5_0::Pubcomp::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid() {
    common::init_tracing();
    let raw = &1234u16.to_be_bytes();
    let (packet, consumed) = mqtt::packet::v5_0::Pubcomp::parse(raw).unwrap();
    assert_eq!(consumed, 2);
    let expected = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_rc() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    let (packet, consumed) = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());
    let expected = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound)
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_bad_rc() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0xA2); // reason code: WildcardSubscriptionsNotSupported (0xA2)
    let err = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid_rc_prop0() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    raw.push(0x00); // property length 0
    let (packet, consumed) = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());
    let expected = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_rc_proplen_over() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    raw.push(0x01); // property length 0
    let err = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid_rc_prop_reason_string() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    raw.push(0x07); // property length
    raw.push(0x1F); // prperty ID: Reason String (0x1F)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");

    let (packet, consumed) = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound)
        .props(vec![mqtt::packet::ReasonString::new("test")
            .unwrap()
            .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_rc_prop_reason_string_twice() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    raw.push(0x0E); // property length
    raw.push(0x1F); // prperty ID: Reason String (0x1F)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");
    raw.push(0x1F); // prperty ID: Reason String (0x1F)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_pid_rc_prop_unmatched() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    raw.push(0x07); // property length
    raw.push(0x03); // prperty ID: Content Type (0x03)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_pid_rc_prop_user_property_twice() {
    common::init_tracing();
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    raw.push(0x1E); // property length

    // First User Property
    raw.push(0x26); // property ID: User Property (0x26)
    raw.push(0x00); // name string length MSB
    raw.push(0x04); // name string length LSB (4)
    raw.extend_from_slice(b"key1");
    raw.push(0x00); // value string length MSB
    raw.push(0x06); // value string length LSB (6)
    raw.extend_from_slice(b"value1");

    // Second User Property
    raw.push(0x26); // property ID: User Property (0x26)
    raw.push(0x00); // name string length MSB
    raw.push(0x04); // name string length LSB (4)
    raw.extend_from_slice(b"key2");
    raw.push(0x00); // value string length MSB
    raw.push(0x06); // value string length LSB (6)
    raw.extend_from_slice(b"value2");

    let (packet, consumed) = mqtt::packet::v5_0::Pubcomp::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Pubcomp::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound)
        .props(vec![
            mqtt::packet::UserProperty::new("key1", "value1")
                .unwrap()
                .into(),
            mqtt::packet::UserProperty::new("key2", "value2")
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pidu32_rc_prop_user_property_twice() {
    common::init_tracing();
    let mut raw = Vec::from(1234u32.to_be_bytes());
    raw.push(0x92); // reason code: PacketIdentifierNotFound
    raw.push(0x1E); // property length

    // First User Property
    raw.push(0x26); // property ID: User Property (0x26)
    raw.push(0x00); // name string length MSB
    raw.push(0x04); // name string length LSB (4)
    raw.extend_from_slice(b"key1");
    raw.push(0x00); // value string length MSB
    raw.push(0x06); // value string length LSB (6)
    raw.extend_from_slice(b"value1");

    // Second User Property
    raw.push(0x26); // property ID: User Property (0x26)
    raw.push(0x00); // name string length MSB
    raw.push(0x04); // name string length LSB (4)
    raw.extend_from_slice(b"key2");
    raw.push(0x00); // value string length MSB
    raw.push(0x06); // value string length LSB (6)
    raw.extend_from_slice(b"value2");

    let (packet, consumed) = mqtt::packet::v5_0::GenericPubcomp::<u32>::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::GenericPubcomp::<u32>::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubcompReasonCode::PacketIdentifierNotFound)
        .props(vec![
            mqtt::packet::UserProperty::new("key1", "value1")
                .unwrap()
                .into(),
            mqtt::packet::UserProperty::new("key2", "value2")
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn test_packet_type() {
    common::init_tracing();
    let packet_type = mqtt::packet::v5_0::Pubcomp::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Pubcomp);
}
