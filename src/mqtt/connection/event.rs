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
use std::fmt;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

use crate::mqtt::packet::GenericPacket;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::result_code::MqttError;

/// Represents different types of MQTT timers
///
/// This enum defines the different kinds of timers used in MQTT protocol operations.
/// Each timer serves a specific purpose in maintaining connection health and protocol compliance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TimerKind {
    /// Timer for sending PINGREQ packets
    ///
    /// This timer is used by MQTT clients to schedule periodic PINGREQ packets
    /// to keep the connection alive. The interval is typically determined by
    /// the keep-alive value negotiated during connection establishment.
    #[serde(rename = "pingreq_send")]
    PingreqSend,

    /// Timer for receiving PINGREQ packets
    ///
    /// This timer is used by MQTT servers (brokers) to detect when a client
    /// has not sent a PINGREQ packet within the expected timeframe, indicating
    /// a potentially disconnected or unresponsive client.
    #[serde(rename = "pingreq_recv")]
    PingreqRecv,

    /// Timer for receiving PINGRESP packets
    ///
    /// This timer is used by MQTT clients to detect when a server has not
    /// responded to a PINGREQ packet with a PINGRESP within the expected
    /// timeframe, indicating a potentially disconnected or unresponsive server.
    #[serde(rename = "pingresp_recv")]
    PingrespRecv,
}

/// Generic MQTT Event - represents events that occur during MQTT operations
///
/// This enum captures all events that would traditionally be handled by callbacks in
/// a callback-based MQTT implementation. Instead of using callbacks, this Sans-I/O
/// library returns events that the user application must process.
///
/// Events are returned from operations like `recv()` and `send()` for the user to process.
/// The user application is responsible for handling each event appropriately, such as
/// sending packets over the network, managing timers, or handling errors.
///
/// # Type Parameters
///
/// * `PacketIdType` - The type used for packet IDs (typically `u16`, but can be `u32` for extended scenarios)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// match event {
///     mqtt::connection::GenericEvent::RequestSendPacket { packet, .. } => {
///         // Send the packet over the network
///         network.send(&packet.to_bytes());
///     },
///     mqtt::connection::GenericEvent::RequestTimerReset { kind, duration_ms } => {
///         // Set or reset a timer
///         timer_manager.set_timer(kind, duration_ms);
///     },
///     // ... handle other events
/// }
/// ```
#[derive(Clone)]
pub enum GenericEvent<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize + 'static,
{
    /// Notification that a packet was received and parsed successfully
    ///
    /// This event is emitted when the MQTT library has successfully received
    /// and parsed an incoming packet. The application should process this packet
    /// according to its type and content.
    ///
    /// # Parameters
    ///
    /// * `GenericPacket<PacketIdType>` - The parsed MQTT packet
    NotifyPacketReceived(GenericPacket<PacketIdType>),

    /// Request to send a packet via the underlying transport
    ///
    /// This event is emitted when the MQTT library needs to send a packet.
    /// The application must send this packet over the network transport.
    /// If sending fails and a packet ID is specified in `release_packet_id_if_send_error`,
    /// the application should call `release_packet_id()` to free the packet ID for reuse.
    ///
    /// # Fields
    ///
    /// * `packet` - The MQTT packet to send
    /// * `release_packet_id_if_send_error` - Optional packet ID to release if sending fails
    RequestSendPacket {
        /// The MQTT packet that needs to be sent over the network
        packet: GenericPacket<PacketIdType>,
        /// Packet ID to release if the send operation fails (QoS > 0 packets only)
        release_packet_id_if_send_error: Option<PacketIdType>,
    },

    /// Notification that a packet ID has been released and is available for reuse
    ///
    /// This event is emitted when a packet ID is no longer in use and can be
    /// assigned to new outgoing packets. This typically happens when:
    /// - A QoS 1 PUBLISH receives its PUBACK
    /// - A QoS 2 PUBLISH completes its full handshake (PUBLISH -> PUBREC -> PUBREL -> PUBCOMP)
    /// - A QoS 2 PUBLISH receives a PUBREC with an error code, terminating the sequence early
    /// - A SUBSCRIBE receives its SUBACK
    /// - An UNSUBSCRIBE receives its UNSUBACK
    ///
    /// # Parameters
    ///
    /// * `PacketIdType` - The packet ID that has been released
    NotifyPacketIdReleased(PacketIdType),

    /// Request to reset or start a timer
    ///
    /// This event is emitted when the MQTT library needs to set up a timer for
    /// protocol operations such as keep-alive pings or timeout detection.
    /// The application should start or reset the specified timer type with
    /// the given duration.
    ///
    /// # Fields
    ///
    /// * `kind` - The type of timer to reset/start
    /// * `duration_ms` - Timer duration in milliseconds
    RequestTimerReset {
        /// The type of timer that needs to be reset or started
        kind: TimerKind,
        /// Duration of the timer in milliseconds
        duration_ms: u64,
    },

