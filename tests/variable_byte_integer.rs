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
use mqtt_protocol_core::mqtt::packet::{DecodeResult, VariableByteInteger};

#[test]
fn test_encode_decode_normal_values() {
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
            other => panic!("decode_stream failed: {:?}", other),
        }
    }
}

#[test]
fn test_encode_too_large_value() {
    let result = VariableByteInteger::from_u32(268435456); // 1 over the max
    assert!(result.is_none());
}

#[test]
fn test_decode_invalid_length() {
    let bytes = vec![0x80, 0x80, 0x80, 0x80, 0x01]; // 5 bytes: invalid
    match VariableByteInteger::decode_stream(&bytes) {
        DecodeResult::Err(_) => {} // expected
        other => panic!("Expected Err, got {:?}", other),
    }
}

#[test]
fn test_decode_incomplete_sequence() {
    let bytes = vec![0x80, 0x80]; // not enough for termination
    match VariableByteInteger::decode_stream(&bytes) {
        DecodeResult::Incomplete => {} // expected
        other => panic!("Expected Incomplete, got {:?}", other),
    }
}
