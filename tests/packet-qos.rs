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
fn test_qos_values() {
    common::init_tracing();
    assert_eq!(mqtt::packet::Qos::AtMostOnce as u8, 0);
    assert_eq!(mqtt::packet::Qos::AtLeastOnce as u8, 1);
    assert_eq!(mqtt::packet::Qos::ExactlyOnce as u8, 2);
}

#[test]
fn test_qos_try_from() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::Qos::try_from(0u8).unwrap(),
        mqtt::packet::Qos::AtMostOnce
    );
    assert_eq!(
        mqtt::packet::Qos::try_from(1u8).unwrap(),
        mqtt::packet::Qos::AtLeastOnce
    );
    assert_eq!(
        mqtt::packet::Qos::try_from(2u8).unwrap(),
        mqtt::packet::Qos::ExactlyOnce
    );

    assert!(mqtt::packet::Qos::try_from(3u8).is_err());
    assert!(mqtt::packet::Qos::try_from(255u8).is_err());
}

#[test]
fn test_qos_display() {
    common::init_tracing();
    assert_eq!(format!("{}", mqtt::packet::Qos::AtMostOnce), "AtMostOnce");
    assert_eq!(format!("{}", mqtt::packet::Qos::AtLeastOnce), "AtLeastOnce");
    assert_eq!(format!("{}", mqtt::packet::Qos::ExactlyOnce), "ExactlyOnce");
}

#[test]
fn test_qos_ordering() {
    common::init_tracing();
    assert!(mqtt::packet::Qos::AtMostOnce < mqtt::packet::Qos::AtLeastOnce);
    assert!(mqtt::packet::Qos::AtLeastOnce < mqtt::packet::Qos::ExactlyOnce);
    assert!(mqtt::packet::Qos::AtMostOnce < mqtt::packet::Qos::ExactlyOnce);
}

#[test]
fn test_qos_equality() {
    common::init_tracing();
    assert_eq!(mqtt::packet::Qos::AtMostOnce, mqtt::packet::Qos::AtMostOnce);
    assert_eq!(
        mqtt::packet::Qos::AtLeastOnce,
        mqtt::packet::Qos::AtLeastOnce
    );
    assert_eq!(
        mqtt::packet::Qos::ExactlyOnce,
        mqtt::packet::Qos::ExactlyOnce
    );

    assert_ne!(
        mqtt::packet::Qos::AtMostOnce,
        mqtt::packet::Qos::AtLeastOnce
    );
    assert_ne!(
        mqtt::packet::Qos::AtLeastOnce,
        mqtt::packet::Qos::ExactlyOnce
    );
}

#[test]
fn test_qos_debug() {
    common::init_tracing();
    let qos = mqtt::packet::Qos::AtLeastOnce;
    let debug_str = format!("{qos:?}");
    assert!(debug_str.contains("AtLeastOnce"));
}