    /// Request to cancel a timer
    ///
    /// This event is emitted when the MQTT library needs to cancel a previously
    /// set timer. This typically happens when the timer is no longer needed,
    /// such as when a PINGRESP is received before the PINGRESP timeout.
    ///
    /// # Parameters
    ///
    /// * `TimerKind` - The type of timer to cancel
    RequestTimerCancel(TimerKind),

    /// Notification that an error occurred during processing
    ///
    /// This event is emitted when the MQTT library encounters an error that
    /// prevents normal operation. The application should handle the error
    /// appropriately, which may include logging, reconnection attempts, or
    /// user notification.
    ///
    /// Note: When handling this error, closing the underlying transport is not required.
    /// If the connection needs to be closed, a separate `RequestClose` event will be emitted.
    ///
    /// # Parameters
    ///
    /// * `MqttError` - The error that occurred
    NotifyError(MqttError),

    /// Request to close the connection
    ///
    /// This event is emitted when the MQTT library determines that the
    /// connection should be closed. This can happen due to protocol violations,
    /// disconnect requests, or other terminal conditions. The application
    /// should close the underlying network connection.
    RequestClose,
}

/// Type alias for Event with u16 packet ID (most common case)
///
/// This is the standard Event type that most applications will use.
/// It uses `u16` for packet IDs, which is the standard MQTT packet ID type
/// supporting values from 1 to 65535.
///
/// For extended scenarios where larger packet ID ranges are needed
/// (such as broker clusters), use `GenericEvent<u32>` directly.
pub type Event = GenericEvent<u16>;

/// Serialization implementation for GenericEvent
///
/// This implementation allows GenericEvent to be serialized to JSON format,
/// which can be useful for logging, debugging, or inter-process communication.
/// Each event variant is serialized with a "type" field indicating the event type.
impl<PacketIdType> Serialize for GenericEvent<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize + 'static,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            GenericEvent::NotifyPacketReceived(packet) => {
                let mut state = serializer.serialize_struct("GenericEvent", 2)?;
                state.serialize_field("type", "notify_packet_received")?;
                state.serialize_field("packet", packet)?;
                state.end()
            }
            GenericEvent::RequestSendPacket {
                packet,
                release_packet_id_if_send_error,
            } => {
                let mut state = serializer.serialize_struct("GenericEvent", 3)?;
                state.serialize_field("type", "request_send_packet")?;
                state.serialize_field("packet", packet)?;
                state.serialize_field(
                    "release_packet_id_if_send_error",
                    release_packet_id_if_send_error,
                )?;
                state.end()
            }
            GenericEvent::NotifyPacketIdReleased(packet_id) => {
                let mut state = serializer.serialize_struct("GenericEvent", 2)?;
                state.serialize_field("type", "notify_packet_id_released")?;
                state.serialize_field("packet_id", packet_id)?;
                state.end()
            }
            GenericEvent::RequestTimerReset { kind, duration_ms } => {
                let mut state = serializer.serialize_struct("GenericEvent", 3)?;
                state.serialize_field("type", "request_timer_reset")?;
                state.serialize_field("kind", kind)?;
                state.serialize_field("duration_ms", duration_ms)?;
                state.end()
            }
            GenericEvent::RequestTimerCancel(kind) => {
                let mut state = serializer.serialize_struct("GenericEvent", 2)?;
                state.serialize_field("type", "request_timer_cancel")?;
                state.serialize_field("kind", kind)?;
                state.end()
            }
            GenericEvent::NotifyError(error) => {
                let mut state = serializer.serialize_struct("GenericEvent", 2)?;
                state.serialize_field("type", "notify_error")?;
                state.serialize_field("error", &format!("{:?}", error))?;
                state.end()
            }
            GenericEvent::RequestClose => {
                let mut state = serializer.serialize_struct("GenericEvent", 1)?;
                state.serialize_field("type", "request_close")?;
                state.end()
            }
        }
    }
}

/// Display implementation for GenericEvent
///
/// Formats the event as a JSON string for human-readable output.
/// This is particularly useful for logging and debugging purposes.
/// If serialization fails, an error message is displayed instead.
impl<PacketIdType> fmt::Display for GenericEvent<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(e) => write!(f, "{{\"error\": \"{}\"}}", e),
        }
    }
}

/// Debug implementation for GenericEvent
///
/// Uses the same JSON formatting as Display for consistent debug output.
/// This ensures that debug output is structured and easily parseable.
impl<PacketIdType> fmt::Debug for GenericEvent<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
