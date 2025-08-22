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

use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        let program = &args[0];
        eprintln!("Usage: {program} <host> <port> <topic> <qos> <payload>");
        eprintln!("Example: {program} localhost 1883 test/topic 0 \"Hello World\"");
        std::process::exit(1);
    }

    let host = &args[1];
    let port: u16 = args[2].parse().map_err(|e| format!("Invalid port: {e}"))?;
    let topic = &args[3];
    let qos: u8 = args[4].parse().map_err(|e| format!("Invalid QoS: {e}"))?;
    let payload = &args[5];

    let qos_level = mqtt::packet::Qos::try_from(qos)
        .expect("Error: Invalid QoS level '{qos}'. Must be 0, 1, or 2");

    let mut stream = TcpStream::connect(format!("{host}:{port}"))?;
    println!("Connected to {host}:{port}");

    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    connection.set_auto_pub_response(true);
    let connect_packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("mqtt_publish_example")
        .unwrap()
        .build()
        .map_err(|e| format!("Failed to build CONNECT packet: {e:?}"))?;

    let events = connection.checked_send(connect_packet);
    handle_events(&mut stream, &mut connection, events)?;

    let mut buffer = [0u8; 1024];
    let n = stream.read(&mut buffer)?;
    if n > 0 {
        let mut cursor = mqtt::common::Cursor::new(&buffer[..n]);
        let events = connection.recv(&mut cursor);
        handle_events(&mut stream, &mut connection, events)?;
    }

    let mut publish_builder = mqtt::packet::v5_0::Publish::builder()
        .topic_name(topic)
        .unwrap()
        .qos(qos_level)
        .payload(payload.as_bytes());

    if qos_level != mqtt::packet::Qos::AtMostOnce {
        let packet_id = connection
            .acquire_packet_id()
            .map_err(|e| format!("Failed to acquire packet ID: {e:?}"))?;
        publish_builder = publish_builder.packet_id(packet_id);
    }

    let publish_packet = publish_builder
        .build()
        .map_err(|e| format!("Failed to build PUBLISH packet: {e:?}"))?;

    let events = connection.checked_send(publish_packet);
    handle_events(&mut stream, &mut connection, events)?;

    if qos_level != mqtt::packet::Qos::AtMostOnce {
        let mut buffer = [0u8; 1024];
        let n = stream.read(&mut buffer)?;
        if n > 0 {
            let mut cursor = mqtt::common::Cursor::new(&buffer[..n]);
            let events = connection.recv(&mut cursor);
            handle_events(&mut stream, &mut connection, events)?;
        }
    }

    println!("Published message to topic '{topic}' with QoS {qos}: {payload}");

    Ok(())
}

fn handle_events(
    stream: &mut TcpStream,
    _connection: &mut mqtt::Connection<mqtt::role::Client>,
    events: Vec<mqtt::Event>,
) -> Result<(), Box<dyn std::error::Error>> {
    for event in events {
        match event {
            mqtt::Event::RequestSendPacket { packet, .. } => {
                let buffer = mqtt::GenericPacketTrait::to_continuous_buffer(&packet);
                stream.write_all(&buffer)?;
                let packet_type = packet.packet_type();
                println!("Sent packet: {packet_type}");
            }
            mqtt::Event::NotifyPacketReceived(packet) => match packet {
                mqtt::packet::Packet::V5_0Connack(connack) => {
                    let reason_code = connack.reason_code();
                    println!("CONNACK received: {reason_code:?}");
                }
                mqtt::packet::Packet::V5_0Puback(puback) => {
                    let packet_id = puback.packet_id();
                    println!("PUBACK received for packet ID: {packet_id}");
                }
                mqtt::packet::Packet::V5_0Pubrec(pubrec) => {
                    let packet_id = pubrec.packet_id();
                    println!("PUBREC received for packet ID: {packet_id}");
                }
                mqtt::packet::Packet::V5_0Pubcomp(pubcomp) => {
                    let packet_id = pubcomp.packet_id();
                    println!("PUBCOMP received for packet ID: {packet_id}");
                }
                _ => {
                    let packet_type = packet.packet_type();
                    println!("Received packet: {packet_type}");
                }
            },
            mqtt::Event::NotifyPacketIdReleased(packet_id) => {
                println!("Packet ID {packet_id} released");
            }
            mqtt::Event::NotifyError(error) => {
                eprintln!("MQTT Error: {error:?}");
            }
            mqtt::Event::RequestClose => {
                println!("Connection close requested");
                return Ok(());
            }
            mqtt::Event::RequestTimerReset { kind, duration_ms } => {
                println!("Timer reset requested: {kind:?} for {duration_ms} ms");
            }
            mqtt::Event::RequestTimerCancel(kind) => {
                println!("Timer cancel requested: {kind:?}");
            }
        }
    }
    Ok(())
}
