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
fn build_fail_nosp() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_norc() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_prop() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into()])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_sei_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::SessionExpiryInterval::new(1).unwrap().into(),
            mqtt::packet::SessionExpiryInterval::new(2).unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_rm_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::ReceiveMaximum::new(1).unwrap().into(),
            mqtt::packet::ReceiveMaximum::new(2).unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_mq_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::MaximumQos::new(0).unwrap().into(),
            mqtt::packet::MaximumQos::new(1).unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_ra_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::RetainAvailable::new(0).unwrap().into(),
            mqtt::packet::RetainAvailable::new(1).unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_mps_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::MaximumPacketSize::new(1024).unwrap().into(),
            mqtt::packet::MaximumPacketSize::new(2048).unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_aci_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::AssignedClientIdentifier::new("cid1")
                .unwrap()
                .into(),
            mqtt::packet::AssignedClientIdentifier::new("cid2")
                .unwrap()
                .into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_tam_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::TopicAliasMaximum::new(1024).unwrap().into(),
            mqtt::packet::TopicAliasMaximum::new(2048).unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_valid_prop_rs_mt() {
    let err = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::ReasonString::new("test1").unwrap().into(),
            mqtt::packet::ReasonString::new("test2").unwrap().into(),
        ])
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// build success tests

#[test]
fn build_succes_svalid_prop() {
    mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![
            mqtt::packet::SessionExpiryInterval::new(60).unwrap().into(),
            mqtt::packet::ReceiveMaximum::new(10).unwrap().into(),
            mqtt::packet::MaximumQos::new(1).unwrap().into(),
            mqtt::packet::RetainAvailable::new(1).unwrap().into(),
            mqtt::packet::MaximumPacketSize::new(2048).unwrap().into(),
            mqtt::packet::AssignedClientIdentifier::new("client123")
                .unwrap()
                .into(),
            mqtt::packet::TopicAliasMaximum::new(5).unwrap().into(),
            mqtt::packet::ReasonString::new("Connection successful")
                .unwrap()
                .into(),
            mqtt::packet::UserProperty::new("key1", "value1")
                .unwrap()
                .into(),
            mqtt::packet::UserProperty::new("key2", "value2")
                .unwrap()
                .into(),
        ])
        .build()
        .expect("Failed to build Connack packet with valid properties");
}

// Display tests

#[test]
fn display_sp_rc() {
    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert_eq!(
        output,
        r#"{"type":"connack","session_present":true,"reason_code":"Success","props":[]}"#
    );
}

// Debug tests

#[test]
fn debug_sp_rc() {
    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{:?}", packet).unwrap();
    assert_eq!(
        output,
        r#"{"type":"connack","session_present":true,"reason_code":"Success","props":[]}"#
    );
}

// Getter tests

#[test]
fn getter_sp_rc() {
    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    assert!(packet.session_present());
    assert_eq!(
        packet.reason_code(),
        mqtt::result_code::ConnectReasonCode::Success
    );
    assert!(packet.props().is_empty());
}

// to_buffers() tests

#[test]
fn to_buffers_sp_rc() {
    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 5);
    assert_eq!(buffers[0].as_ref(), &[0x20]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x03]); // remaining length
    assert_eq!(buffers[2].as_ref(), &[0x01]); // session_present
    assert_eq!(buffers[3].as_ref(), &[0x00]); // reason_code
    assert_eq!(buffers[4].as_ref(), &[0x00]); // property_length
    assert_eq!(packet.size(), 5); // 1 + 1 + 1 + 1 + 1 = 5
}

#[test]
fn to_buffers_sp_rc_prop1() {
    let packet = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(vec![mqtt::packet::SessionExpiryInterval::new(1)
            .unwrap()
            .into()])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 7);
    assert_eq!(buffers[0].as_ref(), &[0x20]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x08]); // remaining length
    assert_eq!(buffers[2].as_ref(), &[0x01]); // session_present
    assert_eq!(buffers[3].as_ref(), &[0x00]); // reason_code
    assert_eq!(buffers[4].as_ref(), &[0x05]); // property_length
    assert_eq!(buffers[5].as_ref(), &[0x11]); // prop_id: session_expiry_interval
    assert_eq!(buffers[6].as_ref(), &[0x00, 0x00, 0x00, 0x01]); // session_expiry_interval

    assert_eq!(packet.size(), 10); // 1 + 1 + 1 + 1 + 1 + 1 + 4 = 10
}

// Parse tests

#[test]
fn parse_no_sp() {
    let raw = &[];
    let err = mqtt::packet::v5_0::Connack::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_no_rc() {
    let raw = &[0x01];
    let err = mqtt::packet::v5_0::Connack::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_no_prop_len() {
    let raw = &[0x01, 0x00];
    let err = mqtt::packet::v5_0::Connack::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_no_props() {
    let raw = &[0x01, 0x00, 0x00];
    let (packet, _) = mqtt::packet::v5_0::Connack::parse(raw)
        .expect("Failed to parse Connack packet with no properties");
    let expected = mqtt::packet::v5_0::Connack::builder()
        .session_present(true)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .expect("Failed to build expected Connack packet");
    assert_eq!(packet, expected);
}
