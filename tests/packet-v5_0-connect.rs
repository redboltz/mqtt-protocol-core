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
mqtt_protocol_core::make_default_aliases!();
mod common;

// Build fail tests
#[test]
fn build_success_no_client_id() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .clean_start(true)
        .build()
        .unwrap();
    assert_eq!(packet.client_id(), "");
    assert!(packet.clean_start());
}

#[test]
fn build_fail_password_without_username() {
    common::init_tracing();
    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .password(b"password")
        .unwrap()
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_invalid_property() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::Binary)
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_duplicate_property() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::SessionExpiryInterval::new(100)
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::SessionExpiryInterval::new(200)
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_success_will_topic_with_empty_payload() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
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

#[test]
fn build_fail_invalid_will_property() {
    common::init_tracing();
    let mut will_props = mqtt::packet::GenericProperties::new();
    will_props.push(mqtt::packet::ReceiveMaximum::new(100).unwrap().into());

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .will_props(will_props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_various_connect_properties() {
    common::init_tracing();
    // Test all properties to cover validate_connect_properties (lines 580-582)

    // Test RequestResponseInformation property
    let mut props1 = mqtt::packet::GenericProperties::new();
    props1.push(
        mqtt::packet::RequestResponseInformation::new(1)
            .unwrap()
            .into(),
    );
    props1.push(
        mqtt::packet::RequestResponseInformation::new(0)
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props1)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);

    // Test RequestProblemInformation property
    let mut props2 = mqtt::packet::GenericProperties::new();
    props2.push(
        mqtt::packet::RequestProblemInformation::new(1)
            .unwrap()
            .into(),
    );
    props2.push(
        mqtt::packet::RequestProblemInformation::new(0)
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props2)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);

    // Test AuthenticationMethod property
    let mut props3 = mqtt::packet::GenericProperties::new();
    props3.push(
        mqtt::packet::AuthenticationMethod::new("method1")
            .unwrap()
            .into(),
    );
    props3.push(
        mqtt::packet::AuthenticationMethod::new("method2")
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props3)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);

    // Test AuthenticationData property
    let mut props4 = mqtt::packet::GenericProperties::new();
    props4.push(
        mqtt::packet::AuthenticationData::new(b"data1".to_vec())
            .unwrap()
            .into(),
    );
    props4.push(
        mqtt::packet::AuthenticationData::new(b"data2".to_vec())
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props4)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_various_will_properties() {
    common::init_tracing();
    // Test all will properties to cover validate_will_properties (lines 617-621)

    // Test MessageExpiryInterval property
    let mut will_props1 = mqtt::packet::GenericProperties::new();
    will_props1.push(
        mqtt::packet::MessageExpiryInterval::new(100)
            .unwrap()
            .into(),
    );
    will_props1.push(
        mqtt::packet::MessageExpiryInterval::new(200)
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .will_props(will_props1)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);

    // Test ContentType property
    let mut will_props2 = mqtt::packet::GenericProperties::new();
    will_props2.push(mqtt::packet::ContentType::new("type1").unwrap().into());
    will_props2.push(mqtt::packet::ContentType::new("type2").unwrap().into());

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .will_props(will_props2)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);

    // Test ResponseTopic property
    let mut will_props3 = mqtt::packet::GenericProperties::new();
    will_props3.push(mqtt::packet::ResponseTopic::new("topic1").unwrap().into());
    will_props3.push(mqtt::packet::ResponseTopic::new("topic2").unwrap().into());

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .will_props(will_props3)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);

    // Test CorrelationData property
    let mut will_props4 = mqtt::packet::GenericProperties::new();
    will_props4.push(
        mqtt::packet::CorrelationData::new(b"data1".to_vec())
            .unwrap()
            .into(),
    );
    will_props4.push(
        mqtt::packet::CorrelationData::new(b"data2".to_vec())
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .will_props(will_props4)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn build_fail_comprehensive_will_properties() {
    common::init_tracing();
    // Test to cover line 634 in validate_will_properties
    let mut will_props = mqtt::packet::GenericProperties::new();
    will_props.push(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String)
            .unwrap()
            .into(),
    );
    will_props.push(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::Binary)
            .unwrap()
            .into(),
    );

    let err = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtMostOnce, false)
        .unwrap()
        .will_props(will_props)
        .build()
        .unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

