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
// tests/store.rs
// Integration tests for Store in async-mqtt-rust crate
mod common;
use common::{mqtt, mqtt_pid32};

/// Helper to create a GenericStorePacket with given id for testing.
fn make_packet_u32(id: u32, qos: mqtt::packet::Qos) -> mqtt_pid32::packet::StorePacket {
    // Create a simple publish packet for testing
    let publish = mqtt_pid32::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("test payload")
        .qos(qos)
        .packet_id(id)
        .build()
        .unwrap();

    mqtt_pid32::packet::StorePacket::V3_1_1Publish(publish)
}

/// Helper to create a GenericStorePacket with u16 packet id.
fn make_packet_u16(id: u16, qos: mqtt::packet::Qos) -> mqtt::packet::StorePacket {
    // Create a simple publish packet for testing
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("test payload")
        .qos(qos)
        .packet_id(id)
        .build()
        .unwrap();

    mqtt::packet::StorePacket::V3_1_1Publish(publish)
}

#[test]
fn test_add_and_get_stored_order() {
    common::init_tracing();
    let mut store = mqtt_pid32::connection::Store::new();
    assert!(store
        .add(make_packet_u32(1, mqtt::packet::Qos::AtLeastOnce))
        .is_ok());
    assert!(store
        .add(make_packet_u32(2, mqtt::packet::Qos::ExactlyOnce))
        .is_ok());
    // Duplicate id should fail
    assert!(store
        .add(make_packet_u32(1, mqtt::packet::Qos::AtLeastOnce))
        .is_err());

    let stored = store.get_stored();
    let ids: Vec<u32> = stored.iter().map(|p| p.packet_id()).collect();
    assert_eq!(ids, vec![1, 2]);
}

#[test]
fn test_erase_by_response_and_id() {
    common::init_tracing();
    let mut store = mqtt_pid32::connection::Store::new();
    let p1 = make_packet_u32(10, mqtt::packet::Qos::ExactlyOnce); // Will expect V3_1_1Pubrec response
    let p2 = make_packet_u32(20, mqtt::packet::Qos::AtLeastOnce); // Will expect V3_1_1Puback response
    store.add(p1).unwrap();
    store.add(p2).unwrap();

    // Erase existing
    assert!(store.erase(mqtt::packet::ResponsePacket::V3_1_1Pubrec, 10));
    let stored = store.get_stored();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].packet_id(), 20);

    // Erase non-existing (wrong resp)
    assert!(!store.erase(mqtt::packet::ResponsePacket::V3_1_1Pubrec, 20));
    // Erase non-existing id
    assert!(!store.erase(mqtt::packet::ResponsePacket::V5_0Pubrec, 30));
}

#[test]
fn test_erase_publish() {
    common::init_tracing();
    let mut store = mqtt_pid32::connection::Store::new();
    // Only one matching packet
    store
        .add(make_packet_u32(42, mqtt::packet::Qos::ExactlyOnce))
        .unwrap();
    assert!(store.erase_publish(42));
    assert!(store.get_stored().is_empty());

    // erase_publish on empty
    assert!(!store.erase_publish(99));
}

#[test]
fn test_clear_and_for_each() {
    common::init_tracing();
    let mut store = mqtt_pid32::connection::Store::new();
    for i in 1..6 {
        // Start from 1 instead of 0 (packet ID 0 is invalid)
        // alternate QoS types
        let qos = if i % 2 == 0 {
            mqtt::packet::Qos::AtLeastOnce
        } else {
            mqtt::packet::Qos::ExactlyOnce
        };
        store.add(make_packet_u32(i, qos)).unwrap();
    }

    // for_each: keep only even IDs
    store.for_each(|packet| packet.packet_id() % 2 == 0);
    let remaining: Vec<u32> = store.get_stored().iter().map(|p| p.packet_id()).collect();
    assert_eq!(remaining, vec![2, 4]);

    // clear
    store.clear();
    assert!(store.get_stored().is_empty());
}

#[test]
fn test_store_type_alias() {
    common::init_tracing();
    // Test that Store is a type alias for GenericStore<u16>
    let mut store = mqtt::connection::Store::new();
    let packet = make_packet_u16(100, mqtt::packet::Qos::AtLeastOnce);

    assert!(store.add(packet).is_ok());
    let stored = store.get_stored();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].packet_id(), 100);
}
