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
// tests/connection-pingreq-interval.rs
// Tests for set_pingreq_send_interval functionality
use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::prelude::*;

#[test]
fn test_set_pingreq_send_interval_enable() {
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);

    // Enable PINGREQ timer with 30 second interval
    let events = connection.set_pingreq_send_interval(30000);

    // Should return one TimerReset event
    assert_eq!(events.len(), 1);
    match &events[0] {
        GenericEvent::RequestTimerReset { kind, duration_ms } => {
            assert_eq!(kind, &TimerKind::PingreqSend);
            assert_eq!(duration_ms, &30000);
        }
        _ => panic!("Expected TimerReset event"),
    }
}

#[test]
fn test_set_pingreq_send_interval_disable() {
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);

    // First enable the timer
    connection.set_pingreq_send_interval(15000);

    // Then disable it with 0 duration
    let events = connection.set_pingreq_send_interval(0);

    // Should return one TimerCancel event
    assert_eq!(events.len(), 1);
    match &events[0] {
        GenericEvent::RequestTimerCancel(kind) => {
            assert_eq!(kind, &TimerKind::PingreqSend);
        }
        _ => panic!("Expected TimerCancel event"),
    }
}

#[test]
fn test_set_pingreq_send_interval_update() {
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Client, u16>::new(mqtt::Version::V5_0);

    // Set initial interval
    let events1 = connection.set_pingreq_send_interval(10000);
    assert_eq!(events1.len(), 1);

    // Update to different interval
    let events2 = connection.set_pingreq_send_interval(20000);
    assert_eq!(events2.len(), 1);
    match &events2[0] {
        GenericEvent::RequestTimerReset { kind, duration_ms } => {
            assert_eq!(kind, &TimerKind::PingreqSend);
            assert_eq!(duration_ms, &20000);
        }
        _ => panic!("Expected TimerReset event"),
    }
}

#[test]
fn test_set_pingreq_send_interval_server_role() {
    let mut connection =
        mqtt::GenericConnection::<mqtt::role::Server, u32>::new(mqtt::Version::V3_1_1);

    // Test with server role and u32 packet ID type
    let events = connection.set_pingreq_send_interval(45000);

    assert_eq!(events.len(), 1);
    match &events[0] {
        GenericEvent::RequestTimerReset { kind, duration_ms } => {
            assert_eq!(kind, &TimerKind::PingreqSend);
            assert_eq!(duration_ms, &45000);
        }
        _ => panic!("Expected TimerReset event"),
    }
}
