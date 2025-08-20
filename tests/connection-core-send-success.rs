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
use mqtt_protocol_core::default_alias;
use mqtt_protocol_core::mqtt;
mod common;

///////////////////////////////////////////////////////////////////////////////

// v3.1.1 client

// disconnected

#[test]
fn v3_1_1_client_send_connect() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);
    let send_packet: mqtt::packet::Packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_session(true)
        .build()
        .expect("Failed to build Connect packet")
        .into();
    let events = con.send(send_packet.clone());
    assert_eq!(events.len(), 1);

    if let mqtt::connection::Event::RequestSendPacket {
        packet,
        release_packet_id_if_send_error,
    } = &events[0]
    {
        assert_eq!(*packet, send_packet);
        assert!(release_packet_id_if_send_error.is_none());
    } else {
        assert!(
            false,
            "Expected RequestSendPacket event, but got: {:?}",
            events[0]
        );
    }
}

// connected
