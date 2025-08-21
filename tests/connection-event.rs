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
use common::{mqtt, mqtt_pid32};

#[test]
fn test_timer_kind_values() {
    common::init_tracing();
    use mqtt::connection::TimerKind;

    // Test all timer kind variants exist
    let _pingreq_send = TimerKind::PingreqSend;
    let _pingreq_recv = TimerKind::PingreqRecv;
    let _pingresp_recv = TimerKind::PingrespRecv;
}

#[test]
fn test_timer_kind_equality() {
    common::init_tracing();
    use mqtt::connection::TimerKind;

    assert_eq!(TimerKind::PingreqSend, TimerKind::PingreqSend);
    assert_eq!(TimerKind::PingreqRecv, TimerKind::PingreqRecv);
    assert_eq!(TimerKind::PingrespRecv, TimerKind::PingrespRecv);

    assert_ne!(TimerKind::PingreqSend, TimerKind::PingreqRecv);
    assert_ne!(TimerKind::PingreqRecv, TimerKind::PingrespRecv);
    assert_ne!(TimerKind::PingreqSend, TimerKind::PingrespRecv);
}

#[test]
fn test_timer_kind_debug() {
    common::init_tracing();
    use mqtt::connection::TimerKind;

    let kind1 = TimerKind::PingreqSend;
    let kind2 = TimerKind::PingreqRecv;
    let kind3 = TimerKind::PingrespRecv;
    assert_eq!(format!("{kind1:?}"), "PingreqSend");
    assert_eq!(format!("{kind2:?}"), "PingreqRecv");
    assert_eq!(format!("{kind3:?}"), "PingrespRecv");
}

#[test]
fn test_timer_kind_serialize() {
    common::init_tracing();
    use mqtt::connection::TimerKind;

    let json1 = serde_json::to_string(&TimerKind::PingreqSend).unwrap();
    assert_eq!(json1, "\"pingreq_send\"");

    let json2 = serde_json::to_string(&TimerKind::PingreqRecv).unwrap();
    assert_eq!(json2, "\"pingreq_recv\"");

    let json3 = serde_json::to_string(&TimerKind::PingrespRecv).unwrap();
    assert_eq!(json3, "\"pingresp_recv\"");
}

#[test]
fn test_event_notify_packet_received() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::packet;

    let pingreq = packet::v5_0::Pingreq::new();
    let generic_packet = packet::GenericPacket::V5_0Pingreq(pingreq);
    let event = Event::NotifyPacketReceived(generic_packet.clone());

    match event {
        Event::NotifyPacketReceived(packet) => {
            assert_eq!(packet, generic_packet);
        }
        _ => panic!("Expected NotifyPacketReceived event"),
    }
}

#[test]
fn test_event_request_send_packet() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::packet;

    let pingresp = packet::v5_0::Pingresp::new();
    let generic_packet = packet::GenericPacket::V5_0Pingresp(pingresp);
    let event = Event::RequestSendPacket {
        packet: generic_packet.clone(),
        release_packet_id_if_send_error: Some(123),
    };

    match event {
        Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } => {
            assert_eq!(packet, generic_packet);
            assert_eq!(release_packet_id_if_send_error, Some(123));
        }
        _ => panic!("Expected RequestSendPacket event"),
    }
}

#[test]
fn test_event_request_send_packet_no_release_id() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::packet;

    let pingresp = packet::v5_0::Pingresp::new();
    let generic_packet = packet::GenericPacket::V5_0Pingresp(pingresp);
    let event = Event::RequestSendPacket {
        packet: generic_packet.clone(),
        release_packet_id_if_send_error: None,
    };

    match event {
        Event::RequestSendPacket {
            packet,
            release_packet_id_if_send_error,
        } => {
            assert_eq!(packet, generic_packet);
            assert_eq!(release_packet_id_if_send_error, None);
        }
        _ => panic!("Expected RequestSendPacket event"),
    }
}

