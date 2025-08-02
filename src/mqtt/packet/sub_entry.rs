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
use crate::mqtt::packet::MqttString;
use crate::mqtt::packet::Qos;
use crate::mqtt::packet::RetainHandling;
use crate::mqtt::result_code::MqttError;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use std::fmt;
use std::io::IoSlice;

/// MQTT Subscription Options
///
/// Represents the subscription options byte used in SUBSCRIBE packets as defined
/// in MQTT v5.0 specification. This single byte contains multiple bit fields that
/// control various aspects of subscription behavior including QoS level, retain
/// handling, and MQTT v5.0 specific flags.
///
/// # Bit Layout
///
/// The subscription options byte is structured as follows:
/// ```text
/// Bit:  7  6  5  4  3  2  1  0
///      [Reserved] [RH] [RAP][NL][QoS]
/// ```
///
/// Where:
/// - **Bits 0-1**: QoS level (0, 1, or 2)
/// - **Bit 2**: No Local flag (NL)
/// - **Bit 3**: Retain As Published flag (RAP)
/// - **Bits 4-5**: Retain Handling option (RH)
/// - **Bits 6-7**: Reserved (must be 0)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Create default subscription options (QoS 0, all flags false)
/// let opts = mqtt::packet::SubOpts::new();
///
/// // Configure specific options
/// let opts = mqtt::packet::SubOpts::new()
///     .set_qos(mqtt::packet::Qos::AtLeastOnce)
///     .set_nl(true)  // No Local flag
///     .set_rap(true) // Retain As Published flag
///     .set_rh(mqtt::packet::RetainHandling::DoNotSendRetained);
///
/// // Parse from byte value
/// let opts = mqtt::packet::SubOpts::from_u8(0x25).unwrap();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubOpts {
    /// Single byte containing all subscription option flags
    sub_opts_buf: [u8; 1],
}

impl SubOpts {
    /// Create new subscription options with default values
    ///
    /// Creates a `SubOpts` instance with all options set to their default values:
    /// - QoS: AtMostOnce (0)
    /// - No Local flag: false
    /// - Retain As Published flag: false
    /// - Retain Handling: SendRetained (0)
    ///
    /// # Returns
    ///
    /// A new `SubOpts` instance with default settings
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new();
    /// assert_eq!(opts.qos(), mqtt::packet::Qos::AtMostOnce);
    /// assert_eq!(opts.nl(), false);
    /// assert_eq!(opts.rap(), false);
    /// ```
    pub fn new() -> Self {
        Self { sub_opts_buf: [0] }
    }

    /// Create subscription options from a byte value
    ///
    /// Parses a subscription options byte and validates that all fields contain
    /// valid values according to the MQTT v5.0 specification. This method performs
    /// comprehensive validation to ensure protocol compliance.
    ///
    /// # Parameters
    ///
    /// * `value` - The subscription options byte to parse
    ///
    /// # Returns
    ///
    /// * `Ok(SubOpts)` - Successfully parsed and validated subscription options
    /// * `Err(MqttError::MalformedPacket)` - If any field contains invalid values
    ///
    /// # Validation Rules
    ///
    /// 1. Reserved bits (6-7) must be 0
    /// 2. QoS value (bits 0-1) must be 0, 1, or 2
    /// 3. Retain Handling value (bits 4-5) must be 0, 1, or 2
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Valid subscription options byte
    /// let opts = mqtt::packet::SubOpts::from_u8(0x25).unwrap();
    ///
    /// // Invalid: reserved bits set
    /// assert!(mqtt::packet::SubOpts::from_u8(0xC0).is_err());
    ///
    /// // Invalid: QoS value 3
    /// assert!(mqtt::packet::SubOpts::from_u8(0x03).is_err());
    /// ```
    pub fn from_u8(value: u8) -> Result<Self, MqttError> {
        // 1. Error if reserved bits (bits 6-7) are not 0
        if (value & 0b1100_0000) != 0 {
            return Err(MqttError::MalformedPacket);
        }

        // 2. Error if QoS is not 0, 1, or 2
        let qos_value = value & 0b0000_0011;
        if qos_value > 2 {
            return Err(MqttError::MalformedPacket);
        }

        // 3. Error if Retain Handling is not 0, 1, or 2
        let rh_value = (value & 0b0011_0000) >> 4;
        if rh_value > 2 {
            return Err(MqttError::MalformedPacket);
        }

        // All validations passed, return SubOpts instance
        Ok(Self {
            sub_opts_buf: [value],
        })
    }

