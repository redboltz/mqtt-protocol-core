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

//! Default type aliases for MQTT packet types with standard buffer sizes
//!
//! This module provides an example of how to use the make_default_aliases macro
//! to create convenient type aliases for MQTT packet types using the default
//! buffer sizes specified by the library:
//! - String buffer size: 32 bytes
//! - Binary buffer size: 32 bytes
//! - Payload buffer size: 128 bytes
//! - Packet ID type: u16
//!
//! # Usage
//!
//! ```ignore
//! // In your application code:
//! mqtt_protocol_core::make_default_aliases!();
//! use mqtt_protocol_core::mqtt;
//!
//! // Now you can use mqtt::packet::v5_0::Publish with default buffer sizes
//! let publish = mqtt::packet::v5_0::Publish::builder()
//!     .topic_name("my/topic")
//!     .payload(b"hello world")
//!     .build()
//!     .unwrap();
//! ```

// This module no longer generates aliases automatically.
// Users should call mqtt_protocol_core::make_default_aliases!(); in their own code.
