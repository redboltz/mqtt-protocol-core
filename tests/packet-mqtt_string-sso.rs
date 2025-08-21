mod common;
use common::mqtt;

#[test]
fn test_small_string_uses_stack() {
    let short_str = "a".repeat(30); // 30 chars + 2 bytes length prefix = 32 bytes total
    let mqtt_str = mqtt::packet::MqttString::new(&short_str).unwrap();

    assert_eq!(mqtt_str.as_str(), short_str);
    assert_eq!(mqtt_str.len(), 30);
    assert_eq!(mqtt_str.size(), 32);
    assert!(!mqtt_str.is_empty());

    let encoded = mqtt_str.as_bytes();
    assert_eq!(encoded.len(), 32);
    assert_eq!(encoded[0], 0x00);
    assert_eq!(encoded[1], 0x1E); // 30 in hex

    let (decoded, consumed) = mqtt::packet::MqttString::decode(encoded).unwrap();
    assert_eq!(decoded.as_str(), short_str);
    assert_eq!(consumed, 32);
}

#[test]
fn test_large_string_uses_heap() {
    let long_str = "b".repeat(100); // 100 chars + 2 bytes length prefix = 102 bytes total
    let mqtt_str = mqtt::packet::MqttString::new(&long_str).unwrap();

    assert_eq!(mqtt_str.as_str(), long_str);
    assert_eq!(mqtt_str.len(), 100);
    assert_eq!(mqtt_str.size(), 102);
    assert!(!mqtt_str.is_empty());

    let encoded = mqtt_str.as_bytes();
    assert_eq!(encoded.len(), 102);
    assert_eq!(encoded[0], 0x00);
    assert_eq!(encoded[1], 0x64); // 100 in hex

    let (decoded, consumed) = mqtt::packet::MqttString::decode(encoded).unwrap();
    assert_eq!(decoded.as_str(), long_str);
    assert_eq!(consumed, 102);
}

#[test]
fn test_boundary_string_exactly_32_bytes() {
    let boundary_str = "c".repeat(30); // exactly 32 bytes total
    let mqtt_str = mqtt::packet::MqttString::new(&boundary_str).unwrap();

    assert_eq!(mqtt_str.as_str(), boundary_str);
    assert_eq!(mqtt_str.len(), 30);
    assert_eq!(mqtt_str.size(), 32);

    let encoded = mqtt_str.as_bytes();
    assert_eq!(encoded.len(), 32);

    let (decoded, consumed) = mqtt::packet::MqttString::decode(encoded).unwrap();
    assert_eq!(decoded.as_str(), boundary_str);
    assert_eq!(consumed, 32);
}

#[test]
fn test_boundary_string_33_bytes() {
    let boundary_str = "d".repeat(31); // 33 bytes total
    let mqtt_str = mqtt::packet::MqttString::new(&boundary_str).unwrap();

    assert_eq!(mqtt_str.as_str(), boundary_str);
    assert_eq!(mqtt_str.len(), 31);
    assert_eq!(mqtt_str.size(), 33);

    let encoded = mqtt_str.as_bytes();
    assert_eq!(encoded.len(), 33);

    let (decoded, consumed) = mqtt::packet::MqttString::decode(encoded).unwrap();
    assert_eq!(decoded.as_str(), boundary_str);
    assert_eq!(consumed, 33);
}

#[test]
fn test_empty_string() {
    let mqtt_str = mqtt::packet::MqttString::new("").unwrap();

    assert_eq!(mqtt_str.as_str(), "");
    assert_eq!(mqtt_str.len(), 0);
    assert_eq!(mqtt_str.size(), 2);
    assert!(mqtt_str.is_empty());

    let encoded = mqtt_str.as_bytes();
    assert_eq!(encoded.len(), 2);
    assert_eq!(encoded[0], 0x00);
    assert_eq!(encoded[1], 0x00);

    let (decoded, consumed) = mqtt::packet::MqttString::decode(encoded).unwrap();
    assert_eq!(decoded.as_str(), "");
    assert_eq!(consumed, 2);
}

#[test]
fn test_utf8_string_small() {
    let utf8_str = "こんにちは"; // 5 Japanese chars = 15 bytes + 2 bytes length prefix = 17 bytes
    let mqtt_str = mqtt::packet::MqttString::new(utf8_str).unwrap();

    assert_eq!(mqtt_str.as_str(), utf8_str);
    assert_eq!(mqtt_str.len(), 15);
    assert_eq!(mqtt_str.size(), 17);

    let encoded = mqtt_str.as_bytes();
    assert_eq!(encoded.len(), 17);

    let (decoded, consumed) = mqtt::packet::MqttString::decode(encoded).unwrap();
    assert_eq!(decoded.as_str(), utf8_str);
    assert_eq!(consumed, 17);
}

#[test]
fn test_utf8_string_large() {
    let utf8_str = "こんにちは".repeat(10); // 50 Japanese chars = 150 bytes + 2 bytes length prefix = 152 bytes
    let mqtt_str = mqtt::packet::MqttString::new(&utf8_str).unwrap();

    assert_eq!(mqtt_str.as_str(), utf8_str);
    assert_eq!(mqtt_str.len(), 150);
    assert_eq!(mqtt_str.size(), 152);

    let encoded = mqtt_str.as_bytes();
    assert_eq!(encoded.len(), 152);

    let (decoded, consumed) = mqtt::packet::MqttString::decode(encoded).unwrap();
    assert_eq!(decoded.as_str(), utf8_str);
    assert_eq!(consumed, 152);
}

#[test]
fn test_clone_and_equality() {
    let small_str = "hello";
    let mqtt_str_small1 = mqtt::packet::MqttString::new(small_str).unwrap();
    let mqtt_str_small2 = mqtt_str_small1.clone();
    assert_eq!(mqtt_str_small1, mqtt_str_small2);

    let large_str = "x".repeat(50);
    let mqtt_str_large1 = mqtt::packet::MqttString::new(&large_str).unwrap();
    let mqtt_str_large2 = mqtt_str_large1.clone();
    assert_eq!(mqtt_str_large1, mqtt_str_large2);

    assert_ne!(mqtt_str_small1, mqtt_str_large1);
}

#[test]
fn test_default() {
    let default_str = mqtt::packet::MqttString::default();

    assert_eq!(default_str.as_str(), "");
    assert_eq!(default_str.len(), 0);
    assert_eq!(default_str.size(), 2);
    assert!(default_str.is_empty());

    let empty_str = mqtt::packet::MqttString::new("").unwrap();
    assert_eq!(default_str, empty_str);
}
