#![cfg(feature = "std")]

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
mqtt_protocol_core::make_default_aliases!();
mod common;
use common::*;

///////////////////////////////////////////////////////////////////////////////

// Test notify_closed method

#[test]
fn notify_closed_basic_disconnection() {
    common::init_tracing();
    // Test basic disconnection without any pending packets
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Establish connection first
    v3_1_1_client_establish_connection(&mut con, true, false);

    // Notify connection closed
    let events = con.notify_closed();

    // Should contain timer cancellation events
    // All timer cancellation events should be RequestTimerCancel
    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {}
            _ => panic!("Unexpected event in basic disconnection: {:?}", event),
        }
    }
}

#[test]
fn notify_closed_with_session_storage_disabled() {
    common::init_tracing();
    // Test disconnection when session storage is disabled (need_store = false)
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Establish connection with clean_session=true (no session storage)
    v3_1_1_client_establish_connection(&mut con, true, false);

    // Simulate some pending packet IDs by sending packets that would create them
    // (In a real implementation, these would be set by sending PUBLISH packets)

    // Notify connection closed
    let events = con.notify_closed();

    // Should process without errors

    // Events should only be timer cancellation or packet ID release events
    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {}
            mqtt::connection::Event::NotifyPacketIdReleased(_) => {}
            _ => panic!(
                "Unexpected event with session storage disabled: {:?}",
                event
            ),
        }
    }
}

#[test]
fn notify_closed_with_session_storage_enabled() {
    common::init_tracing();
    // Test disconnection when session storage is enabled (need_store = true)
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Establish connection with clean_session=false (session storage enabled)
    v3_1_1_client_establish_connection(&mut con, false, true);

    // Notify connection closed
    let events = con.notify_closed();

    // Should process without errors

    // Events should only be timer cancellation events when session is stored
    // (packet IDs are not released when session is stored)
    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {}
            _ => panic!("Unexpected event with session storage enabled: {:?}", event),
        }
    }
}

#[test]
fn notify_closed_v5_0_client() {
    common::init_tracing();
    // Test disconnection with v5.0 client
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Establish connection
    v5_0_client_establish_connection(&mut con);

    // Notify connection closed
    let events = con.notify_closed();

    // Should process without errors

    // Events should only be timer cancellation or packet ID release events
    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {}
            mqtt::connection::Event::NotifyPacketIdReleased(_) => {}
            _ => panic!("Unexpected event in v5.0 disconnection: {:?}", event),
        }
    }
}

#[test]
fn notify_closed_server_role() {
    common::init_tracing();
    // Test disconnection with server role
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);

    // Establish connection
    v3_1_1_server_establish_connection(&mut con, true, false);

    // Notify connection closed
    let events = con.notify_closed();

    // Should process without errors

    // Events should only be timer cancellation or packet ID release events
    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {}
            mqtt::connection::Event::NotifyPacketIdReleased(_) => {}
            _ => panic!("Unexpected event in server disconnection: {:?}", event),
        }
    }
}

#[test]
fn notify_closed_v5_0_server() {
    common::init_tracing();
    // Test disconnection with v5.0 server
    let mut con = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Establish connection
    v5_0_server_establish_connection(&mut con);

    // Notify connection closed
    let events = con.notify_closed();

    // Should process without errors

    // Events should only be timer cancellation or packet ID release events
    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {}
            mqtt::connection::Event::NotifyPacketIdReleased(_) => {}
            _ => panic!("Unexpected event in v5.0 server disconnection: {:?}", event),
        }
    }
}

#[test]
fn notify_closed_multiple_calls() {
    common::init_tracing();
    // Test calling notify_closed multiple times
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Establish connection
    v3_1_1_client_establish_connection(&mut con, true, false);

    // First call to notify_closed
    let events1 = con.notify_closed();

    // Second call to notify_closed (should be safe to call multiple times)
    let events2 = con.notify_closed();

    // Both calls should succeed

    // Second call should typically return fewer or no events since state is already cleared
    assert!(events2.len() <= events1.len());
}

#[test]
fn notify_closed_without_connection() {
    common::init_tracing();
    // Test calling notify_closed without establishing connection first
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Don't establish connection - call notify_closed on disconnected state

    // Notify connection closed
    let events = con.notify_closed();

    // Should process without errors even when not connected

    // Events should only be timer cancellation events (if any)
    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {}
            mqtt::connection::Event::NotifyPacketIdReleased(_) => {}
            _ => panic!(
                "Unexpected event when disconnecting without connection: {:?}",
                event
            ),
        }
    }
}

