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
use core::fmt::Write;
use mqtt_protocol_core::mqtt;

// Build fail tests

#[test]
fn build_fail_nosp() {
    let err = mqtt::packet::v3_1_1::Connack::builder()
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn build_fail_norc() {
    let err = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .build()
        .unwrap_err();

    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Build success tests

#[test]
fn build_success_sp_rc() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    assert!(packet.session_present());
    assert_eq!(
        packet.return_code(),
        mqtt::result_code::ConnectReturnCode::Accepted
    );
}

#[test]
fn build_success_no_sp() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    assert!(!packet.session_present());
    assert_eq!(
        packet.return_code(),
        mqtt::result_code::ConnectReturnCode::Accepted
    );
}

#[test]
fn build_success_different_return_codes() {
    let return_codes = [
        mqtt::result_code::ConnectReturnCode::Accepted,
        mqtt::result_code::ConnectReturnCode::UnacceptableProtocolVersion,
        mqtt::result_code::ConnectReturnCode::IdentifierRejected,
        mqtt::result_code::ConnectReturnCode::ServerUnavailable,
        mqtt::result_code::ConnectReturnCode::BadUserNameOrPassword,
        mqtt::result_code::ConnectReturnCode::NotAuthorized,
    ];

    for return_code in return_codes {
        let packet = mqtt::packet::v3_1_1::Connack::builder()
            .session_present(false)
            .return_code(return_code)
            .build()
            .unwrap();

        assert_eq!(packet.return_code(), return_code);
    }
}

// Display tests

