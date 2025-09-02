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

#[test]
fn receive_keep_alive_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);

    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .clean_session(false)
        .keep_alive(1u16)
        .build()
        .unwrap();
    let bytes = connect.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 2);

    // Check RequestTimerReset event
    if let mqtt::connection::Event::RequestTimerReset { kind, duration_ms } = &events[0] {
        assert_eq!(*kind, mqtt::connection::TimerKind::PingreqRecv);
        assert_eq!(*duration_ms, 1500);
    } else {
        panic!("Expected RequestTimerReset event, got: {:?}", events[0]);
    }

    // Check NotifyPacketReceived event for connect
    if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[1] {
        if let mqtt::packet::GenericPacket::V3_1_1Connect(connect_received) = packet {
            assert_eq!(connect_received.client_id(), "test_client");
            assert_eq!(connect_received.clean_session(), false);
            assert_eq!(connect_received.keep_alive(), 1u16);
        } else {
            panic!("Expected V3_1_1Connect packet, got: {packet:?}");
        }
    } else {
        panic!("Expected NotifyPacketReceived event, got: {:?}", events[1]);
    }
}