#[test]
fn notify_closed_with_acquired_packet_ids() {
    common::init_tracing();
    // Test disconnection when various packet IDs are acquired and packets are sent
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Establish connection with clean_session=true (need_store = false)
    v3_1_1_client_establish_connection(&mut con, true, false);

    // Acquire packet IDs and send various packets

    // 1. Subscribe packet
    let subscribe_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Subscribe");
    let subscribe_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(subscribe_pid)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let _events = con.send(subscribe_packet);

    // 2. Unsubscribe packet
    let unsubscribe_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Unsubscribe");
    let unsubscribe_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(unsubscribe_pid)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let _events = con.send(unsubscribe_packet);

    // 3. Publish QoS1 packet
    let publish_qos1_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Publish QoS1");
    let publish_qos1_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/qos1")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(publish_qos1_pid)
        .payload(b"qos1 payload")
        .build()
        .expect("Failed to build Publish QoS1 packet")
        .into();
    let _events = con.send(publish_qos1_packet);

    // 4. Publish QoS2 packet
    let publish_qos2_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Publish QoS2");
    let publish_qos2_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/qos2")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(publish_qos2_pid)
        .payload(b"qos2 payload")
        .build()
        .expect("Failed to build Publish QoS2 packet")
        .into();
    let _events = con.send(publish_qos2_packet);

    // 5. Pubrel packet
    let pubrel_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Pubrel");
    let pubrel_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(pubrel_pid)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let _events = con.send(pubrel_packet);

    // Notify connection closed - this should release packet IDs since need_store = false
    let events = con.notify_closed();

    // Count different event types
    let mut _timer_cancel_count = 0;
    let mut packet_id_release_count = 0;

    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {
                _timer_cancel_count += 1;
            }
            mqtt::connection::Event::NotifyPacketIdReleased(pid) => {
                packet_id_release_count += 1;
                // Verify that the released packet IDs match the ones we acquired
                assert!(
                    *pid == subscribe_pid
                        || *pid == unsubscribe_pid
                        || *pid == publish_qos1_pid
                        || *pid == publish_qos2_pid
                        || *pid == pubrel_pid,
                    "Unexpected packet ID released: {pid}"
                );
            }
            _ => panic!(
                "Unexpected event in notify_closed with acquired packet IDs: {:?}",
                event
            ),
        }
    }

    // Should have released all packet IDs since need_store = false
    assert_eq!(
        packet_id_release_count, 5,
        "Expected 5 packet IDs to be released"
    );
}

#[test]
fn notify_closed_with_acquired_packet_ids_session_storage() {
    common::init_tracing();
    // Test disconnection with session storage enabled (need_store = true)
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Establish connection with clean_session=false (need_store = true)
    v3_1_1_client_establish_connection(&mut con, false, true);

    // Acquire packet IDs and send various packets

    // 1. Subscribe packet
    let subscribe_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Subscribe");
    let subscribe_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Subscribe::builder()
        .packet_id(subscribe_pid)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let _events = con.send(subscribe_packet);

    // 2. Unsubscribe packet
    let unsubscribe_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Unsubscribe");
    let unsubscribe_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Unsubscribe::builder()
        .packet_id(unsubscribe_pid)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let _events = con.send(unsubscribe_packet);

    // 3. Publish QoS1 packet
    let publish_qos1_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Publish QoS1");
    let publish_qos1_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/qos1")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(publish_qos1_pid)
        .payload(b"qos1 payload")
        .build()
        .expect("Failed to build Publish QoS1 packet")
        .into();
    let _events = con.send(publish_qos1_packet);

    // 4. Publish QoS2 packet
    let publish_qos2_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Publish QoS2");
    let publish_qos2_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/qos2")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(publish_qos2_pid)
        .payload(b"qos2 payload")
        .build()
        .expect("Failed to build Publish QoS2 packet")
        .into();
    let _events = con.send(publish_qos2_packet);

    // 5. Pubrel packet
    let pubrel_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Pubrel");
    let pubrel_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(pubrel_pid)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let _events = con.send(pubrel_packet);

    // Notify connection closed - packet IDs should NOT be released when need_store = true
    let events = con.notify_closed();

    // Count different event types
    let mut _timer_cancel_count = 0;
    let mut packet_id_release_count = 0;

    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {
                _timer_cancel_count += 1;
            }
            mqtt::connection::Event::NotifyPacketIdReleased(pid) => {
                packet_id_release_count += 1;
                // Only SUBACK and UNSUBACK packet IDs should be released even with session storage
                assert!(
                    *pid == subscribe_pid || *pid == unsubscribe_pid,
                    "Unexpected packet ID released with session storage: {pid}"
                );
            }
            _ => panic!(
                "Unexpected event in notify_closed with session storage: {:?}",
                event
            ),
        }
    }

    // Only SUBACK and UNSUBACK packet IDs should be released (publish-related IDs are kept when need_store = true)
    assert_eq!(
        packet_id_release_count, 2,
        "Expected only 2 packet IDs to be released (SUBACK and UNSUBACK)"
    );
}

