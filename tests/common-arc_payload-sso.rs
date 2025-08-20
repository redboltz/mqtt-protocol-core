extern crate alloc;
use alloc::sync::Arc;
use mqtt_protocol_core::default_alias;
use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::common::{GenericArcPayload, IntoPayload};

#[test]
fn test_small_payload_uses_stack() {
    let small_data = b"hello world"; // 11 bytes
    let payload = mqtt::ArcPayload::from(small_data);

    assert_eq!(payload.as_slice(), small_data);
    assert_eq!(payload.len(), 11);
    assert!(!payload.is_empty());
    assert!(payload.arc_data().is_none()); // Small payload, no Arc
}

#[test]
fn test_large_payload_uses_heap() {
    let large_data = "x".repeat(100); // 100 bytes > 32 bytes
    let payload = mqtt::ArcPayload::from(large_data.as_bytes());

    assert_eq!(payload.as_slice(), large_data.as_bytes());
    assert_eq!(payload.len(), 100);
    assert!(!payload.is_empty());
    assert!(payload.arc_data().is_some()); // Large payload, has Arc
}

#[test]
fn test_boundary_exactly_32_bytes() {
    let boundary_data = "a".repeat(32); // exactly 32 bytes
    let payload = mqtt::ArcPayload::from(boundary_data.as_bytes());

    assert_eq!(payload.as_slice(), boundary_data.as_bytes());
    assert_eq!(payload.len(), 32);
    assert!(payload.arc_data().is_none()); // Should use stack at boundary
}

#[test]
fn test_boundary_33_bytes() {
    let boundary_data = "a".repeat(33); // 33 bytes > 32 bytes
    let payload = mqtt::ArcPayload::from(boundary_data.as_bytes());

    assert_eq!(payload.as_slice(), boundary_data.as_bytes());
    assert_eq!(payload.len(), 33);
    assert!(payload.arc_data().is_some()); // Should use heap over boundary
}

#[test]
fn test_empty_payload() {
    let empty_payload = mqtt::ArcPayload::from(&[]);

    assert_eq!(empty_payload.as_slice(), &[] as &[u8]);
    assert_eq!(empty_payload.len(), 0);
    assert!(empty_payload.is_empty());
    assert!(empty_payload.arc_data().is_none()); // Empty uses stack
}

#[test]
fn test_generic_payload_with_different_sizes() {
    type Small16Payload = GenericArcPayload<16>;
    type Large128Payload = GenericArcPayload<128>;

    let data = b"hello world"; // 11 bytes

    let small_payload = Small16Payload::from(data);
    let large_payload = Large128Payload::from(data);

    assert_eq!(small_payload.as_slice(), data);
    assert_eq!(large_payload.as_slice(), data);
    assert_eq!(small_payload.len(), 11);
    assert_eq!(large_payload.len(), 11);

    // Both should use stack for this size
    assert!(small_payload.arc_data().is_none());
    assert!(large_payload.arc_data().is_none());
}

#[test]
fn test_arc_new_method() {
    let data: Arc<[u8]> = Arc::from(&b"hello world"[..]);
    let payload = mqtt::ArcPayload::new(data.clone(), 0, 11);

    assert_eq!(payload.as_slice(), b"hello world");
    assert_eq!(payload.len(), 11);

    // Test slice of Arc data
    let slice_payload = mqtt::ArcPayload::new(data.clone(), 6, 5); // "world"
    assert_eq!(slice_payload.as_slice(), b"world");
    assert_eq!(slice_payload.len(), 5);
}

#[test]
fn test_into_payload_trait() {
    // Test various types using IntoPayload
    let str_payload: mqtt::ArcPayload = "hello".into_payload();
    assert_eq!(str_payload.as_slice(), b"hello");

    let string_payload: mqtt::ArcPayload = String::from("world").into_payload();
    assert_eq!(string_payload.as_slice(), b"world");

    let bytes_payload: mqtt::ArcPayload = b"test".as_slice().into_payload();
    assert_eq!(bytes_payload.as_slice(), b"test");

    let vec_payload: mqtt::ArcPayload = vec![1, 2, 3, 4].into_payload();
    assert_eq!(vec_payload.as_slice(), &[1, 2, 3, 4]);

    let array_payload: mqtt::ArcPayload = [5u8, 6, 7][..].into_payload();
    assert_eq!(array_payload.as_slice(), &[5, 6, 7]);

    let empty_payload: mqtt::ArcPayload = ().into_payload();
    assert!(empty_payload.is_empty());
}

#[test]
fn test_clone_and_equality() {
    let data = b"test data";
    let payload1 = mqtt::ArcPayload::from(data);
    let payload2 = payload1.clone();

    assert_eq!(payload1, payload2);
    assert_eq!(payload1.as_slice(), payload2.as_slice());

    let different_payload = mqtt::ArcPayload::from(b"different");
    assert_ne!(payload1, different_payload);
}

#[test]
fn test_stack_size_boundaries() {
    type Tiny8Payload = GenericArcPayload<8>;
    type Medium32Payload = GenericArcPayload<32>;
    type Large128Payload = GenericArcPayload<128>;

    let test_cases = [(&b""[..], 0), (b"a", 1), (b"hello", 5)];

    for (data, expected_len) in test_cases {
        let tiny = Tiny8Payload::from(data);
        let medium = Medium32Payload::from(data);
        let large = Large128Payload::from(data);

        assert_eq!(tiny.as_slice(), data);
        assert_eq!(medium.as_slice(), data);
        assert_eq!(large.as_slice(), data);

        assert_eq!(tiny.len(), expected_len);
        assert_eq!(medium.len(), expected_len);
        assert_eq!(large.len(), expected_len);
    }

    // Test boundary crossing
    let data10 = b"0123456789"; // 10 bytes
    let data50 = "a".repeat(50);

    let tiny10 = Tiny8Payload::from(data10); // > 8, should use heap
    let medium50 = Medium32Payload::from(data50.as_bytes()); // > 32, should use heap
    let large50 = Large128Payload::from(data50.as_bytes()); // < 128, should use stack

    assert_eq!(tiny10.len(), 10);
    assert_eq!(medium50.len(), 50);
    assert_eq!(large50.len(), 50);

    assert!(tiny10.arc_data().is_some()); // Uses heap
    assert!(medium50.arc_data().is_some()); // Uses heap
    assert!(large50.arc_data().is_none()); // Uses stack
}

#[test]
fn test_default() {
    let default_payload = mqtt::ArcPayload::default();

    assert_eq!(default_payload.as_slice(), &[] as &[u8]);
    assert_eq!(default_payload.len(), 0);
    assert!(default_payload.is_empty());
    assert!(default_payload.arc_data().is_none());
}
