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
fn build_fail_props_without_rc() {
    let err = mqtt::packet::v5_0::Disconnect::builder()
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_prop() {
    let err = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(vec![mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_session_expiry_twice() {
    let err = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(vec![
            mqtt::packet::SessionExpiryInterval::new(30).unwrap().into(),
            mqtt::packet::SessionExpiryInterval::new(60).unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_reason_string_twice() {
    let err = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(vec![
            mqtt::packet::ReasonString::new("test1").unwrap().into(),
            mqtt::packet::ReasonString::new("test2").unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_server_reference_twice() {
    let err = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(vec![
            mqtt::packet::ServerReference::new("server1")
                .unwrap()
                .into(),
            mqtt::packet::ServerReference::new("server2")
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Display tests

#[test]
fn display_empty() {
    let packet = mqtt::packet::v5_0::Disconnect::builder().build().unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(output, r#"{"type":"disconnect"}"#);
}

#[test]
fn display_rc() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"disconnect","reason_code":"NormalDisconnection"}"#
    );
}

#[test]
fn display_rc_props() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::ServerShuttingDown)
        .props(vec![mqtt::packet::ReasonString::new("Server maintenance")
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"disconnect","reason_code":"ServerShuttingDown","props":[{"ReasonString":{"id":31,"val":"Server maintenance"}}]}"#
    );
}

// Debug tests

#[test]
fn debug_empty() {
    let packet = mqtt::packet::v5_0::Disconnect::builder().build().unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(output, r#"{"type":"disconnect"}"#);
}

#[test]
fn debug_rc() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"disconnect","reason_code":"NormalDisconnection"}"#
    );
}

#[test]
fn debug_rc_prop0() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"disconnect","reason_code":"NormalDisconnection","props":[]}"#
    );
}

// Getter tests

#[test]
fn getter_empty() {
    let packet = mqtt::packet::v5_0::Disconnect::builder().build().unwrap();

    assert!(packet.reason_code().is_none());
    assert!(packet.props().is_none());
}

#[test]
fn getter_rc() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .build()
        .unwrap();

    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::DisconnectReasonCode::NormalDisconnection
    );
    assert!(packet.props().is_none());
}

#[test]
fn getter_rc_prop0() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::DisconnectReasonCode::NormalDisconnection
    );
    assert!(packet.props().is_some());
    assert!(packet.props().as_ref().unwrap().is_empty());
}

#[test]
fn getter_rc_props_session_expiry() {
    let props = vec![mqtt::packet::SessionExpiryInterval::new(300)
        .unwrap()
        .into()];
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(props.clone())
        .build()
        .unwrap();

    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::DisconnectReasonCode::NormalDisconnection
    );
    assert!(packet.props().as_ref().unwrap() == &props);
}

#[test]
fn getter_rc_props_mixed() {
    let props = vec![
        mqtt::packet::SessionExpiryInterval::new(300)
            .unwrap()
            .into(),
        mqtt::packet::ReasonString::new("Client disconnecting")
            .unwrap()
            .into(),
        mqtt::packet::UserProperty::new("client", "test_client")
            .unwrap()
            .into(),
        mqtt::packet::ServerReference::new("backup.example.com")
            .unwrap()
            .into(),
    ];
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(props.clone())
        .build()
        .unwrap();

    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::DisconnectReasonCode::NormalDisconnection
    );
    assert!(packet.props().as_ref().unwrap() == &props);
}

// to_buffers() tests

#[test]
fn to_buffers_empty() {
    let packet = mqtt::packet::v5_0::Disconnect::builder().build().unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 2);
    assert_eq!(buffers[0].as_ref(), &[0xe0]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x00]); // remaining length
    assert_eq!(packet.size(), 2); // 1 + 1
}

#[test]
fn to_buffers_rc() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 3);
    assert_eq!(buffers[0].as_ref(), &[0xe0]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x01]); // remaining length
    assert_eq!(buffers[2].as_ref(), &[0x00]); // reason code
    assert_eq!(packet.size(), 3); // 1 + 1 + 1
}

#[test]
fn to_buffers_rc_prop0() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 4);
    assert_eq!(buffers[0].as_ref(), &[0xe0]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x02]); // remaining length
    assert_eq!(buffers[2].as_ref(), &[0x00]); // reason code
    assert_eq!(buffers[3].as_ref(), &[0x00]); // property length
    assert_eq!(packet.size(), 4); // 1 + 1 + 1 + 1
}

#[test]
fn to_buffers_rc_props_session_expiry() {
    let packet = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(vec![mqtt::packet::SessionExpiryInterval::new(300)
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 6);
    assert_eq!(buffers[0].as_ref(), &[0xe0]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x07]); // remaining length (1 + 1 + 5)
    assert_eq!(buffers[2].as_ref(), &[0x00]); // reason code
    assert_eq!(buffers[3].as_ref(), &[0x05]); // property length
    assert_eq!(buffers[4].as_ref(), &[0x11]); // session expiry interval property ID
    assert_eq!(buffers[5].as_ref(), &[0x00, 0x00, 0x01, 0x2c]); // session expiry interval value (300)
    assert_eq!(packet.size(), 9); // 1 + 1 + 1 + 1 + 5
}

