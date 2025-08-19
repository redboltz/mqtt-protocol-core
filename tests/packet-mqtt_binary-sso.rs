use mqtt_protocol_core::mqtt;

#[test]
fn test_small_binary_uses_stack() {
    let small_data = b"hello world"; // 11 bytes + 2 bytes length = 13 bytes total
    let binary = mqtt::packet::MqttBinary::new(small_data).unwrap();

    assert_eq!(binary.as_slice(), small_data);
    assert_eq!(binary.len(), 11);
    assert_eq!(binary.size(), 13); // 2 bytes prefix + 11 bytes data
    assert!(!binary.is_empty());
}

#[test]
fn test_large_binary_uses_heap() {
    let large_data = "x".repeat(100); // 100 bytes + 2 bytes length = 102 bytes > 32 bytes
    let binary = mqtt::packet::MqttBinary::new(large_data.as_bytes()).unwrap();

    assert_eq!(binary.as_slice(), large_data.as_bytes());
    assert_eq!(binary.len(), 100);
    assert_eq!(binary.size(), 102); // 2 bytes prefix + 100 bytes data
    assert!(!binary.is_empty());
}

#[test]
fn test_boundary_exactly_30_bytes_data() {
    let boundary_data = "a".repeat(30); // 30 bytes + 2 bytes length = 32 bytes exactly
    let binary = mqtt::packet::MqttBinary::new(boundary_data.as_bytes()).unwrap();

    assert_eq!(binary.as_slice(), boundary_data.as_bytes());
    assert_eq!(binary.len(), 30);
    assert_eq!(binary.size(), 32); // Should use stack at boundary
}

#[test]
fn test_boundary_31_bytes_data() {
    let boundary_data = "a".repeat(31); // 31 bytes + 2 bytes length = 33 bytes > 32 bytes
    let binary = mqtt::packet::MqttBinary::new(boundary_data.as_bytes()).unwrap();

    assert_eq!(binary.as_slice(), boundary_data.as_bytes());
    assert_eq!(binary.len(), 31);
    assert_eq!(binary.size(), 33); // Should use heap over boundary
}

#[test]
fn test_empty_binary() {
    let empty_binary = mqtt::packet::MqttBinary::new(&[]).unwrap();

    assert_eq!(empty_binary.as_slice(), &[] as &[u8]);
    assert_eq!(empty_binary.len(), 0);
    assert_eq!(empty_binary.size(), 2); // Only length prefix
    assert!(empty_binary.is_empty());
}

#[test]
fn test_binary_data_roundtrip() {
    let test_data = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x7F, 0x42];
    let binary = mqtt::packet::MqttBinary::new(&test_data).unwrap();

    assert_eq!(binary.as_slice(), test_data);
    assert_eq!(binary.len(), 7);
    assert_eq!(binary.size(), 9); // 2 bytes prefix + 7 bytes data

    // Test encode/decode roundtrip
    let encoded = binary.as_bytes();
    let (decoded, consumed) = mqtt::packet::MqttBinary::decode(encoded).unwrap();

    assert_eq!(decoded.as_slice(), test_data);
    assert_eq!(consumed, 9);
}

#[test]
fn test_generic_binary_with_different_sizes() {
    type Small16Binary = mqtt::packet::GenericMqttBinary<16>;
    type Large128Binary = mqtt::packet::GenericMqttBinary<128>;

    let data = b"hello world"; // 11 bytes + 2 prefix = 13 bytes total

    let small_binary = Small16Binary::new(data).unwrap();
    let large_binary = Large128Binary::new(data).unwrap();

    assert_eq!(small_binary.as_slice(), data);
    assert_eq!(large_binary.as_slice(), data);
    assert_eq!(small_binary.len(), 11);
    assert_eq!(large_binary.len(), 11);
    assert_eq!(small_binary.size(), 13);
    assert_eq!(large_binary.size(), 13);
}

