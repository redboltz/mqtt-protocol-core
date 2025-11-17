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

use core::fmt;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

/// Retain Handling Option for MQTT Subscriptions
///
/// This enum defines how retained messages should be handled when establishing
/// a new subscription. It corresponds to bits 4-5 in the Subscription Options
/// byte as specified in the MQTT v5.0 protocol specification.
///
/// Retained messages are messages that the broker stores and delivers to new
/// subscribers immediately upon subscription. The retain handling option
/// controls this behavior, allowing subscribers to specify whether they want
/// to receive existing retained messages.
///
/// # Protocol Specification
///
/// The retain handling option is encoded in bits 4-5 of the Subscription Options:
/// - Bits 4-5: `00` = Send retained messages at subscribe
/// - Bits 4-5: `01` = Send retained messages only for new subscriptions  
/// - Bits 4-5: `10` = Do not send retained messages at subscribe
/// - Bits 4-5: `11` = Reserved (invalid)
///
/// # Use Cases
///
/// - **SendRetained**: Useful when a subscriber always wants the latest retained value
/// - **SendRetainedIfNotExists**: Prevents duplicate retained messages on subscription renewal
/// - **DoNotSendRetained**: For subscribers that only want new messages
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Always receive retained messages when subscribing
/// let always_retained = mqtt::packet::RetainHandling::SendRetained;
///
/// // Only receive retained messages for new subscriptions
/// let new_only = mqtt::packet::RetainHandling::SendRetainedIfNotExists;
///
/// // Never receive retained messages
/// let no_retained = mqtt::packet::RetainHandling::DoNotSendRetained;
///
/// // Convert from byte value
/// let from_byte = mqtt::packet::RetainHandling::try_from(1u8).unwrap();
/// assert_eq!(from_byte, mqtt::packet::RetainHandling::SendRetainedIfNotExists);
/// ```
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, TryFromPrimitive,
)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum RetainHandling {
    /// Send retained messages at the time of the subscribe
    ///
    /// When a subscription is established, the broker will immediately send
    /// any retained messages that match the topic filter to the subscriber.
    /// This is the default behavior and ensures subscribers always receive
    /// the most recent retained value for matching topics.
    ///
    /// This option is suitable for:
    /// - Status monitoring applications that need current state
    /// - Subscribers that always want the latest retained value
    /// - Applications where missing the current retained value would be problematic
    SendRetained = 0,

    /// Send retained messages at subscribe only if the subscription does not currently exist
    ///
    /// The broker will send retained messages only if this is a completely new
    /// subscription. If the client already has an active subscription to the
    /// same topic filter (even with different QoS), retained messages will not
    /// be sent again. This prevents duplicate delivery of retained messages.
    ///
    /// This option is suitable for:
    /// - Preventing duplicate retained messages during connection recovery
    /// - Applications that upgrade/downgrade subscription QoS levels
    /// - Scenarios where processing the same retained message twice is undesirable
    SendRetainedIfNotExists = 1,

    /// Do not send retained messages at the time of the subscribe
    ///
    /// The broker will not send any retained messages when the subscription
    /// is established, regardless of whether matching retained messages exist.
    /// The subscriber will only receive newly published messages after the
    /// subscription is active.
    ///
    /// This option is suitable for:
    /// - Event-driven applications that only care about new events
    /// - Applications where retained messages represent historical data
    /// - Scenarios where the current retained value is not relevant to the subscriber
    DoNotSendRetained = 2,
}

/// Implementation of `Display` for `RetainHandling`
///
/// Formats the retain handling option as a human-readable string representation.
/// This allows retain handling values to be used with `println!`, `format!`, and
/// other string formatting operations.
///
/// # Output Format
///
/// * `RetainHandling::SendRetained` -> "SendRetained"
/// * `RetainHandling::SendRetainedIfNotExists` -> "SendRetainedIfNotExists"
/// * `RetainHandling::DoNotSendRetained` -> "DoNotSendRetained"
impl fmt::Display for RetainHandling {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::SendRetained => "SendRetained",
            Self::SendRetainedIfNotExists => "SendRetainedIfNotExists",
            Self::DoNotSendRetained => "DoNotSendRetained",
        };
        write!(f, "{s}")
    }
}

/// Implementation of `Default` for `RetainHandling`
///
/// Returns `SendRetained` as the default retain handling behavior.
/// This matches the MQTT protocol default where retained messages
/// are sent to new subscribers unless explicitly configured otherwise.
///
/// # Returns
///
/// `RetainHandling::SendRetained` - The default retain handling behavior
impl Default for RetainHandling {
    fn default() -> Self {
        Self::SendRetained
    }
}
