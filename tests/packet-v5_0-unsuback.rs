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

// Build fail tests
#[test]
fn build_fail_empty_reason_codes() {
    let err = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_no_packet_id() {
    let err = mqtt::packet::v5_0::Unsuback::builder()
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_property() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::PayloadFormatIndicator(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::Binary).unwrap(),
    ));

    let err = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_multiple_reason_strings() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("First reason").unwrap(),
    ));
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Second reason").unwrap(),
    ));

    let err = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Build success tests
#[test]
fn build_success_minimal() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
    assert_eq!(packet.reason_codes().len(), 1);
    assert!(packet.props().is_empty());
}

#[test]
fn build_success_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 42u16);
    assert_eq!(packet.reason_codes().len(), 1);
    assert_eq!(packet.props().len(), 2);
}

#[test]
fn build_success_multiple_reason_codes() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(100u16)
        .reason_codes(vec![
            mqtt::result_code::UnsubackReasonCode::Success,
            mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted,
            mqtt::result_code::UnsubackReasonCode::UnspecifiedError,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 100u16);
    assert_eq!(packet.reason_codes().len(), 3);
    assert_eq!(
        packet.reason_codes()[0],
        mqtt::result_code::UnsubackReasonCode::Success
    );
    assert_eq!(
        packet.reason_codes()[1],
        mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted
    );
    assert_eq!(
        packet.reason_codes()[2],
        mqtt::result_code::UnsubackReasonCode::UnspecifiedError
    );
}

// Display tests
#[test]
fn display_minimal() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":1"));
    assert!(display_str.contains("\"reason_codes\""));
}

#[test]
fn display_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":42"));
    assert!(display_str.contains("\"props\""));
    assert!(display_str.contains("\"reason_codes\""));
}

// Debug tests
#[test]
fn debug_minimal() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"packet_id\":1"));
}

#[test]
fn debug_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("test", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(99u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"props\""));
}

// Getter tests
#[test]
fn getter_packet_id() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(12345u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

#[test]
fn getter_reason_codes() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![
            mqtt::result_code::UnsubackReasonCode::Success,
            mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.reason_codes().len(), 2);
    assert_eq!(
        packet.reason_codes()[0],
        mqtt::result_code::UnsubackReasonCode::Success
    );
    assert_eq!(
        packet.reason_codes()[1],
        mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted
    );
}

#[test]
fn getter_props_empty() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    assert!(packet.props().is_empty());
}

#[test]
fn getter_props_with_values() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.props().len(), 2);
}

// to_buffers() tests
#[test]
fn to_buffers_minimal() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert!(!buffers.is_empty());

    // Collect all bytes
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Check fixed header
    assert_eq!(all_bytes[0], 0xB0); // UNSUBACK packet type

    // Should contain packet ID, property length, and reason codes
    assert!(all_bytes.len() > 4);
}

#[test]
fn to_buffers_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Should be larger than minimal case due to properties
    assert!(all_bytes.len() > 10);
    assert_eq!(all_bytes[0], 0xB0);
}

#[test]
fn to_buffers_multiple_reason_codes() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(100u16)
        .reason_codes(vec![
            mqtt::result_code::UnsubackReasonCode::Success,
            mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted,
            mqtt::result_code::UnsubackReasonCode::UnspecifiedError,
        ])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Should contain all three reason codes
    assert!(all_bytes.len() > 6);
}

// Parse tests
#[test]
fn parse_minimal() {
    let original = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        // Skip fixed header and remaining length
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.reason_codes().len(), 1);
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::UnsubackReasonCode::Success
    );
    assert!(parsed.props().is_empty());
}

#[test]
fn parse_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));

    let original = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 42u16);
    assert_eq!(parsed.reason_codes().len(), 1);
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::UnsubackReasonCode::Success
    );
    assert_eq!(parsed.props().len(), 1);
}

#[test]
fn parse_multiple_reason_codes() {
    let original = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(200u16)
        .reason_codes(vec![
            mqtt::result_code::UnsubackReasonCode::Success,
            mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted,
            mqtt::result_code::UnsubackReasonCode::UnspecifiedError,
        ])
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 200u16);
    assert_eq!(parsed.reason_codes().len(), 3);
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::UnsubackReasonCode::Success
    );
    assert_eq!(
        parsed.reason_codes()[1],
        mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted
    );
    assert_eq!(
        parsed.reason_codes()[2],
        mqtt::result_code::UnsubackReasonCode::UnspecifiedError
    );
}

#[test]
fn parse_invalid_too_short() {
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_reason_codes() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x00); // property length = 0

    let err = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_property() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x02); // property length = 2
    data.push(0x01); // PayloadFormatIndicator (invalid for UNSUBACK)
    data.push(0x00); // property value

    let err = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Size tests
#[test]
fn size_minimal() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    let size = packet.size();
    assert!(size > 0);

    // Verify size matches actual buffer size
    let buffers = packet.to_buffers();
    let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(size, actual_size);
}

#[test]
fn size_with_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .props(props)
        .build()
        .unwrap();

    let size = packet.size();
    let buffers = packet.to_buffers();
    let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(size, actual_size);
}

#[test]
fn size_multiple_reason_codes() {
    let packet = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(100u16)
        .reason_codes(vec![
            mqtt::result_code::UnsubackReasonCode::Success,
            mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted,
            mqtt::result_code::UnsubackReasonCode::UnspecifiedError,
        ])
        .build()
        .unwrap();

    let size = packet.size();
    let buffers = packet.to_buffers();
    let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
    assert_eq!(size, actual_size);
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    let original = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::UnsubackReasonCode::Success])
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.reason_codes().len(), parsed.reason_codes().len());
    assert_eq!(original.props().len(), parsed.props().len());
}

#[test]
fn roundtrip_with_all_valid_properties() {
    let mut props = mqtt::packet::Properties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Operation successful").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("client", "test").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("version", "1.0").unwrap(),
    ));

    let original = mqtt::packet::v5_0::Unsuback::builder()
        .packet_id(65535u16)
        .reason_codes(vec![
            mqtt::result_code::UnsubackReasonCode::Success,
            mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted,
            mqtt::result_code::UnsubackReasonCode::UnspecifiedError,
        ])
        .props(props)
        .build()
        .unwrap();

    let buffers = original.to_buffers();
    let mut data = Vec::new();
    for buf in buffers.iter().skip(2) {
        data.extend_from_slice(buf);
    }

    let (parsed, _) = mqtt::packet::v5_0::Unsuback::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.reason_codes().len(), parsed.reason_codes().len());
    assert_eq!(original.props().len(), parsed.props().len());

    // Check specific values
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::UnsubackReasonCode::Success
    );
    assert_eq!(
        parsed.reason_codes()[1],
        mqtt::result_code::UnsubackReasonCode::NoSubscriptionExisted
    );
    assert_eq!(
        parsed.reason_codes()[2],
        mqtt::result_code::UnsubackReasonCode::UnspecifiedError
    );
}
