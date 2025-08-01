/**
 * MIT License
 *
 * Copyright (c) 2025 Takatoshi Kondo
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */
use mqtt_protocol_core::logger;
use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::prelude::*;

#[test]
fn test_packet() {
    logger::init(tracing::Level::TRACE);

    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("test_client")
        .unwrap()
        .build()
        .expect("Failed to build Connect packet");
    println!("print: {}", packet);
    println!("print size: {}", packet.size());
    let v: mqtt::packet::Packet = packet.into();
    println!("print variant: {}", v);
    println!("print variant size: {}", v.size());
    let buffers = v.to_buffers();
    println!("print variant to_buffers length: {}", buffers.len());
}

#[test]
fn test_connection() {
    logger::init(tracing::Level::TRACE);

    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .clean_start(true)
            .keep_alive(60u16)
            .user_name("u1")
            .unwrap()
            .password("secret")
            .unwrap()
            .props(vec![
                mqtt::packet::SessionExpiryInterval::new(120)
                    .unwrap()
                    .into(),
                mqtt::packet::UserProperty::new("key", "val")
                    .unwrap()
                    .into(),
                mqtt::packet::AuthenticationMethod::new("token")
                    .unwrap()
                    .into(),
                mqtt::packet::AuthenticationData::new(vec![0xde, 0xad, 0xbe, 0xef])
                    .unwrap()
                    .into(),
                mqtt::packet::RequestProblemInformation::new(1)
                    .unwrap()
                    .into(),
            ])
            .build()
            .expect("Failed to build Connect packet");
        connection.checked_send(packet);
    }
    {
        let packet: mqtt::packet::Packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .build()
            .expect("Failed to build Connect packet")
            .into();
        connection.checked_send(packet);
    }
}