#[test]
fn test_event_notify_packet_id_released() {
    common::init_tracing();
    use mqtt::connection::Event;

    let event = Event::NotifyPacketIdReleased(456);

    match event {
        Event::NotifyPacketIdReleased(packet_id) => {
            assert_eq!(packet_id, 456);
        }
        _ => panic!("Expected NotifyPacketIdReleased event"),
    }
}

#[test]
fn test_event_request_timer_reset() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    let event = Event::RequestTimerReset {
        kind: TimerKind::PingreqSend,
        duration_ms: 60000,
    };

    match event {
        Event::RequestTimerReset { kind, duration_ms } => {
            assert_eq!(kind, TimerKind::PingreqSend);
            assert_eq!(duration_ms, 60000);
        }
        _ => panic!("Expected RequestTimerReset event"),
    }
}

#[test]
fn test_event_request_timer_cancel() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    let event = Event::RequestTimerCancel(TimerKind::PingrespRecv);

    match event {
        Event::RequestTimerCancel(kind) => {
            assert_eq!(kind, TimerKind::PingrespRecv);
        }
        _ => panic!("Expected RequestTimerCancel event"),
    }
}

#[test]
fn test_event_notify_error() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::result_code::MqttError;

    let error = MqttError::MalformedPacket;
    let event = Event::NotifyError(error);

    match event {
        Event::NotifyError(err) => {
            assert_eq!(err, MqttError::MalformedPacket);
        }
        _ => panic!("Expected NotifyError event"),
    }
}

#[test]
fn test_event_request_close() {
    common::init_tracing();
    use mqtt::connection::Event;

    let event = Event::RequestClose;

    match event {
        Event::RequestClose => {
            // Success - this is what we expect
        }
        _ => panic!("Expected RequestClose event"),
    }
}

#[test]
fn test_event_clone() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    let event1 = Event::RequestTimerReset {
        kind: TimerKind::PingreqSend,
        duration_ms: 30000,
    };
    let event2 = event1.clone();

    match (&event1, &event2) {
        (
            Event::RequestTimerReset {
                kind: k1,
                duration_ms: d1,
            },
            Event::RequestTimerReset {
                kind: k2,
                duration_ms: d2,
            },
        ) => {
            assert_eq!(k1, k2);
            assert_eq!(d1, d2);
        }
        _ => panic!("Expected matching RequestTimerReset events"),
    }
}

#[test]
fn test_event_serialize_notify_packet_received() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::packet;

    let pingreq = packet::v5_0::Pingreq::new();
    let generic_packet = packet::GenericPacket::V5_0Pingreq(pingreq);
    let event = Event::NotifyPacketReceived(generic_packet);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"notify_packet_received\""));
    assert!(json.contains("\"packet\""));
}

#[test]
fn test_event_serialize_request_send_packet() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::packet;

    let pingresp = packet::v5_0::Pingresp::new();
    let generic_packet = packet::GenericPacket::V5_0Pingresp(pingresp);
    let event = Event::RequestSendPacket {
        packet: generic_packet,
        release_packet_id_if_send_error: Some(789),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"request_send_packet\""));
    assert!(json.contains("\"packet\""));
    assert!(json.contains("\"release_packet_id_if_send_error\":789"));
}

#[test]
fn test_event_serialize_request_send_packet_no_release_id() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::packet;

    let pingresp = packet::v5_0::Pingresp::new();
    let generic_packet = packet::GenericPacket::V5_0Pingresp(pingresp);
    let event = Event::RequestSendPacket {
        packet: generic_packet,
        release_packet_id_if_send_error: None,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"request_send_packet\""));
    assert!(json.contains("\"packet\""));
    assert!(json.contains("\"release_packet_id_if_send_error\":null"));
}

#[test]
fn test_event_serialize_notify_packet_id_released() {
    common::init_tracing();
    use mqtt::connection::Event;

    let event = Event::NotifyPacketIdReleased(101);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"notify_packet_id_released\""));
    assert!(json.contains("\"packet_id\":101"));
}

#[test]
fn test_event_serialize_request_timer_reset() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    let event = Event::RequestTimerReset {
        kind: TimerKind::PingreqRecv,
        duration_ms: 45000,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"request_timer_reset\""));
    assert!(json.contains("\"kind\":\"pingreq_recv\""));
    assert!(json.contains("\"duration_ms\":45000"));
}

