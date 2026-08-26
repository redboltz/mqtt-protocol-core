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
/// Protocol-level maximum packet size (no limit)
///
/// 1 byte fixed header + 4 bytes remaining length + 268,435,455 bytes body.
pub const MQTT_PACKET_SIZE_NO_LIMIT: u32 = 1 + 4 + 128 * 128 * 128 * 128;

/// Options fixed at `Connection` construction time
///
/// These settings describe local capabilities that must be effective from the
/// very first received byte and must not change during the lifetime of the
/// connection. They are therefore given once via
/// `Connection::with_options()` instead of setter methods.
///
/// `ConnectionOptions::default()` reproduces the behavior of `Connection::new()`.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let options = mqtt::connection::ConnectionOptions::new().maximum_packet_size_recv(1024 * 1024);
/// let con = mqtt::Connection::<mqtt::role::Server>::with_options(mqtt::Version::V5_0, options);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ConnectionOptions {
    /// Maximum packet size this connection accepts on receive
    ///
    /// Enforced from the first received byte, before any CONNECT/CONNACK
    /// negotiation, on both MQTT v3.1.1 and v5.0. A packet whose total size
    /// exceeds this limit is rejected as soon as its Remaining Length is
    /// decoded, before any buffer for the packet body is allocated.
    ///
    /// On MQTT v5.0, if the CONNECT/CONNACK packet being sent has no
    /// `MaximumPacketSize` property, one carrying this value is added
    /// automatically. An explicit `MaximumPacketSize` property is allowed
    /// only if it is not larger than this value; a larger value is rejected
    /// with `MqttError::ProtocolError` because it would promise the peer a
    /// size that cannot be accepted.
    ///
    /// Default: `MQTT_PACKET_SIZE_NO_LIMIT`.
    pub maximum_packet_size_recv: u32,
}

impl Default for ConnectionOptions {
    fn default() -> Self {
        Self {
            maximum_packet_size_recv: MQTT_PACKET_SIZE_NO_LIMIT,
        }
    }
}

impl ConnectionOptions {
    /// Create options with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the maximum packet size accepted on receive
    ///
    /// Values larger than `MQTT_PACKET_SIZE_NO_LIMIT` are clamped to it.
    /// `0` is not a valid MQTT value and is treated as no limit.
    pub fn maximum_packet_size_recv(mut self, size: u32) -> Self {
        self.maximum_packet_size_recv = if size == 0 {
            MQTT_PACKET_SIZE_NO_LIMIT
        } else {
            size.min(MQTT_PACKET_SIZE_NO_LIMIT)
        };
        self
    }
}