    /// Get the QoS level from subscription options
    ///
    /// Extracts and returns the Quality of Service level from bits 0-1
    /// of the subscription options byte. The QoS level determines the
    /// delivery guarantee for messages matching this subscription.
    ///
    /// # Returns
    ///
    /// The QoS level as a `Qos` enum value:
    /// - `Qos::AtMostOnce` for value 0
    /// - `Qos::AtLeastOnce` for value 1  
    /// - `Qos::ExactlyOnce` for value 2
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    /// assert_eq!(opts.qos(), mqtt::packet::Qos::AtLeastOnce);
    /// ```
    pub fn qos(&self) -> Qos {
        // Extract bits 0-1 value
        let qos_value = self.sub_opts_buf[0] & 0b0000_0011;

        // Safe conversion (only uses values 0, 1, 2)
        match qos_value {
            0 => Qos::AtMostOnce,
            1 => Qos::AtLeastOnce,
            2 => Qos::ExactlyOnce,
            _ => unreachable!("Invalid QoS value: {}, this should never happen", qos_value),
        }
    }

    /// Set the QoS level in subscription options
    ///
    /// Updates bits 0-1 of the subscription options byte with the specified
    /// QoS level. This method uses a builder pattern, consuming and returning
    /// the `SubOpts` instance to allow method chaining.
    ///
    /// # Parameters
    ///
    /// * `qos` - The QoS level to set
    ///
    /// # Returns
    ///
    /// The updated `SubOpts` instance with the new QoS level
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new()
    ///     .set_qos(mqtt::packet::Qos::ExactlyOnce);
    /// assert_eq!(opts.qos(), mqtt::packet::Qos::ExactlyOnce);
    /// ```
    pub fn set_qos(mut self, qos: Qos) -> Self {
        self.sub_opts_buf[0] &= 0b1111_1100;
        self.sub_opts_buf[0] |= qos as u8;
        self
    }

    /// Get the No Local flag from subscription options
    ///
    /// Extracts the No Local flag from bit 2 of the subscription options byte.
    /// When set to true, messages published by this client will not be forwarded
    /// back to it, even if it has a matching subscription.
    ///
    /// This flag is useful for preventing message loops in scenarios where
    /// a client both publishes and subscribes to the same topics.
    ///
    /// # Returns
    ///
    /// `true` if the No Local flag is set, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new().set_nl(true);
    /// assert_eq!(opts.nl(), true);
    /// ```
    pub fn nl(&self) -> bool {
        (self.sub_opts_buf[0] & 0b0000_0100) != 0
    }

    /// Set the No Local flag in subscription options
    ///
    /// Updates bit 2 of the subscription options byte with the specified
    /// No Local flag value. This method uses a builder pattern, consuming
    /// and returning the `SubOpts` instance to allow method chaining.
    ///
    /// # Parameters
    ///
    /// * `nl` - The No Local flag value to set
    ///
    /// # Returns
    ///
    /// The updated `SubOpts` instance with the new No Local flag
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Enable No Local to prevent message loops
    /// let opts = mqtt::packet::SubOpts::new().set_nl(true);
    /// assert_eq!(opts.nl(), true);
    /// ```
    pub fn set_nl(mut self, nl: bool) -> Self {
        if nl {
            self.sub_opts_buf[0] |= 0b0000_0100; // Set bit 2
        } else {
            self.sub_opts_buf[0] &= !0b0000_0100; // Clear bit 2
        }
        self
    }