#[test]
fn test_event_serialize_request_timer_cancel() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    let event = Event::RequestTimerCancel(TimerKind::PingrespRecv);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"request_timer_cancel\""));
    assert!(json.contains("\"kind\":\"pingresp_recv\""));
}

#[test]
fn test_event_serialize_notify_error() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::result_code::MqttError;

    let event = Event::NotifyError(MqttError::MalformedPacket);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"notify_error\""));
    assert!(json.contains("\"error\""));
    assert!(json.contains("MalformedPacket"));
}

#[test]
fn test_event_serialize_request_close() {
    common::init_tracing();
    use mqtt::connection::Event;

    let event = Event::RequestClose;

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"request_close\""));
}

#[test]
fn test_event_display() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    let event = Event::RequestTimerReset {
        kind: TimerKind::PingreqSend,
        duration_ms: 30000,
    };

    let display_str = format!("{event}");
    assert!(display_str.contains("request_timer_reset"));
    assert!(display_str.contains("pingreq_send"));
    assert!(display_str.contains("30000"));
}

#[test]
fn test_event_debug() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    let event = Event::RequestTimerCancel(TimerKind::PingrespRecv);

    let debug_str = format!("{event:?}");
    assert!(debug_str.contains("request_timer_cancel"));
    assert!(debug_str.contains("pingresp_recv"));
}

#[test]
fn test_generic_event_with_u32() {
    common::init_tracing();
    use mqtt_pid32::connection::Event;

    let event: Event = Event::NotifyPacketIdReleased(0x12345678);

    match event {
        Event::NotifyPacketIdReleased(packet_id) => {
            assert_eq!(packet_id, 0x12345678);
        }
        _ => panic!("Expected NotifyPacketIdReleased event"),
    }
}

#[test]
fn test_generic_event_serialize_with_u32() {
    common::init_tracing();
    use mqtt_pid32::connection::Event;

    let event: Event = Event::NotifyPacketIdReleased(0x87654321);

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"type\":\"notify_packet_id_released\""));
    assert!(json.contains("\"packet_id\":2271560481")); // 0x87654321 in decimal
}

#[test]
fn test_event_type_alias() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::result_code::MqttError;

    // Test that Event is indeed GenericEvent<u16>
    let event: Event = Event::NotifyError(MqttError::MalformedPacket);
    let _: mqtt::connection::Event = event;
}

#[test]
fn test_all_timer_kinds() {
    common::init_tracing();
    use mqtt::connection::{Event, TimerKind};

    // Test all timer kinds can be used in events
    let events = vec![
        Event::RequestTimerReset {
            kind: TimerKind::PingreqSend,
            duration_ms: 1000,
        },
        Event::RequestTimerReset {
            kind: TimerKind::PingreqRecv,
            duration_ms: 2000,
        },
        Event::RequestTimerReset {
            kind: TimerKind::PingrespRecv,
            duration_ms: 3000,
        },
        Event::RequestTimerCancel(TimerKind::PingreqSend),
        Event::RequestTimerCancel(TimerKind::PingreqRecv),
        Event::RequestTimerCancel(TimerKind::PingrespRecv),
    ];

    // Verify we can serialize all timer kinds
    for event in events {
        let _json = serde_json::to_string(&event).unwrap();
    }
}

#[test]
fn test_all_mqtt_errors_in_event() {
    common::init_tracing();
    use mqtt::connection::Event;
    use mqtt::result_code::MqttError;

    // Test various error types can be used in events
    let errors = vec![
        MqttError::MalformedPacket,
        MqttError::ProtocolError,
        MqttError::UnspecifiedError,
        MqttError::ImplementationSpecificError,
        MqttError::TopicFilterInvalid,
        MqttError::TopicNameInvalid,
        MqttError::ReceiveMaximumExceeded,
    ];

    for error in errors {
        let event = Event::NotifyError(error);
        let _json = serde_json::to_string(&event).unwrap();
        let _display = format!("{event}");
    }
}