// Build success tests
#[test]
fn build_success_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
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
    assert!(packet.props().is_empty());
}

#[test]
fn build_success_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::SessionExpiryInterval::new(3600)
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::UserProperty::new("key", "value")
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_start(false)
        .keep_alive(60)
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "test_client");
    assert!(!packet.clean_start());
    assert_eq!(packet.keep_alive(), 60);
    assert_eq!(packet.props().len(), 2);
}

#[test]
fn build_success_with_credentials() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
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
    let mut will_props = mqtt::packet::GenericProperties::new();
    will_props.push(mqtt::packet::WillDelayInterval::new(30).unwrap().into());

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .will_message(
            "will/topic",
            b"will_payload",
            mqtt::packet::Qos::AtLeastOnce,
            true,
        )
        .unwrap()
        .will_props(will_props)
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "test_client");
    assert!(packet.will_flag());
    assert_eq!(packet.will_qos(), mqtt::packet::Qos::AtLeastOnce);
    assert!(packet.will_retain());
    assert_eq!(packet.will_topic().unwrap(), "will/topic");
    assert_eq!(packet.will_payload().unwrap(), b"will_payload");
    assert_eq!(packet.will_props().len(), 1);
}

#[test]
fn build_success_all_features_comprehensive() {
    common::init_tracing();
    // Test all possible builder settings and properties

    // Create comprehensive connect properties
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::SessionExpiryInterval::new(7200)
            .unwrap()
            .into(),
    );
    props.push(mqtt::packet::ReceiveMaximum::new(512).unwrap().into());
    props.push(mqtt::packet::MaximumPacketSize::new(65536).unwrap().into());
    props.push(mqtt::packet::TopicAliasMaximum::new(10).unwrap().into());
    props.push(
        mqtt::packet::RequestResponseInformation::new(1)
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::RequestProblemInformation::new(1)
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::AuthenticationMethod::new("SCRAM-SHA-256")
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::AuthenticationData::new(b"auth_data_123".to_vec())
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::UserProperty::new("client_type", "rust_client")
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::UserProperty::new("version", "1.0.0")
            .unwrap()
            .into(),
    );

    // Create comprehensive will properties
    let mut will_props = mqtt::packet::GenericProperties::new();
    will_props.push(mqtt::packet::WillDelayInterval::new(60).unwrap().into());
    will_props.push(
        mqtt::packet::PayloadFormatIndicator::new(mqtt::packet::PayloadFormat::String)
            .unwrap()
            .into(),
    );
    will_props.push(
        mqtt::packet::MessageExpiryInterval::new(3600)
            .unwrap()
            .into(),
    );
    will_props.push(
        mqtt::packet::ContentType::new("application/json")
            .unwrap()
            .into(),
    );
    will_props.push(
        mqtt::packet::ResponseTopic::new("response/topic")
            .unwrap()
            .into(),
    );
    will_props.push(
        mqtt::packet::CorrelationData::new(b"correlation_123".to_vec())
            .unwrap()
            .into(),
    );
    will_props.push(
        mqtt::packet::UserProperty::new("will_type", "last_will")
            .unwrap()
            .into(),
    );
    will_props.push(
        mqtt::packet::UserProperty::new("priority", "high")
            .unwrap()
            .into(),
    );

    // Build packet with all possible features
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("comprehensive_test_client_id")
        .unwrap()
        .clean_start(false)
        .keep_alive(600)
        .props(props)
        .user_name("test_user_name")
        .unwrap()
        .password(b"test_password_123")
        .unwrap()
        .will_message(
            "device/status/last_will",
            b"{\"status\":\"offline\",\"timestamp\":1640995200}",
            mqtt::packet::Qos::ExactlyOnce,
            true,
        )
        .unwrap()
        .will_props(will_props)
        .build()
        .unwrap();

    // Verify all basic settings
    assert_eq!(packet.client_id(), "comprehensive_test_client_id");
    assert!(!packet.clean_start());
    assert_eq!(packet.keep_alive(), 600);
    assert_eq!(packet.protocol_name(), "MQTT");
    assert_eq!(packet.protocol_version(), 5);

    // Verify flags
    assert!(packet.will_flag());
    assert_eq!(packet.will_qos(), mqtt::packet::Qos::ExactlyOnce);
    assert!(packet.will_retain());
    assert!(packet.user_name_flag());
    assert!(packet.password_flag());

    // Verify optional fields
    assert_eq!(packet.will_topic().unwrap(), "device/status/last_will");
    assert_eq!(
        packet.will_payload().unwrap(),
        b"{\"status\":\"offline\",\"timestamp\":1640995200}"
    );
    assert_eq!(packet.user_name().unwrap(), "test_user_name");
    assert_eq!(packet.password().unwrap(), b"test_password_123");

    // Verify properties count
    assert_eq!(packet.props().len(), 10); // 8 unique properties + 2 user properties
    assert_eq!(packet.will_props().len(), 8); // 6 unique properties + 2 user properties

    // Verify packet size calculation
    let size = packet.size();
    assert!(size > 100); // Should be reasonably large with all features

    #[cfg(feature = "std")]
    {
        // Verify packet can be serialized to buffers
        let buffers = packet.to_buffers();
        assert!(!buffers.is_empty());

        // Verify size matches actual buffer size
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);

        // Verify to_buffers() and to_continuous_buffer() produce same result
        let continuous = packet.to_continuous_buffer();
        let mut from_buffers = Vec::new();
        for buf in buffers {
            from_buffers.extend_from_slice(&buf);
        }
        assert_eq!(continuous, from_buffers);
    }
}

