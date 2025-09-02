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
use common::*;

#[test]
fn client_recv_publish_qos0_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    v3_1_1_client_establish_connection(&mut connection, true, false);

    // Create and receive QoS1 PUBLISH A
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload A")
        .build()
        .unwrap();
    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, publish_a.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}

#[test]
fn client_recv_publish_qos0_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    v5_0_client_establish_connection(&mut connection);

    // Create and receive QoS1 PUBLISH A
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"payload A")
        .build()
        .unwrap();
    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(events.len(), 1);
    match &events[0] {
        mqtt::connection::Event::NotifyPacketReceived(packet) => {
            assert_eq!(*packet, publish_a.into());
        }
        _ => panic!("Expected NotifyPacketReceived event, got {:?}", events[1]),
    }
}
