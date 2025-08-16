use core::fmt;
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
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

/// MQTT Quality of Service levels
///
/// Defines the delivery guarantee levels for MQTT message publishing and subscription.
/// Each QoS level provides different guarantees about message delivery, with higher
/// levels providing stronger guarantees at the cost of increased overhead.
///
/// The QoS level is specified in PUBLISH packets and affects how the message is
/// handled by both the sender and receiver according to the MQTT protocol specification.
///
/// # QoS Level Descriptions
///
/// - **QoS 0 (At Most Once)**: Fire-and-forget delivery. Messages are delivered
///   at most once, or not at all. No acknowledgment is required.
/// - **QoS 1 (At Least Once)**: Acknowledged delivery. Messages are delivered
///   at least once. Duplicates may occur but will be handled by the receiver.
/// - **QoS 2 (Exactly Once)**: Assured delivery. Messages are delivered exactly
///   once using a four-part handshake protocol.
///
/// # Protocol Behavior
///
/// Each QoS level has specific protocol requirements:
///
/// - **QoS 0**: PUBLISH packet is sent without expecting acknowledgment
/// - **QoS 1**: PUBLISH packet must be acknowledged with PUBACK
/// - **QoS 2**: PUBLISH packet initiates a handshake: PUBLISH -> PUBREC -> PUBREL -> PUBCOMP
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Fire-and-forget message delivery
/// let qos0 = mqtt::packet::Qos::AtMostOnce;
///
/// // Acknowledged delivery with possible duplicates
/// let qos1 = mqtt::packet::Qos::AtLeastOnce;
///
/// // Exactly-once delivery with handshake
/// let qos2 = mqtt::packet::Qos::ExactlyOnce;
///
/// // Convert from byte value
/// let qos_from_byte = mqtt::packet::Qos::try_from(1u8).unwrap();
/// assert_eq!(qos_from_byte, mqtt::packet::Qos::AtLeastOnce);
/// ```
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive,
)]
#[repr(u8)]
pub enum Qos {
    /// QoS level 0: At most once delivery
    ///
    /// Messages are delivered at most once, or not at all. There is no
    /// acknowledgment of delivery and no retry mechanism. This is the
    /// fastest delivery method but provides no guarantee that the message
    /// will be received.
    ///
    /// This level is suitable for:
    /// - High-frequency sensor data where missing occasional readings is acceptable
    /// - Applications where message loss is tolerable
    /// - Scenarios where network bandwidth and battery life are critical
    AtMostOnce = 0,

    /// QoS level 1: At least once delivery
    ///
    /// Messages are delivered at least once. The receiver acknowledges
    /// delivery with a PUBACK packet. If the sender doesn't receive the
    /// acknowledgment within a reasonable time, it retransmits the message.
    /// Duplicate messages may occur and should be handled by the application.
    ///
    /// This level is suitable for:
    /// - Applications that can handle duplicate messages
    /// - Scenarios where message delivery is important but duplicates are acceptable
    /// - Most general-purpose messaging use cases
    AtLeastOnce = 1,

    /// QoS level 2: Exactly once delivery
    ///
    /// Messages are delivered exactly once using a four-part handshake
    /// protocol (PUBLISH -> PUBREC -> PUBREL -> PUBCOMP). This guarantees
    /// that messages are neither lost nor duplicated, but requires the
    /// most network traffic and processing overhead.
    ///
    /// This level is suitable for:
    /// - Financial transactions or billing information
    /// - Critical control messages
    /// - Applications where duplicate processing would cause serious problems
    ExactlyOnce = 2,
}

/// Implementation of `Display` for `Qos`
///
/// Formats the QoS level as a human-readable string representation.
/// This allows QoS values to be used with `println!`, `format!`, and
/// other string formatting operations.
///
/// # Output Format
///
/// * `Qos::AtMostOnce` -> "AtMostOnce"
/// * `Qos::AtLeastOnce` -> "AtLeastOnce"
/// * `Qos::ExactlyOnce` -> "ExactlyOnce"
impl fmt::Display for Qos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::AtMostOnce => "AtMostOnce",
            Self::AtLeastOnce => "AtLeastOnce",
            Self::ExactlyOnce => "ExactlyOnce",
        };
        write!(f, "{s}")
    }
}