// Display tests
#[test]
fn display_minimal() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::SessionExpiryInterval::new(3600)
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"type\":\"connect\""));
    assert!(display_str.contains("\"client_id\":\"test\""));
    assert!(display_str.contains("\"clean_start\":true"));
    assert!(display_str.contains("\"keep_alive\":0"));
    assert!(display_str.contains("\"props\"")); // covers line 496 path where props is not empty
}

#[test]
fn display_with_will() {
    common::init_tracing();
    let mut will_props = mqtt::packet::GenericProperties::new();
    will_props.push(mqtt::packet::WillDelayInterval::new(30).unwrap().into());

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::AtLeastOnce, true)
        .unwrap()
        .will_props(will_props)
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains("\"will_qos\":\"AtLeastOnce\""));
    assert!(display_str.contains("\"will_retain\":true"));
    assert!(display_str.contains("\"will_topic\":\"topic\""));
    assert!(display_str.contains("\"will_props\"")); // covers line 522
}

#[test]
fn display_with_password_masked() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
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
fn display_with_small_will_payload() {
    common::init_tracing();
    // Create a small will payload to test hex formatting
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .will_message(
            "topic",
            &[0x01, 0x02, 0x03],
            mqtt::packet::Qos::AtMostOnce,
            false,
        )
        .unwrap()
        .build()
        .unwrap();

    let display_str = format!("{packet}");
    assert!(display_str.contains(r#"\u0001\u0002\u0003"#));
}

// Debug tests
#[test]
fn debug_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"type\":\"connect\""));
    assert!(debug_str.contains("\"client_id\":\"test\""));
}

#[test]
fn debug_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::UserProperty::new("test", "value")
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let debug_str = format!("{packet:?}");
    assert!(debug_str.contains("\"props\""));
}

// Getter tests
#[test]
fn getter_client_id() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("my_client_id")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.client_id(), "my_client_id");
}

