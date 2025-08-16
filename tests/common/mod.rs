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
use mqtt_protocol_core::mqtt;

pub fn v3_1_1_client_connecting(
    con: &mut mqtt::Connection<mqtt::role::Client>,
    clean_session: bool,
) {
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_session(clean_session)
        .build()
        .expect("Failed to build Connect packet");
    let _ = con.checked_send(packet);
}

pub fn v3_1_1_client_establish_connection(
    con: &mut mqtt::Connection<mqtt::role::Client>,
    clean_session: bool,
    session_present: bool,
) {
    v3_1_1_client_connecting(con, clean_session);
    let packet = mqtt::packet::v3_1_1::Connack::builder()
        .session_present(session_present)
        .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
        .build()
        .expect("Failed to build Connack packet");
    let flattened: Vec<u8> = packet.to_continuous_buffer();
    let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
    let _ = con.recv(&mut cursor);
}

#[allow(dead_code)]
pub fn v3_1_1_server_connecting(
    con: &mut mqtt::Connection<mqtt::role::Server>,
    clean_session: bool,
) {
    let packet = mqtt::packet::v3_1_1::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .clean_session(clean_session)
        .build()
        .expect("Failed to build Connect packet");
    let flattened: Vec<u8> = packet.to_continuous_buffer();
    let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
    let _ = con.recv(&mut cursor);
}

pub fn v3_1_1_server_establish_connection(
    con: &mut mqtt::Connection<mqtt::role::Server>,
    clean_session: bool,
    session_present: bool,
) {
    v3_1_1_server_connecting(con, clean_session);
    {
        let packet = mqtt::packet::v3_1_1::Connack::builder()
            .session_present(session_present)
            .return_code(mqtt::result_code::ConnectReturnCode::Accepted)
            .build()
            .expect("Failed to build Connack packet");
        let _ = con.checked_send(packet);
    }
}

#[cfg(feature = "std")]
#[allow(dead_code)]
pub fn v5_0_client_establish_connection(con: &mut mqtt::Connection<mqtt::role::Client>) {
    {
        let packet = mqtt::packet::v5_0::Connect::builder()
            .client_id("cid1")
            .unwrap()
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
}

#[cfg(feature = "std")]
#[allow(dead_code)]
pub fn v5_0_server_connecting(con: &mut mqtt::Connection<mqtt::role::Server>) {
    let packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("cid1")
        .unwrap()
        .build()
        .expect("Failed to build Connect packet");
    let flattened: Vec<u8> = packet.to_continuous_buffer();
    let mut cursor = mqtt::common::Cursor::new(&flattened[..]);
    let _ = con.recv(&mut cursor);
}

#[cfg(feature = "std")]
#[allow(dead_code)]
pub fn v5_0_server_establish_connection(con: &mut mqtt::Connection<mqtt::role::Server>) {
    v5_0_server_connecting(con);
    {
        let packet = mqtt::packet::v5_0::Connack::builder()
            .session_present(false)
            .reason_code(mqtt::result_code::ConnectReasonCode::Success)
            .build()
            .expect("Failed to build Connack packet");
        let _ = con.checked_send(packet);
    }
}
