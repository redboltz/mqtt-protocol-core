#![cfg_attr(not(feature = "std"), no_std)]

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
//!     common::Cursor,
//! };
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
//!
//! ## No-std Support
//!
//! This library fully supports `no_std` environments for embedded systems.
//! To use in a `no_std` environment, disable the default `std` feature:
//!
//! ```toml
//! [dependencies]
//! mqtt-protocol-core = { version = "0.5.1", default-features = false }
//! ```
//!
//! **No-std usage example:**
//!
//! ```rust,no_run,ignore
//! #![no_std]
//! extern crate alloc;
//!
//! use alloc::{vec::Vec, string::String};
//! use mqtt_protocol_core::mqtt::{
//!     Connection, Version,
//!     connection::role::Client,
//!     packet::v5_0::Connect,
//!     common::Cursor,
//! };
//!
//! fn main() {
//!     // Create client connection
//!     let mut client = Connection::<Client>::new(Version::V5_0);
//!
//!     // Create CONNECT packet
//!     let connect = Connect::builder()
//!         .client_id("embedded-client")
//!         .unwrap()
//!         .clean_start(true)
//!         .build()
//!         .unwrap();
//!
//!     // Send packet and handle events
//!     let events = client.send(connect.into());
//!
//!     // Process events in your embedded application
//!     for event in events {
//!         match event {
//!             // Handle RequestSendPacket, NotifyPacketReceived, etc.
//!             _ => {}
//!         }
//!     }
//! }
//! ```
//!
//! ## Feature Flags
//!
//! ### Core Features
//!
//! - **`std`** (default): Enables standard library support, including `std::io::IoSlice` for vectored I/O
//! - **`tracing`**: Enables logging support via the `tracing` crate. When disabled, trace statements compile to no-ops with zero overhead
//!
//! ```toml
//! # Enable tracing support (independent of std)
//! [dependencies]
//! mqtt-protocol-core = { version = "0.5.1", default-features = false, features = ["tracing"] }
//!
//! # Use with std but without tracing overhead
//! [dependencies]
//! mqtt-protocol-core = { version = "0.5.1", default-features = false, features = ["std"] }
//!
//! # Full-featured (std + tracing)
//! [dependencies]
//! mqtt-protocol-core = { version = "0.5.1", features = ["tracing"] }
//! ```
//!
//! ### Small String Optimization (SSO) Features
//!
//! These features optimize memory usage for small strings and binary data by storing them
//! on the stack instead of allocating on the heap:
//!
//! - **`sso-min-32bit`**: Minimal optimization for 32-bit environments
//!   - MqttString/MqttBinary: 12 bytes total buffer
//!   - ArcPayload: 15 bytes data buffer
//!
//! - **`sso-min-64bit`**: Recommended optimization for 64-bit environments
//!   - MqttString/MqttBinary: 24 bytes total buffer
//!   - ArcPayload: 31 bytes data buffer
//!
//! - **`sso-lv10`**: Level 10 optimization for moderate performance gains
//!   - MqttString/MqttBinary: 24 bytes total buffer
//!   - ArcPayload: 127 bytes data buffer
//!
//! - **`sso-lv20`**: Level 20 optimization for maximum performance
//!   - MqttString/MqttBinary: 48 bytes total buffer
//!   - ArcPayload: 255 bytes data buffer
//!
//! ```toml
//! # Use specific SSO optimization level
//! [dependencies]
//! mqtt-protocol-core = { version = "0.5.1", features = ["sso-lv10"] }
//!
//! # Combine with other features
//! [dependencies]
//! mqtt-protocol-core = { version = "0.5.1", features = ["std", "sso-lv20", "tracing"] }
//! ```
//!
//! #### ⚠️ **Critical: SSO Feature Flag Propagation**
//!
//! **When your crate depends on mqtt-protocol-core and is consumed by other crates,
//! you should re-export all SSO feature flags to ensure proper feature selection.**
//!
//! **Multiple SSO Features:** When multiple SSO features are enabled simultaneously,
//! the system automatically selects the **largest buffer size** from the enabled features.
//! This allows safe usage with `--all-features` and prevents compilation errors.
//!
//! **Feature Selection Priority:**
//! 1. `sso-lv20` (highest): 48-byte String/Binary, 255-byte ArcPayload
//! 2. `sso-lv10` or `sso-min-64bit`: 24-byte String/Binary, 127-byte ArcPayload
//! 3. `sso-min-32bit` (lowest): 12-byte String/Binary, 15-byte ArcPayload
//!
//! **Required pattern for library authors:**
//!
//! ```toml
//! # Your crate's Cargo.toml
//! [dependencies]
//! mqtt-protocol-core = { version = "0.5.1", features = ["sso-lv10"] }
//!
//! [features]
//! # MANDATORY: Re-export ALL SSO features to allow downstream configuration
//! sso-min-32bit = ["mqtt-protocol-core/sso-min-32bit"]
//! sso-min-64bit = ["mqtt-protocol-core/sso-min-64bit"]
//! sso-lv10 = ["mqtt-protocol-core/sso-lv10"]
//! sso-lv20 = ["mqtt-protocol-core/sso-lv20"]
//! ```
//!
//! This pattern ensures that when multiple dependency crates enable different SSO levels,
//! the final application receives the maximum optimization level from all dependencies.
//!
//! **Key points for no-std usage:**
//! - Use `extern crate alloc;` to enable heap allocations
//! - Import types from `alloc` crate instead of `std`
//! - `IoSlice` functionality is not available in `no_std` mode
//! - Tracing can be enabled independently of `std` for embedded debugging

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

// Always use alloc types for consistency between std and no-std
#[macro_use]
extern crate alloc;

// Common prelude with alloc types
pub mod prelude {
    pub use alloc::{boxed::Box, format, string::String, vec, vec::Vec};

    #[cfg(feature = "std")]
    pub use std::io::IoSlice;
}

pub mod mqtt;