    /// Get the Retain As Published flag from subscription options
    ///
    /// Extracts the Retain As Published flag from bit 3 of the subscription
    /// options byte. When set to true, messages forwarded to this subscription
    /// will keep their original RETAIN flag value. When false, forwarded
    /// messages will have their RETAIN flag set to 0.
    ///
    /// This flag affects how the broker handles the RETAIN flag when forwarding
    /// messages to this specific subscription.
    ///
    /// # Returns
    ///
    /// `true` if the Retain As Published flag is set, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new().set_rap(true);
    /// assert_eq!(opts.rap(), true);
    /// ```
    pub fn rap(&self) -> bool {
        (self.sub_opts_buf[0] & 0b0000_1000) != 0
    }

    /// Set the Retain As Published flag in subscription options
    ///
    /// Updates bit 3 of the subscription options byte with the specified
    /// Retain As Published flag value. This method uses a builder pattern,
    /// consuming and returning the `SubOpts` instance to allow method chaining.
    ///
    /// # Parameters
    ///
    /// * `rap` - The Retain As Published flag value to set
    ///
    /// # Returns
    ///
    /// The updated `SubOpts` instance with the new Retain As Published flag
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Preserve original RETAIN flag in forwarded messages
    /// let opts = mqtt::packet::SubOpts::new().set_rap(true);
    /// assert_eq!(opts.rap(), true);
    /// ```
    pub fn set_rap(mut self, rap: bool) -> Self {
        if rap {
            self.sub_opts_buf[0] |= 0b0000_1000; // Set bit 3
        } else {
            self.sub_opts_buf[0] &= !0b0000_1000; // Clear bit 3
        }
        self
    }

    /// Get the Retain Handling option from subscription options
    ///
    /// Extracts and returns the Retain Handling option from bits 4-5
    /// of the subscription options byte. This option controls how retained
    /// messages are handled when the subscription is established.
    ///
    /// # Returns
    ///
    /// The retain handling option as a `RetainHandling` enum value:
    /// - `RetainHandling::SendRetained` for value 0
    /// - `RetainHandling::SendRetainedIfNotExists` for value 1
    /// - `RetainHandling::DoNotSendRetained` for value 2
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new()
    ///     .set_rh(mqtt::packet::RetainHandling::DoNotSendRetained);
    /// assert_eq!(opts.rh(), mqtt::packet::RetainHandling::DoNotSendRetained);
    /// ```
    pub fn rh(&self) -> RetainHandling {
        let rh_value = (self.sub_opts_buf[0] & 0b0011_0000) >> 4;

        match rh_value {
            0 => RetainHandling::SendRetained,
            1 => RetainHandling::SendRetainedIfNotExists,
            2 => RetainHandling::DoNotSendRetained,
            _ => unreachable!(
                "Invalid RetainHandling value: {}, this should never happen",
                rh_value
            ),
        }
    }

    /// Set the Retain Handling option in subscription options
    ///
    /// Updates bits 4-5 of the subscription options byte with the specified
    /// retain handling option. This method uses a builder pattern, consuming
    /// and returning the `SubOpts` instance to allow method chaining.
    ///
    /// # Parameters
    ///
    /// * `rh` - The retain handling option to set
    ///
    /// # Returns
    ///
    /// The updated `SubOpts` instance with the new retain handling option
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new()
    ///     .set_rh(mqtt::packet::RetainHandling::SendRetainedIfNotExists);
    /// assert_eq!(opts.rh(), mqtt::packet::RetainHandling::SendRetainedIfNotExists);
    /// ```
    pub fn set_rh(mut self, rh: RetainHandling) -> Self {
        self.sub_opts_buf[0] &= 0b1100_1111;
        self.sub_opts_buf[0] |= (rh as u8) << 4;
        self
    }

