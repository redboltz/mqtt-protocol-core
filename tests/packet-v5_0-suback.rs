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
use mqtt_protocol_core::mqtt;
mod common;

// Build fail tests
#[test]
fn build_fail_empty_reason_codes() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_no_packet_id() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Suback::builder()
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_invalid_property() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::PayloadFormatIndicator(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::Binary).unwrap(),
    ));

    let err = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_multiple_reason_strings() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("First reason").unwrap(),
    ));
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Second reason").unwrap(),
    ));

    let err = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Build success tests
#[test]
fn build_success_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
    assert_eq!(packet.reason_codes().len(), 1);
    assert!(packet.props().is_empty());
}

#[test]
fn build_success_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos1])
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 42u16);
    assert_eq!(packet.reason_codes().len(), 1);
    assert_eq!(packet.props().len(), 2);
}

#[test]
fn build_success_multiple_reason_codes() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(100u16)
        .reason_codes(vec![
            mqtt::result_code::SubackReasonCode::GrantedQos0,
            mqtt::result_code::SubackReasonCode::GrantedQos1,
            mqtt::result_code::SubackReasonCode::GrantedQos2,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 100u16);
    assert_eq!(packet.reason_codes().len(), 3);
    assert_eq!(
        packet.reason_codes()[0],
        mqtt::result_code::SubackReasonCode::GrantedQos0
    );
    assert_eq!(
        packet.reason_codes()[1],
        mqtt::result_code::SubackReasonCode::GrantedQos1
    );
    assert_eq!(
        packet.reason_codes()[2],
        mqtt::result_code::SubackReasonCode::GrantedQos2
    );
}

// Display tests
#[test]
fn display_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":1"));
    assert!(display_str.contains("\"reason_codes\""));
}

#[test]
fn display_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos1])
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
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"packet_id\":1"));
}

#[test]
fn debug_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("test", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(99u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos2])
        .props(props)
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"props\""));
}

// Getter tests
#[test]
fn getter_packet_id() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(12345u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

#[test]
fn getter_reason_codes() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![
            mqtt::result_code::SubackReasonCode::GrantedQos0,
            mqtt::result_code::SubackReasonCode::GrantedQos1,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.reason_codes().len(), 2);
    assert_eq!(
        packet.reason_codes()[0],
        mqtt::result_code::SubackReasonCode::GrantedQos0
    );
    assert_eq!(
        packet.reason_codes()[1],
        mqtt::result_code::SubackReasonCode::GrantedQos1
    );
}

#[test]
fn getter_props_empty() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();

    assert!(packet.props().is_empty());
}

#[test]
fn getter_props_with_values() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.props().len(), 2);
}

// to_buffers() tests
#[cfg(feature = "std")]
#[test]
fn to_buffers_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
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
    assert_eq!(all_bytes[0], 0x90); // SUBACK packet type

    // Should contain packet ID, property length, and reason codes
    assert!(all_bytes.len() > 4);
}

#[cfg(feature = "std")]
#[test]
fn to_buffers_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos1])
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
    assert_eq!(all_bytes[0], 0x90);
}

#[cfg(feature = "std")]
#[test]
fn to_buffers_multiple_reason_codes() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(100u16)
        .reason_codes(vec![
            mqtt::result_code::SubackReasonCode::GrantedQos0,
            mqtt::result_code::SubackReasonCode::GrantedQos1,
            mqtt::result_code::SubackReasonCode::GrantedQos2,
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
    common::init_tracing();
    let original = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, consumed) = mqtt::packet::v5_0::Suback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.reason_codes().len(), 1);
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::SubackReasonCode::GrantedQos0
    );
    assert!(parsed.props().is_empty());
}

#[test]
fn parse_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));

    let original = mqtt::packet::v5_0::Suback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos1])
        .props(props)
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, consumed) = mqtt::packet::v5_0::Suback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 42u16);
    assert_eq!(parsed.reason_codes().len(), 1);
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::SubackReasonCode::GrantedQos1
    );
    assert_eq!(parsed.props().len(), 1);
}

