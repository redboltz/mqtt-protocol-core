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
fn test_retain_handling_values() {
    common::init_tracing();
    assert_eq!(mqtt::packet::RetainHandling::SendRetained as u8, 0);
    assert_eq!(
        mqtt::packet::RetainHandling::SendRetainedIfNotExists as u8,
        1
    );
    assert_eq!(mqtt::packet::RetainHandling::DoNotSendRetained as u8, 2);
}

#[test]
fn test_retain_handling_try_from() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::RetainHandling::try_from(0u8).unwrap(),
        mqtt::packet::RetainHandling::SendRetained
    );
    assert_eq!(
        mqtt::packet::RetainHandling::try_from(1u8).unwrap(),
        mqtt::packet::RetainHandling::SendRetainedIfNotExists
    );
    assert_eq!(
        mqtt::packet::RetainHandling::try_from(2u8).unwrap(),
        mqtt::packet::RetainHandling::DoNotSendRetained
    );

    assert!(mqtt::packet::RetainHandling::try_from(3u8).is_err());
    assert!(mqtt::packet::RetainHandling::try_from(255u8).is_err());
}

#[test]
fn test_retain_handling_display() {
    common::init_tracing();
    assert_eq!(
        format!("{}", mqtt::packet::RetainHandling::SendRetained),
        "SendRetained"
    );
    assert_eq!(
        format!("{}", mqtt::packet::RetainHandling::SendRetainedIfNotExists),
        "SendRetainedIfNotExists"
    );
    assert_eq!(
        format!("{}", mqtt::packet::RetainHandling::DoNotSendRetained),
        "DoNotSendRetained"
    );
}

#[test]
fn test_retain_handling_default() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::RetainHandling::default(),
        mqtt::packet::RetainHandling::SendRetained
    );
}

#[test]
fn test_retain_handling_ordering() {
    common::init_tracing();
    assert!(
        mqtt::packet::RetainHandling::SendRetained
            < mqtt::packet::RetainHandling::SendRetainedIfNotExists
    );
    assert!(
        mqtt::packet::RetainHandling::SendRetainedIfNotExists
            < mqtt::packet::RetainHandling::DoNotSendRetained
    );
    assert!(
        mqtt::packet::RetainHandling::SendRetained
            < mqtt::packet::RetainHandling::DoNotSendRetained
    );
}

#[test]
fn test_retain_handling_equality() {
    common::init_tracing();
    assert_eq!(
        mqtt::packet::RetainHandling::SendRetained,
        mqtt::packet::RetainHandling::SendRetained
    );
    assert_eq!(
        mqtt::packet::RetainHandling::SendRetainedIfNotExists,
        mqtt::packet::RetainHandling::SendRetainedIfNotExists
    );
    assert_eq!(
        mqtt::packet::RetainHandling::DoNotSendRetained,
        mqtt::packet::RetainHandling::DoNotSendRetained
    );

    assert_ne!(
        mqtt::packet::RetainHandling::SendRetained,
        mqtt::packet::RetainHandling::SendRetainedIfNotExists
    );
    assert_ne!(
        mqtt::packet::RetainHandling::SendRetainedIfNotExists,
        mqtt::packet::RetainHandling::DoNotSendRetained
    );
}

#[test]
fn test_retain_handling_debug() {
    common::init_tracing();
    let rh = mqtt::packet::RetainHandling::DoNotSendRetained;
    let debug_str = format!("{rh:?}");
    assert!(debug_str.contains("DoNotSendRetained"));
}