#[test]
fn getter_optional_fields_none() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    // Test Option getters with flags not set (covers lines 136, 144, 152, 160)
    assert_eq!(packet.will_topic(), None);
    assert_eq!(packet.will_payload(), None);
    assert_eq!(packet.user_name(), None);
    assert_eq!(packet.password(), None);
}

#[test]
fn getter_flags() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .clean_start(false)
        .user_name("user")
        .unwrap()
        .password(b"pass")
        .unwrap()
        .will_message("topic", b"payload", mqtt::packet::Qos::ExactlyOnce, true)
        .unwrap()
        .build()
        .unwrap();

    assert!(!packet.clean_start());
    assert!(packet.will_flag());
    assert_eq!(packet.will_qos(), mqtt::packet::Qos::ExactlyOnce);
    assert!(packet.will_retain());
    assert!(packet.user_name_flag());
    assert!(packet.password_flag());

    // Test Option getters with will flag set (covers lines 136, 144, 152, 160)
    assert_eq!(packet.will_topic().unwrap(), "topic");
    assert_eq!(packet.will_payload().unwrap(), b"payload");
    assert_eq!(packet.user_name().unwrap(), "user");
    assert_eq!(packet.password().unwrap(), b"pass");
}

#[test]
fn getter_keep_alive() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .keep_alive(300)
        .build()
        .unwrap();

    assert_eq!(packet.keep_alive(), 300);
}

#[test]
fn getter_protocol_info() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    assert_eq!(packet.protocol_name(), "MQTT");
    assert_eq!(packet.protocol_version(), 5);
}

#[test]
fn getter_props_empty() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    assert!(packet.props().is_empty());
}

#[test]
fn getter_props_with_values() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::ReceiveMaximum::new(100).unwrap().into());
    props.push(
        mqtt::packet::UserProperty::new("key", "value")
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    assert_eq!(packet.props().len(), 2);
}

// to_buffers() tests
#[test]
#[cfg(feature = "std")]
fn to_buffers_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    assert!(!buffers.is_empty());

    // Collect all bytes
    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Check fixed header
    assert_eq!(all_bytes[0], 0x10); // CONNECT packet type

    // Should contain protocol name, version, flags, keep alive, properties, client ID
    assert!(all_bytes.len() > 15);

    // Check protocol name "MQTT"
    assert_eq!(&all_bytes[2..8], &[0x00, 0x04, b'M', b'Q', b'T', b'T']);

    // Check protocol version (5)
    assert_eq!(all_bytes[8], 0x05);

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    assert_eq!(all_bytes, continuous);
}

#[test]
#[cfg(feature = "std")]
fn to_buffers_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::SessionExpiryInterval::new(3600)
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .props(props)
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Should be larger than minimal case due to properties
    assert!(all_bytes.len() > 20);
    assert_eq!(all_bytes[0], 0x10);

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    assert_eq!(all_bytes, continuous);
}

#[test]
#[cfg(feature = "std")]
fn to_buffers_with_will() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
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

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Should contain will section
    assert!(all_bytes.len() > 30);

    // Check will flag is set (bit 2)
    let connect_flags = all_bytes[9];
    assert!((connect_flags & 0b0000_0100) != 0);

    // Check will QoS (bits 3-4)
    assert_eq!((connect_flags >> 3) & 0x03, 1); // AtLeastOnce

    // Check will retain (bit 5)
    assert!((connect_flags & 0b0010_0000) != 0);

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    assert_eq!(all_bytes, continuous);
}

#[test]
#[cfg(feature = "std")]
fn to_buffers_with_credentials() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .user_name("username")
        .unwrap()
        .password(b"password")
        .unwrap()
        .build()
        .unwrap();

    let buffers = packet.to_buffers();
    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(&buf);
    }

    // Check username flag (bit 7) and password flag (bit 6)
    let connect_flags = all_bytes[9];
    assert!((connect_flags & 0b1000_0000) != 0); // username flag
    assert!((connect_flags & 0b0100_0000) != 0); // password flag

    // Verify to_buffers() and to_continuous_buffer() produce same result
    let continuous = packet.to_continuous_buffer();
    assert_eq!(all_bytes, continuous);
}

