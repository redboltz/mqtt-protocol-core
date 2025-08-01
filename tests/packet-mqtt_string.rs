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
use mqtt_protocol_core::mqtt::packet::MqttString;

#[test]
fn test_mqttstring_creation() {
    let s = MqttString::new("test").unwrap();
    assert_eq!(s.as_str(), "test");
    assert_eq!(s.len(), 4);
    assert_eq!(s.size(), 6); // 2 bytes length + 4 bytes content
}

#[test]
fn test_mqttstring_empty() {
    let s = MqttString::new("").unwrap();
    assert_eq!(s.as_str(), "");
    assert_eq!(s.len(), 0);
    assert_eq!(s.size(), 2); // 2 bytes length only
    assert!(s.is_empty());
}

#[test]
fn test_mqttstring_decode() {
    let data = [0, 4, b't', b'e', b's', b't'];
    let (s, consumed) = MqttString::decode(&data).unwrap();
    assert_eq!(s.as_str(), "test");
    assert_eq!(consumed, 6);
}

#[test]
fn test_mqttstring_utf8() {
    let s = MqttString::new("こんにちは").unwrap();
    assert_eq!(s.as_str(), "こんにちは");

    // 15 bytes as UTF-8
    assert_eq!(s.len(), "こんにちは".len());
    assert_eq!(s.size(), 2 + "こんにちは".len());
}

#[test]
fn test_mqttstring_operations() {
    let s = MqttString::new("hello world").unwrap();
    assert!(s.contains('o'));
    assert!(s.starts_with("hello"));
    assert!(s.ends_with("world"));
    assert!(!s.contains('z'));
}

#[test]
fn test_mqttstring_comparison() {
    let s1 = MqttString::new("hello").unwrap();

    // Comparison with string literal
    assert_eq!(s1, "hello");
    assert_ne!(s1, "world");

    // Comparison with String
    let string = "hello".to_string();
    assert_eq!(s1, string);

    // Comparison with another MqttString
    let s2 = MqttString::new("hello").unwrap();
    let s3 = MqttString::new("world").unwrap();
    assert_eq!(s1, s2);
    assert_ne!(s1, s3);
}

#[test]
fn test_mqttstring_hash() {
    use std::collections::HashMap;

    let mut map = HashMap::new();
    map.insert(MqttString::new("key1"), "value1");
    map.insert(MqttString::new("key2"), "value2");

    assert_eq!(map.get(&MqttString::new("key1")), Some(&"value1"));
    assert_eq!(map.get(&MqttString::new("key2")), Some(&"value2"));
    assert_eq!(map.get(&MqttString::new("key3")), None);
}

#[test]
fn test_mqttstring_serde() {
    let s = MqttString::new("test_string").unwrap();

    // Serialize
    let serialized = serde_json::to_string(&s).unwrap();
    assert_eq!(serialized, "\"test_string\"");
}

#[test]
fn test_mqttstring_deref() {
    let s = MqttString::new("test").unwrap();

    // Call standard string methods through Deref
    assert_eq!(s.len(), 4);
    assert_eq!(s.chars().count(), 4);
    assert_eq!(s.to_uppercase(), "TEST");
}