#[test]
fn notify_closed_v5_0_with_acquired_packet_ids() {
    common::init_tracing();
    // Test disconnection with v5.0 and acquired packet IDs
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Establish connection
    v5_0_client_establish_connection(&mut con);

    // Acquire packet IDs and send various v5.0 packets

    // 1. Subscribe packet
    let subscribe_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Subscribe");
    let subscribe_packet: mqtt::packet::Packet = mqtt::packet::v5_0::Subscribe::builder()
        .packet_id(subscribe_pid)
        .entries(vec![mqtt::packet::SubEntry::new(
            "test/topic",
            mqtt::packet::SubOpts::default(),
        )
        .unwrap()])
        .build()
        .expect("Failed to build Subscribe packet")
        .into();
    let _events = con.send(subscribe_packet);

    // 2. Unsubscribe packet
    let unsubscribe_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Unsubscribe");
    let unsubscribe_packet: mqtt::packet::Packet = mqtt::packet::v5_0::Unsubscribe::builder()
        .packet_id(unsubscribe_pid)
        .entries(vec!["test/topic"])
        .unwrap()
        .build()
        .expect("Failed to build Unsubscribe packet")
        .into();
    let _events = con.send(unsubscribe_packet);

    // 3. Publish QoS1 packet
    let publish_qos1_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Publish QoS1");
    let publish_qos1_packet: mqtt::packet::Packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/qos1")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(publish_qos1_pid)
        .payload(b"qos1 payload")
        .build()
        .expect("Failed to build Publish QoS1 packet")
        .into();
    let _events = con.send(publish_qos1_packet);

    // 4. Publish QoS2 packet
    let publish_qos2_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Publish QoS2");
    let publish_qos2_packet: mqtt::packet::Packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/qos2")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(publish_qos2_pid)
        .payload(b"qos2 payload")
        .build()
        .expect("Failed to build Publish QoS2 packet")
        .into();
    let _events = con.send(publish_qos2_packet);

    // 5. Pubrel packet
    let pubrel_pid = con
        .acquire_packet_id()
        .expect("Failed to acquire packet ID for Pubrel");
    let pubrel_packet: mqtt::packet::Packet = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(pubrel_pid)
        .build()
        .expect("Failed to build Pubrel packet")
        .into();
    let _events = con.send(pubrel_packet);

    // Notify connection closed
    let events = con.notify_closed();

    // Count different event types
    let mut _timer_cancel_count = 0;
    let mut packet_id_release_count = 0;

    for event in &events {
        match event {
            mqtt::connection::Event::RequestTimerCancel(_) => {
                _timer_cancel_count += 1;
            }
            mqtt::connection::Event::NotifyPacketIdReleased(pid) => {
                packet_id_release_count += 1;
                // Verify that the released packet IDs match the ones we acquired
                assert!(
                    *pid == subscribe_pid
                        || *pid == unsubscribe_pid
                        || *pid == publish_qos1_pid
                        || *pid == publish_qos2_pid
                        || *pid == pubrel_pid,
                    "Unexpected packet ID released in v5.0: {pid}"
                );
            }
            _ => panic!(
                "Unexpected event in v5.0 notify_closed with acquired packet IDs: {:?}",
                event
            ),
        }
    }

    // Should have released all packet IDs (default is need_store = false)
    assert_eq!(
        packet_id_release_count, 5,
        "Expected 5 packet IDs to be released in v5.0"
    );
}
