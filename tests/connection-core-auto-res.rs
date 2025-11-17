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
fn auto_pub_response_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

    // Send CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Create and receive QoS1 PUBLISH A
    let packet_id_a = 1u16;
    let publish_a = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(Some(packet_id_a))
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBACK send request event with same packet_id
    let mut puback_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Puback(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_a {
                puback_found = true;
                break;
            }
        }
    }
    assert!(
        puback_found,
        "PUBACK with packet_id {} should be found in events",
        packet_id_a
    );

    // Create and receive QoS2 PUBLISH B
    let packet_id_b = 2u16;
    let publish_b = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(Some(packet_id_b))
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let bytes = publish_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREC send request event with same packet_id
    let mut pubrec_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pubrec(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubrec_found = true;
                break;
            }
        }
    }
    assert!(
        pubrec_found,
        "PUBREC with packet_id {} should be found in events",
        packet_id_b
    );

    // Send PUBREC B
    let pubrec_b = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(packet_id_b)
        .build()
        .unwrap();

    let _events = connection.send(pubrec_b.into());

    // Receive PUBREL B
    let pubrel_b = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(packet_id_b)
        .build()
        .unwrap();

    let bytes = pubrel_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBCOMP send request event with same packet_id
    let mut pubcomp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pubcomp(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubcomp_found = true;
                break;
            }
        }
    }
    assert!(
        pubcomp_found,
        "PUBCOMP with packet_id {} should be found in events",
        packet_id_b
    );
}

#[test]
fn auto_pub_response_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

    // Send CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Create and receive QoS1 PUBLISH A
    let packet_id_a = 1u16;
    let publish_a = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/a")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(Some(packet_id_a))
        .payload(b"payload A".to_vec())
        .build()
        .unwrap();

    let bytes = publish_a.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBACK send request event with same packet_id
    let mut puback_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Puback(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_a {
                puback_found = true;
                break;
            }
        }
    }
    assert!(
        puback_found,
        "PUBACK with packet_id {} should be found in events",
        packet_id_a
    );

    // Create and receive QoS2 PUBLISH B
    let packet_id_b = 2u16;
    let publish_b = mqtt::packet::v5_0::Publish::builder()
        .topic_name("topic/b")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(Some(packet_id_b))
        .payload(b"payload B".to_vec())
        .build()
        .unwrap();

    let bytes = publish_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREC send request event with same packet_id
    let mut pubrec_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pubrec(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubrec_found = true;
                break;
            }
        }
    }
    assert!(
        pubrec_found,
        "PUBREC with packet_id {} should be found in events",
        packet_id_b
    );

    // Send PUBREC B
    let pubrec_b = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(packet_id_b)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let _events = connection.send(pubrec_b.into());

    // Receive PUBREL B
    let pubrel_b = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(packet_id_b)
        .reason_code(mqtt::result_code::PubrelReasonCode::Success)
        .build()
        .unwrap();

    let bytes = pubrel_b.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBCOMP send request event with same packet_id
    let mut pubcomp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pubcomp(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id_b {
                pubcomp_found = true;
                break;
            }
        }
    }
    assert!(
        pubcomp_found,
        "PUBCOMP with packet_id {} should be found in events",
        packet_id_b
    );
}

#[test]
fn qos2_pubrel_send_request_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V3_1_1);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

    // Send CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Acquire packet ID and send QoS2 PUBLISH
    let packet_id = connection.acquire_packet_id().unwrap();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(Some(packet_id))
        .payload(b"test payload".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish.into());

    // Receive PUBREC
    let pubrec = mqtt::packet::v3_1_1::Pubrec::builder()
        .packet_id(packet_id)
        .build()
        .unwrap();

    let bytes = pubrec.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREL send request event with same packet_id
    let mut pubrel_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pubrel(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id {
                pubrel_found = true;
                break;
            }
        }
    }
    assert!(
        pubrel_found,
        "PUBREL with packet_id {} should be found in events",
        packet_id
    );
}

#[test]
fn qos2_pubrel_send_request_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Enable automatic publish response
    connection.set_auto_pub_response(true);

    // Send CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let _events = connection.send(connect.into());

    // Receive CONNACK
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let bytes = connack.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Acquire packet ID and send QoS2 PUBLISH
    let packet_id = connection.acquire_packet_id().unwrap();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(Some(packet_id))
        .payload(b"test payload".to_vec())
        .build()
        .unwrap();

    let _events = connection.send(publish.into());

    // Receive PUBREC
    let pubrec = mqtt::packet::v5_0::Pubrec::builder()
        .packet_id(packet_id)
        .reason_code(mqtt::result_code::PubrecReasonCode::Success)
        .build()
        .unwrap();

    let bytes = pubrec.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PUBREL send request event with same packet_id
    let mut pubrel_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pubrel(p),
            ..
        } = event
        {
            if p.packet_id() == packet_id {
                pubrel_found = true;
                break;
            }
        }
    }
    assert!(
        pubrel_found,
        "PUBREL with packet_id {} should be found in events",
        packet_id
    );
}

#[test]
fn auto_ping_response_server_v3_1_1() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V3_1_1);

    // Enable automatic ping response
    connection.set_auto_ping_response(true);

    // Receive CONNECT
    let connect = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let bytes = connect.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Send CONNACK
    let connack = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(false)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .unwrap();

    let _events = connection.send(connack.into());

    // Receive PINGREQ
    let pingreq = mqtt::packet::v3_1_1::Pingreq::new();

    let bytes = pingreq.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PINGRESP send request event
    let mut pingresp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V3_1_1Pingresp(_),
            ..
        } = event
        {
            pingresp_found = true;
            break;
        }
    }
    assert!(pingresp_found, "PINGRESP should be found in events");
}

#[test]
fn auto_ping_response_server_v5_0() {
    common::init_tracing();
    let mut connection = mqtt::Connection::<mqtt::role::Server>::new(mqtt::Version::V5_0);

    // Enable automatic ping response
    connection.set_auto_ping_response(true);

    // Receive CONNECT
    let connect = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .unwrap();

    let bytes = connect.to_continuous_buffer();
    let _events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Send CONNACK
    let connack = mqtt::packet::v5_0::Connack::builder()
        .session_present(false)
        .reason_code(mqtt::result_code::ConnectReasonCode::Success)
        .build()
        .unwrap();

    let _events = connection.send(connack.into());

    // Receive PINGREQ
    let pingreq = mqtt::packet::v5_0::Pingreq::new();

    let bytes = pingreq.to_continuous_buffer();
    let events = connection.recv(&mut mqtt::common::Cursor::new(&bytes));

    // Find PINGRESP send request event
    let mut pingresp_found = false;
    for event in &events {
        if let mqtt::connection::Event::RequestSendPacket {
            packet: mqtt::packet::Packet::V5_0Pingresp(_),
            ..
        } = event
        {
            pingresp_found = true;
            break;
        }
    }
    assert!(pingresp_found, "PINGRESP should be found in events");
}