#[test]
fn test_decode_boundary_conditions() {
    // Test decoding at stack/heap boundary
    let small_data = "a".repeat(30); // 30 bytes + 2 prefix = 32 bytes exactly
    let small_binary = mqtt::packet::MqttBinary::new(small_data.as_bytes()).unwrap();
    let encoded_small = small_binary.as_bytes();
    let (decoded_small, consumed_small) = mqtt::packet::MqttBinary::decode(encoded_small).unwrap();

    assert_eq!(decoded_small.as_slice(), small_data.as_bytes());
    assert_eq!(consumed_small, 32);

    let large_data = "a".repeat(31); // 31 bytes + 2 prefix = 33 bytes
    let large_binary = mqtt::packet::MqttBinary::new(large_data.as_bytes()).unwrap();
    let encoded_large = large_binary.as_bytes();
    let (decoded_large, consumed_large) = mqtt::packet::MqttBinary::decode(encoded_large).unwrap();

    assert_eq!(decoded_large.as_slice(), large_data.as_bytes());
    assert_eq!(consumed_large, 33);
}

#[test]
fn test_clone_and_equality() {
    let data = b"test data";
    let binary1 = mqtt::packet::MqttBinary::new(data).unwrap();
    let binary2 = binary1.clone();

    assert_eq!(binary1, binary2);
    assert_eq!(binary1.as_slice(), binary2.as_slice());

    let different_binary = mqtt::packet::MqttBinary::new(b"different").unwrap();
    assert_ne!(binary1, different_binary);
}

#[test]
fn test_stack_size_boundaries() {
    type Tiny8Binary = mqtt::packet::GenericMqttBinary<8>;
    type Medium32Binary = mqtt::packet::GenericMqttBinary<32>;
    type Large128Binary = mqtt::packet::GenericMqttBinary<128>;

    let test_cases = [(&b""[..], 0, 2), (b"a", 1, 3), (b"hello", 5, 7)];

    for (data, expected_len, expected_size) in test_cases {
        let tiny = Tiny8Binary::new(data).unwrap();
        let medium = Medium32Binary::new(data).unwrap();
        let large = Large128Binary::new(data).unwrap();

        assert_eq!(tiny.as_slice(), data);
        assert_eq!(medium.as_slice(), data);
        assert_eq!(large.as_slice(), data);

        assert_eq!(tiny.len(), expected_len);
        assert_eq!(medium.len(), expected_len);
        assert_eq!(large.len(), expected_len);

        assert_eq!(tiny.size(), expected_size);
        assert_eq!(medium.size(), expected_size);
        assert_eq!(large.size(), expected_size);
    }

    // Test boundary crossing
    let data10 = b"0123456789"; // 10 bytes + 2 prefix = 12 bytes
    let data50 = "a".repeat(50); // 50 bytes + 2 prefix = 52 bytes

    let tiny10 = Tiny8Binary::new(data10).unwrap(); // 12 > 8, should use heap
    let medium50 = Medium32Binary::new(data50.as_bytes()).unwrap(); // 52 > 32, should use heap
    let large50 = Large128Binary::new(data50.as_bytes()).unwrap(); // 52 < 128, should use stack

    assert_eq!(tiny10.len(), 10);
    assert_eq!(medium50.len(), 50);
    assert_eq!(large50.len(), 50);

    assert_eq!(tiny10.size(), 12);
    assert_eq!(medium50.size(), 52);
    assert_eq!(large50.size(), 52);
}

#[test]
fn test_default() {
    let default_binary = mqtt::packet::MqttBinary::default();

    assert_eq!(default_binary.as_slice(), &[] as &[u8]);
    assert_eq!(default_binary.len(), 0);
    assert_eq!(default_binary.size(), 2); // Only length prefix
    assert!(default_binary.is_empty());
}

#[test]
fn test_try_from_str() {
    let text = "hello world";
    let binary = mqtt::packet::MqttBinary::try_from(text).unwrap();

    assert_eq!(binary.as_slice(), text.as_bytes());
    assert_eq!(binary.len(), 11);
    assert_eq!(binary.size(), 13);
}

#[test]
fn test_as_bytes_vs_as_slice() {
    let data = b"test";
    let binary = mqtt::packet::MqttBinary::new(data).unwrap();

    let as_bytes = binary.as_bytes();
    let as_slice = binary.as_slice();

    // as_bytes should include length prefix
    assert_eq!(as_bytes.len(), 6); // 2 bytes prefix + 4 bytes data
    assert_eq!(as_bytes[0], 0x00); // length high byte
    assert_eq!(as_bytes[1], 0x04); // length low byte
    assert_eq!(&as_bytes[2..], data);

    // as_slice should be just the data
    assert_eq!(as_slice.len(), 4);
    assert_eq!(as_slice, data);
}
