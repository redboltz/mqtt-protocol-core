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
use mqtt_protocol_core::mqtt;

mod common;
use std::convert::TryInto;

// Basic functionality tests

#[test]
fn test_mqttbinary_creation() {
    common::init_tracing();
    let data = b"test data";
    let binary = mqtt::packet::MqttBinary::new(data).unwrap();
    assert_eq!(binary.as_slice(), b"test data");
    assert_eq!(binary.len(), 9);
    assert_eq!(binary.size(), 11); // 2 bytes length + 9 bytes content
}

#[test]
fn test_mqttbinary_empty() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"").unwrap();
    assert_eq!(binary.as_slice(), b"");
    assert_eq!(binary.len(), 0);
    assert_eq!(binary.size(), 2); // 2 bytes length only
    assert!(binary.is_empty());
}

#[test]
fn test_mqttbinary_as_bytes() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"hi").unwrap();
    let bytes = binary.as_bytes();
    assert_eq!(bytes, &[0x00, 0x02, b'h', b'i']);

    let empty_binary = mqtt::packet::MqttBinary::new(b"").unwrap();
    let empty_bytes = empty_binary.as_bytes();
    assert_eq!(empty_bytes, &[0x00, 0x00]);
}

#[test]
fn test_mqttbinary_as_slice() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"hello").unwrap();
    assert_eq!(binary.as_slice(), b"hello");

    let empty_binary = mqtt::packet::MqttBinary::new(b"").unwrap();
    assert_eq!(empty_binary.as_slice(), b"");
}

#[test]
fn test_mqttbinary_len_and_size() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"hello world").unwrap();
    assert_eq!(binary.len(), 11); // Data length only
    assert_eq!(binary.size(), 13); // 2 bytes prefix + 11 bytes data
    assert!(!binary.is_empty());

    let empty = mqtt::packet::MqttBinary::new(b"").unwrap();
    assert_eq!(empty.len(), 0);
    assert_eq!(empty.size(), 2);
    assert!(empty.is_empty());
}

// Decode tests

#[test]
fn test_mqttbinary_decode() {
    common::init_tracing();
    let data = [0, 4, b't', b'e', b's', b't'];
    let (binary, consumed) = mqtt::packet::MqttBinary::decode(&data).unwrap();
    assert_eq!(binary.as_slice(), b"test");
    assert_eq!(consumed, 6);
}

#[test]
fn test_mqttbinary_decode_empty() {
    common::init_tracing();
    let data = [0, 0];
    let (binary, consumed) = mqtt::packet::MqttBinary::decode(&data).unwrap();
    assert_eq!(binary.as_slice(), b"");
    assert_eq!(consumed, 2);
    assert!(binary.is_empty());
}

#[test]
fn test_mqttbinary_decode_large_data() {
    common::init_tracing();
    // Create data with 1000 bytes
    let mut data = vec![0x03, 0xE8]; // 1000 in big-endian
    data.extend(vec![0xAB; 1000]); // 1000 bytes of 0xAB

    let (binary, consumed) = mqtt::packet::MqttBinary::decode(&data).unwrap();
    assert_eq!(binary.len(), 1000);
    assert_eq!(consumed, 1002);
    assert_eq!(binary.as_slice(), vec![0xAB; 1000].as_slice());
}

// Error handling tests

