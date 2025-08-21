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

#[test]
fn test_cursor_new() {
    common::init_tracing();
    let data = &b"hello"[..];
    let cursor = mqtt::common::Cursor::new(data);
    assert_eq!(cursor.position(), 0);
    assert_eq!(cursor.get_ref(), &data);
}

#[test]
fn test_cursor_position_and_set_position() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello world"[..]);

    // Initial position should be 0
    assert_eq!(cursor.position(), 0);

    // Set position and verify
    cursor.set_position(5);
    assert_eq!(cursor.position(), 5);

    // Set position beyond data length
    cursor.set_position(100);
    assert_eq!(cursor.position(), 100);
}

#[test]
fn test_cursor_get_ref() {
    common::init_tracing();
    let data = &b"test data"[..];
    let cursor = mqtt::common::Cursor::new(data);
    assert_eq!(cursor.get_ref(), &data);
}

#[test]
fn test_remaining_slice() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello world"[..]);

    // Initially should return entire slice
    assert_eq!(cursor.remaining_slice(), b"hello world");

    // After setting position
    cursor.set_position(6);
    assert_eq!(cursor.remaining_slice(), b"world");

    // At end
    cursor.set_position(11);
    assert_eq!(cursor.remaining_slice(), b"");

    // Beyond end
    cursor.set_position(20);
    assert_eq!(cursor.remaining_slice(), b"");
}

#[test]
fn test_read_u8() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"abc"[..]);

    // Read each byte
    assert_eq!(cursor.read_u8(), Some(b'a'));
    assert_eq!(cursor.position(), 1);

    assert_eq!(cursor.read_u8(), Some(b'b'));
    assert_eq!(cursor.position(), 2);

    assert_eq!(cursor.read_u8(), Some(b'c'));
    assert_eq!(cursor.position(), 3);

    // Try to read beyond end
    assert_eq!(cursor.read_u8(), None);
    assert_eq!(cursor.position(), 3);
}

#[test]
fn test_read_bytes() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello world"[..]);

    // Read first 5 bytes
    let chunk = cursor.read_bytes(5);
    assert_eq!(chunk, Some(&b"hello"[..]));
    assert_eq!(cursor.position(), 5);

    // Read next byte (space)
    let chunk = cursor.read_bytes(1);
    assert_eq!(chunk, Some(&b" "[..]));
    assert_eq!(cursor.position(), 6);

    // Read remaining bytes
    let chunk = cursor.read_bytes(5);
    assert_eq!(chunk, Some(&b"world"[..]));
    assert_eq!(cursor.position(), 11);

    // Try to read beyond end
    let chunk = cursor.read_bytes(1);
    assert_eq!(chunk, None);
    assert_eq!(cursor.position(), 11); // Position shouldn't change on failure
}

#[test]
fn test_read_bytes_zero_length() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello"[..]);

    // Read zero bytes should succeed and return empty slice
    let chunk = cursor.read_bytes(0);
    assert_eq!(chunk, Some(&b""[..]));
    assert_eq!(cursor.position(), 0); // Position shouldn't change
}

#[test]
fn test_read_bytes_exact_length() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"test"[..]);

    // Read exactly the length of the data
    let chunk = cursor.read_bytes(4);
    assert_eq!(chunk, Some(&b"test"[..]));
    assert_eq!(cursor.position(), 4);

    // Try to read more
    let chunk = cursor.read_bytes(1);
    assert_eq!(chunk, None);
}

#[test]
fn test_read() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello world"[..]);
    let mut buf = [0u8; 5];

    // Read first 5 bytes
    let n = cursor.read(&mut buf).unwrap();
    assert_eq!(n, 5);
    assert_eq!(&buf, b"hello");
    assert_eq!(cursor.position(), 5);

    // Read next 3 bytes into smaller buffer
    let mut buf2 = [0u8; 3];
    let n = cursor.read(&mut buf2).unwrap();
    assert_eq!(n, 3);
    assert_eq!(&buf2, b" wo");
    assert_eq!(cursor.position(), 8);

    // Try to read more than available
    let mut buf3 = [0u8; 10];
    let n = cursor.read(&mut buf3).unwrap();
    assert_eq!(n, 3); // Only 3 bytes remaining
    assert_eq!(&buf3[..3], b"rld");
    assert_eq!(cursor.position(), 11);

    // Read from empty cursor
    let n = cursor.read(&mut buf3).unwrap();
    assert_eq!(n, 0);
    assert_eq!(cursor.position(), 11);
}

