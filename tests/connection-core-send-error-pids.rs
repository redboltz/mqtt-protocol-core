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

// Tests for cleanup of internal tracking state (store / pid_puback / pid_pubrec)
// when a v5.0 PUBLISH cannot be sent due to error.
//
// The three error paths exercised here are inside `process_send_v5_0_publish`:
//   - empty topic_name with invalid TopicAlias
//   - non-empty topic_name with out-of-range TopicAlias
//   - ReceiveMaximumExceeded
//
// After each error the connection must release the packet_id AND remove the
// publish entry that was tentatively recorded in `store` / `pid_puback` /
// `pid_pubrec`. Otherwise the released packet_id, once reused, would either
// panic on `store.add(...)` (the bug Jonas reported) or leave dangling entries
// in the QoS tracking sets.

use mqtt_protocol_core::mqtt;
mod common;

///////////////////////////////////////////////////////////////////////////////
// helpers

fn connect_v5_0_client_with_props(
    con: &mut mqtt::Connection<mqtt::role::Client>,
    connack_props: Vec<mqtt::packet::Property>,
    session_expiry_interval: Option<u32>,
) {
    let connect_props: Vec<mqtt::packet::Property> = match session_expiry_interval {
        Some(v) => vec![mqtt::packet::SessionExpiryInterval::new(v).unwrap().into()],
        None => vec![],
    };
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .props(connect_props)
        .build()
        .expect("Failed to build Connect packet");
    let _ = con.checked_send(packet);

    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .props(connack_props)
        .build()
        .expect("Failed to build Connack packet");
    let bytes = connack.to_continuous_buffer();
    let _ = con.recv(&mut mqtt::common::Cursor::new(&bytes));
}

///////////////////////////////////////////////////////////////////////////////
// ReceiveMaximumExceeded — store cleanup

// need_store=true: the publish that triggers ReceiveMaximumExceeded has already
// been added to the store. The fix must remove it so the store reflects only
// the truly in-flight packet.
#[test]
fn v5_0_receive_maximum_exceeded_store_not_leaked_qos1() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connect_v5_0_client_with_props(
        &mut con,
        vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()],
        Some(0xffffffff),
    );

    let pid_a = con.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_a)
        .payload(b"A".to_vec())
        .build()
        .unwrap();
    let _ = con.send(publish_a.into());

    let pid_b = con.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_b)
        .payload(b"B".to_vec())
        .build()
        .unwrap();
    let events = con.send(publish_b.into());

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::ReceiveMaximumExceeded)
    ));
    assert!(matches!(
        events[1],
        mqtt::connection::Event::NotifyPacketIdReleased(pid) if pid == pid_b
    ));

    let stored = con.get_stored_packets();
    assert_eq!(stored.len(), 1, "only publish A should remain in store");
    match &stored[0] {
        mqtt::packet::GenericStorePacket::V5_0Publish(p) => {
            assert_eq!(p.packet_id(), Some(pid_a));
            assert_eq!(p.topic_name(), "topic/a");
        }
        other => panic!("expected V5_0Publish, got {other:?}"),
    }
}