    /// Get the raw subscription options byte buffer
    ///
    /// Returns a reference to the internal byte buffer containing the
    /// encoded subscription options. This can be used for direct serialization
    /// to the MQTT wire format.
    ///
    /// # Returns
    ///
    /// A reference to the single-byte buffer containing the encoded options
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    /// let buffer = opts.to_buffer();
    /// assert_eq!(buffer[0] & 0x03, 1); // QoS bits should be 01
    /// ```
    pub fn to_buffer(&self) -> &[u8; 1] {
        &self.sub_opts_buf
    }
}
/// Implementation of `Default` for `SubOpts`
///
/// Creates subscription options with all default values.
/// This is equivalent to calling `SubOpts::new()`.
impl Default for SubOpts {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of `Display` for `SubOpts`
///
/// Formats the subscription options as a JSON string for human-readable output.
/// This is particularly useful for logging and debugging purposes.
/// If JSON serialization fails, an error message is displayed instead.
impl fmt::Display for SubOpts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(e) => write!(f, "{{\"error\": \"{}\"}}", e),
        }
    }
}

/// Implementation of `Serialize` for `SubOpts`
///
/// Serializes the subscription options to a structured format with individual
/// fields for each option. This allows the options to be serialized to JSON
/// format with clear field names and values.
///
/// # Serialized Fields
///
/// - `qos`: Quality of Service level as a string
/// - `nl`: No Local flag as a boolean
/// - `rap`: Retain As Published flag as a boolean
/// - `rh`: Retain Handling option as a string
impl Serialize for SubOpts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Define field count
        let mut state = serializer.serialize_struct("SubOpts", 4)?;

        // Serialize each option
        state.serialize_field("qos", &self.qos().to_string())?;
        state.serialize_field("nl", &self.nl())?;
        state.serialize_field("rap", &self.rap())?;
        state.serialize_field("rh", &self.rh().to_string())?;

        state.end()
    }
}

/// MQTT Subscription Entry
///
/// Represents a single subscription entry consisting of a topic filter and
/// subscription options. This structure is used in SUBSCRIBE packets to
/// specify what topics to subscribe to and how messages should be handled.
///
/// Each subscription entry contains:
/// - A topic filter string that may include wildcards (`+` and `#`)
/// - Subscription options that control message delivery behavior
///
/// # Topic Filter Format
///
/// Topic filters follow MQTT specification rules:
/// - Single-level wildcard: `+` matches any single level
/// - Multi-level wildcard: `#` matches any number of levels (must be last)
/// - Example: `home/+/temperature` or `sensors/#`
///
/// # Wire Format
///
/// In the MQTT wire protocol, each subscription entry is encoded as:
/// 1. Topic filter as an MQTT string (2-byte length + UTF-8 bytes)
/// 2. Subscription options as a single byte
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Basic subscription with default options
/// let entry = mqtt::packet::SubEntry::new(
///     "sensors/temperature",
///     mqtt::packet::SubOpts::new()
/// ).unwrap();
///
/// // Subscription with custom options
/// let opts = mqtt::packet::SubOpts::new()
///     .set_qos(mqtt::packet::Qos::AtLeastOnce)
///     .set_nl(true);
/// let entry = mqtt::packet::SubEntry::new("home/+/status", opts).unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubEntry {
    /// The topic filter string for this subscription
    topic_filter: MqttString,
    /// The subscription options controlling message delivery
    sub_opts: SubOpts,
}

impl SubEntry {
    /// Create a new subscription entry
    ///
    /// Creates a `SubEntry` with the specified topic filter and subscription options.
    /// The topic filter is validated to ensure it's a valid UTF-8 string and within
    /// the MQTT size limits (maximum 65,535 bytes).
    ///
    /// # Parameters
    ///
    /// * `topic_filter` - The topic filter string (may contain wildcards + and #)
    /// * `sub_opts` - The subscription options controlling message delivery
    ///
    /// # Returns
    ///
    /// * `Ok(SubEntry)` - Successfully created subscription entry
    /// * `Err(MqttError::MalformedPacket)` - If topic filter exceeds maximum length
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Basic subscription
    /// let entry = mqtt::packet::SubEntry::new(
    ///     "sensors/temperature",
    ///     mqtt::packet::SubOpts::new()
    /// ).unwrap();
    ///
    /// // Subscription with wildcards and custom options
    /// let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    /// let entry = mqtt::packet::SubEntry::new("home/+/status", opts).unwrap();
    /// ```
    pub fn new(topic_filter: impl AsRef<str>, sub_opts: SubOpts) -> Result<Self, MqttError> {
        let topic_filter = MqttString::new(topic_filter)?;
        Ok(Self {
            topic_filter,
            sub_opts,
        })
    }