#[test]
fn test_read_exact() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello world"[..]);
    let mut buf = [0u8; 5];

    // Read exactly 5 bytes
    cursor.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"hello");
    assert_eq!(cursor.position(), 5);

    // Read exactly 1 byte
    let mut buf2 = [0u8; 1];
    cursor.read_exact(&mut buf2).unwrap();
    assert_eq!(&buf2, b" ");
    assert_eq!(cursor.position(), 6);

    // Try to read more than available
    let mut buf3 = [0u8; 10];
    let result = cursor.read_exact(&mut buf3);
    assert_eq!(result, Err(mqtt::common::CursorError::UnexpectedEof));
    assert_eq!(cursor.position(), 6); // Position shouldn't change on failure
}

#[test]
fn test_read_exact_zero_length() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello"[..]);
    let mut buf = [0u8; 0];

    // Read zero bytes should succeed
    cursor.read_exact(&mut buf).unwrap();
    assert_eq!(cursor.position(), 0); // Position shouldn't change
}

#[test]
fn test_read_exact_at_end() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"ab"[..]);
    cursor.set_position(2);

    let mut buf = [0u8; 1];
    let result = cursor.read_exact(&mut buf);
    assert_eq!(result, Err(mqtt::common::CursorError::UnexpectedEof));
}

#[test]
fn test_cursor_with_different_types() {
    common::init_tracing();
    // Test with Vec<u8>
    let data_vec = vec![1, 2, 3, 4, 5];
    let mut cursor_vec = mqtt::common::Cursor::new(data_vec);
    let mut buf = [0u8; 3];
    cursor_vec.read_exact(&mut buf).unwrap();
    assert_eq!(buf, [1, 2, 3]);

    // Test with String (as bytes)
    let data_string = "hello".to_string();
    let mut cursor_string = mqtt::common::Cursor::new(data_string);
    let mut buf2 = [0u8; 2];
    cursor_string.read_exact(&mut buf2).unwrap();
    assert_eq!(&buf2, b"he");
}

#[test]
fn test_cursor_error_debug() {
    common::init_tracing();
    let error = mqtt::common::CursorError::UnexpectedEof;
    let debug_str = format!("{error:?}");
    assert_eq!(debug_str, "UnexpectedEof");
}

#[test]
fn test_cursor_error_equality() {
    common::init_tracing();
    let error1 = mqtt::common::CursorError::UnexpectedEof;
    let error2 = mqtt::common::CursorError::UnexpectedEof;
    assert_eq!(error1, error2);
    assert_eq!(error1.clone(), error2);
}

#[test]
fn test_mixed_operations() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"abcdefghij"[..]);

    // Mix different read operations
    assert_eq!(cursor.read_u8(), Some(b'a'));
    assert_eq!(cursor.position(), 1);

    let chunk = cursor.read_bytes(3).unwrap();
    assert_eq!(chunk, b"bcd");
    assert_eq!(cursor.position(), 4);

    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf).unwrap();
    assert_eq!(&buf, b"ef");
    assert_eq!(cursor.position(), 6);

    assert_eq!(cursor.remaining_slice(), b"ghij");

    let mut buf2 = [0u8; 5];
    let n = cursor.read(&mut buf2).unwrap();
    assert_eq!(n, 4);
    assert_eq!(&buf2[..4], b"ghij");
    assert_eq!(cursor.position(), 10);
}

#[test]
fn test_position_overflow_safety() {
    common::init_tracing();
    let mut cursor = mqtt::common::Cursor::new(&b"hello"[..]);

    // Set position to very large value
    cursor.set_position(u64::MAX);
    assert_eq!(cursor.position(), u64::MAX);

    // Operations should handle overflow gracefully
    assert_eq!(cursor.remaining_slice(), b"");
    assert_eq!(cursor.read_u8(), None);
    assert_eq!(cursor.read_bytes(1), None);

    let mut buf = [0u8; 1];
    let result = cursor.read_exact(&mut buf);
    assert_eq!(result, Err(mqtt::common::CursorError::UnexpectedEof));

    let n = cursor.read(&mut buf).unwrap();
    assert_eq!(n, 0);
}
