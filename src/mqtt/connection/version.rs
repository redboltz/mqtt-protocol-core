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

/// MQTT protocol version enumeration
///
/// Represents the supported MQTT protocol versions. Each variant corresponds
/// to the protocol level value as defined in the MQTT specification.
/// This is used throughout the library to handle version-specific behavior
/// and feature availability.
///
/// # Protocol Level Values
///
/// The numeric values correspond to the Protocol Level field in the CONNECT packet:
/// - v3.1.1 uses protocol level 4
/// - v5.0 uses protocol level 5
/// - `Undetermined` is used by servers to accept any supported version
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::version::Version;
///
/// // Client specifies a specific version
/// let client_version = Version::V5_0;
///
/// // Server accepts any supported version
/// let server_version = Version::Undetermined;
///
/// match server_version {
///     Version::V3_1_1 => println!("Using MQTT v3.1.1"),
///     Version::V5_0 => println!("Using MQTT v5.0"),
///     Version::Undetermined => println!("Waiting for client CONNECT to determine version"),
/// }
/// ```
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Version {
    /// Version to be determined by incoming CONNECT packet
    ///
    /// Used by MQTT servers to indicate that the protocol version should be
    /// determined based on the protocol level in the incoming CONNECT packet
    /// from the client. The server will accept either v3.1.1 or v5.0 connections
    /// and adapt accordingly. Not a valid protocol level value itself.
    Undetermined = 0,

    /// MQTT version 3.1.1 (protocol level 4)
    ///
    /// The widely adopted MQTT v3.1.1 specification. This version provides
    /// the core MQTT functionality including QoS levels, retained messages,
    /// last will and testament, and persistent sessions.
    V3_1_1 = 4,

    /// MQTT version 5.0 (protocol level 5)
    ///
    /// The latest MQTT specification with enhanced features including:
    /// - User properties and reason codes
    /// - Topic aliases and subscription identifiers  
    /// - Message expiry and session expiry intervals
    /// - Enhanced authentication (AUTH packets)
    /// - Improved error handling and flow control
    V5_0 = 5,
}
