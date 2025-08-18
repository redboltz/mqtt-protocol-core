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
fn build_fail_empty_return_codes() {
    let err = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_no_packet_id() {
    let err = mqtt::packet::v3_1_1::Suback::builder()
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_zero_packet_id() {
    let err = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(0u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Build success tests
#[test]
fn build_success_minimal() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();
    assert_eq!(packet.packet_id(), 1u16);
    assert_eq!(packet.return_codes().len(), 1);
}

#[test]
fn build_success_multiple_return_codes() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(100u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos1,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos2,
            mqtt::result_code::SubackReturnCode::Failure,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 100u16);
    assert_eq!(packet.return_codes().len(), 4);
    assert_eq!(
        packet.return_codes()[0],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos0
    );
    assert_eq!(
        packet.return_codes()[1],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos1
    );
    assert_eq!(
        packet.return_codes()[2],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos2
    );
    assert_eq!(
        packet.return_codes()[3],
        mqtt::result_code::SubackReturnCode::Failure
    );
}

// Display tests
#[test]
fn display_minimal() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":1"));
    assert!(display_str.contains("\"return_codes\""));
}

#[test]
fn display_multiple_return_codes() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(42u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos1,
            mqtt::result_code::SubackReturnCode::Failure,
        ])
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"packet_id\":42"));
    assert!(display_str.contains("\"return_codes\""));
}

// Debug tests
#[test]
fn debug_minimal() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"packet_id\":1"));
}

#[test]
fn debug_multiple_return_codes() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(99u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos2,
            mqtt::result_code::SubackReturnCode::Failure,
        ])
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"packet_id\":99"));
}

// Getter tests
#[test]
fn getter_packet_id() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(12345u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 12345u16);
}

#[test]
fn getter_return_codes() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos1,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.return_codes().len(), 2);
    assert_eq!(
        packet.return_codes()[0],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos0
    );
    assert_eq!(
        packet.return_codes()[1],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos1
    );
}

// to_buffers() tests
#[test]
#[cfg(feature = "std")]
fn to_buffers_minimal() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
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

    // Should contain packet ID and return codes
    assert!(all_bytes.len() > 3);
}

#[test]
#[cfg(feature = "std")]
fn to_buffers_multiple_return_codes() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(100u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos1,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos2,
            mqtt::result_code::SubackReturnCode::Failure,
        ])
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Should contain all four return codes
    assert!(all_bytes.len() > 6);
}

// Parse tests
#[test]
fn parse_minimal() {
    let original = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            // Skip fixed header and remaining length
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data);
    }

    let data: &[u8] = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Suback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 1u16);
    assert_eq!(parsed.return_codes().len(), 1);
    assert_eq!(
        parsed.return_codes()[0],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos0
    );
}

#[test]
fn parse_multiple_return_codes() {
    let original = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(200u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos1,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos2,
            mqtt::result_code::SubackReturnCode::Failure,
        ])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data);
    }

    let data: &[u8] = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Suback::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.packet_id(), 200u16);
    assert_eq!(parsed.return_codes().len(), 4);
    assert_eq!(
        parsed.return_codes()[0],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos0
    );
    assert_eq!(
        parsed.return_codes()[1],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos1
    );
    assert_eq!(
        parsed.return_codes()[2],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos2
    );
    assert_eq!(
        parsed.return_codes()[3],
        mqtt::result_code::SubackReturnCode::Failure
    );
}

#[test]
fn parse_invalid_too_short() {
    let data = [0x00]; // Too short for packet ID
    let err = mqtt::packet::v3_1_1::Suback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_no_return_codes() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID

    let err = mqtt::packet::v3_1_1::Suback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_return_code() {
    let mut data = Vec::new();
    data.extend_from_slice(&(1u16).to_be_bytes()); // packet ID
    data.push(0xFF); // Invalid return code

    let err = mqtt::packet::v3_1_1::Suback::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Size tests
#[test]
fn size_minimal() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();

    let size = packet.size();
    assert!(size > 0);

    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

#[test]
fn size_multiple_return_codes() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(100u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos1,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos2,
            mqtt::result_code::SubackReturnCode::Failure,
        ])
        .build()
        .unwrap();

    let size = packet.size();

    let actual_size = packet.to_continuous_buffer().len();
    assert_eq!(size, actual_size);

    #[cfg(feature = "std")]
    {
        let buffers = packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

// Parse/serialize roundtrip tests
#[test]
fn roundtrip_minimal() {
    let original = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
        ])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data);
    }

    let data: &[u8] = &continuous[2..];
    let (parsed, _) = mqtt::packet::v3_1_1::Suback::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.return_codes().len(), parsed.return_codes().len());
}

#[test]
fn roundtrip_all_return_codes() {
    let original = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(65535u16)
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos1,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos2,
            mqtt::result_code::SubackReturnCode::Failure,
        ])
        .build()
        .unwrap();

    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data);
    }

    let data: &[u8] = &continuous[2..];
    let (parsed, _) = mqtt::packet::v3_1_1::Suback::parse(&data).unwrap();
    assert_eq!(original.packet_id(), parsed.packet_id());
    assert_eq!(original.return_codes().len(), parsed.return_codes().len());

    // Check specific values
    assert_eq!(
        parsed.return_codes()[0],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos0
    );
    assert_eq!(
        parsed.return_codes()[1],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos1
    );
    assert_eq!(
        parsed.return_codes()[2],
        mqtt::result_code::SubackReturnCode::SuccessMaximumQos2
    );
    assert_eq!(
        parsed.return_codes()[3],
        mqtt::result_code::SubackReturnCode::Failure
    );
}

// Edge case tests
#[test]
fn test_large_packet_id() {
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(65534u16) // Maximum valid packet ID
        .return_codes(vec![
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos2,
        ])
        .build()
        .unwrap();

    assert_eq!(packet.packet_id(), 65534u16);
}

#[test]
fn test_many_return_codes() {
    let return_codes = vec![mqtt::result_code::SubackReturnCode::SuccessMaximumQos0; 100];
    let packet = mqtt::packet::v3_1_1::Suback::builder()
        .packet_id(1000u16)
        .return_codes(return_codes)
        .build()
        .unwrap();

    assert_eq!(packet.return_codes().len(), 100);
    for code in packet.return_codes() {
        assert_eq!(
            code,
            mqtt::result_code::SubackReturnCode::SuccessMaximumQos0
        );
    }
}

#[test]
fn test_packet_type() {
    let packet_type = mqtt::packet::v3_1_1::Suback::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Suback);
}