// Parse tests

#[test]
fn parse_empty() {
    let raw = &[];
    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(raw).unwrap();
    assert_eq!(consumed, 0);
    let expected = mqtt::packet::v5_0::Disconnect::builder().build().unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc() {
    let raw = &[0x00]; // reason code: NormalDisconnection
    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(raw).unwrap();
    assert_eq!(consumed, 1);
    let expected = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_server_busy() {
    let raw = &[0x89]; // reason code: ServerBusy
    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(raw).unwrap();
    assert_eq!(consumed, 1);
    let expected = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::ServerBusy)
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_bad_rc() {
    let raw = &[0xFF]; // invalid reason code
    let err = mqtt::packet::v5_0::Disconnect::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_rc_prop0() {
    let raw = &[0x00, 0x00]; // reason code + property length 0
    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(raw).unwrap();
    assert_eq!(consumed, 2);
    let expected = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_proplen_over() {
    let raw = &[0x00, 0x01]; // reason code + property length 1 but no property data
    let err = mqtt::packet::v5_0::Disconnect::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_rc_prop_session_expiry() {
    let mut raw = vec![0x00]; // reason code: NormalDisconnection
    raw.push(0x05); // property length
    raw.push(0x11); // property ID: Session Expiry Interval (0x11)
    raw.extend_from_slice(&300u32.to_be_bytes()); // 300 seconds

    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
        .props(vec![mqtt::packet::SessionExpiryInterval::new(300)
            .unwrap()
            .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_prop_session_expiry_twice() {
    let mut raw = vec![0x00]; // reason code: NormalDisconnection
    raw.push(0x0A); // property length
    raw.push(0x11); // property ID: Session Expiry Interval (0x11)
    raw.extend_from_slice(&300u32.to_be_bytes()); // 300 seconds
    raw.push(0x11); // property ID: Session Expiry Interval (0x11) again
    raw.extend_from_slice(&600u32.to_be_bytes()); // 600 seconds

    let err = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_reason_string() {
    let mut raw = vec![0x8B]; // reason code: ServerShuttingDown
    raw.push(0x15); // property length
    raw.push(0x1F); // property ID: Reason String (0x1F)
    raw.push(0x00); // string length MSB
    raw.push(0x12); // string length LSB (18)
    raw.extend_from_slice(b"Server maintenance");

    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::ServerShuttingDown)
        .props(vec![mqtt::packet::ReasonString::new("Server maintenance")
            .unwrap()
            .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_prop_reason_string_twice() {
    let mut raw = vec![0x00]; // reason code: NormalDisconnection
    raw.push(0x0E); // property length
    raw.push(0x1F); // property ID: Reason String (0x1F)
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");
    raw.push(0x1F); // property ID: Reason String (0x1F) again
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_server_reference() {
    let mut raw = vec![0x9D]; // reason code: ServerMoved
    raw.push(0x12); // property length (1 + 2 + 15)
    raw.push(0x1C); // property ID: Server Reference (0x1C)
    raw.push(0x00); // string length MSB
    raw.push(0x0F); // string length LSB (15)
    raw.extend_from_slice(b"new.example.com");

    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::ServerMoved)
        .props(vec![mqtt::packet::ServerReference::new("new.example.com")
            .unwrap()
            .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_prop_server_reference_twice() {
    let mut raw = vec![0x00]; // reason code: NormalDisconnection
    raw.push(0x14); // property length
    raw.push(0x1C); // property ID: Server Reference (0x1C)
    raw.push(0x00); // string length MSB
    raw.push(0x07); // string length LSB (7)
    raw.extend_from_slice(b"server1");
    raw.push(0x1C); // property ID: Server Reference (0x1C) again
    raw.push(0x00); // string length MSB
    raw.push(0x07); // string length LSB (7)
    raw.extend_from_slice(b"server2");

    let err = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_unmatched() {
    let mut raw = vec![0x00]; // reason code: NormalDisconnection
    raw.push(0x07); // property length
    raw.push(0x03); // property ID: Content Type (0x03) - not allowed in DISCONNECT
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_user_property_twice() {
    let mut raw = vec![0x00]; // reason code: NormalDisconnection
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

    let (packet, consumed) = mqtt::packet::v5_0::Disconnect::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Disconnect::builder()
        .reason_code(mqtt::result_code::DisconnectReasonCode::NormalDisconnection)
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
    let packet_type = mqtt::packet::v5_0::Disconnect::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Disconnect);
}
