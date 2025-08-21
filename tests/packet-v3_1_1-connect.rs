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
mod common;
use common::mqtt;

// Build fail tests
#[test]
fn build_success_no_client_id() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .clean_start(true)
        .build()
        .unwrap();
    assert_eq!(packet.client_id(), "");
    assert!(packet.clean_start());

    // Test packet_type method
    let packet_type = mqtt::packet::v3_1_1::Connect::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Connect);
}

#[test]
fn build_fail_password_without_username() {
    common::init_tracing();
    let err = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .password(b"password")
        .unwrap()
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_success_will_topic_with_empty_payload() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(packet.client_id(), "test");
    assert!(packet.will_flag());
    assert_eq!(packet.will_topic().unwrap(), "topic");
    assert_eq!(packet.will_payload().unwrap(), b"");
}

// Build success tests
#[test]
fn build_success_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "test_client");
    assert!(packet.clean_start());
    assert_eq!(packet.keep_alive(), 0);
    assert!(!packet.will_flag());
    assert!(!packet.user_name_flag());
    assert!(!packet.password_flag());
}

#[test]
fn build_success_with_credentials() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .user_name("username")
        .unwrap()
        .password(b"password")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "test_client");
    assert!(packet.user_name_flag());
    assert!(packet.password_flag());
    assert_eq!(packet.user_name().unwrap(), "username");
    assert_eq!(packet.password().unwrap(), b"password");
}

#[test]
fn build_success_with_will() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .will_message(
            "will/topic",
            b"will_payload",
            mqtt::packet::Qos::AtLeastOnce,
            true,
        )
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "test_client");
    assert!(packet.will_flag());
    assert_eq!(packet.will_qos(), mqtt::packet::Qos::AtLeastOnce);
    assert!(packet.will_retain());
    assert_eq!(packet.will_topic().unwrap(), "will/topic");
    assert_eq!(packet.will_payload().unwrap(), b"will_payload");
}

#[test]
fn build_success_clean_start_false() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_start(false)
        .keep_alive(60)
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "test_client");
    assert!(!packet.clean_start());
    assert_eq!(packet.keep_alive(), 60);
}

#[test]
fn build_success_all_features() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("comprehensive_test")
        .unwrap()
        .clean_start(false)
        .keep_alive(300)
        .user_name("testuser")
        .unwrap()
        .password(b"testpass")
        .unwrap()
        .will_message(
            "will/topic",
            b"will_message",
            mqtt::packet::Qos::ExactlyOnce,
            true,
        )
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "comprehensive_test");
    assert!(!packet.clean_start());
    assert_eq!(packet.keep_alive(), 300);
    assert!(packet.user_name_flag());
    assert!(packet.password_flag());
    assert_eq!(packet.user_name().unwrap(), "testuser");
    assert_eq!(packet.password().unwrap(), b"testpass");
    assert!(packet.will_flag());
    assert_eq!(packet.will_qos(), mqtt::packet::Qos::ExactlyOnce);
    assert!(packet.will_retain());
    assert_eq!(packet.will_topic().unwrap(), "will/topic");
    assert_eq!(packet.will_payload().unwrap(), b"will_message");
}

// Display tests
#[test]
fn display_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"type\":\"connect\""));
    assert!(display_str.contains("\"client_id\":\"test\""));
    assert!(display_str.contains("\"clean_start\":true"));
    assert!(display_str.contains("\"keep_alive\":0"));
}

#[test]
fn display_with_will() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtLeastOnce, true)
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"will_qos\":\"AtLeastOnce\""));
    assert!(display_str.contains("\"will_retain\":true"));
    assert!(display_str.contains("\"will_topic\":\"topic\""));
    assert!(display_str.contains("\"will_payload\":\"payload\""));
}

#[test]
fn display_with_password_masked() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .user_name("user")
        .unwrap()
        .password(b"secret")
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"user_name\":\"user\""));
    assert!(display_str.contains("\"password\":\"*****\""));
}

#[test]
fn display_with_binary_will_payload() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message(
            "topic",
            &[0x00, 0x01, 0xFF],
            mqtt::packet::Qos::AtMostOnce,
            false,
        )
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"will_payload\":[0,1,255]"));
}

// Debug tests
#[test]
fn debug_output() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("debug_test")
        .unwrap()
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"type\":\"connect\""));
    assert!(debug_str.contains("\"client_id\":\"debug_test\""));
}

// Getter tests
#[test]
fn getter_protocol_info() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.protocol_name(), "MQTT");
    assert_eq!(packet.protocol_version(), 0x04);
}

#[test]
fn getter_flags() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .clean_start(false)
        .user_name("user")
        .unwrap()
        .password(b"pass")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtLeastOnce, true)
        .unwrap()
        .build()
        .unwrap();

    assert!(!packet.clean_start());
    assert!(packet.will_flag());
    assert_eq!(packet.will_qos(), mqtt::packet::Qos::AtLeastOnce);
    assert!(packet.will_retain());
    assert!(packet.password_flag());
    assert!(packet.user_name_flag());
}

#[test]
fn getter_optional_fields() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.will_topic(), None);
    assert_eq!(packet.will_payload(), None);
    assert_eq!(packet.user_name(), None);
    assert_eq!(packet.password(), None);
}