#[test]
fn v5_0_receive_maximum_exceeded_store_not_leaked_qos2() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connect_v5_0_client_with_props(
        &mut con,
        vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()],
        Some(0xffffffff),
    );

    let pid_a = con.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(pid_a)
        .payload(b"A".to_vec())
        .build()
        .unwrap();
    let _ = con.send(publish_a.into());

    let pid_b = con.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(pid_b)
        .payload(b"B".to_vec())
        .build()
        .unwrap();
    let events = con.send(publish_b.into());

    assert_eq!(events.len(), 2);
    assert!(matches!(
        events[0],
        mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::ReceiveMaximumExceeded)
    ));
    assert!(matches!(
        events[1],
        mqtt::connection::Event::NotifyPacketIdReleased(pid) if pid == pid_b
    ));

    let stored = con.get_stored_packets();
    assert_eq!(stored.len(), 1);
    match &stored[0] {
        mqtt::packet::GenericStorePacket::V5_0Publish(p) => {
            assert_eq!(p.packet_id(), Some(pid_a));
        }
        other => panic!("expected V5_0Publish, got {other:?}"),
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReceiveMaximumExceeded — packet_id reusable without panic
//
// This is the exact failure mode the original PR was reproducing: after
// ReceiveMaximumExceeded released the packet_id, the next publish reuses it
// and the unconditional `store.add(...).unwrap()` panicked with
// PacketIdentifierConflict because the store entry was still there.

#[test]
fn v5_0_receive_maximum_exceeded_packet_id_reusable() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connect_v5_0_client_with_props(
        &mut con,
        vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()],
        Some(0xffffffff),
    );

    let pid_a = con.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_a)
        .payload(b"A".to_vec())
        .build()
        .unwrap();
    let _ = con.send(publish_a.into());

    let pid_b = con.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_b)
        .payload(b"B".to_vec())
        .build()
        .unwrap();
    let _ = con.send(publish_b.into());

    // Drain capacity: PUBACK(A) so a new publish can be admitted.
    let puback_a = mqtt::packet::v5_0::Puback::builder()
        .packet_id(pid_a)
        .build()
        .unwrap();
    let bytes = puback_a.to_continuous_buffer();
    let _ = con.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Reacquire — the manager may hand back the previously released pid_b.
    let pid_c = con.acquire_packet_id().unwrap();
    let publish_c = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/c")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_c)
        .payload(b"C".to_vec())
        .build()
        .unwrap();
    let events = con.send(publish_c.into());

    // Without the fix this would have panicked inside send() before any event
    // is produced. Reaching here at all is the headline assertion.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, mqtt::connection::Event::RequestSendPacket { .. })),
        "publish C must be admitted after the released pid is reused"
    );

    let stored = con.get_stored_packets();
    assert_eq!(stored.len(), 1);
    match &stored[0] {
        mqtt::packet::GenericStorePacket::V5_0Publish(p) => {
            assert_eq!(p.packet_id(), Some(pid_c));
            assert_eq!(p.topic_name(), "topic/c");
        }
        other => panic!("expected V5_0Publish, got {other:?}"),
    }
}

///////////////////////////////////////////////////////////////////////////////
// ReceiveMaximumExceeded — pid_puback / pid_pubrec cleanup
//
// publish_send_count is incremented AFTER the ReceiveMaximumExceeded check,
// so the failing publish has not consumed a slot. If the subsequent PUBACK
// for the in-flight publish is processed correctly and a third publish is
// admitted, the bookkeeping (count, pid_puback) must be consistent with
// having only ever sent one packet. A leak in pid_puback would not panic but
// would leave the released pid_b lingering — exercised here via reuse.

#[test]
fn v5_0_receive_maximum_exceeded_publish_count_consistent_qos1() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connect_v5_0_client_with_props(
        &mut con,
        vec![mqtt::packet::ReceiveMaximum::new(1).unwrap().into()],
        Some(0xffffffff),
    );

    assert_eq!(con.get_receive_maximum_vacancy_for_send(), Some(1));

    let pid_a = con.acquire_packet_id().unwrap();
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_a)
        .payload(b"A".to_vec())
        .build()
        .unwrap();
    let _ = con.send(publish_a.into());
    assert_eq!(con.get_receive_maximum_vacancy_for_send(), Some(0));

    let pid_b = con.acquire_packet_id().unwrap();
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_b)
        .payload(b"B".to_vec())
        .build()
        .unwrap();
    let _ = con.send(publish_b.into());

    // The failed publish must not have consumed a flow-control slot.
    assert_eq!(con.get_receive_maximum_vacancy_for_send(), Some(0));

    // PUBACK(A) frees the slot back to 1.
    let puback_a = mqtt::packet::v5_0::Puback::builder()
        .packet_id(pid_a)
        .build()
        .unwrap();
    let bytes = puback_a.to_continuous_buffer();
    let _ = con.recv(&mut mqtt::common::Cursor::new(&bytes));
    assert_eq!(con.get_receive_maximum_vacancy_for_send(), Some(1));

    // Store is now empty.
    assert!(con.get_stored_packets().is_empty());
}

