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
use mqtt_protocol_core::mqtt;
mod common;

#[test]
fn test_escape_ascii() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::escape_binary_json_string(b"abc").unwrap(),
        "abc"
    );
}

#[test]
fn test_escape_specials() {
    common::init_tracing();
    // Valid UTF-8 special characters are preserved as-is (not pre-escaped)
    assert_eq!(
        mqtt::packet::escape_binary_json_string(b"\n\r\t\"\\").unwrap(),
        "\n\r\t\"\\"
    );
}

#[test]
fn test_escape_non_ascii() {
    common::init_tracing();
    // Use bytes that definitely cannot form valid UTF-8 (continuation bytes without start)
    let result = mqtt::packet::escape_binary_json_string(b"\x80\x81\x82\x83");
    assert_eq!(result, None);
}

#[test]
fn test_escape_valid_utf8() {
    common::init_tracing();
    // Valid UTF-8 string should be preserved as-is
    assert_eq!(
        mqtt::packet::escape_binary_json_string("Hello world".as_bytes()).unwrap(),
        "Hello world"
    );

    // Valid UTF-8 with Japanese characters
    assert_eq!(
        mqtt::packet::escape_binary_json_string("こんにちは".as_bytes()).unwrap(),
        "こんにちは"
    );
}
