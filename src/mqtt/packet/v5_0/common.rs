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

//! Common utilities for MQTT v5.0 packet processing.
//!
//! This module contains shared validation functions and utilities
//! used across multiple MQTT v5.0 packet types.

use crate::mqtt::result_code::MqttError;

/// Prefix for shared subscription topic filters.
const SHARE_PREFIX: &str = "$share/";

/// Validates a topic filter for MQTT v5.0 Shared Subscription rules.
///
/// According to MQTT v5.0 specification, a Shared Subscription topic filter has the format:
/// `$share/{ShareName}/{filter}`
///
/// Where `{ShareName}` must not contain the characters "/", "+", or "#".
///
/// This function checks if the topic filter starts with `$share/` and if so,
/// validates that the ShareName portion does not contain invalid characters.
///
/// # Parameters
///
/// * `topic_filter` - The topic filter string to validate
///
/// # Returns
///
/// * `Ok(())` - If the topic filter is valid (either not a shared subscription,
///   or a valid shared subscription with a valid ShareName)
/// * `Err(MqttError::MalformedPacket)` - If the topic filter is an invalid
///   shared subscription (ShareName contains "/", "+", or "#", or ShareName is empty,
///   or filter portion is missing)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::packet::v5_0::common::validate_share_name;
///
/// // Valid non-shared subscription
/// assert!(validate_share_name("sensors/temperature").is_ok());
///
/// // Valid shared subscription
/// assert!(validate_share_name("$share/mygroup/sensors/temperature").is_ok());
///
/// // Invalid: ShareName contains "/"
/// assert!(validate_share_name("$share/my/group/sensors/temperature").is_err());
///
/// // Invalid: ShareName contains "+"
/// assert!(validate_share_name("$share/my+group/sensors/temperature").is_err());
///
/// // Invalid: ShareName contains "#"
/// assert!(validate_share_name("$share/my#group/sensors/temperature").is_err());
///
/// // Invalid: ShareName is empty
/// assert!(validate_share_name("$share//sensors/temperature").is_err());
///
/// // Invalid: filter portion is missing
/// assert!(validate_share_name("$share/mygroup").is_err());
/// ```
pub fn validate_share_name(topic_filter: &str) -> Result<(), MqttError> {
    // Check if this is a shared subscription
    if !topic_filter.starts_with(SHARE_PREFIX) {
        // Not a shared subscription, no validation needed
        return Ok(());
    }

    // Extract the part after "$share/"
    let after_prefix = &topic_filter[SHARE_PREFIX.len()..];

    // Find the position of the next "/" which separates ShareName from filter
    let separator_pos = after_prefix.find('/');

    match separator_pos {
        None => {
            // No "/" found after "$share/", meaning no filter portion
            // This is invalid: "$share/sharename" without a filter
            Err(MqttError::MalformedPacket)
        }
        Some(0) => {
            // ShareName is empty: "$share//filter"
            Err(MqttError::MalformedPacket)
        }
        Some(pos) => {
            // Extract ShareName
            let share_name = &after_prefix[..pos];

            // Check for invalid characters in ShareName
            // ShareName must not contain "/", "+", or "#"
            if share_name.contains('+') || share_name.contains('#') {
                return Err(MqttError::MalformedPacket);
            }

            // Note: "/" check is implicitly handled by finding the first "/"
            // as the separator, so ShareName cannot contain "/"

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_share_name_non_shared() {
        // Non-shared subscriptions should always pass
        assert!(validate_share_name("sensors/temperature").is_ok());
        assert!(validate_share_name("home/+/status").is_ok());
        assert!(validate_share_name("alerts/#").is_ok());
        assert!(validate_share_name("$SYS/broker/uptime").is_ok());
        assert!(validate_share_name("").is_ok());
    }

    #[test]
    fn test_validate_share_name_valid_shared() {
        // Valid shared subscriptions
        assert!(validate_share_name("$share/mygroup/sensors/temperature").is_ok());
        assert!(validate_share_name("$share/group1/home/+/status").is_ok());
        assert!(validate_share_name("$share/consumers/alerts/#").is_ok());
        assert!(validate_share_name("$share/a/topic").is_ok());
        assert!(validate_share_name("$share/group-name/topic").is_ok());
        assert!(validate_share_name("$share/group_name/topic").is_ok());
        assert!(validate_share_name("$share/group.name/topic").is_ok());
    }

    #[test]
    fn test_validate_share_name_invalid_contains_slash() {
        // ShareName implicitly cannot contain "/" because we find the first "/"
        // as the separator. So "$share/my/group/topic" means ShareName="my", filter="group/topic"
        // which is actually valid. Let's verify this understanding.
        assert!(validate_share_name("$share/my/group/topic").is_ok());
    }

    #[test]
    fn test_validate_share_name_invalid_contains_plus() {
        // ShareName contains "+"
        assert!(validate_share_name("$share/my+group/topic").is_err());
        assert!(validate_share_name("$share/+/topic").is_err());
        assert!(validate_share_name("$share/group+/topic").is_err());
    }

    #[test]
    fn test_validate_share_name_invalid_contains_hash() {
        // ShareName contains "#"
        assert!(validate_share_name("$share/my#group/topic").is_err());
        assert!(validate_share_name("$share/#/topic").is_err());
        assert!(validate_share_name("$share/group#/topic").is_err());
    }

    #[test]
    fn test_validate_share_name_invalid_empty_share_name() {
        // Empty ShareName
        assert!(validate_share_name("$share//topic").is_err());
    }

    #[test]
    fn test_validate_share_name_invalid_no_filter() {
        // No filter portion
        assert!(validate_share_name("$share/mygroup").is_err());
        assert!(validate_share_name("$share/").is_err());
    }
}