#[test]
fn parse_multiple_reason_codes() {
    common::init_tracing();
    let original = mqtt::packet::v5_0::Suback::builder()
        .packet_id(200u16)
        .reason_codes(vec![
            mqtt::result_code::SubackReasonCode::GrantedQos0,
            mqtt::result_code::SubackReasonCode::GrantedQos1,
            mqtt::result_code::SubackReasonCode::GrantedQos2,
        ])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, consumed) = mqtt::packet::v5_0::Suback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 200u16);
    assert_eq!(parsed.reason_codes().len(), 3);
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::SubackReasonCode::GrantedQos0
    );
    assert_eq!(
        parsed.reason_codes()[1],
        mqtt::result_code::SubackReasonCode::GrantedQos1
    );
    assert_eq!(
        parsed.reason_codes()[2],
        mqtt::result_code::SubackReasonCode::GrantedQos2
    );
}

#[test]
fn parse_invalid_too_short() {
    common::init_tracing();
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v5_0::Suback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_reason_codes() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x00); // property length = 0

    let err = mqtt::packet::v5_0::Suback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_property() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0x02); // property length = 2
    data.push(0x01); // PayloadFormatIndicator (invalid for SUBACK)
    data.push(0x00); // property value

    let err = mqtt::packet::v5_0::Suback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Size tests
#[test]
fn size_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();

    let size = packet.size();
    assert!(size > 0);

    // Verify size matches actual buffer size
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let buffers_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, buffers_size);
    }
}

#[test]
fn size_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Success").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("key", "value").unwrap(),
    ));

    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(42u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos1])
        .props(props)
        .build()
        .unwrap();

    let size = packet.size();
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let buffers_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, buffers_size);
    }
}

#[test]
fn size_multiple_reason_codes() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Suback::builder()
        .packet_id(100u16)
        .reason_codes(vec![
            mqtt::result_code::SubackReasonCode::GrantedQos0,
            mqtt::result_code::SubackReasonCode::GrantedQos1,
            mqtt::result_code::SubackReasonCode::GrantedQos2,
        ])
        .build()
        .unwrap();

    let size = packet.size();
    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let buffers_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, buffers_size);
    }
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    common::init_tracing();
    let original = mqtt::packet::v5_0::Suback::builder()
        .packet_id(1u16)
        .reason_codes(vec![mqtt::result_code::SubackReasonCode::GrantedQos0])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, _) = mqtt::packet::v5_0::Suback::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.reason_codes().len(), parsed.reason_codes().len());
    assert_eq!(original.props().len(), parsed.props().len());
}

#[test]
fn roundtrip_with_all_valid_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::Property::ReasonString(
        mqtt::packet::ReasonString::new("Operation successful").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("client", "test").unwrap(),
    ));
    props.push(mqtt::packet::Property::UserProperty(
        mqtt::packet::UserProperty::new("version", "1.0").unwrap(),
    ));

    let original = mqtt::packet::v5_0::Suback::builder()
        .packet_id(65535u16)
        .reason_codes(vec![
            mqtt::result_code::SubackReasonCode::GrantedQos0,
            mqtt::result_code::SubackReasonCode::GrantedQos1,
            mqtt::result_code::SubackReasonCode::UnspecifiedError,
        ])
        .props(props)
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length
    let (parsed, _) = mqtt::packet::v5_0::Suback::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.reason_codes().len(), parsed.reason_codes().len());
    assert_eq!(original.props().len(), parsed.props().len());

    // Check specific values
    assert_eq!(
        parsed.reason_codes()[0],
        mqtt::result_code::SubackReasonCode::GrantedQos0
    );
    assert_eq!(
        parsed.reason_codes()[1],
        mqtt::result_code::SubackReasonCode::GrantedQos1
    );
    assert_eq!(
        parsed.reason_codes()[2],
        mqtt::result_code::SubackReasonCode::UnspecifiedError
    );
}

#[test]
fn test_packet_type() {
    common::init_tracing();
    let packet_type = mqtt::packet::v5_0::Suback::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Suback);
}
