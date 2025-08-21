
mod common;
use common::mqtt;

#[test]
fn test_generic_mqtt_string_with_different_sizes() {
    // Test 16-byte stack buffer
    type SmallString = mqtt::packet::GenericMqttString<16>;
    let small_str = SmallString::new("hello").unwrap();
    assert_eq!(small_str.as_str(), "hello");
    assert_eq!(small_str.size(), 7);

    // Test 64-byte stack buffer
    type LargeString = mqtt::packet::GenericMqttString<64>;
    let large_str = LargeString::new(&"a".repeat(50)).unwrap();
    assert_eq!(large_str.as_str(), "a".repeat(50));
    assert_eq!(large_str.size(), 52);
}

#[test]
fn test_custom_stack_sizes() {
    // Test 128-byte stack buffer
    type Custom128String = mqtt::packet::GenericMqttString<128>;
    let custom_str = Custom128String::new(&"a".repeat(100)).unwrap();
    assert_eq!(custom_str.as_str(), "a".repeat(100));
    assert_eq!(custom_str.size(), 102);
}

#[test]
fn test_default_vs_custom_sizes() {
    let default_str = mqtt::packet::MqttString::new("test").unwrap();

    type Custom16String = mqtt::packet::GenericMqttString<16>;
    let custom_str = Custom16String::new("test").unwrap();

    assert_eq!(default_str.as_str(), custom_str.as_str());
    assert_eq!(default_str.size(), custom_str.size());

    let boundary_str_30 = "a".repeat(30);

    let default_boundary = mqtt::packet::MqttString::new(&boundary_str_30).unwrap();
    assert_eq!(default_boundary.size(), 32);

    let custom_boundary = Custom16String::new(&boundary_str_30).unwrap();
    assert_eq!(custom_boundary.size(), 32);

    assert_eq!(default_boundary.as_str(), custom_boundary.as_str());
}

#[test]
fn test_stack_size_boundaries() {
    type Tiny8String = mqtt::packet::GenericMqttString<8>;
    type Medium32String = mqtt::packet::GenericMqttString<32>;
    type Large128String = mqtt::packet::GenericMqttString<128>;

    let test_cases = [("", 2), ("a", 3), ("hello", 7)];

    for (content, expected_size) in test_cases {
        let tiny = Tiny8String::new(content).unwrap();
        let medium = Medium32String::new(content).unwrap();
        let large = Large128String::new(content).unwrap();

        assert_eq!(tiny.as_str(), content);
        assert_eq!(medium.as_str(), content);
        assert_eq!(large.as_str(), content);

        assert_eq!(tiny.size(), expected_size);
        assert_eq!(medium.size(), expected_size);
        assert_eq!(large.size(), expected_size);
    }

    // Test longer strings
    let str10 = "a".repeat(10);
    let str30 = "a".repeat(30);
    let str100 = "a".repeat(100);

    let tiny10 = Tiny8String::new(&str10).unwrap();
    let medium30 = Medium32String::new(&str30).unwrap();
    let large100 = Large128String::new(&str100).unwrap();

    assert_eq!(tiny10.size(), 12);
    assert_eq!(medium30.size(), 32);
    assert_eq!(large100.size(), 102);
}

#[test]
fn test_compatibility_between_sizes() {
    let content = "test string for compatibility";

    let default_str = mqtt::packet::MqttString::new(content).unwrap();
    let custom64_str = mqtt::packet::GenericMqttString::<64>::new(content).unwrap();
    let custom16_str = mqtt::packet::GenericMqttString::<16>::new(content).unwrap();

    assert_eq!(default_str.as_bytes(), custom64_str.as_bytes());
    assert_eq!(default_str.as_bytes(), custom16_str.as_bytes());

    let encoded = default_str.as_bytes();

    let (decoded_default, _) = mqtt::packet::MqttString::decode(encoded).unwrap();
    let (decoded_64, _) = mqtt::packet::GenericMqttString::<64>::decode(encoded).unwrap();
    let (decoded_16, _) = mqtt::packet::GenericMqttString::<16>::decode(encoded).unwrap();

    assert_eq!(decoded_default.as_str(), content);
    assert_eq!(decoded_64.as_str(), content);
    assert_eq!(decoded_16.as_str(), content);
}