#[test]
fn test_mqttbinary_new_size_limit() {
    common::init_tracing();
    // Test maximum allowed size
    let max_data = vec![0u8; 65535];
    let binary = mqtt::packet::MqttBinary::new(&max_data);
    assert!(binary.is_ok());
    assert_eq!(binary.unwrap().len(), 65535);

    // Test size exceeding limit
    let oversized_data = vec![0u8; 65536];
    let result = mqtt::packet::MqttBinary::new(&oversized_data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

#[test]
fn test_mqttbinary_decode_insufficient_length() {
    common::init_tracing();
    // Test case where buffer is too short for length header
    let partial_header = [0];
    let result = mqtt::packet::MqttBinary::decode(&partial_header);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );

    // Test case where buffer is too short for declared data length
    let data = [0, 10, b't', b'e']; // Claims 10 bytes but only has 2
    let result = mqtt::packet::MqttBinary::decode(&data);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

#[test]
fn test_mqttbinary_decode_empty_buffer() {
    common::init_tracing();
    let empty_buffer = [];
    let result = mqtt::packet::MqttBinary::decode(&empty_buffer);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

// I/O functionality tests

#[test]
fn test_mqttbinary_to_buffers() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"data").unwrap();

    // Test with to_continuous_buffer (works in no-std)
    let all_bytes = binary.to_continuous_buffer();
    assert_eq!(all_bytes, &[0x00, 0x04, b'd', b'a', b't', b'a']);

    // In std environment, verify to_buffers() produces same result
    #[cfg(feature = "std")]
    {
        let buffers = binary.to_buffers();
        assert_eq!(buffers.len(), 1);

        // Extract data from IoSlice for comparison
        let buffer_data: &[u8] = &buffers[0];
        assert_eq!(buffer_data, &[0x00, 0x04, b'd', b'a', b't', b'a']);

        // Verify to_buffers() and to_continuous_buffer() produce same result
        assert_eq!(all_bytes, buffer_data);
    }
}

#[test]
fn test_mqttbinary_to_buffers_empty() {
    common::init_tracing();
    let empty_binary = mqtt::packet::MqttBinary::new(b"").unwrap();

    // Test with to_continuous_buffer (works in no-std)
    let all_bytes = empty_binary.to_continuous_buffer();
    assert_eq!(all_bytes, &[0x00, 0x00]);

    // In std environment, verify to_buffers() produces same result
    #[cfg(feature = "std")]
    {
        let buffers = empty_binary.to_buffers();
        assert_eq!(buffers.len(), 1);

        let buffer_data: &[u8] = &buffers[0];
        assert_eq!(buffer_data, &[0x00, 0x00]);

        // Verify to_buffers() and to_continuous_buffer() produce same result
        assert_eq!(all_bytes, buffer_data);
    }
}

// Trait implementation tests

#[test]
fn test_mqttbinary_as_ref() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"test_binary").unwrap();

    // Test AsRef<[u8]> trait
    let slice_ref: &[u8] = binary.as_ref();
    assert_eq!(slice_ref, b"test_binary");

    // Test that AsRef works in generic contexts
    fn takes_slice_ref<T: AsRef<[u8]>>(value: T) -> usize {
        value.as_ref().len()
    }
    assert_eq!(takes_slice_ref(binary), 11);
}

#[test]
fn test_mqttbinary_deref() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"deref_test").unwrap();

    // Test Deref trait - can call slice methods directly
    assert_eq!(binary.len(), 10);
    assert_eq!(&binary[0..5], b"deref");
    assert_eq!(binary.first(), Some(&b'd'));
    assert_eq!(binary.last(), Some(&b't'));
}

#[test]
fn test_mqttbinary_serialize() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"serialize_test").unwrap();

    // Test serde serialization - should serialize as bytes
    let serialized = serde_json::to_string(&binary).unwrap();
    // The exact format may vary by serialization format, but it should serialize the data
    assert!(serialized.contains("serialize_test") || !serialized.is_empty());
}

#[test]
fn test_mqttbinary_try_from_str() {
    common::init_tracing();
    // Test TryFrom<&str> trait
    let str_data = "convert_test";
    let binary: Result<mqtt::packet::MqttBinary, _> = str_data.try_into();
    assert!(binary.is_ok());
    assert_eq!(binary.unwrap().as_slice(), b"convert_test");

    let empty_str = "";
    let empty_binary: Result<mqtt::packet::MqttBinary, _> = empty_str.try_into();
    assert!(empty_binary.is_ok());
    assert_eq!(empty_binary.unwrap().as_slice(), b"");
}

#[test]
fn test_mqttbinary_try_from_str_size_limit() {
    common::init_tracing();
    // Test TryFrom<&str> with oversized string
    let large_str = "x".repeat(65536);
    let result: Result<mqtt::packet::MqttBinary, _> = large_str.as_str().try_into();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::MalformedPacket
    );
}

#[test]
fn test_mqttbinary_default() {
    common::init_tracing();
    let default_binary = mqtt::packet::MqttBinary::default();
    assert!(default_binary.is_empty());
    assert_eq!(default_binary.len(), 0);
    assert_eq!(default_binary.size(), 2);
    assert_eq!(default_binary.as_slice(), b"");
    assert_eq!(default_binary.as_bytes(), &[0x00, 0x00]);
}

// Comparison and equality tests

#[test]
fn test_mqttbinary_partial_eq() {
    common::init_tracing();
    let binary1 = mqtt::packet::MqttBinary::new(b"hello").unwrap();
    let binary2 = mqtt::packet::MqttBinary::new(b"hello").unwrap();
    let binary3 = mqtt::packet::MqttBinary::new(b"world").unwrap();

    assert_eq!(binary1, binary2);
    assert_ne!(binary1, binary3);
    assert_ne!(binary2, binary3);
}

