//! # MQTT Protocol Core
//!
//! A Sans-I/O style MQTT protocol library for Rust that supports both MQTT v5.0 and v3.1.1.
//!
//! This library provides a pure protocol implementation without any I/O operations,
//! making it suitable for use with any async runtime or synchronous I/O framework.
//! All operations are synchronous and the library focuses solely on MQTT protocol
//! message parsing, validation, and generation.
//!
//! ## Features
//!
//! - **Sans-I/O Design**: Pure protocol implementation with no I/O dependencies
//! - **Dual Version Support**: Full support for both MQTT v3.1.1 and v5.0
//! - **Generic Packet ID**: Supports custom packet ID types (u16, u32) for broker clustering
//! - **Zero-Copy Payload**: Efficient payload handling with `ArcPayload`
//! - **Comprehensive**: All MQTT packet types and features supported
//! - **Type Safety**: Compile-time role and version checking
//!
//! ## Quick Start
//!
//! ### Basic Client Connection
//!
//! ```rust,no_run
//! use mqtt_protocol_core::mqtt::{
//!     Connection, Version,
//!     connection::role::Client,
//!     packet::v5_0::Connect,
//! };
//!
//! // Create a client connection for MQTT v5.0
//! let mut client = Connection::<Client>::new(Version::V5_0);
//!
//! // Create a CONNECT packet
//! let connect = Connect::builder()
//!     .client_id("my-client")
//!     .unwrap()
//!     .clean_start(true)
//!     .build()
//!     .unwrap();
//!
//! // Send the packet through the connection
//! let events = client.send(connect.into());
//! ```
//!
//! ### Server with Version Auto-Detection
//!
//! ```rust,no_run
//! use mqtt_protocol_core::mqtt::{
//!     Connection, Version,
//!     connection::role::Server,
//! };
//!
//! // Create a server that accepts any MQTT version
//! let mut server = Connection::<Server>::new(Version::Undetermined);
//!
//! // The server will automatically adapt to the client's protocol version
//! // when it receives a CONNECT packet
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`mqtt::connection`] - Connection state management and packet processing
//! - [`mqtt::packet`] - MQTT packet definitions for v3.1.1 and v5.0
//! - [`mqtt::Version`] - Protocol version handling
//! - [`mqtt::ArcPayload`] - Efficient payload management
//!
//! ## Sans-I/O Pattern
//!
//! This library follows the Sans-I/O pattern, meaning it handles protocol logic
//! without performing any I/O operations. Instead, it returns events that tell
//! your application what actions to take:
//!
//! ```rust,no_run
//! use mqtt_protocol_core::mqtt::{
//!     Connection, Version,
//!     connection::{role::Client, event::GenericEvent},
//! };
//! use std::io::Cursor;
//!
//! let mut client = Connection::<Client>::new(Version::V5_0);
//! let data = &[0u8; 0][..];
//! let mut data_cursor = Cursor::new(data);
//! let events = client.recv(&mut data_cursor);
//!
//! for event in events {
//!     match event {
//!         GenericEvent::RequestSendPacket { packet, .. } => {
//!             // Send packet over network
//!         }
//!         GenericEvent::NotifyPacketReceived(packet) => {
//!             // Handle received packet
//!         }
//!         // ... other events
//!         _ => {}
//!     }
//! }
//! ```
//!
//! ## Generic Packet ID Support
//!
//! The library supports custom packet ID types for advanced use cases like
//! broker clustering, where u32 packet IDs can prevent ID exhaustion:
//!
//! ```rust,no_run
//! use mqtt_protocol_core::mqtt::{GenericConnection, connection::role::Server};
//!
//! // Use u32 packet IDs instead of standard u16
//! let mut server = GenericConnection::<Server, u32>::new(
//!     mqtt_protocol_core::mqtt::Version::V5_0
//! );
//! ```

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
pub mod logger;
pub mod mqtt;