    /// Get the topic filter as a string slice
    ///
    /// Returns the topic filter string for this subscription entry.
    /// The topic filter may contain MQTT wildcards (`+` for single level,
    /// `#` for multiple levels).
    ///
    /// # Returns
    ///
    /// A string slice containing the topic filter
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let entry = mqtt::packet::SubEntry::new("sensors/+/temperature",
    ///                                        mqtt::packet::SubOpts::new()).unwrap();
    /// assert_eq!(entry.topic_filter(), "sensors/+/temperature");
    /// ```
    pub fn topic_filter(&self) -> &str {
        &self.topic_filter.as_str()
    }

    /// Get the subscription options
    ///
    /// Returns a reference to the subscription options that control
    /// how messages matching this topic filter should be delivered.
    ///
    /// # Returns
    ///
    /// A reference to the `SubOpts` containing the subscription options
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    /// let entry = mqtt::packet::SubEntry::new("test/topic", opts).unwrap();
    /// assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::AtLeastOnce);
    /// ```
    pub fn sub_opts(&self) -> &SubOpts {
        &self.sub_opts
    }

    /// Set the topic filter for this subscription entry
    ///
    /// Updates the topic filter with a new value. The new topic filter
    /// is validated to ensure it's valid UTF-8 and within size limits.
    ///
    /// # Parameters
    ///
    /// * `topic_filter` - The new topic filter string
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Topic filter updated successfully
    /// * `Err(MqttError::MalformedPacket)` - If topic filter exceeds maximum length
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mut entry = mqtt::packet::SubEntry::new("old/topic",
    ///                                           mqtt::packet::SubOpts::new()).unwrap();
    /// entry.set_topic_filter("new/topic".to_string()).unwrap();
    /// assert_eq!(entry.topic_filter(), "new/topic");
    /// ```
    pub fn set_topic_filter(&mut self, topic_filter: String) -> Result<(), MqttError> {
        self.topic_filter = MqttString::new(topic_filter)?;
        Ok(())
    }

    /// Set the subscription options for this entry
    ///
    /// Updates the subscription options that control how messages
    /// matching this topic filter should be delivered.
    ///
    /// # Parameters
    ///
    /// * `sub_opts` - The new subscription options
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mut entry = mqtt::packet::SubEntry::new("test/topic",
    ///                                           mqtt::packet::SubOpts::new()).unwrap();
    /// let new_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::ExactlyOnce);
    /// entry.set_sub_opts(new_opts);
    /// assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::ExactlyOnce);
    /// ```
    pub fn set_sub_opts(&mut self, sub_opts: SubOpts) {
        self.sub_opts = sub_opts;
    }

    /// Create IoSlice buffers for efficient network I/O
    ///
    /// Returns a vector of `IoSlice` objects that can be used for vectored I/O
    /// operations, allowing zero-copy writes to network sockets. The buffers
    /// contain the complete wire format representation of this subscription entry.
    ///
    /// # Returns
    ///
    /// A vector of `IoSlice` buffers containing:
    /// 1. Topic filter with length prefix
    /// 2. Subscription options byte
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let entry = mqtt::packet::SubEntry::new("test/topic",
    ///                                        mqtt::packet::SubOpts::new()).unwrap();
    /// let buffers = entry.to_buffers();
    /// // Can be used with vectored write operations
    /// // socket.write_vectored(&buffers)?;
    /// ```
    pub fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        let mut buffers = self.topic_filter.to_buffers();
        buffers.push(IoSlice::new(self.sub_opts.to_buffer()));
        buffers
    }

    /// Get the total encoded size of this subscription entry
    ///
    /// Returns the number of bytes this subscription entry will occupy
    /// in the MQTT wire format, including the topic filter with its
    /// length prefix and the subscription options byte.
    ///
    /// # Returns
    ///
    /// The total size in bytes for the wire format representation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let entry = mqtt::packet::SubEntry::new("test",
    ///                                        mqtt::packet::SubOpts::new()).unwrap();
    /// // Size = 2 bytes (length prefix) + 4 bytes ("test") + 1 byte (options) = 7 bytes
    /// assert_eq!(entry.size(), 7);
    /// ```
    pub fn size(&self) -> usize {
        self.topic_filter.size() + self.sub_opts.to_buffer().len()
    }

    /// Parse a subscription entry from byte data
    ///
    /// Decodes a subscription entry from the MQTT wire format, which consists
    /// of a topic filter (MQTT string with length prefix) followed by a
    /// subscription options byte.
    ///
    /// # Parameters
    ///
    /// * `data` - Byte buffer containing the encoded subscription entry
    ///
    /// # Returns
    ///
    /// * `Ok((SubEntry, bytes_consumed))` - Successfully parsed entry and number of bytes consumed
    /// * `Err(MqttError::MalformedPacket)` - If the data is malformed or incomplete
    ///
    /// # Wire Format
    ///
    /// 1. Topic filter as MQTT string (2-byte length + UTF-8 bytes)
    /// 2. Subscription options as single byte
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // Buffer containing: length=4, "test", options=0x01
    /// let buffer = &[0x00, 0x04, b't', b'e', b's', b't', 0x01];
    /// let (entry, consumed) = mqtt::packet::SubEntry::parse(buffer).unwrap();
    ///
    /// assert_eq!(entry.topic_filter(), "test");
    /// assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::AtLeastOnce);
    /// assert_eq!(consumed, 7);
    /// ```
    pub fn parse(data: &[u8]) -> Result<(Self, usize), MqttError> {
        let mut cursor = 0;

        // 1. Parse topic filter
        let (topic_filter, consumed) = MqttString::decode(&data[cursor..])?;
        cursor += consumed;

        // 2. Parse subscription options
        if cursor >= data.len() {
            return Err(MqttError::MalformedPacket);
        }

        // Parse subscription options (1 byte)
        let sub_opts = SubOpts::from_u8(data[cursor])?;
        cursor += 1;

        Ok((
            Self {
                topic_filter,
                sub_opts,
            },
            cursor,
        ))
    }
}

/// Implementation of `Default` for `SubEntry`
///
/// Creates a subscription entry with default values:
/// - Empty topic filter string
/// - Default subscription options (QoS 0, all flags false)
///
/// Note: An empty topic filter is not valid for actual MQTT usage
/// but provides a default state for initialization purposes.
impl Default for SubEntry {
    fn default() -> Self {
        Self {
            topic_filter: MqttString::new(String::new()).unwrap(),
            sub_opts: SubOpts::default(),
        }
    }
}

/// Implementation of `Display` for `SubEntry`
///
/// Formats the subscription entry as a JSON string for human-readable output.
/// This is particularly useful for logging and debugging purposes.
/// If JSON serialization fails, an error message is displayed instead.
impl fmt::Display for SubEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(e) => write!(f, "{{\"error\": \"{}\"}}", e),
        }
    }
}

/// Implementation of `Serialize` for `SubEntry`
///
/// Serializes the subscription entry to a structured format with separate
/// fields for the topic filter and subscription options. This provides a
/// clear JSON representation suitable for debugging and logging.
///
/// # Serialized Fields
///
/// - `topic_filter`: The topic filter string
/// - `options`: The subscription options as a structured object
impl Serialize for SubEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Define field count
        let mut state = serializer.serialize_struct("SubEntry", 2)?;

        // Serialize topic filter and options
        state.serialize_field("topic_filter", self.topic_filter())?;
        state.serialize_field("options", self.sub_opts())?;

        state.end()
    }
}