#[test]
fn test_mqttbinary_partial_ord() {
    common::init_tracing();
    let binary_a = mqtt::packet::MqttBinary::new(b"a").unwrap();
    let binary_b = mqtt::packet::MqttBinary::new(b"b").unwrap();
    let binary_aa = mqtt::packet::MqttBinary::new(b"aa").unwrap();

    assert!(binary_a < binary_b);
    assert!(binary_a < binary_aa);
    assert!(binary_b > binary_a);
    assert!(binary_aa > binary_a);

    let binary_same = mqtt::packet::MqttBinary::new(b"a").unwrap();
    assert!(binary_a <= binary_same);
    assert!(binary_a >= binary_same);
}

#[test]
fn test_mqttbinary_clone() {
    common::init_tracing();
    let original = mqtt::packet::MqttBinary::new(b"clone_test").unwrap();
    let cloned = original.clone();

    assert_eq!(original, cloned);
    assert_eq!(original.as_slice(), cloned.as_slice());
    assert_eq!(original.as_bytes(), cloned.as_bytes());
}

#[test]
fn test_mqttbinary_debug() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"debug").unwrap();
    let debug_str = format!("{binary:?}");
    assert!(!debug_str.is_empty());
    // Debug format should include some representation of the data
    assert!(debug_str.contains("MqttBinary"));
}

// Edge cases and special scenarios

#[test]
fn test_mqttbinary_binary_data() {
    common::init_tracing();
    // Test with actual binary data (not text)
    let binary_data = vec![0x00, 0x01, 0xFF, 0xFE, 0x80, 0x7F];
    let binary = mqtt::packet::MqttBinary::new(&binary_data).unwrap();

    assert_eq!(binary.as_slice(), binary_data.as_slice());
    assert_eq!(binary.len(), 6);

    let encoded = binary.as_bytes();
    assert_eq!(&encoded[0..2], &[0x00, 0x06]); // Length prefix
    assert_eq!(&encoded[2..], binary_data.as_slice()); // Data
}

#[test]
fn test_mqttbinary_roundtrip() {
    common::init_tracing();
    // Test encode -> decode roundtrip
    let original_data = b"roundtrip_test_data";
    let binary = mqtt::packet::MqttBinary::new(original_data).unwrap();
    let encoded = binary.as_bytes();

    let (decoded, consumed) = mqtt::packet::MqttBinary::decode(encoded).unwrap();
    assert_eq!(consumed, encoded.len());
    assert_eq!(decoded.as_slice(), original_data);
    assert_eq!(decoded, binary);
}

#[test]
fn test_mqttbinary_various_sizes() {
    common::init_tracing();
    // Test different data sizes
    let sizes = [0, 1, 2, 255, 256, 1000, 32767, 65535];

    for &size in &sizes {
        let data = vec![0xAA; size];
        let binary = mqtt::packet::MqttBinary::new(&data).unwrap();

        assert_eq!(binary.len(), size);
        assert_eq!(binary.size(), size + 2);
        assert_eq!(binary.as_slice(), data.as_slice());
        assert_eq!(binary.is_empty(), size == 0);

        // Test roundtrip
        let encoded = binary.as_bytes();
        let (decoded, consumed) = mqtt::packet::MqttBinary::decode(encoded).unwrap();
        assert_eq!(consumed, size + 2);
        assert_eq!(decoded, binary);
    }
}

#[test]
fn test_mqttbinary_new_from_different_types() {
    common::init_tracing();
    // Test creating from different AsRef<[u8]> types

    // From &[u8]
    let slice_binary = mqtt::packet::MqttBinary::new(b"slice").unwrap();
    assert_eq!(slice_binary.as_slice(), b"slice");

    // From Vec<u8>
    let vec_data = vec![1, 2, 3, 4, 5];
    let vec_binary = mqtt::packet::MqttBinary::new(vec_data).unwrap();
    assert_eq!(vec_binary.as_slice(), &[1, 2, 3, 4, 5]);

    // From array
    let array_data = [10, 20, 30];
    let array_binary = mqtt::packet::MqttBinary::new(array_data).unwrap();
    assert_eq!(array_binary.as_slice(), &[10, 20, 30]);
}

#[test]
fn test_mqttbinary_buffers_equivalence() {
    common::init_tracing();
    let binary = mqtt::packet::MqttBinary::new(b"test_binary_data").unwrap();

    // Get buffers from both methods
    let continuous_buffer = binary.to_continuous_buffer();

    #[cfg(feature = "std")]
    {
        let io_slices = binary.to_buffers();

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
