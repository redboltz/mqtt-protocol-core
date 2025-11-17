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
use mqtt_protocol_core::mqtt::connection::{GenericStore, Store};
use mqtt_protocol_core::mqtt::packet::v3_1_1;
use mqtt_protocol_core::mqtt::packet::Qos;
use mqtt_protocol_core::mqtt::packet::{GenericStorePacket, ResponsePacket};
mod common;

/// Helper to create a GenericStorePacket with given id for testing.
fn make_packet_u32(id: u32, qos: Qos) -> GenericStorePacket<u32> {
    // Create a simple publish packet for testing
    let publish = v3_1_1::GenericPublish::<u32>::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("test payload")
        .qos(qos)
        .packet_id(Some(id))
        .build()
        .unwrap();

    GenericStorePacket::V3_1_1Publish(publish)
}

/// Helper to create a GenericStorePacket with u16 packet id.
fn make_packet_u16(id: u16, qos: Qos) -> GenericStorePacket<u16> {
    // Create a simple publish packet for testing
    let publish = v3_1_1::GenericPublish::<u16>::builder()
        .topic_name("test/topic")
        .unwrap()
        .payload("test payload")
        .qos(qos)
        .packet_id(Some(id))
        .build()
        .unwrap();

    GenericStorePacket::V3_1_1Publish(publish)
}

#[test]
fn test_add_and_get_stored_order() {
    common::init_tracing();
    let mut store = GenericStore::<u32>::new();
    assert!(store.add(make_packet_u32(1, Qos::AtLeastOnce)).is_ok());
    assert!(store.add(make_packet_u32(2, Qos::ExactlyOnce)).is_ok());
    // Duplicate id should fail
    assert!(store.add(make_packet_u32(1, Qos::AtLeastOnce)).is_err());

    let stored = store.get_stored();
    let ids: Vec<u32> = stored.iter().map(|p| p.packet_id()).collect();
    assert_eq!(ids, vec![1, 2]);
}

#[test]
fn test_erase_by_response_and_id() {
    common::init_tracing();
    let mut store = GenericStore::<u32>::new();
    let p1 = make_packet_u32(10, Qos::ExactlyOnce); // Will expect V3_1_1Pubrec response
    let p2 = make_packet_u32(20, Qos::AtLeastOnce); // Will expect V3_1_1Puback response
    store.add(p1).unwrap();
    store.add(p2).unwrap();

    // Erase existing
    assert!(store.erase(ResponsePacket::V3_1_1Pubrec, 10));
    let stored = store.get_stored();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].packet_id(), 20);

    // Erase non-existing (wrong resp)
    assert!(!store.erase(ResponsePacket::V3_1_1Pubrec, 20));
    // Erase non-existing id
    assert!(!store.erase(ResponsePacket::V5_0Pubrec, 30));
}

#[test]
fn test_erase_publish() {
    common::init_tracing();
    let mut store = GenericStore::<u32>::new();
    // Only one matching packet
    store.add(make_packet_u32(42, Qos::ExactlyOnce)).unwrap();
    assert!(store.erase_publish(42));
    assert!(store.get_stored().is_empty());

    // erase_publish on empty
    assert!(!store.erase_publish(99));
}

#[test]
fn test_clear_and_for_each() {
    common::init_tracing();
    let mut store = GenericStore::<u32>::new();
    for i in 1..6 {
        // Start from 1 instead of 0 (packet ID 0 is invalid)
        // alternate QoS types
        let qos = if i % 2 == 0 {
            Qos::AtLeastOnce
        } else {
            Qos::ExactlyOnce
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
    let mut store = Store::new();
    let packet = make_packet_u16(100, Qos::AtLeastOnce);

    assert!(store.add(packet).is_ok());
    let stored = store.get_stored();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].packet_id(), 100);
}
