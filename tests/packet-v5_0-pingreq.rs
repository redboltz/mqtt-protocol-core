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
use std::fmt::Write;

// Build tests

#[test]
fn build_success() {
    let packet = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    // PINGREQ has no fields to check
    assert_eq!(packet.size(), 2); // fixed header (1) + remaining length (1)
}

// Display tests

#[test]
fn display() {
    let packet = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    let mut output = String::new();
    write!(&mut output, "{}", packet).unwrap();
    assert_eq!(output, r#"{"type":"pingreq"}"#);
}

// Debug tests

#[test]
fn debug() {
    let packet = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    let mut output = String::new();
    write!(&mut output, "{:?}", packet).unwrap();
    assert_eq!(output, r#"{"type":"pingreq"}"#);
}

// to_buffers() tests

#[test]
fn to_buffers() {
    let packet = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    let buffers = packet.to_buffers();
    assert_eq!(buffers.len(), 2);
    assert_eq!(buffers[0].as_ref(), &[0xc0]); // fixed header (PINGREQ packet type)
    assert_eq!(buffers[1].as_ref(), &[0x00]); // remaining length (0)
    assert_eq!(packet.size(), 2); // 1 + 1
}

// Parse tests

#[test]
fn parse_empty() {
    let raw = &[];
    let (packet, consumed) = mqtt::packet::v5_0::Pingreq::parse(raw).unwrap();
    assert_eq!(consumed, 0);
    let expected = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();
    assert_eq!(packet, expected);
}

#[test]
fn parse_extra_data() {
    // PINGREQ should parse successfully even if there's extra data
    // The extra data is ignored as PINGREQ has no variable header or payload
    let raw = &[0x01, 0x02, 0x03];
    let (packet, consumed) = mqtt::packet::v5_0::Pingreq::parse(raw).unwrap();
    assert_eq!(consumed, 0); // No data consumed for PINGREQ
    let expected = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();
    assert_eq!(packet, expected);
}

// Size tests

#[test]
fn size() {
    let packet = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    // PINGREQ packet size is always 2 bytes:
    // - Fixed header: 1 byte (0xC0)
    // - Remaining length: 1 byte (0x00)
    assert_eq!(packet.size(), 2);
}

// Equality tests

#[test]
fn equality() {
    let packet1 = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    let packet2 = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    assert_eq!(packet1, packet2);
}

// Clone tests

#[test]
fn clone() {
    let packet1 = mqtt::packet::v5_0::Pingreq::builder().build().unwrap();

    let packet2 = packet1.clone();

    assert_eq!(packet1, packet2);
}
