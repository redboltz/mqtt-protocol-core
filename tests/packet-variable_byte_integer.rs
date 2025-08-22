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
use mqtt::packet::{DecodeResult, VariableByteInteger};

#[test]
fn test_encode_decode_normal_values() {
    common::init_tracing();
    let test_cases = [
        (0, vec![0x00]),
        (127, vec![0x7F]),
        (128, vec![0x80, 0x01]),
        (16383, vec![0xFF, 0x7F]),
        (16384, vec![0x80, 0x80, 0x01]),
        (2097151, vec![0xFF, 0xFF, 0x7F]),
        (2097152, vec![0x80, 0x80, 0x80, 0x01]),
        (268435455, vec![0xFF, 0xFF, 0xFF, 0x7F]),
    ];

    for (value, expected_bytes) in test_cases {
        let vbi = VariableByteInteger::from_u32(value).unwrap();
        assert_eq!(vbi.as_bytes(), expected_bytes.as_slice());

        match VariableByteInteger::decode_stream(&expected_bytes) {
            DecodeResult::Ok(decoded, size) => {
                assert_eq!(decoded.to_u32(), value);
                assert_eq!(size, expected_bytes.len());
            }
            other => panic!("decode_stream failed: {other:?}"),
        }
    }
}

#[test]
fn test_encode_too_large_value() {
    common::init_tracing();
    let result = VariableByteInteger::from_u32(268435456); // 1 over the max
    assert!(result.is_none());
}

#[test]
fn test_decode_invalid_length() {
    common::init_tracing();
    let bytes = vec![0x80, 0x80, 0x80, 0x80, 0x01]; // 5 bytes: invalid
    match VariableByteInteger::decode_stream(&bytes) {
        DecodeResult::Err(_) => {} // expected
        other => panic!("Expected Err, got {other:?}"),
    }
}

#[test]
fn test_decode_incomplete_sequence() {
    common::init_tracing();
    let bytes = vec![0x80, 0x80]; // not enough for termination
    match VariableByteInteger::decode_stream(&bytes) {
        DecodeResult::Incomplete => {} // expected
        other => panic!("Expected Incomplete, got {other:?}"),
    }
}

#[test]
#[cfg(feature = "std")]
fn test_to_buffers() {
    common::init_tracing();
    let vbi = VariableByteInteger::from_u32(128).unwrap(); // [0x80, 0x01]
    let buffers = vbi.to_buffers();
    assert_eq!(buffers.len(), 1);
    assert_eq!(buffers[0].len(), 2);
    assert_eq!(buffers[0][0], 0x80);
    assert_eq!(buffers[0][1], 0x01);
}

#[test]
fn test_decode_stream_value_too_large() {
    common::init_tracing();
    // Create a sequence that would decode to a value > MAX
    let bytes = vec![0x80, 0x80, 0x80, 0x80]; // This creates a value that's too large
    match VariableByteInteger::decode_stream(&bytes) {
        DecodeResult::Err(msg) => {
            assert!(msg.contains("too large") || msg.contains("too many bytes"));
        }
        other => panic!("Expected Err for value too large, got {other:?}"),
    }
}

#[test]
fn test_serialize() {
    common::init_tracing();
    let vbi = VariableByteInteger::from_u32(12345).unwrap();
    let serialized = serde_json::to_string(&vbi).unwrap();
    assert_eq!(serialized, "12345");
}

#[test]
fn test_display() {
    common::init_tracing();
    let vbi = VariableByteInteger::from_u32(42).unwrap();
    assert_eq!(format!("{vbi}"), "42");

    let vbi = VariableByteInteger::from_u32(268435455).unwrap(); // MAX value
    assert_eq!(format!("{vbi}"), "268435455");
}

#[test]
fn test_from_conversion() {
    common::init_tracing();
    let vbi = VariableByteInteger::from_u32(1000).unwrap();
    let value: u32 = vbi.into();
    assert_eq!(value, 1000);
}

#[test]
fn test_try_from_conversion() {
    common::init_tracing();
    // Test successful conversion
    let vbi = VariableByteInteger::try_from(500u32).unwrap();
    assert_eq!(vbi.to_u32(), 500);

    // Test failed conversion (value too large)
    let result = VariableByteInteger::try_from(268435456u32); // 1 over MAX
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Value too large");
}

#[test]
fn test_size_method() {
    common::init_tracing();
    let vbi1 = VariableByteInteger::from_u32(0).unwrap();
    assert_eq!(vbi1.size(), 1);

    let vbi2 = VariableByteInteger::from_u32(128).unwrap();
    assert_eq!(vbi2.size(), 2);

    let vbi3 = VariableByteInteger::from_u32(16384).unwrap();
    assert_eq!(vbi3.size(), 3);

    let vbi4 = VariableByteInteger::from_u32(2097152).unwrap();
    assert_eq!(vbi4.size(), 4);
}

#[test]
fn test_as_bytes_method() {
    common::init_tracing();
    let vbi = VariableByteInteger::from_u32(16383).unwrap(); // [0xFF, 0x7F]
    let bytes = vbi.as_bytes();
    assert_eq!(bytes, &[0xFF, 0x7F]);
}

#[test]
fn test_decode_result_variants() {
    common::init_tracing();
    // Test Incomplete case with short buffer
    let short_bytes = vec![0x80]; // Continuation bit set but no more data
    match VariableByteInteger::decode_stream(&short_bytes) {
        DecodeResult::Incomplete => {} // expected
        other => panic!("Expected Incomplete, got {other:?}"),
    }

    // Test Ok case
    let valid_bytes = vec![0x00]; // Simple case: value 0
    match VariableByteInteger::decode_stream(&valid_bytes) {
        DecodeResult::Ok(vbi, consumed) => {
            assert_eq!(vbi.to_u32(), 0);
            assert_eq!(consumed, 1);
        }
        other => panic!("Expected Ok, got {other:?}"),
    }
}

#[test]
fn test_max_constant() {
    common::init_tracing();
    assert_eq!(VariableByteInteger::MAX, 0x0FFF_FFFF);

    // Test that MAX value can be encoded
    let vbi = VariableByteInteger::from_u32(VariableByteInteger::MAX).unwrap();
    assert_eq!(vbi.to_u32(), VariableByteInteger::MAX);

    // Test that MAX + 1 cannot be encoded
    let result = VariableByteInteger::from_u32(VariableByteInteger::MAX + 1);
    assert!(result.is_none());
}