// Parse tests
#[test]
fn parse_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let all_bytes = packet.to_continuous_buffer();
    // Skip fixed header (1 byte) and remaining length (variable)
    let mut skip_bytes = 1; // Fixed header
    if all_bytes.len() > 1 {
        let remaining_len = all_bytes[1];
        if remaining_len & 0x80 == 0 {
            skip_bytes += 1; // Single byte remaining length
        } else {
            skip_bytes += 2; // Multi-byte remaining length (simplified)
        }
    }
    let data = &all_bytes[skip_bytes..];

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter().skip(2) {
            // Skip fixed header and remaining length
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(data, buffers_data.as_slice());
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "test");
    assert!(parsed.clean_start());
    assert_eq!(parsed.keep_alive(), 0);
    assert!(parsed.props().is_empty());
}

#[test]
fn parse_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::ReceiveMaximum::new(100).unwrap().into());

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .keep_alive(60)
        .props(props)
        .build()
        .unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let all_bytes = packet.to_continuous_buffer();
    // Skip fixed header (1 byte) and remaining length (variable)
    let mut skip_bytes = 1; // Fixed header
    if all_bytes.len() > 1 {
        let remaining_len = all_bytes[1];
        if remaining_len & 0x80 == 0 {
            skip_bytes += 1; // Single byte remaining length
        } else {
            skip_bytes += 2; // Multi-byte remaining length (simplified)
        }
    }
    let data = &all_bytes[skip_bytes..];

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter().skip(2) {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(data, buffers_data.as_slice());
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "test_client");
    assert_eq!(parsed.keep_alive(), 60);
    assert_eq!(parsed.props().len(), 1);
}

#[test]
fn parse_with_credentials() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .user_name("username")
        .unwrap()
        .password(b"password")
        .unwrap()
        .build()
        .unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let all_bytes = packet.to_continuous_buffer();
    // Skip fixed header (1 byte) and remaining length (variable)
    let mut skip_bytes = 1; // Fixed header
    if all_bytes.len() > 1 {
        let remaining_len = all_bytes[1];
        if remaining_len & 0x80 == 0 {
            skip_bytes += 1; // Single byte remaining length
        } else {
            skip_bytes += 2; // Multi-byte remaining length (simplified)
        }
    }
    let data = &all_bytes[skip_bytes..];

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter().skip(2) {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(data, buffers_data.as_slice());
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "test");
    assert_eq!(parsed.user_name().unwrap(), "username");
    assert_eq!(parsed.password().unwrap(), b"password");
}

#[test]
fn parse_with_will() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
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

    // Use to_continuous_buffer for no-std compatibility
    let all_bytes = packet.to_continuous_buffer();
    // Skip fixed header (1 byte) and remaining length (variable)
    let mut skip_bytes = 1; // Fixed header
    if all_bytes.len() > 1 {
        let remaining_len = all_bytes[1];
        if remaining_len & 0x80 == 0 {
            skip_bytes += 1; // Single byte remaining length
        } else {
            skip_bytes += 2; // Multi-byte remaining length (simplified)
        }
    }
    let data = &all_bytes[skip_bytes..];

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = packet.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter().skip(2) {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(data, buffers_data.as_slice());
    }

    let (parsed, consumed) = mqtt::packet::v5_0::Connect::parse(&data).unwrap();
    assert_eq!(consumed, data.len());
    assert_eq!(parsed.client_id(), "test");
    assert!(parsed.will_flag());
    assert_eq!(parsed.will_qos(), mqtt::packet::Qos::AtLeastOnce);
    assert!(parsed.will_retain());
    assert_eq!(parsed.will_topic().unwrap(), "will/topic");
    assert_eq!(parsed.will_payload().unwrap(), b"will_payload");
}

