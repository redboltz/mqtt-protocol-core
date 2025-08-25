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
fn v5_0_send_stored_oversize() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    // store QoS1 publish with no MaximumPacketSize
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("cid1")
            .unwrap()
            .props(vec![mqtt::packet::SessionExpiryInterval::new(0xffffffff)
                .unwrap()
                .into()])
            .build()
            .expect("Failed to build Connect packet");
        let _ = con.checked_send(packet);
    }
    {
        let packet = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .expect("Failed to build Connack packet");
        let flattened: Vec<u8> = packet.to_continuous_buffer();
        let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
        let _ = con.recv(&mut cursor);
    }
    let pid = con.acquire_packet_id().unwrap();
    {
        let packet = mqtt::packet::v5_0::Publish::builder()
            .packet_id(pid)
            .qos(mqtt::packet::Qos::AtLeastOnce)
            .topic_name("t")
            .unwrap()
            .payload("0123456789abcdef")
            .build()
            .expect("Failed to build Publish packet");
        let _ = con.checked_send(packet);
    }
    con.notify_closed();

    // send_store QoS1 publish but MaximumPacketSize is too small
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("cid1")
            .unwrap()
            .clean_start(false)
            .build()
            .expect("Failed to build Connect packet");
        let _ = con.checked_send(packet);
    }
    {
        let packet = mqtt::packet::v5_0::Connack::builder()
            .session_present(true)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::MaximumPacketSize::new(15)
                .unwrap()
                .into()])
            .build()
            .expect("Failed to build Connack packet");
        let flattened: Vec<u8> = packet.to_continuous_buffer();
        let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
        let events = con.recv(&mut cursor);
        assert_eq!(events.len(), 2);
        if let mqtt::connection::Event::NotifyPacketIdReleased(packet_id) = &events[0] {
            assert_eq!(*packet_id, pid);
        } else {
            panic!(
                "Expected NotifyPacketIdReleased event, got: {:?}",
                events[0]
            );
        }
        if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[1] {
            if let mqtt::packet::GenericPacket::V5_0Connack(connack) = packet {
                assert_eq!(connack.session_present(), true);
                assert_eq!(
                    connack.reason_code(),
                    mqtt::result_code::ConnectReasonCode::Success
                );
            } else {
                panic!("Expected V5_0Connack packet, got: {:?}", packet);
            }
        } else {
            panic!("Expected NotifyPacketReceived event, got: {:?}", events[1]);
        }
    }
    con.notify_closed();

    // Not send_store QoS1 publish even if MaximumPacketSize is not set
    // because the publish packet has already been erased
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("cid1")
            .unwrap()
            .clean_start(false)
            .build()
            .expect("Failed to build Connect packet");
        let _ = con.checked_send(packet);
    }
    {
        let packet = mqtt::packet::v5_0::Connack::builder()
            .session_present(true)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .props(vec![mqtt::packet::MaximumPacketSize::new(15)
                .unwrap()
                .into()])
            .build()
            .expect("Failed to build Connack packet");
        let flattened: Vec<u8> = packet.to_continuous_buffer();
        let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
        let events = con.recv(&mut cursor);
        assert_eq!(events.len(), 1);
        if let mqtt::connection::Event::NotifyPacketReceived(packet) = &events[0] {
            if let mqtt::packet::GenericPacket::V5_0Connack(connack) = packet {
                assert_eq!(connack.session_present(), true);
                assert_eq!(
                    connack.reason_code(),
                    mqtt::result_code::ConnectReasonCode::Success
                );
            } else {
                panic!("Expected V5_0Connack packet, got: {:?}", packet);
            }
        } else {
            panic!("Expected NotifyPacketReceived event, got: {:?}", events[1]);
        }
    }
}
