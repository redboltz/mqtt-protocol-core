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
    // AUTH packets cannot have props without reason_code
    let err = mqtt::packet::v5_0::Auth::builder()
        .props(mqtt::packet::Properties::new())
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_prop() {
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .props(vec![mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_auth_data_without_method() {
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .props(vec![mqtt::packet::AuthenticationData::new(vec![
            1, 2, 3, 4,
        ])
        .unwrap()
        .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_auth_method_twice() {
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .props(vec![
            mqtt::packet::AuthenticationMethod::new("method1")
                .unwrap()
                .into(),
            mqtt::packet::AuthenticationMethod::new("method2")
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_reason_string_twice() {
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .props(vec![
            mqtt::packet::ReasonString::new("reason1").unwrap().into(),
            mqtt::packet::ReasonString::new("reason2").unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_continue_auth_without_auth_method() {
    // ContinueAuthentication reason code without Authentication Method should fail
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![mqtt::packet::ReasonString::new(
            "Continue authentication",
        )
        .unwrap()
        .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_reauth_without_auth_method() {
    // ReAuthenticate reason code without Authentication Method should fail
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ReAuthenticate)
        .props(vec![mqtt::packet::UserProperty::new("client", "test")
            .unwrap()
            .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_continue_auth_no_props() {
    // ContinueAuthentication reason code without any properties should fail
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_auth_data_twice() {
    // Authentication Data property can only appear once
    let err = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![
            mqtt::packet::AuthenticationMethod::new("SCRAM-SHA-256")
                .unwrap()
                .into(),
            mqtt::packet::AuthenticationData::new(vec![1, 2, 3])
                .unwrap()
                .into(),
            mqtt::packet::AuthenticationData::new(vec![4, 5, 6])
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
    let packet = mqtt::packet::v5_0::Auth::builder().build().unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(output, r#"{"type":"auth"}"#);
}

#[test]
fn display_rc() {
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"auth","reason_code":"Success","props":[]}"#
    );
}

#[test]
fn display_rc_props() {
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![mqtt::packet::AuthenticationMethod::new(
            "SCRAM-SHA-256",
        )
        .unwrap()
        .into()])
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"auth","reason_code":"ContinueAuthentication","props":[{"AuthenticationMethod":{"id":21,"val":"SCRAM-SHA-256"}}]}"#
    );
}

// Debug tests

#[test]
fn debug_empty() {
    let packet = mqtt::packet::v5_0::Auth::builder().build().unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(output, r#"{"type":"auth"}"#);
}

#[test]
fn debug_rc() {
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"auth","reason_code":"Success","props":[]}"#
    );
}

#[test]
fn debug_rc_prop0() {
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert_eq!(
        output,
        r#"{"type":"auth","reason_code":"Success","props":[]}"#
    );
}

// Getter tests

#[test]
fn getter_empty() {
    let packet = mqtt::packet::v5_0::Auth::builder().build().unwrap();

    assert!(packet.reason_code().is_none());
    assert!(packet.props().is_none());
}

#[test]
fn getter_rc() {
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();

    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::AuthReasonCode::Success
    );
    assert!(packet.props().is_some());
    assert!(packet.props().as_ref().unwrap().is_empty());
}

#[test]
fn getter_rc_props_auth_method() {
    let props = vec![mqtt::packet::AuthenticationMethod::new("SCRAM-SHA-256")
        .unwrap()
        .into()];
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(props.clone())
        .build()
        .unwrap();

    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::AuthReasonCode::ContinueAuthentication
    );
    assert!(packet.props().as_ref().unwrap() == &props);
}

#[test]
fn getter_rc_props_mixed() {
    let props = vec![
        mqtt::packet::AuthenticationMethod::new("SCRAM-SHA-256")
            .unwrap()
            .into(),
        mqtt::packet::AuthenticationData::new(vec![1, 2, 3, 4])
            .unwrap()
            .into(),
        mqtt::packet::ReasonString::new("Continue authentication")
            .unwrap()
            .into(),
        mqtt::packet::UserProperty::new("client", "test_client")
            .unwrap()
            .into(),
    ];
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(props.clone())
        .build()
        .unwrap();

    assert!(packet.reason_code().is_some());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::AuthReasonCode::ContinueAuthentication
    );
    assert!(packet.props().as_ref().unwrap() == &props);
}

// to_buffers() tests

#[test]
fn to_buffers_empty() {
    let packet = mqtt::packet::v5_0::Auth::builder().build().unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 2);
    assert_eq!(buffers[0].as_ref(), &[0xf0]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x00]); // remaining length
    assert_eq!(packet.size(), 2); // 1 + 1
}

#[test]
fn to_buffers_rc() {
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 4);
    assert_eq!(buffers[0].as_ref(), &[0xf0]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x02]); // remaining length
    assert_eq!(buffers[2].as_ref(), &[0x00]); // reason code
    assert_eq!(buffers[3].as_ref(), &[0x00]); // property length
    assert_eq!(packet.size(), 4); // 1 + 1 + 1 + 1
}

#[test]
fn to_buffers_rc_props_auth_method() {
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![mqtt::packet::AuthenticationMethod::new(
            "SCRAM-SHA-256",
        )
        .unwrap()
        .into()])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();

    assert_eq!(buffers.len(), 6);
    assert_eq!(buffers[0].as_ref(), &[0xf0]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x12]); // remaining length (1 + 1 + 16)
    assert_eq!(buffers[2].as_ref(), &[0x18]); // reason code
    assert_eq!(buffers[3].as_ref(), &[0x10]); // property length (16)
    assert_eq!(buffers[4].as_ref(), &[0x15]); // authentication method property ID
    assert_eq!(
        buffers[5].as_ref(),
        &[
            0x00, 0x0d, b'S', b'C', b'R', b'A', b'M', b'-', b'S', b'H', b'A', b'-', b'2', b'5',
            b'6'
        ]
    ); // method string
    assert_eq!(packet.size(), 20); // 1 + 1 + 1 + 1 + 16
}

// Parse tests

#[test]
fn parse_empty() {
    let raw = &[];
    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(raw).unwrap();
    assert_eq!(consumed, 0);
    let expected = mqtt::packet::v5_0::Auth::builder().build().unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc() {
    let raw = &[0x00, 0x00]; // reason code: Success + property length 0
    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(raw).unwrap();
    assert_eq!(consumed, 2);
    let expected = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_fail_continue_auth_no_auth_method() {
    let raw = &[0x18, 0x00]; // reason code: ContinueAuthentication + property length 0
    let err = mqtt::packet::v5_0::Auth::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_bad_rc() {
    let raw = &[0xFF, 0x00]; // invalid reason code
    let err = mqtt::packet::v5_0::Auth::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_rc_proplen_over() {
    let raw = &[0x00, 0x01]; // reason code + property length 1 but no property data
    let err = mqtt::packet::v5_0::Auth::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_rc_prop_auth_method() {
    let mut raw = vec![0x18]; // reason code: ContinueAuthentication
    raw.push(0x10); // property length
    raw.push(0x15); // property ID: Authentication Method (0x15)
    raw.push(0x00); // string length MSB
    raw.push(0x0d); // string length LSB (13)
    raw.extend_from_slice(b"SCRAM-SHA-256");

    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![mqtt::packet::AuthenticationMethod::new(
            "SCRAM-SHA-256",
        )
        .unwrap()
        .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_prop_auth_method_twice() {
    let mut raw = vec![0x00]; // reason code: Success
    raw.push(0x14); // property length
    raw.push(0x15); // property ID: Authentication Method (0x15)
    raw.push(0x00); // string length MSB
    raw.push(0x07); // string length LSB (7)
    raw.extend_from_slice(b"method1");
    raw.push(0x15); // property ID: Authentication Method (0x15) again
    raw.push(0x00); // string length MSB
    raw.push(0x07); // string length LSB (0x07)
    raw.extend_from_slice(b"method2");

    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_auth_data() {
    let mut raw = vec![0x18]; // reason code: ContinueAuthentication
    raw.push(0x0E); // property length
    raw.push(0x15); // property ID: Authentication Method (0x15)
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");
    raw.push(0x16); // property ID: Authentication Data (0x16)
    raw.push(0x00); // binary length MSB
    raw.push(0x04); // binary length LSB (4)
    raw.extend_from_slice(&[1, 2, 3, 4]);

    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![
            mqtt::packet::AuthenticationMethod::new("test")
                .unwrap()
                .into(),
            mqtt::packet::AuthenticationData::new(vec![1, 2, 3, 4])
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_prop_auth_data_without_method() {
    let mut raw = vec![0x00]; // reason code: Success
    raw.push(0x07); // property length
    raw.push(0x16); // property ID: Authentication Data (0x16) - not allowed without method
    raw.push(0x00); // binary length MSB
    raw.push(0x04); // binary length LSB (4)
    raw.extend_from_slice(&[1, 2, 3, 4]);

    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_reason_string() {
    let mut raw = vec![0x00]; // reason code: Success
    raw.push(0x14); // property length
    raw.push(0x1F); // property ID: Reason String (0x1F)
    raw.push(0x00); // string length MSB
    raw.push(0x11); // string length LSB (17)
    raw.extend_from_slice(b"Authentication OK");

    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
        .props(vec![mqtt::packet::ReasonString::new("Authentication OK")
            .unwrap()
            .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_rc_prop_reason_string_twice() {
    let mut raw = vec![0x00]; // reason code: Success
    raw.push(0x0e); // property length
    raw.push(0x1F); // property ID: Reason String (0x1F)
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");
    raw.push(0x1F); // property ID: Reason String (0x1F) again
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_unmatched() {
    let mut raw = vec![0x00]; // reason code: Success
    raw.push(0x07); // property length
    raw.push(0x03); // property ID: Content Type (0x03) - not allowed in AUTH
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");

    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_rc_prop_user_property_twice() {
    let mut raw = vec![0x00]; // reason code: Success
    raw.push(0x1e); // property length

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

    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::Success)
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

// Positive tests

#[test]
fn build_success_continue_auth_with_auth_method() {
    // ContinueAuthentication reason code with Authentication Method should succeed
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![mqtt::packet::AuthenticationMethod::new(
            "SCRAM-SHA-256",
        )
        .unwrap()
        .into()])
        .build()
        .unwrap();

    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::AuthReasonCode::ContinueAuthentication
    );
    assert!(packet.props().is_some());
}

#[test]
fn build_success_reauth_with_auth_method() {
    // ReAuthenticate reason code with Authentication Method should succeed
    let packet = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ReAuthenticate)
        .props(vec![
            mqtt::packet::AuthenticationMethod::new("OAUTH2")
                .unwrap()
                .into(),
            mqtt::packet::ReasonString::new("Re-authenticating")
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap();

    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::AuthReasonCode::ReAuthenticate
    );
    assert!(packet.props().is_some());
    assert_eq!(packet.props().as_ref().unwrap().len(), 2);
}

#[test]
fn build_success_auth_method_omitted() {
    // AUTH packet with no reason code and no properties (Authentication Method can be omitted)
    let packet = mqtt::packet::v5_0::Auth::builder().build().unwrap();

    assert!(packet.reason_code().is_none());
    assert!(packet.props().is_none());
}

#[test]
fn parse_fail_continue_auth_without_auth_method() {
    // ContinueAuthentication without Authentication Method should fail
    let mut raw = vec![0x18]; // reason code: ContinueAuthentication
    raw.push(0x0a); // property length
    raw.push(0x1F); // property ID: Reason String (0x1F)
    raw.push(0x00); // string length MSB
    raw.push(0x07); // string length LSB (7)
    raw.extend_from_slice(b"testing");

    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_fail_reauth_without_auth_method() {
    // ReAuthenticate without Authentication Method should fail
    let mut raw = vec![0x19]; // reason code: ReAuthenticate
    raw.push(0x0e); // property length

    // First User Property
    raw.push(0x26); // property ID: User Property (0x26)
    raw.push(0x00); // name string length MSB
    raw.push(0x04); // name string length LSB (4)
    raw.extend_from_slice(b"test");
    raw.push(0x00); // value string length MSB
    raw.push(0x04); // value string length LSB (4)
    raw.extend_from_slice(b"data");

    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_success_continue_auth_with_auth_method() {
    // ContinueAuthentication with Authentication Method should succeed
    let mut raw = vec![0x18]; // reason code: ContinueAuthentication
    raw.push(0x10); // property length
    raw.push(0x15); // property ID: Authentication Method (0x15)
    raw.push(0x00); // string length MSB
    raw.push(0x0d); // string length LSB (13)
    raw.extend_from_slice(b"SCRAM-SHA-256");

    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());

    let expected = mqtt::packet::v5_0::Auth::builder()
        .reason_code(mqtt::result_code::AuthReasonCode::ContinueAuthentication)
        .props(vec![mqtt::packet::AuthenticationMethod::new(
            "SCRAM-SHA-256",
        )
        .unwrap()
        .into()])
        .build()
        .unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_continue_auth_with_auth_method() {
    // Raw data: reason code (0x18) + property length + auth method property
    let auth_method = "SCRAM-SHA-256";
    let mut raw = Vec::new();
    raw.push(0x18); // ContinueAuthentication reason code

    // Calculate property length
    let prop_len = 1 + 2 + auth_method.len(); // identifier + length + value
    raw.push(prop_len as u8); // property length

    // Add AuthenticationMethod property
    raw.push(0x15); // AuthenticationMethod identifier
    raw.extend_from_slice(&(auth_method.len() as u16).to_be_bytes());
    raw.extend_from_slice(auth_method.as_bytes());

    let (packet, consumed) = mqtt::packet::v5_0::Auth::parse(&raw).unwrap();
    assert_eq!(consumed, raw.len());
    assert_eq!(
        packet.reason_code().unwrap(),
        mqtt::result_code::AuthReasonCode::ContinueAuthentication
    );
    assert!(packet.props().is_some());
}

#[test]
fn parse_fail_auth_data_twice() {
    // Authentication Data property should not appear twice
    let mut raw = vec![0x18]; // reason code: ContinueAuthentication
    raw.push(0x13); // property length (1+2+13) + (1+2+4) + (1+2+4) + (1+2+4) = 19

    // Authentication Method
    raw.push(0x15); // property ID: Authentication Method (0x15)
    raw.push(0x00); // string length MSB
    raw.push(0x04); // string length LSB (4)
    raw.extend_from_slice(b"test");

    // First Authentication Data
    raw.push(0x16); // property ID: Authentication Data (0x16)
    raw.push(0x00); // binary length MSB
    raw.push(0x03); // binary length LSB (3)
    raw.extend_from_slice(&[1, 2, 3]);

    // Second Authentication Data (should fail)
    raw.push(0x16); // property ID: Authentication Data (0x16)
    raw.push(0x00); // binary length MSB
    raw.push(0x03); // binary length LSB (3)
    raw.extend_from_slice(&[4, 5, 6]);

    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_fail_non_success_no_props() {
    // ContinueAuthentication with no properties should fail
    let raw = [0x18]; // reason code: ContinueAuthentication, no property length
    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_fail_reauth_no_props() {
    // ReAuthenticate with no properties should fail
    let raw = [0x19]; // reason code: ReAuthenticate, no property length
    let err = mqtt::packet::v5_0::Auth::parse(&raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn test_packet_type() {
    let packet_type = mqtt::packet::v5_0::Auth::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Auth);
}
