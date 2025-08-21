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

#[test]
fn test_mqttstring_creation() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("test").unwrap();
    assert_eq!(s.as_str(), "test");
    assert_eq!(s.len(), 4);
    assert_eq!(s.size(), 6); // 2 bytes length + 4 bytes content
}

#[test]
fn test_mqttstring_as_bytes() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("hi").unwrap();
    let bytes = s.as_bytes();
    assert_eq!(bytes, &[0x00, 0x02, b'h', b'i']);

    let empty_s = mqtt::packet::MqttString::new("").unwrap();
    let empty_bytes = empty_s.as_bytes();
    assert_eq!(empty_bytes, &[0x00, 0x00]);
}

#[test]
fn test_mqttstring_empty() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("").unwrap();
    assert_eq!(s.as_str(), "");
    assert_eq!(s.len(), 0);
    assert_eq!(s.size(), 2); // 2 bytes length only
    assert!(s.is_empty());
}

#[test]
fn test_mqttstring_decode() {
    common::init_tracing();
    let data = [0, 4, b't', b'e', b's', b't'];
    let (s, consumed) = mqtt::packet::MqttString::decode(&data).unwrap();
    assert_eq!(s.as_str(), "test");
    assert_eq!(consumed, 6);
}

#[test]
fn test_mqttstring_decode_insufficient_length() {
    common::init_tracing();
    // Test case where buffer is too short for declared string length
    let data = [0, 10, b't', b'e']; // Claims 10 bytes but only has 2
    let result = mqtt::packet::MqttString::decode(&data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );

    // Test case with only partial length header
    let partial_header = [0];
    let result = mqtt::packet::MqttString::decode(&partial_header);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

#[test]
fn test_mqttstring_decode_invalid_utf8() {
    common::init_tracing();
    // Test case with invalid UTF-8 sequence
    let data = [0, 4, 0xFF, 0xFE, 0xFD, 0xFC]; // Invalid UTF-8 bytes
    let result = mqtt::packet::MqttString::decode(&data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

#[test]
fn test_mqttstring_utf8() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("こんにちは").unwrap();
    assert_eq!(s.as_str(), "こんにちは");

    // 15 bytes as UTF-8
    assert_eq!(s.len(), "こんにちは".len());
    assert_eq!(s.size(), 2 + "こんにちは".len());
}

#[test]
fn test_mqttstring_operations() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("hello world").unwrap();
    assert!(s.contains('o'));
    assert!(s.starts_with("hello"));
    assert!(s.ends_with("world"));
    assert!(!s.contains('z'));
}

#[test]
fn test_mqttstring_comparison() {
    common::init_tracing();
    let s1 = mqtt::packet::MqttString::new("hello").unwrap();

    // Comparison with string literal (usually PartialEq<&str>)
    assert_eq!(s1, "hello");
    assert_ne!(s1, "world");

    // Comparison with String
    let string = "hello".to_string();
    assert_eq!(s1, string);

    // Comparison with another mqtt::packet::MqttString
    let s2 = mqtt::packet::MqttString::new("hello").unwrap();
    let s3 = mqtt::packet::MqttString::new("world").unwrap();
    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

#[test]
fn test_mqttstring_partial_eq_str() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("test").unwrap();

    // Test PartialEq<str> - this requires dereferencing a &str to get str
    let str_ref = "test";
    assert_eq!(s, *str_ref); // This uses PartialEq<str>
    assert_ne!(s, *"other"); // This uses PartialEq<str>

    // Another way to force PartialEq<str> usage with Box<str>
    let boxed_str: Box<str> = "test".into();
    assert_eq!(s, *boxed_str); // This uses PartialEq<str>

    // Test with String deref to str
    let string = String::from("test");
    assert_eq!(s, *string); // This dereferences String to str, using PartialEq<str>
}

#[test]
fn test_mqttstring_as_ref_str() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("test_string").unwrap();

    // Test AsRef<str> trait
    let str_ref: &str = s.as_ref();
    assert_eq!(str_ref, "test_string");

    // Test that AsRef works in generic contexts
    fn takes_str_ref<T: AsRef<str>>(value: T) -> usize {
        value.as_ref().len()
    }
    assert_eq!(takes_str_ref(s), 11);
}

#[test]
fn test_mqttstring_display() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("display_test").unwrap();

    // Test Display trait
    assert_eq!(format!("{s}"), "display_test");
    assert_eq!(format!("{s}"), "display_test");

    let empty_s = mqtt::packet::MqttString::new("").unwrap();
    assert_eq!(format!("{empty_s}"), "");
}

#[test]
fn test_mqttstring_try_from_string() {
    common::init_tracing();
    // Test TryFrom<String> trait
    let owned_string = "test_convert".to_string();
    let s: Result<mqtt::packet::MqttString, _> = owned_string.try_into();
    assert!(s.is_ok());
    assert_eq!(s.unwrap().as_str(), "test_convert");

    let empty_string = String::new();
    let empty_s: Result<mqtt::packet::MqttString, _> = empty_string.try_into();
    assert!(empty_s.is_ok());
    assert_eq!(empty_s.unwrap().as_str(), "");
}

#[test]
fn test_mqttstring_hash() {
    common::init_tracing();
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(mqtt::packet::MqttString::new("key1"), "value1");
    map.insert(mqtt::packet::MqttString::new("key2"), "value2");

    assert_eq!(
        map.get(&mqtt::packet::MqttString::new("key1")),
        Some(&"value1")
    );
    assert_eq!(
        map.get(&mqtt::packet::MqttString::new("key2")),
        Some(&"value2")
    );
    assert_eq!(map.get(&mqtt::packet::MqttString::new("key3")), None);
}

#[test]
fn test_mqttstring_serde() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("test_string").unwrap();

    // Serialize
    let serialized = serde_json::to_string(&s).unwrap();
    assert_eq!(serialized, "\"test_string\"");
}

#[test]
fn test_mqttstring_deref() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("test").unwrap();

    // Call standard string methods through Deref
    assert_eq!(s.len(), 4);
    assert_eq!(s.chars().count(), 4);
    assert_eq!(s.to_uppercase(), "TEST");
}

#[test]
fn test_mqttstring_buffers_equivalence() {
    common::init_tracing();
    let s = mqtt::packet::MqttString::new("test_data").unwrap();

    // Get buffers from both methods
    let continuous_buffer = s.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        let io_slices = s.to_buffers();

        // Concatenate IoSlice buffers
        let mut concatenated = Vec::new();
        for slice in io_slices {
            concatenated.extend_from_slice(&slice);
        }

        // Verify they produce identical results
        assert_eq!(continuous_buffer, concatenated);
    }

    // Verify the continuous buffer is not empty
    assert!(!continuous_buffer.is_empty());
}