#[test]
fn parse_invalid_too_short() {
    common::init_tracing();
    let data = [0x00]; // Too short for protocol name
    let err = mqtt::packet::v5_0::Connect::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

#[test]
fn parse_invalid_protocol_name() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x04]);
    data.extend_from_slice(b"FAKE"); // Wrong protocol name
    data.push(0x05); // version
    data.push(0x02); // flags
    data.extend_from_slice(&[0x00, 0x00]); // keep alive
    data.push(0x00); // properties length

    let err = mqtt::packet::v5_0::Connect::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_protocol_version() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x04]);
    data.extend_from_slice(b"MQTT");
    data.push(0x04); // wrong version (should be 5)
    data.push(0x02); // flags
    data.extend_from_slice(&[0x00, 0x00]); // keep alive
    data.push(0x00); // properties length

    let err = mqtt::packet::v5_0::Connect::parse(&data).unwrap_err();
    assert_eq!(
        err,
        mqtt::result_code::MqttError::UnsupportedProtocolVersion
    );
}

#[test]
fn parse_invalid_password_without_username() {
    common::init_tracing();
    let mut data = Vec::new();
    data.extend_from_slice(&[0x00, 0x04]);
    data.extend_from_slice(b"MQTT");
    data.push(0x05); // version
    data.push(0x02 | 0x40); // clean start + password flag but no username flag
    data.extend_from_slice(&[0x00, 0x00]); // keep alive
    data.push(0x00); // properties length
    data.extend_from_slice(&[0x00, 0x04]); // client id length
    data.extend_from_slice(b"test"); // client id
    data.extend_from_slice(&[0x00, 0x08]); // password length
    data.extend_from_slice(b"password"); // password

    let err = mqtt::packet::v5_0::Connect::parse(&data).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::ProtocolError);
}

#[test]
fn parse_invalid_short_data() {
    common::init_tracing();
    // Test various short data scenarios to cover lines 217, 228, 236

    // Too short for protocol version (covers line 217)
    let short_data1 = &[0x00, 0x04, b'M', b'Q', b'T', b'T'];
    let err = mqtt::packet::v5_0::Connect::parse(short_data1).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);

    // Too short for connect flags (covers line 228)
    let short_data2 = &[0x00, 0x04, b'M', b'Q', b'T', b'T', 0x05];
    let err = mqtt::packet::v5_0::Connect::parse(short_data2).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);

    // Too short for keep alive (covers line 236)
    let short_data3 = &[0x00, 0x04, b'M', b'Q', b'T', b'T', 0x05, 0x02];
    let err = mqtt::packet::v5_0::Connect::parse(short_data3).unwrap_err();
    assert_eq!(err, mqtt::result_code::MqttError::MalformedPacket);
}

