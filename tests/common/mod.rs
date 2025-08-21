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

// Generate default type aliases for all tests
mqtt_protocol_core::make_default_aliases!();

// Generate u32 packet ID aliases for testing
mqtt_protocol_core::make_type_size_aliases!(mqtt_pid32, u32, 32, 32, 128);

#[cfg(feature = "std")]
use std::sync::Once;

#[cfg(feature = "std")]
static INIT: Once = Once::new();

/// Automatic tracing initialization for ALL tests in std environments
///
/// Environment variables:
/// - `RUST_LOG`: Standard Rust logging (takes precedence if set)
/// - `MQTT_LOG_LEVEL`: Set log level (trace, debug, info, warn, error). Default: warn
///
/// Usage examples:
/// - `cargo test --features tracing` (default warn level)
/// - `MQTT_LOG_LEVEL=trace cargo test --features tracing`
/// - `RUST_LOG=debug cargo test --features tracing`
#[cfg(feature = "std")]
fn auto_init_tracing() {
    INIT.call_once(|| {
        // Try RUST_LOG first, then MQTT_LOG_LEVEL, then default to warn
        let filter = if let Ok(rust_log) = std::env::var("RUST_LOG") {
            tracing_subscriber::EnvFilter::new(rust_log)
        } else {
            let level = std::env::var("MQTT_LOG_LEVEL").unwrap_or_else(|_| "warn".to_string());
            tracing_subscriber::EnvFilter::new(format!("mqtt_protocol_core={level}"))
        };

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(true)
            .with_test_writer()
            .init();
    });
}

// no-std環境用のstub関数
#[cfg(not(feature = "std"))]
fn auto_init_tracing() {
    // no-op for no-std environments
}

pub fn init_tracing() {
    auto_init_tracing();
}

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