#[test]
#[cfg(feature = "std")]
fn display_sp_rc() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert!(output.contains(r#""type":"connack""#));
    assert!(output.contains(r#""session_present":true"#));
    assert!(output.contains(r#""return_code":"Accepted""#));
}

#[test]
#[cfg(feature = "std")]
fn display_no_sp() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::NotAuthorized)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet}").unwrap();
    assert!(output.contains(r#""type":"connack""#));
    assert!(output.contains(r#""session_present":false"#));
    assert!(output.contains(r#""return_code":"NotAuthorized""#));
}

// Debug tests

#[test]
#[cfg(feature = "std")]
fn debug_sp_rc() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let mut output = String::new();
    write!(&mut output, "{packet:?}").unwrap();
    assert!(output.contains(r#""type":"connack""#));
    assert!(output.contains(r#""session_present":true"#));
    assert!(output.contains(r#""return_code":"Accepted""#));
}

// Getter tests

#[test]
fn getter_sp_rc() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    assert!(packet.session_present());
    assert_eq!(
        packet.return_code(),
        mqtt::result_code::ConnectReturnCode::Accepted
    );
}

#[test]
fn getter_no_sp() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::ServerUnavailable)
        .build()
        .unwrap();

    assert!(!packet.session_present());
    assert_eq!(
        packet.return_code(),
        mqtt::result_code::ConnectReturnCode::ServerUnavailable
    );
}

// to_buffers() tests

#[test]
#[cfg(feature = "std")]
fn to_buffers_sp_rc() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 4);
    assert_eq!(buffers[0].as_ref(), &[0x20]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x02]); // remaining length
    assert_eq!(buffers[2].as_ref(), &[0x01]); // session_present = true
    assert_eq!(buffers[3].as_ref(), &[0x00]); // return_code = Accepted

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    let mut from_buffers = Vec::new();
    for buf in buffers {
        from_buffers.extend_from_slice(&buf);
    }
    assert_eq!(continuous, from_buffers);
    assert_eq!(packet.size(), 4); // 1 + 1 + 1 + 1 = 4
}

#[test]
#[cfg(feature = "std")]
fn to_buffers_no_sp() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::NotAuthorized)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 4);
    assert_eq!(buffers[0].as_ref(), &[0x20]); // fixed header
    assert_eq!(buffers[1].as_ref(), &[0x02]); // remaining length
    assert_eq!(buffers[2].as_ref(), &[0x00]); // session_present = false
    assert_eq!(buffers[3].as_ref(), &[0x05]); // return_code = NotAuthorized (5)
    assert_eq!(packet.size(), 4); // 1 + 1 + 1 + 1 = 4

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    let mut from_buffers = Vec::new();
    for buf in buffers {
        from_buffers.extend_from_slice(&buf);
    }
    assert_eq!(continuous, from_buffers);
}

#[test]
#[cfg(feature = "std")]
fn to_buffers_different_return_codes() {
    let test_cases = [
        (mqtt::result_code::ConnectReturnCode::Accepted, 0x00),
        (
            mqtt::result_code::ConnectReturnCode::UnacceptableProtocolVersion,
            0x01,
        ),
        (
            mqtt::result_code::ConnectReturnCode::IdentifierRejected,
            0x02,
        ),
        (
            mqtt::result_code::ConnectReturnCode::ServerUnavailable,
            0x03,
        ),
        (
            mqtt::result_code::ConnectReturnCode::BadUserNameOrPassword,
            0x04,
        ),
        (mqtt::result_code::ConnectReturnCode::NotAuthorized, 0x05),
    ];

    for (return_code, expected_byte) in test_cases {
        let packet = mqtt::packet::v3_1_1::Connack::builder()
            .session_present(false)
            .return_code(return_code)
            .build()
            .unwrap();

        let buffers = packet.to_buffers();
        assert_eq!(buffers[3].as_ref(), &[expected_byte]);
    }
}

// Parse tests

#[test]
fn parse_incomplete_data() {
    let raw = &[];
    let err = mqtt::packet::v3_1_1::Connack::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_missing_return_code() {
    let raw = &[0x01]; // only ack_flags, missing return_code
    let err = mqtt::packet::v3_1_1::Connack::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_sp_accepted() {
    let raw = &[0x01, 0x00]; // session_present=true, return_code=Accepted
    let (packet, consumed) =
        mqtt::packet::v3_1_1::Connack::parse(raw).expect("Failed to parse Connack packet");

    assert_eq!(consumed, 2);
    assert!(packet.session_present());
    assert_eq!(
        packet.return_code(),
        mqtt::result_code::ConnectReturnCode::Accepted
    );
}

#[test]
fn parse_no_sp_accepted() {
    let raw = &[0x00, 0x00]; // session_present=false, return_code=Accepted
    let (packet, consumed) =
        mqtt::packet::v3_1_1::Connack::parse(raw).expect("Failed to parse Connack packet");

    assert_eq!(consumed, 2);
    assert!(!packet.session_present());
    assert_eq!(
        packet.return_code(),
        mqtt::result_code::ConnectReturnCode::Accepted
    );
}

#[test]
fn parse_different_return_codes() {
    let test_cases = [
        (0x00, mqtt::result_code::ConnectReturnCode::Accepted),
        (
            0x01,
            mqtt::result_code::ConnectReturnCode::UnacceptableProtocolVersion,
        ),
        (
            0x02,
            mqtt::result_code::ConnectReturnCode::IdentifierRejected,
        ),
        (
            0x03,
            mqtt::result_code::ConnectReturnCode::ServerUnavailable,
        ),
        (
            0x04,
            mqtt::result_code::ConnectReturnCode::BadUserNameOrPassword,
        ),
        (0x05, mqtt::result_code::ConnectReturnCode::NotAuthorized),
    ];

    for (return_code_byte, expected_return_code) in test_cases {
        let raw = &[0x00, return_code_byte]; // session_present=false, various return codes
        let (packet, consumed) =
            mqtt::packet::v3_1_1::Connack::parse(raw).expect("Failed to parse Connack packet");

        assert_eq!(consumed, 2);
        assert!(!packet.session_present());
        assert_eq!(packet.return_code(), expected_return_code);
    }
}

#[test]
fn parse_invalid_return_code() {
    let raw = &[0x00, 0xFF]; // session_present=false, invalid return_code=255
    let err = mqtt::packet::v3_1_1::Connack::parse(raw).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_sp_with_error_codes() {
    let error_codes = [
        (
            0x01,
            mqtt::result_code::ConnectReturnCode::UnacceptableProtocolVersion,
        ),
        (
            0x02,
            mqtt::result_code::ConnectReturnCode::IdentifierRejected,
        ),
        (
            0x03,
            mqtt::result_code::ConnectReturnCode::ServerUnavailable,
        ),
        (
            0x04,
            mqtt::result_code::ConnectReturnCode::BadUserNameOrPassword,
        ),
        (0x05, mqtt::result_code::ConnectReturnCode::NotAuthorized),
    ];

    for (return_code_byte, expected_return_code) in error_codes {
        let raw = &[0x01, return_code_byte]; // session_present=true with error codes
        let (packet, consumed) =
            mqtt::packet::v3_1_1::Connack::parse(raw).expect("Failed to parse Connack packet");

        assert_eq!(consumed, 2);
        assert!(packet.session_present());
        assert_eq!(packet.return_code(), expected_return_code);
    }
}

// Size tests

#[test]
fn size_calculation() {
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    // CONNACK packet should always be 4 bytes: fixed header (1) + remaining length (1) + ack flags (1) + return code (1)
    assert_eq!(packet.size(), 4);
}

#[test]
fn size_different_values() {
    let test_cases = [
        (true, mqtt::result_code::ConnectReturnCode::Accepted),
        (false, mqtt::result_code::ConnectReturnCode::Accepted),
        (true, mqtt::result_code::ConnectReturnCode::NotAuthorized),
        (
            false,
            mqtt::result_code::ConnectReturnCode::ServerUnavailable,
        ),
    ];

    for (session_present, return_code) in test_cases {
        let packet = mqtt::packet::v3_1_1::Connack::builder()
            .session_present(session_present)
            .return_code(return_code)
            .build()
            .unwrap();

        // Size should always be 4 bytes regardless of values
        assert_eq!(packet.size(), 4);
    }
}

// Roundtrip tests (build -> to_buffers -> parse)

#[test]
fn roundtrip_sp_accepted() {
    let original = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Also verify to_buffers() produces same result when std is available
        let buffers = original.to_buffers();
        let mut from_buffers = Vec::new();
        for buffer in &buffers {
            from_buffers.extend_from_slice(buffer);
        }
        assert_eq!(continuous, from_buffers);
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length

    let (parsed, consumed) = mqtt::packet::v3_1_1::Connack::parse(data).unwrap();
    assert_eq!(consumed, 2);
    assert_eq!(parsed.session_present(), original.session_present());
    assert_eq!(parsed.return_code(), original.return_code());
}

#[test]
fn roundtrip_no_sp_error() {
    let original = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::NotAuthorized)
        .build()
        .unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let continuous = original.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Also verify to_buffers() produces same result when std is available
        let buffers = original.to_buffers();
        let mut from_buffers = Vec::new();
        for buffer in &buffers {
            from_buffers.extend_from_slice(buffer);
        }
        assert_eq!(continuous, from_buffers);
    }

    let data = &continuous[2..]; // Skip fixed header and remaining length

    let (parsed, consumed) = mqtt::packet::v3_1_1::Connack::parse(data).unwrap();
    assert_eq!(consumed, 2);
    assert_eq!(parsed.session_present(), original.session_present());
    assert_eq!(parsed.return_code(), original.return_code());
}

#[test]
fn test_packet_type() {
    let packet_type = mqtt::packet::v3_1_1::Connack::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Connack);
}

#[test]
fn test_connack_buffers_equivalence() {
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(true)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    // Get buffers from both methods
    let continuous_buffer = connack.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        let io_slices = connack.to_buffers();

        // Concatenate IoSlice buffers
        let mut concatenated = Vec::new();
        for slice in io_slices {
            concatenated.extend_from_slice(&slice);
        }

        // Verify they produce identical results
        assert_eq!(continuous_buffer, concatenated);
    }

    // Verify the continuous buffer is not empty and has correct size
    assert!(!continuous_buffer.is_empty());
    assert_eq!(continuous_buffer.len(), connack.size());
}