// to_buffers() tests
#[test]
#[cfg(feature = "std")]
fn to_buffers_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert!(buffers.len() >= 7); // At minimum: fixed_header, remaining_length, protocol_name, version, flags, keep_alive, client_id

    // Check protocol name
    assert_eq!(buffers[2].as_ref(), &[0x00, 0x04, b'M', b'Q', b'T', b'T']);
    // Check protocol version
    assert_eq!(buffers[3].as_ref(), &[0x04]);

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
fn to_buffers_with_will() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    // Should have additional buffers for will topic and will payload
    assert!(buffers.len() > 7);

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
fn to_buffers_with_credentials() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .user_name("user")
        .unwrap()
        .password(b"pass")
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    // Should have additional buffers for username and password
    assert!(buffers.len() > 7);

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    let mut from_buffers = Vec::new();
    for buf in buffers {
        from_buffers.extend_from_slice(&buf);
    }
    assert_eq!(continuous, from_buffers);
}

// Parse tests
#[test]
fn parse_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            // Skip fixed header and remaining length
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "test");
    assert!(parsed.clean_start());
    assert_eq!(parsed.keep_alive(), 0);
}

#[test]
fn parse_with_credentials() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .user_name("username")
        .unwrap()
        .password(b"password")
        .unwrap()
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "test_client");
    assert!(parsed.user_name_flag());
    assert!(parsed.password_flag());
    assert_eq!(parsed.user_name().unwrap(), "username");
    assert_eq!(parsed.password().unwrap(), b"password");
}

#[test]
fn parse_with_will() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message(
            "will_topic",
            b"will_payload",
            mqtt::packet::Qos::AtLeastOnce,
            true,
        )
        .unwrap()
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "test");
    assert!(parsed.will_flag());
    assert_eq!(parsed.will_qos(), mqtt::packet::Qos::AtLeastOnce);
    assert!(parsed.will_retain());
    assert_eq!(parsed.will_topic().unwrap(), "will_topic");
    assert_eq!(parsed.will_payload().unwrap(), b"will_payload");
}

#[test]
fn parse_with_all_features() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("full_test")
        .unwrap()
        .clean_start(false)
        .keep_alive(120)
        .user_name("testuser")
        .unwrap()
        .password(b"testpass")
        .unwrap()
        .will_message(
            "will/topic",
            b"will_msg",
            mqtt::packet::Qos::ExactlyOnce,
            true,
        )
        .unwrap()
        .build()
        .unwrap();

    let continuous = packet.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter() {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(continuous, buffers_data.as_slice());
    }

    let data = &continuous[2..];
    let (parsed, consumed) = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "full_test");
    assert!(!parsed.clean_start());
    assert_eq!(parsed.keep_alive(), 120);
    assert!(parsed.user_name_flag());
    assert!(parsed.password_flag());
    assert_eq!(parsed.user_name().unwrap(), "testuser");
    assert_eq!(parsed.password().unwrap(), b"testpass");
    assert!(parsed.will_flag());
    assert_eq!(parsed.will_qos(), mqtt::packet::Qos::ExactlyOnce);
    assert!(parsed.will_retain());
    assert_eq!(parsed.will_topic().unwrap(), "will/topic");
    assert_eq!(parsed.will_payload().unwrap(), b"will_msg");
}

#[test]
fn parse_invalid_too_short() {
    common::init_tracing();
    let data = vec![0x00]; // Too short
    let err = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_protocol_name() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x04, b'M', b'Q', b'T', b'X']); // Wrong protocol name
    data.push(0x04); // version
    data.push(0x02); // flags
    data.extend_from_slice(&[0x00, 0x3C]); // keep alive
    data.extend_from_slice(&[0x00, 0x04]); // client id length
    data.extend_from_slice(b"test"); // client id

    let err = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_protocol_version() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x04, b'M', b'Q', b'T', b'T']); // protocol name
    data.push(0x05); // Wrong version (should be 0x04 for v3.1.1)
    data.push(0x02); // flags
    data.extend_from_slice(&[0x00, 0x3C]); // keep alive
    data.extend_from_slice(&[0x00, 0x04]); // client id length
    data.extend_from_slice(b"test"); // client id

    let err = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap_err();
    assert_eq!(
        err,
        mqtt::result_code::MqttError::UnsupportedProtocolVersion
    );
}

#[test]
fn parse_invalid_password_without_username() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x04, b'M', b'Q', b'T', b'T']); // protocol name
    data.push(0x04); // version
    data.push(0x40); // flags (password flag set but not username flag)
    data.extend_from_slice(&[0x00, 0x3C]); // keep alive
    data.extend_from_slice(&[0x00, 0x04]); // client id length
    data.extend_from_slice(b"test"); // client id
    data.extend_from_slice(&[0x00, 0x08]); // password length
    data.extend_from_slice(b"password"); // password

    let err = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_short_data() {
    common::init_tracing();
    // Test various incomplete packet scenarios
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x04, b'M', b'Q', b'T', b'T']); // protocol name
    data.push(0x04); // version
    data.push(0x02); // flags
                     // Missing keep alive and rest

    let err = mqtt::packet::v3_1_1::Connect::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Size tests
#[test]
fn size_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    let expected_size = 1 + 1 + // fixed header + remaining length
        6 + 1 + 1 + 2 + // protocol name + version + flags + keep alive
        2 + 4; // client id length + client id
    assert_eq!(packet.size(), expected_size);
}

#[test]
fn size_with_features() {
    common::init_tracing();
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test")
        .unwrap()
        .user_name("user")
        .unwrap()
        .password(b"pass")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .build()
        .unwrap();

    // Calculate expected size
    let variable_header_payload = 6 + 1 + 1 + 2 + // protocol name + version + flags + keep alive
        2 + 4 + // client id
        2 + 5 + 2 + 7 + // will topic + will payload
        2 + 4 + 2 + 4; // username + password

    let expected_size = 1 + 1 + variable_header_payload; // fixed header + remaining length + payload
    assert_eq!(packet.size(), expected_size);
}

#[test]
fn test_packet_type() {
    common::init_tracing();
    let packet_type = mqtt::packet::v3_1_1::Connect::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Connect);
}
