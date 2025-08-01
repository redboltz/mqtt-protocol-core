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
use std::fmt::Write;

// Build fail tests

#[test]
fn build_fail_nopid() {
    let err = mqtt::packet::v5_0::Pubrec::builder().build().unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_pid0() {
    let err = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(0)
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_props_without_rc() {
    let err = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_prop() {
    let err = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .props(vec![mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_mt() {
    let err = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
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
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(output, r#"{"type":"pubrec","packet_id":1234}"#);
}

#[test]
fn display_pid_rc() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"pubrec","packet_id":1234,"reason_code":"Success"}"#
    );
}

// Debug tests

#[test]
fn debug_pid() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(output, r#"{"type":"pubrec","packet_id":1234}"#);
}

#[test]
fn debug_pid_rc() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"pubrec","packet_id":1234,"reason_code":"Success"}"#
    );
}

#[test]
fn debug_pid_rc_prop0() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"pubrec","packet_id":1234,"reason_code":"Success","props":[]}"#
    );
}

// Getter tests

#[test]
fn getter_pid() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_none());
    assert!(packet.props().is_none());
}

#[test]
fn getter_pid_rc() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::PubrecReasonCode::Success
    );
    assert!(packet.props().is_none());
}

#[test]
fn getter_pid_rc_prop0() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::PubrecReasonCode::Success
    );
    assert!(packet.props().is_some());
    assert!(packet.props().as_ref().unwrap().is_empty());
}

#[test]
fn getter_pid_rc_prop_mt() {
    let props = vec![
        mqtt::packet::UserProperty::new("key1", "value1")
            .unwrap()
            .into(),
        mqtt::packet::UserProperty::new("key2", "value2")
            .unwrap()
            .into(),
    ];
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .props(props.clone())
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 1234);
    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::PubrecReasonCode::Success
    );
    assert!(packet.props().as_ref().unwrap() == &props);
}

// to_buffers() tests

#[test]
fn to_buffers_pid() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 3);
    assert_eq!(buffers[0].as_ref(), &[0x50]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x02]); // remaining length
    assert_eq!(buffers[2].as_ref(), &1234u16.to_be_bytes()); // packet_id
    assert_eq!(packet.size(), 4); // 1 + 1 + 2
}

#[test]
fn to_buffers_pid_rc() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 4);
    assert_eq!(buffers[0].as_ref(), &[0x50]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x03]); // remaining length
    assert_eq!(buffers[2].as_ref(), &1234u16.to_be_bytes()); // packet_id
    assert_eq!(buffers[3].as_ref(), &[0x00]); // reason code
    assert_eq!(packet.size(), 5); // 1 + 1 + 2 + 1
}

#[test]
fn to_buffers_pid_rc_prop0() {
    let packet = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 5);
    assert_eq!(buffers[0].as_ref(), &[0x50]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x04]); // remaining length
    assert_eq!(buffers[2].as_ref(), &1234u16.to_be_bytes()); // packet_id
    assert_eq!(buffers[3].as_ref(), &[0x00]); // reason code
    assert_eq!(buffers[4].as_ref(), &[0x00]); // property length
    assert_eq!(packet.size(), 6); // 1 + 1 + 2 + 1 + 1
}

// Parse tests

#[test]
fn parse_empty() {
    let raw = &[];
    let err = mqtt::packet::v5_0::Pubrec::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid_incomplete() {
    let raw = &[0x00];
    let err = mqtt::packet::v5_0::Pubrec::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid0() {
    let raw = &[0x00, 0x00];
    let err = mqtt::packet::v5_0::Pubrec::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid() {
    let raw = &1234u16.to_be_bytes();
    let (packet, consumed) = mqtt::packet::v5_0::Pubrec::parse(raw).unwrap();
    assert_eq!(consumed, 2);
    let expected = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_rc() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
    let (packet, consumed) = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());
    let expected = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::UnspecifiedError)
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_bad_rc() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0xA2); // reason code: WildcardSubscriptionsNotSupported (0xA2)
    let err = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid_rc_prop0() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
    raw.push(0x00); // property length 0
    let (packet, consumed) = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());
    let expected = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::UnspecifiedError)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_rc_proplen_over() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
    raw.push(0x01); // property length 0
    let err = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_pid_rc_prop_reason_string() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
    raw.push(0x07); // property length
    raw.push(0x1F); // prperty ID: Reason String (0x1F)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");

    let (packet, consumed) = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::UnspecifiedError)
        .props(vec![mqtt::packet::ReasonString::new("test")
            .unwrap()
            .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_pid_rc_prop_reason_string_twice() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
    raw.push(0x0E); // property length
    raw.push(0x1F); // prperty ID: Reason String (0x1F)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");
    raw.push(0x1F); // prperty ID: Reason String (0x1F)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_pid_rc_prop_unmatched() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
    raw.push(0x07); // property length
    raw.push(0x03); // prperty ID: Content Type (0x03)
    raw.push(0x00); // string length
    raw.push(0x04); // string length
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_pid_rc_prop_user_property_twice() {
    let mut raw = Vec::from(1234u16.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
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

    let (packet, consumed) = mqtt::packet::v5_0::Pubrec::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::UnspecifiedError)
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
    let mut raw = Vec::from(1234u32.to_be_bytes());
    raw.push(0x80); // reason code: UnspecifiedError
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

    let (packet, consumed) = mqtt::packet::v5_0::GenericPubrec::<u32>::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::GenericPubrec::<u32>::builder()
        .packet_id(1234)
        .reason_code(mqtt::result_code::PubrecReasonCode::UnspecifiedError)
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