// Size tests
#[test]
fn size_minimal() {
    common::init_tracing();
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    let size = packet.size();
    assert!(size > 0);

    #[cfg(feature = "std")]
    {
        // Verify size matches actual buffer size
        let buffers = packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

#[test]
fn size_with_properties() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::SessionExpiryInterval::new(3600)
            .unwrap()
            .into(),
    );
    props.push(
        mqtt::packet::UserProperty::new("key", "value")
            .unwrap()
            .into(),
    );

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .props(props)
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

#[test]
fn size_with_all_fields() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(mqtt::packet::ReceiveMaximum::new(100).unwrap().into());

    let mut will_props = mqtt::packet::GenericProperties::new();
    will_props.push(mqtt::packet::WillDelayInterval::new(30).unwrap().into());

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .user_name("username")
        .unwrap()
        .password(b"password")
        .unwrap()
        .will_message(
            "will/topic",
            b"will_payload",
            mqtt::packet::Qos::AtLeastOnce,
            true,
        )
        .unwrap()
        .props(props)
        .will_props(will_props)
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

// Roundtrip tests
#[test]
fn roundtrip_minimal() {
    common::init_tracing();
    let original = mqtt::packet::v5_0::Connect::builder()
        .client_id("test")
        .unwrap()
        .build()
        .unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let all_bytes = original.to_continuous_buffer();
    // Skip fixed header (1 byte) and remaining length (variable)
    let mut skip_bytes = 1; // Fixed header
    if all_bytes.len() > 1 {
        let remaining_len = all_bytes[1];
        if remaining_len & 0x80 == 0 {
            skip_bytes += 1; // Single byte remaining length
        } else {
            skip_bytes += 2; // Multi-byte remaining length (simplified)
        }
    }
    let data = &all_bytes[skip_bytes..];

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter().skip(2) {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(data, buffers_data.as_slice());
    }

    let (parsed, _) = mqtt::packet::v5_0::Connect::parse(&data).unwrap();
    assert_eq!(original.client_id(), parsed.client_id());
    assert_eq!(original.clean_start(), parsed.clean_start());
    assert_eq!(original.keep_alive(), parsed.keep_alive());
    assert_eq!(original.props().len(), parsed.props().len());
}

#[test]
fn roundtrip_with_all_features() {
    common::init_tracing();
    let mut props = mqtt::packet::GenericProperties::new();
    props.push(
        mqtt::packet::SessionExpiryInterval::new(3600)
            .unwrap()
            .into(),
    );
    props.push(mqtt::packet::ReceiveMaximum::new(100).unwrap().into());
    props.push(
        mqtt::packet::UserProperty::new("client", "test")
            .unwrap()
            .into(),
    );

    let mut will_props = mqtt::packet::GenericProperties::new();
    will_props.push(mqtt::packet::WillDelayInterval::new(30).unwrap().into());
    will_props.push(
        mqtt::packet::UserProperty::new("will", "test")
            .unwrap()
            .into(),
    );

    let original = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client_123")
        .unwrap()
        .clean_start(false)
        .keep_alive(300)
        .user_name("test_user")
        .unwrap()
        .password(b"test_password")
        .unwrap()
        .will_message(
            "device/status",
            b"offline",
            mqtt::packet::Qos::ExactlyOnce,
            true,
        )
        .unwrap()
        .props(props)
        .will_props(will_props)
        .build()
        .unwrap();

    // Use to_continuous_buffer for no-std compatibility
    let all_bytes = original.to_continuous_buffer();
    // Skip fixed header (1 byte) and remaining length (variable)
    let mut skip_bytes = 1; // Fixed header
    if all_bytes.len() > 1 {
        let remaining_len = all_bytes[1];
        if remaining_len & 0x80 == 0 {
            skip_bytes += 1; // Single byte remaining length
        } else {
            skip_bytes += 2; // Multi-byte remaining length (simplified)
        }
    }
    let data = &all_bytes[skip_bytes..];

    #[cfg(feature = "std")]
    {
        // Verify consistency with to_buffers()
        let buffers = original.to_buffers();
        let mut buffers_data = Vec::new();
        for buf in buffers.iter().skip(2) {
            buffers_data.extend_from_slice(buf);
        }
        assert_eq!(data, buffers_data.as_slice());
    }

    let (parsed, _) = mqtt::packet::v5_0::Connect::parse(&data).unwrap();
    assert_eq!(original.client_id(), parsed.client_id());
    assert_eq!(original.clean_start(), parsed.clean_start());
    assert_eq!(original.keep_alive(), parsed.keep_alive());
    assert_eq!(original.props().len(), parsed.props().len());
    assert_eq!(original.will_props().len(), parsed.will_props().len());
    assert_eq!(original.user_name(), parsed.user_name());
    assert_eq!(original.password(), parsed.password());
    assert_eq!(original.will_topic(), parsed.will_topic());
    assert_eq!(original.will_payload(), parsed.will_payload());
    assert_eq!(original.will_qos(), parsed.will_qos());
    assert_eq!(original.will_retain(), parsed.will_retain());
}

#[test]
fn test_packet_type() {
    common::init_tracing();
    let packet_type = mqtt::packet::v5_0::Connect::packet_type();
    assert_eq!(packet_type, mqtt::packet::PacketType::Connect);
}