///////////////////////////////////////////////////////////////////////////////
// Invalid (out-of-range) TopicAlias on send

#[test]
fn v5_0_send_topic_alias_out_of_range_store_not_leaked() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connect_v5_0_client_with_props(
        &mut con,
        vec![mqtt::packet::TopicAliasMaximum::new(2).unwrap().into()],
        Some(0xffffffff),
    );

    let pid = con.acquire_packet_id().unwrap();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid)
        .props(vec![mqtt::packet::TopicAlias::new(5).unwrap().into()])
        .payload(b"A".to_vec())
        .build()
        .unwrap();
    let events = con.send(publish.into());

    assert!(events.iter().any(|e| matches!(
        e,
        mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketNotAllowedToSend)
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        mqtt::connection::Event::NotifyPacketIdReleased(p) if *p == pid
    )));

    assert!(con.get_stored_packets().is_empty());
}

// After the out-of-range error, the released packet_id must be safe to reuse:
// the store entry tentatively added for the failed publish must have been
// erased, otherwise `store.add(...).unwrap()` panics on reuse.
#[test]
fn v5_0_send_topic_alias_out_of_range_packet_id_reusable() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connect_v5_0_client_with_props(
        &mut con,
        vec![mqtt::packet::TopicAliasMaximum::new(2).unwrap().into()],
        Some(0xffffffff),
    );

    let pid_first = con.acquire_packet_id().unwrap();
    let bad_publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_first)
        .props(vec![mqtt::packet::TopicAlias::new(5).unwrap().into()])
        .payload(b"A".to_vec())
        .build()
        .unwrap();
    let _ = con.send(bad_publish.into());

    let pid_reused = con.acquire_packet_id().unwrap();
    let ok_publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_reused)
        .payload(b"B".to_vec())
        .build()
        .unwrap();
    let events = con.send(ok_publish.into());

    assert!(events
        .iter()
        .any(|e| matches!(e, mqtt::connection::Event::RequestSendPacket { .. })));

    let stored = con.get_stored_packets();
    assert_eq!(stored.len(), 1);
    match &stored[0] {
        mqtt::packet::GenericStorePacket::V5_0Publish(p) => {
            assert_eq!(p.packet_id(), Some(pid_reused));
            assert_eq!(p.topic_name(), "topic/b");
        }
        other => panic!("expected V5_0Publish, got {other:?}"),
    }
}

///////////////////////////////////////////////////////////////////////////////
// Empty topic_name + invalid TopicAlias on send
//
// Reachable only when need_store=false (otherwise the earlier validation block
// inside the need_store branch returns first). In this case the publish has
// not been added to the store and `erase_publish` is a no-op, but the
// pid_puback/pid_pubrec entry inserted right before this check IS present and
// must be removed.

#[test]
fn v5_0_send_empty_topic_no_alias_packet_id_released() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    // No SessionExpiryInterval, no TopicAliasMaximum — need_store stays false
    // and topic_alias_send is None, so validate_topic_alias always returns None.
    connect_v5_0_client_with_props(&mut con, vec![], None);

    let pid = con.acquire_packet_id().unwrap();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid)
        .props(vec![mqtt::packet::TopicAlias::new(1).unwrap().into()])
        .payload(b"A".to_vec())
        .build()
        .unwrap();
    let events = con.send(publish.into());

    assert!(events.iter().any(|e| matches!(
        e,
        mqtt::connection::Event::NotifyError(mqtt::result_code::MqttError::PacketNotAllowedToSend)
    )));
    assert!(events.iter().any(|e| matches!(
        e,
        mqtt::connection::Event::NotifyPacketIdReleased(p) if *p == pid
    )));
    assert!(con.get_stored_packets().is_empty());

    // The released pid must be reusable for a subsequent valid publish.
    let pid_reused = con.acquire_packet_id().unwrap();
    let next = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/x")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(pid_reused)
        .payload(b"X".to_vec())
        .build()
        .unwrap();
    let events = con.send(next.into());
    assert!(events
        .iter()
        .any(|e| matches!(e, mqtt::connection::Event::RequestSendPacket { .. })));
}
