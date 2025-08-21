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
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::marker::PhantomData;

use crate::mqtt_internal::common::tracing::{error, info, trace, warn};
use crate::mqtt_internal::common::Cursor;
use crate::mqtt_internal::common::HashSet;
use super::event::{GenericEvent, TimerKind};
use super::GenericStore;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
enum ConnectionStatus {
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "connecting")]
    Connecting,
    #[serde(rename = "connected")]
    Connected,
}
use super::packet_builder::{
    PacketBuildResult, PacketBuilder, PacketData, RawPacket,
};
use super::packet_id_manager::PacketIdManager;
use super::role;
use super::role::RoleType;
use super::sendable::Sendable;
use super::version::*;
use crate::mqtt_internal::packet::v3_1_1;
use crate::mqtt_internal::packet::v5_0;
use crate::mqtt_internal::packet::GenericPacket;
use crate::mqtt_internal::packet::GenericStorePacket;
use crate::mqtt_internal::packet::IsPacketId;
use crate::mqtt_internal::packet::Qos;
use crate::mqtt_internal::packet::ResponsePacket;
use crate::mqtt_internal::packet::{GenericProperty, TopicAliasRecv, TopicAliasSend};
use crate::mqtt_internal::prelude::GenericPacketTrait;
use crate::mqtt_internal::result_code::{
    ConnectReasonCode, ConnectReturnCode, DisconnectReasonCode, MqttError, PubrecReasonCode,
};

/// MQTT protocol maximum packet size limit
/// 1 (fixed header) + 4 (remaining length) + 128^4 (maximum remaining length value)
const MQTT_PACKET_SIZE_NO_LIMIT: u32 = 1 + 4 + 128 * 128 * 128 * 128;

/// Calculate total packet size from remaining length
///
/// The total packet size consists of:
/// - 1 byte for the fixed header
/// - 1-4 bytes for the remaining length encoding
/// - The remaining length value itself
fn remaining_length_to_total_size(remaining_length: u32) -> u32 {
    let remaining_length_bytes = if remaining_length < 128 {
        1
    } else if remaining_length < 16384 {
        2
    } else if remaining_length < 2097152 {
        3
    } else {
        4
    };

    1 + remaining_length_bytes + remaining_length
}

/// Generic MQTT Connection - Core Sans-I/O MQTT protocol implementation
///
/// This struct represents the core MQTT protocol logic in a Sans-I/O (synchronous I/O-independent) design.
/// It handles MQTT packet processing, state management, and protocol compliance without performing
/// any actual network I/O operations. Instead, it returns events that the application must handle.
///
/// # Type Parameters
///
/// * `Role` - The connection role (Client or Server), determining allowed operations
/// * `PacketIdType` - The type used for packet IDs (typically `u16`, but can be `u32` for extended scenarios)
///
/// # Key Features
///
/// - **Sans-I/O Design**: No network I/O is performed directly; events are returned for external handling
/// - **Protocol Compliance**: Implements MQTT v3.1.1 and v5.0 specifications
/// - **State Management**: Tracks connection state, packet IDs, and protocol flows
/// - **Configurable Behavior**: Supports various configuration options for different use cases
/// - **Generic Packet ID Support**: Can use u16 or u32 packet IDs for different deployment scenarios
///
/// # Usage
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// let mut connection = mqtt::connection::Connection::<mqtt::connection::role::Client>::new(mqtt::version::Version::V5_0);
///
/// // Send a packet
/// let events = connection.send(publish_packet);
/// for event in events {
///     match event {
///         mqtt::connection::Event::RequestSendPacket { packet, .. } => {
///             // Send packet over network
///         },
///         // Handle other events...
///     }
/// }
///
/// // Receive data
/// let mut cursor = std::io::Cursor::new(received_data);
/// let events = connection.recv(&mut cursor);
/// // Process events...
/// ```
pub struct GenericConnection<
    Role,
    PacketIdType,
    const STRING_BUFFER_SIZE: usize = 32,
    const BINARY_BUFFER_SIZE: usize = 32,
    const PAYLOAD_BUFFER_SIZE: usize = 32,
> where
    Role: RoleType,
    PacketIdType: IsPacketId,
{
    _marker: PhantomData<Role>,

    protocol_version: Version,

    pid_man: PacketIdManager<PacketIdType>,
    pid_suback: HashSet<PacketIdType>,
    pid_unsuback: HashSet<PacketIdType>,
    pid_puback: HashSet<PacketIdType>,
    pid_pubrec: HashSet<PacketIdType>,
    pid_pubcomp: HashSet<PacketIdType>,

    need_store: bool,
    // Store for retransmission packets
    store: GenericStore<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>,

    offline_publish: bool,
    auto_pub_response: bool,
    auto_ping_response: bool,

    // Auto map topic alias for sending
    auto_map_topic_alias_send: bool,
    // Auto replace topic alias for sending
    auto_replace_topic_alias_send: bool,
    // Topic alias management for receiving
    topic_alias_recv: Option<TopicAliasRecv>,
    // Topic alias management for sending
    topic_alias_send: Option<TopicAliasSend>,

    publish_send_max: Option<u16>,
    // Maximum number of concurrent PUBLISH packets for receiving
    publish_recv_max: Option<u16>,
    // Maximum number of concurrent PUBLISH packets for sending
    // Current count of PUBLISH packets being sent
    publish_send_count: u16,

    // Set of received PUBLISH packets (for flow control)
    publish_recv: HashSet<PacketIdType>,

    // Maximum packet size for sending
    maximum_packet_size_send: u32,
    // Maximum packet size for receiving
    maximum_packet_size_recv: u32,

    // Connection state
    status: ConnectionStatus,

    // PINGREQ send interval in milliseconds
    pingreq_send_interval_ms: Option<u64>,
    // PINGREQ receive timeout in milliseconds
    pingreq_recv_timeout_ms: Option<u64>,
    // PINGRESP receive timeout in milliseconds
    pingresp_recv_timeout_ms: Option<u64>,

    // QoS2 PUBLISH packet handling state (for duplicate detection)
    qos2_publish_handled: HashSet<PacketIdType>,
    // QoS2 PUBLISH packet processing state
    qos2_publish_processing: HashSet<PacketIdType>,

    // Timer state flags
    pingreq_send_set: bool,
    pingreq_recv_set: bool,
    pingresp_recv_set: bool,

    packet_builder: PacketBuilder,
    // Client/Server mode flag
    is_client: bool,
}

/// Type alias for Connection with u16 packet ID (standard case)
///
/// This is the standard Connection type that most applications will use.
/// It uses `u16` for packet IDs, which is the standard MQTT packet ID type
/// supporting values from 1 to 65535.
///
/// For extended scenarios where larger packet ID ranges are needed
/// (such as broker clusters), use `GenericConnection<Role, u32>` directly.
///
/// # Type Parameters
///
/// * `Role` - The connection role (typically `role::Client` or `role::Server`)
pub type Connection<Role> = GenericConnection<Role, u16>;

impl<
        Role,
        PacketIdType,
        const STRING_BUFFER_SIZE: usize,
        const BINARY_BUFFER_SIZE: usize,
        const PAYLOAD_BUFFER_SIZE: usize,
    >
    GenericConnection<
        Role,
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
where
    Role: RoleType,
    PacketIdType: IsPacketId,
{
    /// Create a new MQTT connection instance
    ///
    /// Initializes a new MQTT connection with the specified protocol version.
    /// The connection starts in a disconnected state and must be activated through
    /// the connection handshake process (CONNECT/CONNACK).
    ///
    /// # Parameters
    ///
    /// * `version` - The MQTT protocol version to use (V3_1_1 or V5_0)
    ///
    /// # Returns
    ///
    /// A new `GenericConnection` instance ready for use
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mut client = mqtt::connection::Connection::<mqtt::connection::role::Client>::new(mqtt::version::Version::V5_0);
    /// let mut server = mqtt::connection::Connection::<mqtt::connection::role::Server>::new(mqtt::version::Version::V3_1_1);
    /// ```
    pub fn new(version: Version) -> Self {
        Self {
            _marker: PhantomData,
            protocol_version: version,
            pid_man: PacketIdManager::new(),
            pid_suback: HashSet::new(),
            pid_unsuback: HashSet::new(),
            pid_puback: HashSet::new(),
            pid_pubrec: HashSet::new(),
            pid_pubcomp: HashSet::new(),
            need_store: false,
            store: GenericStore::new(),
            offline_publish: false,
            auto_pub_response: false,
            auto_ping_response: false,
            auto_map_topic_alias_send: false,
            auto_replace_topic_alias_send: false,
            topic_alias_recv: None,
            topic_alias_send: None,
            publish_send_max: None,
            publish_recv_max: None,
            publish_send_count: 0,
            publish_recv: HashSet::new(),
            maximum_packet_size_send: MQTT_PACKET_SIZE_NO_LIMIT,
            maximum_packet_size_recv: MQTT_PACKET_SIZE_NO_LIMIT,
            status: ConnectionStatus::Disconnected,
            pingreq_send_interval_ms: None,
            pingreq_recv_timeout_ms: None,
            pingresp_recv_timeout_ms: None,
            qos2_publish_handled: HashSet::new(),
            qos2_publish_processing: HashSet::new(),
            pingreq_send_set: false,
            pingreq_recv_set: false,
            pingresp_recv_set: false,
            packet_builder: PacketBuilder::new(),
            is_client: false,
        }
    }

    // public

    /// Send MQTT packet with compile-time role checking (experimental)
    ///
    /// This method provides compile-time verification that the packet being sent
    /// is allowed for the current connection role (Client or Server). It only works
    /// when the `Role` type parameter is concrete (not generic).
    ///
    /// This is an experimental API that may be subject to change. For general use,
    /// consider using the `send()` method instead.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The packet type that must implement `Sendable<Role, PacketIdType>`
    ///
    /// # Parameters
    ///
    /// * `packet` - The MQTT packet to send
    ///
    /// # Returns
    ///
    /// A vector of events that the application must process
    ///
    /// # Compile-time Safety
    ///
    /// If the packet type is not allowed for the current role, this will result
    /// in a compile-time error, preventing protocol violations at development time.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// // This works for concrete roles
    /// let mut client = mqtt::connection::Connection::<mqtt::connection::role::Client>::new(mqtt::version::Version::V5_0);
    /// let events = client.checked_send(connect_packet); // OK - clients can send CONNECT
    ///
    /// // This would cause a compile error
    /// // let events = client.checked_send(connack_packet); // Error - clients cannot send CONNACK
    /// ```
    pub fn checked_send<T>(
        &mut self,
        packet: T,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    where
        T: Sendable<
            Role,
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >,
    {
        // dispatch concrete packet or generic packet
        packet.dispatch_send(self)
    }

    /// Send MQTT packet with runtime role validation
    ///
    /// This is the primary method for sending MQTT packets. It accepts any `GenericPacket`
    /// and performs runtime validation to ensure the packet is allowed for the current
    /// connection role. This provides flexibility when the exact packet type is not known
    /// at compile time.
    ///
    /// # Parameters
    ///
    /// * `packet` - The MQTT packet to send
    ///
    /// # Returns
    ///
    /// A vector of events that the application must process. If the packet is not allowed
    /// for the current role, a `NotifyError` event will be included.
    ///
    /// # Validation
    ///
    /// The method validates that:
    /// - The packet type is allowed for the connection role (Client vs Server)
    /// - Protocol version compatibility
    /// - Connection state requirements
    /// - Packet ID management for QoS > 0 packets
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mut client = mqtt::connection::Connection::<mqtt::connection::role::Client>::new(mqtt::version::Version::V5_0);
    /// let events = client.send(mqtt::packet::GenericPacket::V5_0(mqtt::packet::v5_0::Packet::Connect(connect_packet)));
    ///
    /// for event in events {
    ///     match event {
    ///         mqtt::connection::Event::RequestSendPacket { packet, .. } => {
    ///             // Send packet over network
    ///         },
    ///         mqtt::connection::Event::NotifyError(error) => {
    ///             // Handle validation errors
    ///         },
    ///         // Handle other events...
    ///     }
    /// }
    /// ```
    pub fn send(
        &mut self,
        packet: GenericPacket<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        use core::any::TypeId;

        let role_id = TypeId::of::<Role>();
        let client_id = TypeId::of::<role::Client>();
        let server_id = TypeId::of::<role::Server>();
        let any_id = TypeId::of::<role::Any>();

        // Check version compatibility between connection and packet
        let packet_version = packet.protocol_version();

        // Return error if versions don't match
        if self.protocol_version != packet_version {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        match packet {
            // CONNECT - Client/Any can send
            GenericPacket::V3_1_1Connect(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v3_1_1_connect(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Connect(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v5_0_connect(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            // CONNACK - Server/Any can send
            GenericPacket::V3_1_1Connack(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v3_1_1_connack(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Connack(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v5_0_connack(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            // PUBLISH - Any role can send
            GenericPacket::V3_1_1Publish(p) => self.process_send_v3_1_1_publish(p),
            GenericPacket::V5_0Publish(p) => self.process_send_v5_0_publish(p),
            // PUBACK/PUBREC/PUBREL/PUBCOMP - Any role can send
            GenericPacket::V3_1_1Puback(p) => self.process_send_v3_1_1_puback(p),
            GenericPacket::V5_0Puback(p) => self.process_send_v5_0_puback(p),
            GenericPacket::V3_1_1Pubrec(p) => self.process_send_v3_1_1_pubrec(p),
            GenericPacket::V5_0Pubrec(p) => self.process_send_v5_0_pubrec(p),
            GenericPacket::V3_1_1Pubrel(p) => self.process_send_v3_1_1_pubrel(p),
            GenericPacket::V5_0Pubrel(p) => self.process_send_v5_0_pubrel(p),
            GenericPacket::V3_1_1Pubcomp(p) => self.process_send_v3_1_1_pubcomp(p),
            GenericPacket::V5_0Pubcomp(p) => self.process_send_v5_0_pubcomp(p),
            // SUBSCRIBE/UNSUBSCRIBE - Client/Any can send
            GenericPacket::V3_1_1Subscribe(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v3_1_1_subscribe(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Subscribe(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v5_0_subscribe(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V3_1_1Unsubscribe(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v3_1_1_unsubscribe(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Unsubscribe(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v5_0_unsubscribe(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            // SUBACK/UNSUBACK - Server/Any can send
            GenericPacket::V3_1_1Suback(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v3_1_1_suback(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Suback(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v5_0_suback(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V3_1_1Unsuback(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v3_1_1_unsuback(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Unsuback(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v5_0_unsuback(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            // PINGREQ - Client/Any can send
            GenericPacket::V3_1_1Pingreq(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v3_1_1_pingreq(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Pingreq(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v5_0_pingreq(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            // PINGRESP - Server/Any can send
            GenericPacket::V3_1_1Pingresp(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v3_1_1_pingresp(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            GenericPacket::V5_0Pingresp(p) => {
                if role_id == server_id || role_id == any_id {
                    self.process_send_v5_0_pingresp(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            // DISCONNECT(v3.1.1) - Client/Any role can send
            GenericPacket::V3_1_1Disconnect(p) => {
                if role_id == client_id || role_id == any_id {
                    self.process_send_v3_1_1_disconnect(p)
                } else {
                    vec![GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::PacketNotAllowedToSend
                    )]
                }
            }
            // DISCONNECT(v5.0) - Any role can send
            GenericPacket::V5_0Disconnect(p) => self.process_send_v5_0_disconnect(p),
            // AUTH - Any role can send (v5.0 only)
            GenericPacket::V5_0Auth(p) => self.process_send_v5_0_auth(p),
        }
    }

    /// Receive and process incoming MQTT data
    ///
    /// This method processes raw bytes received from the network and attempts to
    /// parse them into MQTT packets. It handles packet fragmentation and can
    /// process multiple complete packets from a single data buffer.
    ///
    /// # Parameters
    ///
    /// * `data` - A cursor over the received data bytes. The cursor position will
    ///   be advanced as data is consumed.
    ///
    /// # Returns
    ///
    /// A vector of events generated from processing the received data:
    /// - `NotifyPacketReceived` for successfully parsed packets
    /// - `NotifyError` for parsing errors or protocol violations
    /// - Additional events based on packet processing (timers, responses, etc.)
    ///
    /// # Behavior
    ///
    /// - Handles partial packets (data will be buffered until complete)
    /// - Processes multiple complete packets in sequence
    /// - Validates packet structure and protocol compliance
    /// - Updates internal connection state based on received packets
    /// - Generates appropriate response events (ACKs, etc.)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use mqtt_protocol_core::mqtt;
    ///
    /// let mut connection = mqtt::connection::Connection::<mqtt::connection::role::Client>::new(mqtt::version::Version::V5_0);
    /// let received_data = [/* network data */];
    /// let mut cursor = std::io::Cursor::new(&received_data[..]);
    ///
    /// let events = connection.recv(&mut cursor);
    /// for event in events {
    ///     match event {
    ///         mqtt::connection::Event::NotifyPacketReceived(packet) => {
    ///             // Process received packet
    ///         },
    ///         mqtt::connection::Event::NotifyError(error) => {
    ///             // Handle parsing/protocol errors
    ///         },
    ///         // Handle other events...
    ///     }
    /// }
    /// ```
    pub fn recv(
        &mut self,
        data: &mut Cursor<&[u8]>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match self.packet_builder.feed(data) {
            PacketBuildResult::Complete(raw_packet) => {
                events.extend(self.process_recv_packet(raw_packet));
            }
            PacketBuildResult::Incomplete => {}
            PacketBuildResult::Error(e) => {
                self.cancel_timers(&mut events);
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    /// Notify that a timer has fired (Event-based API)
    ///
    /// This method should be called when the I/O layer detects that a timer has expired.
    /// It handles the timer event appropriately and returns events that need to be processed.
    /// Notify that a timer has fired
    ///
    /// This method should be called by the application when a previously requested
    /// timer expires. The connection will take appropriate action based on the timer type.
    ///
    /// # Parameters
    ///
    /// * `kind` - The type of timer that fired
    ///
    /// # Returns
    ///
    /// Events generated from timer processing (e.g., sending PINGREQ, connection timeouts)
    pub fn notify_timer_fired(
        &mut self,
        kind: TimerKind,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match kind {
            TimerKind::PingreqSend => {
                // Reset timer flag
                self.pingreq_send_set = false;

                // Send PINGREQ if connected
                if self.status == ConnectionStatus::Connected {
                    match self.protocol_version {
                        Version::V3_1_1 => {
                            if let Ok(pingreq) = v3_1_1::Pingreq::builder().build() {
                                events.extend(self.process_send_v3_1_1_pingreq(pingreq));
                            }
                        }
                        Version::V5_0 => {
                            if let Ok(pingreq) = v5_0::Pingreq::builder().build() {
                                events.extend(self.process_send_v5_0_pingreq(pingreq));
                            }
                        }
                        Version::Undetermined => {
                            unreachable!("Protocol version should be set before sending PINGREQ");
                        }
                    }
                }
            }
            TimerKind::PingreqRecv => {
                // Reset timer flag
                self.pingreq_recv_set = false;

                match self.protocol_version {
                    Version::V3_1_1 => {
                        // V3.1.1: Close connection
                        events.push(
                            GenericEvent::<
                                PacketIdType,
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                                PAYLOAD_BUFFER_SIZE,
                            >::RequestClose,
                        );
                    }
                    Version::V5_0 => {
                        // V5.0: Send DISCONNECT with keep_alive_timeout if connected
                        if self.status == ConnectionStatus::Connected {
                            if let Ok(disconnect) = v5_0::GenericDisconnect::<
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                            >::builder()
                            .reason_code(DisconnectReasonCode::KeepAliveTimeout)
                            .build()
                            {
                                events.extend(self.process_send_v5_0_disconnect(disconnect));
                            }
                        }
                    }
                    Version::Undetermined => {
                        unreachable!("Protocol version should be set before receiving PINGREQ");
                    }
                }
            }
            TimerKind::PingrespRecv => {
                // Reset timer flag
                self.pingresp_recv_set = false;

                match self.protocol_version {
                    Version::V3_1_1 => {
                        // V3.1.1: Close connection
                        events.push(
                            GenericEvent::<
                                PacketIdType,
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                                PAYLOAD_BUFFER_SIZE,
                            >::RequestClose,
                        );
                    }
                    Version::V5_0 => {
                        // V5.0: Send DISCONNECT with keep_alive_timeout if connected
                        if self.status == ConnectionStatus::Connected {
                            if let Ok(disconnect) = v5_0::GenericDisconnect::<
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                            >::builder()
                            .reason_code(DisconnectReasonCode::KeepAliveTimeout)
                            .build()
                            {
                                events.extend(self.process_send_v5_0_disconnect(disconnect));
                            }
                        }
                    }
                    Version::Undetermined => {
                        unreachable!("Protocol version should be set before receiving PINGRESP");
                    }
                }
            }
        }

        events
    }

    /// Notify that the connection has been closed by the I/O layer (Event-based API)
    ///
    /// This method should be called when the I/O layer detects that the socket has been closed.
    /// It updates the internal state appropriately and returns events that need to be processed.
    /// Notify that the underlying connection has been closed
    ///
    /// This method should be called when the network connection is closed,
    /// either intentionally or due to network issues.
    ///
    /// # Returns
    ///
    /// Events generated from connection closure processing
    pub fn notify_closed(
        &mut self,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        // Reset packet size limits to MQTT protocol maximum
        self.maximum_packet_size_send = MQTT_PACKET_SIZE_NO_LIMIT;
        self.maximum_packet_size_recv = MQTT_PACKET_SIZE_NO_LIMIT;

        // Set status to disconnected
        self.status = ConnectionStatus::Disconnected;

        // Clear topic alias management
        self.topic_alias_send = None;
        self.topic_alias_recv = None;

        // Release packet IDs for SUBACK
        for packet_id in self.pid_suback.drain() {
            if self.pid_man.is_used_id(packet_id) {
                self.pid_man.release_id(packet_id);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketIdReleased(packet_id));
            }
        }

        // Release packet IDs for UNSUBACK
        for packet_id in self.pid_unsuback.drain() {
            if self.pid_man.is_used_id(packet_id) {
                self.pid_man.release_id(packet_id);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketIdReleased(packet_id));
            }
        }

        // If not storing session state, clear QoS2 states and release publish-related packet IDs
        if !self.need_store {
            self.qos2_publish_processing.clear();
            self.qos2_publish_handled.clear();

            // Release packet IDs for PUBACK
            for packet_id in self.pid_puback.drain() {
                if self.pid_man.is_used_id(packet_id) {
                    self.pid_man.release_id(packet_id);
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketIdReleased(packet_id));
                }
            }

            // Release packet IDs for PUBREC
            for packet_id in self.pid_pubrec.drain() {
                if self.pid_man.is_used_id(packet_id) {
                    self.pid_man.release_id(packet_id);
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketIdReleased(packet_id));
                }
            }

            // Release packet IDs for PUBCOMP
            for packet_id in self.pid_pubcomp.drain() {
                if self.pid_man.is_used_id(packet_id) {
                    self.pid_man.release_id(packet_id);
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketIdReleased(packet_id));
                }
            }
        }

        // Cancel all timers
        self.cancel_timers(&mut events);

        events
    }

    /// Set the PINGREQ send interval
    ///
    /// Sets the interval for sending PINGREQ packets to maintain the connection alive.
    /// When changed, this may generate timer-related events to update the ping schedule.
    ///
    /// # Parameters
    ///
    /// * `duration_ms` - The interval in milliseconds between PINGREQ packets
    ///
    /// # Returns
    ///
    /// Events generated from updating the ping interval
    pub fn set_pingreq_send_interval(
        &mut self,
        duration_ms: u64,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        if duration_ms == 0 {
            self.pingreq_send_interval_ms = None;
            self.pingreq_send_set = false;
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestTimerCancel(TimerKind::PingreqSend));
        } else {
            self.pingreq_send_interval_ms = Some(duration_ms);
            self.pingreq_send_set = true;
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestTimerReset {
                kind: TimerKind::PingreqSend,
                duration_ms,
            });
        }

        events
    }

    /// Get the remaining capacity for sending PUBLISH packets
    ///
    /// Returns the number of additional PUBLISH packets that can be sent
    /// without exceeding the receive maximum limit.
    ///
    /// # Returns
    ///
    /// The remaining capacity for outgoing PUBLISH packets, or `None` if no limit is set
    pub fn get_receive_maximum_vacancy_for_send(&self) -> Option<u16> {
        // If publish_recv_max is set, return the remaining capacity
        self.publish_send_max
            .map(|max| max.saturating_sub(self.publish_send_count))
    }

    /// Enable or disable offline publishing
    ///
    /// When enabled, PUBLISH packets can be sent even when disconnected.
    /// They will be queued and sent once the connection is established.
    ///
    /// # Parameters
    ///
    /// * `enable` - Whether to enable offline publishing
    pub fn set_offline_publish(&mut self, enable: bool) {
        self.offline_publish = enable;
        if self.offline_publish {
            self.need_store = true;
        }
    }

    /// Enable or disable automatic PUBLISH response generation
    ///
    /// When enabled, appropriate response packets (PUBACK, PUBREC, PUBREL, and PUBCOMP.)
    /// are automatically generated for received PUBLISH packets.
    ///
    /// # Parameters
    ///
    /// * `enable` - Whether to enable automatic responses
    pub fn set_auto_pub_response(&mut self, enable: bool) {
        self.auto_pub_response = enable;
    }

    /// Enable or disable automatic PING response generation
    ///
    /// When enabled, PINGRESP packets are automatically sent in response to PINGREQ.
    ///
    /// # Parameters
    ///
    /// * `enable` - Whether to enable automatic PING responses
    pub fn set_auto_ping_response(&mut self, enable: bool) {
        self.auto_ping_response = enable;
    }

    /// Enable or disable automatic topic alias mapping for outgoing packets
    ///
    /// When enabled, the connection will automatically map topics to aliases
    /// for outgoing PUBLISH packets to reduce bandwidth usage. This includes:
    /// - Applying existing registered topic aliases when available
    /// - Allocating new topic aliases for unregistered topics
    /// - Using LRU algorithm to overwrite the least recently used alias when all aliases are in use
    ///
    /// # Parameters
    ///
    /// * `enable` - Whether to enable automatic topic alias mapping
    pub fn set_auto_map_topic_alias_send(&mut self, enable: bool) {
        self.auto_map_topic_alias_send = enable;
    }

    /// Enable or disable automatic topic alias replacement for outgoing packets
    ///
    /// When enabled, the connection will automatically apply existing registered
    /// topic aliases to outgoing PUBLISH packets when aliases are available.
    /// This only uses previously registered aliases and does not allocate new ones.
    ///
    /// # Parameters
    ///
    /// * `enable` - Whether to enable automatic topic alias replacement
    pub fn set_auto_replace_topic_alias_send(&mut self, enable: bool) {
        self.auto_replace_topic_alias_send = enable;
    }

    /// Set PINGREQ receive timeout
    pub fn set_pingresp_recv_timeout(&mut self, timeout_ms: Option<u64>) {
        self.pingresp_recv_timeout_ms = timeout_ms;
    }

    /// Acquire a new packet ID for outgoing packets
    ///
    /// # Returns
    ///
    /// A unique packet ID, or an error if none are available
    pub fn acquire_packet_id(&mut self) -> Result<PacketIdType, MqttError> {
        self.pid_man.acquire_unique_id()
    }

    /// Register a packet ID as in use
    ///
    /// Manually registers a specific packet ID as being in use, preventing
    /// it from being allocated by `acquire_packet_id()`.
    ///
    /// # Parameters
    ///
    /// * `packet_id` - The packet ID to register as in use
    ///
    /// # Returns
    ///
    /// `Ok(())` if successful, or an error if the packet ID is already in use
    pub fn register_packet_id(&mut self, packet_id: PacketIdType) -> Result<(), MqttError> {
        self.pid_man.register_id(packet_id)
    }

    /// Release packet ID (Event-based API)
    /// Release a packet ID for reuse
    ///
    /// This method releases a packet ID, making it available for future use.
    /// It also generates a `NotifyPacketIdReleased` event.
    ///
    /// # Parameters
    ///
    /// * `packet_id` - The packet ID to release
    ///
    /// # Returns
    ///
    /// Events generated from releasing the packet ID
    pub fn release_packet_id(
        &mut self,
        packet_id: PacketIdType,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        if self.pid_man.is_used_id(packet_id) {
            self.pid_man.release_id(packet_id);
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyPacketIdReleased(packet_id));
        }

        events
    }

    /// Get the set of QoS 2 PUBLISH packet IDs that have been handled
    ///
    /// Returns a copy of the set containing packet IDs of QoS 2 PUBLISH packets
    /// that have been successfully processed and handled.
    ///
    /// # Returns
    ///
    /// A `HashSet` containing packet IDs of handled QoS 2 PUBLISH packets
    pub fn get_qos2_publish_handled(&self) -> HashSet<PacketIdType> {
        self.qos2_publish_handled.clone()
    }

    /// Restore the set of QoS 2 PUBLISH packet IDs that have been handled
    ///
    /// Restores the internal state of handled QoS 2 PUBLISH packet IDs,
    /// typically used when resuming a connection from persistent storage.
    ///
    /// # Parameters
    ///
    /// * `pids` - A `HashSet` containing packet IDs of previously handled QoS 2 PUBLISH packets
    pub fn restore_qos2_publish_handled(&mut self, pids: HashSet<PacketIdType>) {
        self.qos2_publish_handled = pids;
    }

    /// Restore previously stored packets
    ///
    /// This method restores packets that were previously stored for persistence,
    /// typically called when resuming a session.
    ///
    /// # Parameters
    ///
    /// * `packets` - Vector of packets to restore
    pub fn restore_packets(
        &mut self,
        packets: Vec<
            GenericStorePacket<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >,
        >,
    ) {
        for packet in packets {
            match &packet {
                GenericStorePacket::V3_1_1Publish(p) => {
                    // Add to appropriate QoS tracking set
                    match p.qos() {
                        Qos::AtLeastOnce => {
                            self.pid_puback.insert(p.packet_id().unwrap());
                        }
                        Qos::ExactlyOnce => {
                            self.pid_pubrec.insert(p.packet_id().unwrap());
                        }
                        _ => {
                            // QoS 0 shouldn't be in store, but handle gracefully
                            warn!("QoS 0 packet found in store, skipping");
                            continue;
                        }
                    }
                    // Register packet ID and add to store
                    let packet_id = p.packet_id().unwrap();
                    if self.pid_man.register_id(packet_id).is_ok() {
                        if let Err(_e) = self.store.add(packet) {
                            error!("Failed to add packet to store: {:?}", _e);
                        }
                    } else {
                        error!("Packet ID {} has already been used. Skip it", packet_id);
                    }
                }
                GenericStorePacket::V5_0Publish(p) => {
                    // Add to appropriate QoS tracking set
                    match p.qos() {
                        Qos::AtLeastOnce => {
                            self.pid_puback.insert(p.packet_id().unwrap());
                        }
                        Qos::ExactlyOnce => {
                            self.pid_pubrec.insert(p.packet_id().unwrap());
                        }
                        _ => {
                            // QoS 0 shouldn't be in store, but handle gracefully
                            warn!("QoS 0 packet found in store, skipping");
                            continue;
                        }
                    }
                    // Register packet ID and add to store
                    let packet_id = p.packet_id().unwrap();
                    if self.pid_man.register_id(packet_id).is_ok() {
                        if let Err(_e) = self.store.add(packet) {
                            error!("Failed to add packet to store: {:?}", _e);
                        }
                    } else {
                        error!("Packet ID {} has already been used. Skip it", packet_id);
                    }
                }
                GenericStorePacket::V3_1_1Pubrel(p) => {
                    // Pubrel packets expect PUBCOMP response
                    self.pid_pubcomp.insert(p.packet_id());
                    // Register packet ID and add to store
                    let packet_id = p.packet_id();
                    if self.pid_man.register_id(packet_id).is_ok() {
                        if let Err(_e) = self.store.add(packet) {
                            error!("Failed to add packet to store: {:?}", _e);
                        }
                    } else {
                        error!("Packet ID {} has already been used. Skip it", packet_id);
                    }
                }
                GenericStorePacket::V5_0Pubrel(p) => {
                    // Pubrel packets expect PUBCOMP response
                    self.pid_pubcomp.insert(p.packet_id());
                    // Register packet ID and add to store
                    let packet_id = p.packet_id();
                    if self.pid_man.register_id(packet_id).is_ok() {
                        if let Err(_e) = self.store.add(packet) {
                            error!("Failed to add packet to store: {:?}", _e);
                        }
                    } else {
                        error!("Packet ID {} has already been used. Skip it", packet_id);
                    }
                }
            }
        }
    }

    /// Get stored packets for persistence
    ///
    /// Returns packets that need to be stored for potential retransmission.
    /// This is useful for implementing persistent sessions.
    ///
    /// # Returns
    ///
    /// Vector of packets that should be persisted
    pub fn get_stored_packets(
        &self,
    ) -> Vec<
        GenericStorePacket<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >,
    > {
        self.store.get_stored()
    }

    /// Get the MQTT protocol version being used
    ///
    /// # Returns
    ///
    /// The protocol version (V3_1_1 or V5_0)
    pub fn get_protocol_version(&self) -> Version {
        self.protocol_version
    }

    /// Check if a PUBLISH packet is currently being processed
    ///
    /// # Parameters
    ///
    /// * `packet_id` - The packet ID to check
    ///
    /// # Returns
    ///
    /// True if the packet ID is in use for PUBLISH processing
    pub fn is_publish_processing(&self, packet_id: PacketIdType) -> bool {
        self.qos2_publish_processing.contains(&packet_id)
    }

    /// Regulate packet for store (remove/resolve topic alias)
    ///
    /// This method prepares a V5.0 publish packet for storage by resolving topic aliases
    /// and removing TopicAlias properties to ensure the packet can be retransmitted correctly.
    pub fn regulate_for_store(
        &self,
        mut packet: v5_0::GenericPublish<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >,
    ) -> Result<
        v5_0::GenericPublish<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >,
        MqttError,
    > {
        if packet.topic_name().is_empty() {
            // Topic is empty, need to resolve from topic alias
            if let Some(props) = packet.props() {
                if let Some(topic_alias) = Self::get_topic_alias_from_props(props) {
                    if let Some(ref topic_alias_send) = self.topic_alias_send {
                        if let Some(topic) = topic_alias_send.peek(topic_alias) {
                            // Found topic for alias, add topic and remove alias property
                            packet = packet.remove_topic_alias_add_topic(topic.to_string())?;
                        } else {
                            return Err(MqttError::PacketNotRegulated);
                        }
                    } else {
                        return Err(MqttError::PacketNotRegulated);
                    }
                } else {
                    return Err(MqttError::PacketNotRegulated);
                }
            } else {
                return Err(MqttError::PacketNotRegulated);
            }
        } else {
            // Topic is not empty, just remove TopicAlias property if present
            packet = packet.remove_topic_alias();
        }

        Ok(packet)
    }

    // private

    /// Initialize connection state based on client/server role
    ///
    /// Resets all connection-specific state including:
    /// - Publish flow control counters and limits
    /// - Topic alias management
    /// - QoS2 processing state
    /// - Packet ID tracking sets
    /// - Store requirement flag
    ///
    /// # Parameters
    /// * `is_client` - true for client mode, false for server mode
    fn initialize(&mut self, is_client: bool) {
        self.publish_send_max = None;
        self.publish_recv_max = None;
        self.publish_send_count = 0;
        self.topic_alias_send = None;
        self.topic_alias_recv = None;
        self.publish_recv.clear();
        self.qos2_publish_processing.clear();
        self.need_store = false;
        self.pid_suback.clear();
        self.pid_unsuback.clear();
        self.is_client = is_client;
    }

    fn clear_store_related(&mut self) {
        self.pid_man.clear();
        self.pid_puback.clear();
        self.pid_pubrec.clear();
        self.pid_pubcomp.clear();
        self.store.clear();
    }

    /// Send all stored packets for retransmission
    fn send_stored(
        &mut self,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();
        self.store.for_each(|packet| {
            if packet.size() > self.maximum_packet_size_send as usize {
                let packet_id = packet.packet_id();
                self.pid_man.release_id(packet_id);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketIdReleased(packet_id));
                return false; // Remove from store
            }
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestSendPacket {
                packet: packet.clone().into(),
                release_packet_id_if_send_error: None,
            });
            true // Keep in store
        });

        events
    }

    /// Validate topic alias and return the associated topic name
    ///
    /// Checks if the topic alias is valid and retrieves the corresponding topic name
    /// from the topic alias send manager.
    ///
    /// # Parameters
    /// * `topic_alias_opt` - Optional topic alias value
    ///
    /// # Returns
    /// * `Some(topic_name)` if the topic alias is valid and found
    /// * `None` if the topic alias is invalid, not provided, or not found
    fn validate_topic_alias(&mut self, topic_alias_opt: Option<u16>) -> Option<String> {
        let topic_alias = topic_alias_opt?;

        if !self.validate_topic_alias_range(topic_alias) {
            return None;
        }

        let topic_alias_send = self.topic_alias_send.as_mut()?;
        // LRU updated here
        let topic = topic_alias_send.get(topic_alias)?;

        Some(topic.to_string())
    }

    /// Validate that topic alias is within the allowed range
    ///
    /// Checks if the topic alias value is valid according to the configured
    /// topic alias maximum for sending.
    ///
    /// # Parameters
    /// * `topic_alias` - Topic alias value to validate
    ///
    /// # Returns
    /// * `true` if the topic alias is within valid range
    /// * `false` if invalid or topic alias sending is not configured
    fn validate_topic_alias_range(&self, topic_alias: u16) -> bool {
        let topic_alias_send = match &self.topic_alias_send {
            Some(tas) => tas,
            None => {
                error!("topic_alias is set but topic_alias_maximum is 0");
                return false;
            }
        };

        if topic_alias == 0 || topic_alias > topic_alias_send.max() {
            error!("topic_alias is set but out of range");
            return false;
        }

        true
    }

    /// Process v3.1.1 CONNECT packet - C++ constexpr if implementation
    pub(crate) fn process_send_v3_1_1_connect(
        &mut self,
        packet: v3_1_1::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        info!("send connect v3.1.1: {packet}");

        if self.status != ConnectionStatus::Disconnected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }

        let mut events = Vec::new();
        self.initialize(true);
        self.status = ConnectionStatus::Connecting;

        // Extract keep_alive and set pingreq_send_interval_ms if != 0
        let keep_alive = packet.keep_alive();
        if keep_alive != 0 && self.pingreq_send_interval_ms.is_none() {
            self.pingreq_send_interval_ms = Some(keep_alive as u64 * 1000);
        }

        // Handle clean_session flag
        if packet.clean_start() {
            self.clear_store_related();
        } else {
            self.need_store = true;
        }

        // Clear topic alias for sending
        self.topic_alias_send = None;

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    /// Process v5.0 CONNECT packet - C++ constexpr if implementation
    pub(crate) fn process_send_v5_0_connect(
        &mut self,
        packet: v5_0::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        info!("send connect v5.0: {packet}");
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Disconnected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        self.initialize(true);
        self.status = ConnectionStatus::Connecting;

        // Extract keep_alive and set pingreq_send_interval_ms if != 0
        let keep_alive = packet.keep_alive();
        if keep_alive != 0 && self.pingreq_send_interval_ms.is_none() {
            self.pingreq_send_interval_ms = Some(keep_alive as u64 * 1000);
        }

        // Handle clean_start flag
        if packet.clean_start() {
            self.clear_store_related();
        }

        // Process properties
        for prop in packet.props() {
            match prop {
                GenericProperty::TopicAliasMaximum(val) => {
                    if val.val() != 0 {
                        self.topic_alias_recv = Some(TopicAliasRecv::new(val.val()));
                    }
                }
                GenericProperty::ReceiveMaximum(val) => {
                    debug_assert!(val.val() != 0, "ReceiveMaximum must not be 0");
                    self.publish_recv_max = Some(val.val());
                }
                GenericProperty::MaximumPacketSize(val) => {
                    debug_assert!(val.val() != 0, "MaximumPacketSize must not be 0");
                    self.maximum_packet_size_recv = val.val();
                }
                GenericProperty::SessionExpiryInterval(val) => {
                    if val.val() != 0 {
                        self.need_store = true;
                    }
                }
                _ => {
                    // Ignore other properties (equivalent to [](auto const&){} in C++)
                }
            }
        }

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_connack(
        &mut self,
        packet: v3_1_1::Connack,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        info!("send connack v3.1.1: {packet}");
        if self.status != ConnectionStatus::Connecting {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();
        if packet.return_code() == ConnectReturnCode::Accepted {
            self.status = ConnectionStatus::Connected;
        } else {
            self.status = ConnectionStatus::Disconnected;
        }

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        events.extend(self.send_stored());
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_connack(
        &mut self,
        packet: v5_0::GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        info!("send connack v5.0: {packet}");
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connecting {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();

        if packet.reason_code() == ConnectReasonCode::Success {
            self.status = ConnectionStatus::Connected;

            // Process properties
            for prop in packet.props() {
                match prop {
                    GenericProperty::TopicAliasMaximum(val) => {
                        if val.val() != 0 {
                            self.topic_alias_recv = Some(TopicAliasRecv::new(val.val()));
                        }
                    }
                    GenericProperty::ReceiveMaximum(val) => {
                        debug_assert!(val.val() != 0, "ReceiveMaximum must not be 0");
                        self.publish_recv_max = Some(val.val());
                    }
                    GenericProperty::MaximumPacketSize(val) => {
                        debug_assert!(val.val() != 0, "MaximumPacketSize must not be 0");
                        self.maximum_packet_size_recv = val.val();
                    }
                    _ => {
                        // Ignore other properties
                    }
                }
            }
        } else {
            self.status = ConnectionStatus::Disconnected;
            self.cancel_timers(&mut events);
        }

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        events.extend(self.send_stored());
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_publish(
        &mut self,
        packet: v3_1_1::GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();
        let mut release_packet_id_if_send_error: Option<PacketIdType> = None;

        if packet.qos() == Qos::AtLeastOnce || packet.qos() == Qos::ExactlyOnce {
            // Register packet ID for QoS 1 or 2
            let packet_id = packet.packet_id().unwrap();
            if self.status != ConnectionStatus::Connected
                && !self.need_store
                && !self.offline_publish
            {
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(
                    MqttError::PacketNotAllowedToSend
                ));
                if self.pid_man.is_used_id(packet_id) {
                    self.pid_man.release_id(packet_id);
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketIdReleased(packet_id));
                }
                return events;
            }
            if !self.pid_man.is_used_id(packet_id) {
                error!("packet_id {packet_id} must be acquired or registered");
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(
                    MqttError::PacketIdentifierInvalid
                ));
                return events;
            }
            if self.need_store
                && (self.status != ConnectionStatus::Disconnected || self.offline_publish)
            {
                let store_packet = packet.clone().set_dup(true);
                self.store.add(store_packet.try_into().unwrap()).unwrap();
            } else {
                release_packet_id_if_send_error = Some(packet_id);
            }
            if packet.qos() == Qos::ExactlyOnce {
                self.qos2_publish_processing.insert(packet_id);
                self.pid_pubrec.insert(packet_id);
            } else {
                self.pid_puback.insert(packet_id);
            }
        } else if self.status != ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend));
            return events;
        }

        if self.status == ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestSendPacket {
                packet: packet.into(),
                release_packet_id_if_send_error,
            });
        }
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_publish(
        &mut self,
        mut packet: v5_0::GenericPublish<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }

        let mut events = Vec::new();
        let mut release_packet_id_if_send_error: Option<PacketIdType> = None;
        let mut topic_alias_validated = false;
        if packet.qos() == Qos::AtLeastOnce || packet.qos() == Qos::ExactlyOnce {
            let packet_id = packet.packet_id().unwrap();
            if self.status != ConnectionStatus::Connected
                && !self.need_store
                && !self.offline_publish
            {
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(
                    MqttError::PacketNotAllowedToSend
                ));
                if self.pid_man.is_used_id(packet_id) {
                    self.pid_man.release_id(packet_id);
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketIdReleased(packet_id));
                }
                return events;
            }

            // Extract topic_name from TopicAlias and remove TopicAlias property, then store it
            if !self.pid_man.is_used_id(packet_id) {
                error!("packet_id {packet_id} must be acquired or registered");
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(
                    MqttError::PacketIdentifierInvalid
                ));
                return events;
            }

            if self.need_store
                && (self.status != ConnectionStatus::Disconnected || self.offline_publish)
            {
                let ta_opt = Self::get_topic_alias_from_props_opt(packet.props());
                if packet.topic_name().is_empty() {
                    // Topic name is empty, must validate topic alias
                    let topic_opt = self.validate_topic_alias(ta_opt);
                    if topic_opt.is_none() {
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyError(
                            MqttError::PacketNotAllowedToSend
                        ));
                        if self.pid_man.is_used_id(packet_id) {
                            self.pid_man.release_id(packet_id);
                            events.push(GenericEvent::<
                                PacketIdType,
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                                PAYLOAD_BUFFER_SIZE,
                            >::NotifyPacketIdReleased(
                                packet_id
                            ));
                        }
                        return events;
                    }
                    topic_alias_validated = true;
                    let store_packet = packet
                        .clone()
                        .remove_topic_alias_add_topic(topic_opt.unwrap())
                        .unwrap()
                        .set_dup(true);
                    // TBD validate_maximum_packet_size(store_packet.size());
                    self.store.add(store_packet.try_into().unwrap()).unwrap();
                } else {
                    // Topic name is not empty, remove topic alias if present
                    let store_packet = packet.clone().remove_topic_alias().set_dup(true);
                    self.store.add(store_packet.try_into().unwrap()).unwrap();
                }
            } else {
                release_packet_id_if_send_error = Some(packet_id);
            }
            if packet.qos() == Qos::ExactlyOnce {
                self.qos2_publish_processing.insert(packet_id);
                self.pid_pubrec.insert(packet_id);
            } else {
                self.pid_puback.insert(packet_id);
            }
        } else if self.status != ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend));
            return events;
        }

        let packet_id_opt = packet.packet_id();
        let ta_opt = Self::get_topic_alias_from_props_opt(packet.props());
        if packet.topic_name().is_empty() {
            // process manually provided TopicAlias
            if !topic_alias_validated && self.validate_topic_alias(ta_opt).is_none() {
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(
                    MqttError::PacketNotAllowedToSend
                ));
                if let Some(packet_id) = packet_id_opt {
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                }
                return events;
            }
        } else if let Some(ta) = ta_opt {
            // Topic alias is provided
            if self.validate_topic_alias_range(ta) {
                trace!(
                    "topic alias: {} - {} is registered.",
                    packet.topic_name(),
                    ta
                );
                if let Some(ref mut topic_alias_send) = self.topic_alias_send {
                    topic_alias_send.insert_or_update(packet.topic_name(), ta);
                }
            } else {
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(
                    MqttError::PacketNotAllowedToSend
                ));
                if let Some(packet_id) = packet_id_opt {
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                }
                return events;
            }
        } else if self.status == ConnectionStatus::Connected {
            // process auto applying TopicAlias if the option is enabled
            if self.auto_map_topic_alias_send {
                if let Some(ref mut topic_alias_send) = self.topic_alias_send {
                    if let Some(found_ta) = topic_alias_send.find_by_topic(packet.topic_name()) {
                        trace!(
                            "topic alias: {} - {} is found.",
                            packet.topic_name(),
                            found_ta
                        );
                        packet = packet.remove_topic_add_topic_alias(found_ta);
                    } else {
                        let lru_ta = topic_alias_send.get_lru_alias();
                        topic_alias_send.insert_or_update(packet.topic_name(), lru_ta);
                        packet = packet.remove_topic_add_topic_alias(lru_ta);
                    }
                }
            } else if self.auto_replace_topic_alias_send {
                if let Some(ref topic_alias_send) = self.topic_alias_send {
                    if let Some(found_ta) = topic_alias_send.find_by_topic(packet.topic_name()) {
                        trace!(
                            "topic alias: {} - {} is found.",
                            packet.topic_name(),
                            found_ta
                        );
                        packet = packet.remove_topic_add_topic_alias(found_ta);
                    }
                }
            }
        }

        // Check receive_maximum for sending (QoS 1 and 2 packets)
        if packet.qos() == Qos::AtLeastOnce || packet.qos() == Qos::ExactlyOnce {
            if let Some(max) = self.publish_send_max {
                if self.publish_send_count == max {
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(
                        MqttError::ReceiveMaximumExceeded
                    ));
                    if let Some(packet_id) = packet_id_opt {
                        if self.pid_man.is_used_id(packet_id) {
                            self.pid_man.release_id(packet_id);
                            events.push(GenericEvent::<
                                PacketIdType,
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                                PAYLOAD_BUFFER_SIZE,
                            >::NotifyPacketIdReleased(
                                packet_id
                            ));
                        }
                    }
                    return events;
                }
                self.publish_send_count += 1;
            }
        }

        if self.status == ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestSendPacket {
                packet: packet.into(),
                release_packet_id_if_send_error,
            });
        }
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_puback(
        &mut self,
        packet: v3_1_1::GenericPuback<PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_puback(
        &mut self,
        packet: v5_0::GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        self.publish_recv.remove(&packet.packet_id());

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_pubrec(
        &mut self,
        packet: v3_1_1::GenericPubrec<PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_pubrec(
        &mut self,
        packet: v5_0::GenericPubrec<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        let packet_id = packet.packet_id();

        if let Some(rc) = packet.reason_code() {
            if rc.is_failure() {
                self.publish_recv.remove(&packet_id);
                self.qos2_publish_handled.remove(&packet_id);
            }
        }

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_pubrel(
        &mut self,
        packet: v3_1_1::GenericPubrel<PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected && !self.need_store {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();
        let packet_id = packet.packet_id();
        if !self.pid_man.is_used_id(packet_id) {
            error!("packet_id {packet_id} must be acquired or registered");
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(
                MqttError::PacketIdentifierInvalid
            ));
            return events;
        }
        if self.need_store {
            self.store.add(packet.clone().try_into().unwrap()).unwrap();
        }

        if self.status == ConnectionStatus::Connected {
            self.pid_pubcomp.insert(packet_id);
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestSendPacket {
                packet: packet.into(),
                release_packet_id_if_send_error: None,
            });
        }
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_pubrel(
        &mut self,
        packet: v5_0::GenericPubrel<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected && !self.need_store {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        let packet_id = packet.packet_id();
        if !self.pid_man.is_used_id(packet_id) {
            error!("packet_id {packet_id} must be acquired or registered");
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(
                MqttError::PacketIdentifierInvalid
            ));
            return events;
        }
        if self.need_store {
            self.store.add(packet.clone().try_into().unwrap()).unwrap();
        }

        if self.status == ConnectionStatus::Connected {
            self.pid_pubcomp.insert(packet_id);
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestSendPacket {
                packet: packet.into(),
                release_packet_id_if_send_error: None,
            });
        }
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_pubcomp(
        &mut self,
        packet: v3_1_1::GenericPubcomp<PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_pubcomp(
        &mut self,
        packet: v5_0::GenericPubcomp<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        self.publish_recv.remove(&packet.packet_id());

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_subscribe(
        &mut self,
        packet: v3_1_1::GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();
        let packet_id = packet.packet_id();
        if self.status != ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend));
            if self.pid_man.is_used_id(packet_id) {
                self.pid_man.release_id(packet_id);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketIdReleased(packet_id));
            }
            return events;
        }
        if !self.pid_man.is_used_id(packet_id) {
            error!("packet_id {packet_id} must be acquired or registered");
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(
                MqttError::PacketIdentifierInvalid
            ));
            return events;
        }
        self.pid_suback.insert(packet_id);

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: Some(packet_id),
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_subscribe(
        &mut self,
        packet: v5_0::GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }

        let mut events = Vec::new();
        let packet_id = packet.packet_id();
        if self.status != ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend));
            if self.pid_man.is_used_id(packet_id) {
                self.pid_man.release_id(packet_id);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketIdReleased(packet_id));
            }
            return events;
        }
        if !self.pid_man.is_used_id(packet_id) {
            error!("packet_id {packet_id} must be acquired or registered");
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(
                MqttError::PacketIdentifierInvalid
            ));
            return events;
        }
        self.pid_suback.insert(packet_id);

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: Some(packet_id),
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_suback(
        &mut self,
        packet: v3_1_1::GenericSuback<PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_suback(
        &mut self,
        packet: v5_0::GenericSuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_unsubscribe(
        &mut self,
        packet: v3_1_1::GenericUnsubscribe<PacketIdType, STRING_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();
        let packet_id = packet.packet_id();
        if self.status != ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend));
            if self.pid_man.is_used_id(packet_id) {
                self.pid_man.release_id(packet_id);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketIdReleased(packet_id));
            }
            return events;
        }
        if !self.pid_man.is_used_id(packet_id) {
            error!("packet_id {packet_id} must be acquired or registered");
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(
                MqttError::PacketIdentifierInvalid
            ));
            return events;
        }
        self.pid_unsuback.insert(packet_id);

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: Some(packet_id),
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_unsubscribe(
        &mut self,
        packet: v5_0::GenericUnsubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }

        let mut events = Vec::new();
        let packet_id = packet.packet_id();
        if self.status != ConnectionStatus::Connected {
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend));
            if self.pid_man.is_used_id(packet_id) {
                self.pid_man.release_id(packet_id);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketIdReleased(packet_id));
            }
            return events;
        }
        if !self.pid_man.is_used_id(packet_id) {
            error!("packet_id {packet_id} must be acquired or registered");
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(
                MqttError::PacketIdentifierInvalid
            ));
            return events;
        }
        self.pid_unsuback.insert(packet_id);

        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: Some(packet_id),
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_unsuback(
        &mut self,
        packet: v3_1_1::GenericUnsuback<PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_unsuback(
        &mut self,
        packet: v5_0::GenericUnsuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_pingreq(
        &mut self,
        packet: v3_1_1::Pingreq,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        if let Some(timeout_ms) = self.pingresp_recv_timeout_ms {
            self.pingreq_send_set = true;
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestTimerReset {
                kind: TimerKind::PingrespRecv,
                duration_ms: timeout_ms,
            });
        }
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_pingreq(
        &mut self,
        packet: v5_0::Pingreq,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        if let Some(timeout_ms) = self.pingresp_recv_timeout_ms {
            self.pingreq_send_set = true;
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestTimerReset {
                kind: TimerKind::PingrespRecv,
                duration_ms: timeout_ms,
            });
        }
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_pingresp(
        &mut self,
        packet: v3_1_1::Pingresp,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v5_0_pingresp(
        &mut self,
        packet: v5_0::Pingresp,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    pub(crate) fn process_send_v3_1_1_disconnect(
        &mut self,
        packet: v3_1_1::Disconnect,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }
        let mut events = Vec::new();
        self.status = ConnectionStatus::Disconnected;
        self.cancel_timers(&mut events);
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        events.push(GenericEvent::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::RequestClose);

        events
    }

    pub(crate) fn process_send_v5_0_disconnect(
        &mut self,
        packet: v5_0::GenericDisconnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status != ConnectionStatus::Connected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        self.status = ConnectionStatus::Disconnected;
        self.cancel_timers(&mut events);
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        events.push(GenericEvent::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::RequestClose);

        events
    }

    pub(crate) fn process_send_v5_0_auth(
        &mut self,
        packet: v5_0::GenericAuth<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        if !self.validate_maximum_packet_size_send(packet.size()) {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge)];
        }
        if self.status == ConnectionStatus::Disconnected {
            return vec![GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketNotAllowedToSend)];
        }

        let mut events = Vec::new();
        events.push(GenericEvent::<
            PacketIdType,
            STRING_BUFFER_SIZE,
            BINARY_BUFFER_SIZE,
            PAYLOAD_BUFFER_SIZE,
        >::RequestSendPacket {
            packet: packet.into(),
            release_packet_id_if_send_error: None,
        });
        self.send_post_process(&mut events);

        events
    }

    fn send_post_process(
        &mut self,
        events: &mut Vec<
            GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>,
        >,
    ) {
        if self.is_client {
            if let Some(timeout_ms) = self.pingreq_send_interval_ms {
                self.pingreq_send_set = true;
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::RequestTimerReset {
                    kind: TimerKind::PingreqSend,
                    duration_ms: timeout_ms,
                });
            }
        }
    }

    fn validate_maximum_packet_size_send(&self, size: usize) -> bool {
        if size > self.maximum_packet_size_send as usize {
            error!("packet size over maximum_packet_size for sending");
            return false;
        }
        true
    }

    fn process_recv_packet(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        // packet size limit validation (v3.1.1 is always satisfied)
        let total_size = remaining_length_to_total_size(raw_packet.remaining_length());
        if total_size > self.maximum_packet_size_recv {
            // This happens only when protocol version is V5.0.
            // On v3.1.1, the maximum packet size is always 268435455 (2^32 - 1).
            // If the packet size is over 268434555, feed() return an error.
            // maximum_packet_size_recv is set by sending CONNECT or CONNACK packet.
            // So DISCONNECT packet is the right choice to notify the error.
            let disconnect_packet =
                v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                    .reason_code(DisconnectReasonCode::PacketTooLarge)
                    .build()
                    .unwrap();
            // Send disconnect packet directly without generic constraints
            events.extend(self.process_send_v5_0_disconnect(disconnect_packet));
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::NotifyError(MqttError::PacketTooLarge));
            return events;
        }

        let packet_type = raw_packet.packet_type();
        let _flags = raw_packet.flags();
        match self.protocol_version {
            Version::V3_1_1 => {
                match packet_type {
                    1 => {
                        // CONNECT
                        events.extend(self.process_recv_v3_1_1_connect(raw_packet));
                    }
                    2 => {
                        // CONNACK
                        events.extend(self.process_recv_v3_1_1_connack(raw_packet));
                    }
                    3 => {
                        // PUBLISH
                        events.extend(self.process_recv_v3_1_1_publish(raw_packet));
                    }
                    4 => {
                        // PUBACK
                        events.extend(self.process_recv_v3_1_1_puback(raw_packet));
                    }
                    5 => {
                        // PUBREC
                        events.extend(self.process_recv_v3_1_1_pubrec(raw_packet));
                    }
                    6 => {
                        // PUBREL
                        events.extend(self.process_recv_v3_1_1_pubrel(raw_packet));
                    }
                    7 => {
                        // PUBCOMP
                        events.extend(self.process_recv_v3_1_1_pubcomp(raw_packet));
                    }
                    8 => {
                        // SUBSCRIBE
                        events.extend(self.process_recv_v3_1_1_subscribe(raw_packet));
                    }
                    9 => {
                        // SUBACK
                        events.extend(self.process_recv_v3_1_1_suback(raw_packet));
                    }
                    10 => {
                        // UNSUBSCRIBE
                        events.extend(self.process_recv_v3_1_1_unsubscribe(raw_packet));
                    }
                    11 => {
                        // UNSUBACK
                        events.extend(self.process_recv_v3_1_1_unsuback(raw_packet));
                    }
                    12 => {
                        // PINGREQ
                        events.extend(self.process_recv_v3_1_1_pingreq(raw_packet));
                    }
                    13 => {
                        // PINGRESP
                        events.extend(self.process_recv_v3_1_1_pingresp(raw_packet));
                    }
                    14 => {
                        // DISCONNECT
                        events.extend(self.process_recv_v3_1_1_disconnect(raw_packet));
                    }
                    // invalid packet type
                    _ => {
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyError(
                            MqttError::MalformedPacket
                        ));
                    }
                }
            }
            Version::V5_0 => {
                match packet_type {
                    1 => {
                        // CONNECT
                        events.extend(self.process_recv_v5_0_connect(raw_packet));
                    }
                    2 => {
                        // CONNACK
                        events.extend(self.process_recv_v5_0_connack(raw_packet));
                    }
                    3 => {
                        // PUBLISH
                        events.extend(self.process_recv_v5_0_publish(raw_packet));
                    }
                    4 => {
                        // PUBACK
                        events.extend(self.process_recv_v5_0_puback(raw_packet));
                    }
                    5 => {
                        // PUBREC
                        events.extend(self.process_recv_v5_0_pubrec(raw_packet));
                    }
                    6 => {
                        // PUBREL
                        events.extend(self.process_recv_v5_0_pubrel(raw_packet));
                    }
                    7 => {
                        // PUBCOMP
                        events.extend(self.process_recv_v5_0_pubcomp(raw_packet));
                    }
                    8 => {
                        // SUBSCRIBE
                        events.extend(self.process_recv_v5_0_subscribe(raw_packet));
                    }
                    9 => {
                        // SUBACK
                        events.extend(self.process_recv_v5_0_suback(raw_packet));
                    }
                    10 => {
                        // UNSUBSCRIBE
                        events.extend(self.process_recv_v5_0_unsubscribe(raw_packet));
                    }
                    11 => {
                        // UNSUBACK
                        events.extend(self.process_recv_v5_0_unsuback(raw_packet));
                    }
                    12 => {
                        // PINGREQ
                        events.extend(self.process_recv_v5_0_pingreq(raw_packet));
                    }
                    13 => {
                        // PINGRESP
                        events.extend(self.process_recv_v5_0_pingresp(raw_packet));
                    }
                    14 => {
                        // DISCONNECT
                        events.extend(self.process_recv_v5_0_disconnect(raw_packet));
                    }
                    15 => {
                        // AUTH
                        events.extend(self.process_recv_v5_0_auth(raw_packet));
                    }
                    // invalid packet type
                    _ => {
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyError(
                            MqttError::MalformedPacket
                        ));
                    }
                }
            }
            Version::Undetermined => {
                match packet_type {
                    1 => {
                        // CONNECT
                        if raw_packet.remaining_length() < 7 {
                            events.push(GenericEvent::<
                                PacketIdType,
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                                PAYLOAD_BUFFER_SIZE,
                            >::NotifyError(
                                MqttError::MalformedPacket
                            ));
                            return events;
                        }
                        match raw_packet.data_as_slice()[6] {
                            // Protocol Version
                            4 => {
                                self.protocol_version = Version::V3_1_1;
                                events.extend(self.process_recv_v3_1_1_connect(raw_packet));
                            }
                            5 => {
                                self.protocol_version = Version::V5_0;
                                events.extend(self.process_recv_v5_0_connect(raw_packet));
                            }
                            _ => {
                                events.push(GenericEvent::<
                                    PacketIdType,
                                    STRING_BUFFER_SIZE,
                                    BINARY_BUFFER_SIZE,
                                    PAYLOAD_BUFFER_SIZE,
                                >::NotifyError(
                                    MqttError::UnsupportedProtocolVersion
                                ));
                            }
                        }
                    }
                    _ => {
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyError(
                            MqttError::MalformedPacket
                        ));
                    }
                }
            }
        }

        events
    }

    fn process_recv_v3_1_1_connect(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();
        match v3_1_1::GenericConnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                if self.status != ConnectionStatus::Disconnected {
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                    return events;
                }
                self.initialize(false);
                self.status = ConnectionStatus::Connecting;
                if packet.keep_alive() > 0 {
                    self.pingreq_recv_timeout_ms =
                        Some((packet.keep_alive() as u64) * 1000 * 3 / 2);
                }
                if packet.clean_session() {
                    self.clear_store_related();
                } else {
                    self.need_store = true;
                }
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                if self.status == ConnectionStatus::Disconnected {
                    self.status = ConnectionStatus::Connecting;
                    let rc = match e {
                        MqttError::ClientIdentifierNotValid => {
                            ConnectReturnCode::IdentifierRejected
                        }
                        MqttError::BadUserNameOrPassword => {
                            ConnectReturnCode::BadUserNameOrPassword
                        }
                        MqttError::UnsupportedProtocolVersion => {
                            ConnectReturnCode::UnacceptableProtocolVersion
                        }
                        _ => ConnectReturnCode::NotAuthorized, // TBD close could be better
                    };
                    let connack = v3_1_1::Connack::builder().return_code(rc).build().unwrap();
                    let connack_events = self.process_send_v3_1_1_connack(connack);
                    events.extend(connack_events);
                } else {
                    events.push(
                        GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::RequestClose,
                    );
                }
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_connect(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();
        match v5_0::GenericConnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                if self.status != ConnectionStatus::Disconnected {
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                    return events;
                }
                self.initialize(false);
                self.status = ConnectionStatus::Connecting;
                if packet.keep_alive() > 0 {
                    self.pingreq_recv_timeout_ms =
                        Some((packet.keep_alive() as u64) * 1000 * 3 / 2);
                }
                if packet.clean_start() {
                    self.clear_store_related();
                }
                packet.props().iter().for_each(|prop| match prop {
                    GenericProperty::TopicAliasMaximum(p) => {
                        self.topic_alias_send = Some(TopicAliasSend::new(p.val()));
                    }
                    GenericProperty::ReceiveMaximum(p) => {
                        self.publish_send_max = Some(p.val());
                    }
                    GenericProperty::MaximumPacketSize(p) => {
                        self.maximum_packet_size_send = p.val();
                    }
                    GenericProperty::SessionExpiryInterval(p) => {
                        if p.val() != 0 {
                            self.need_store = true;
                        }
                    }
                    _ => {}
                });
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                if self.status == ConnectionStatus::Disconnected {
                    self.status = ConnectionStatus::Connecting;
                    let rc = match e {
                        MqttError::ClientIdentifierNotValid => {
                            ConnectReasonCode::ClientIdentifierNotValid
                        }
                        MqttError::BadUserNameOrPassword => {
                            ConnectReasonCode::BadAuthenticationMethod
                        }
                        MqttError::UnsupportedProtocolVersion => {
                            ConnectReasonCode::UnsupportedProtocolVersion
                        }
                        _ => ConnectReasonCode::UnspecifiedError,
                    };
                    let connack =
                        v5_0::GenericConnack::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                            .reason_code(rc)
                            .build()
                            .unwrap();
                    let connack_events = self.process_send_v5_0_connack(connack);
                    events.extend(connack_events);
                } else {
                    let disconnect =
                        v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder(
                        )
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                    let disconnect_events = self.process_send_v5_0_disconnect(disconnect);
                    events.extend(disconnect_events);
                }
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_connack(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::Connack::parse(raw_packet.data_as_slice()) {
            Ok((packet, _consumed)) => {
                if packet.return_code() == ConnectReturnCode::Accepted {
                    self.status = ConnectionStatus::Connected;
                    if packet.session_present() {
                        events.extend(self.send_stored());
                    } else {
                        self.clear_store_related();
                    }
                }
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(
                    GenericPacket::V3_1_1Connack(packet)
                ));
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_connack(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericConnack::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _consumed)) => {
                if packet.reason_code() == ConnectReasonCode::Success {
                    self.status = ConnectionStatus::Connected;

                    // Process properties
                    for prop in packet.props() {
                        match prop {
                            GenericProperty::TopicAliasMaximum(val) => {
                                if val.val() > 0 {
                                    self.topic_alias_send = Some(TopicAliasSend::new(val.val()));
                                }
                            }
                            GenericProperty::ReceiveMaximum(val) => {
                                assert!(val.val() != 0);
                                self.publish_send_max = Some(val.val());
                            }
                            GenericProperty::MaximumPacketSize(val) => {
                                assert!(val.val() != 0);
                                self.maximum_packet_size_send = val.val();
                            }
                            GenericProperty::ServerKeepAlive(val) => {
                                // Set PINGREQ send interval if this is a client
                                let timeout_ms = (val.val() as u64) * 1000;
                                self.pingreq_send_interval_ms = Some(timeout_ms);
                            }
                            _ => {
                                // Ignore other properties
                            }
                        }
                    }

                    if packet.session_present() {
                        events.extend(self.send_stored());
                    } else {
                        self.clear_store_related();
                    }
                }
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                if self.status == ConnectionStatus::Connected {
                    let disconnect =
                        v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder(
                        )
                        .reason_code(e.into())
                        .build()
                        .unwrap();
                    let disconnect_events = self.process_send_v5_0_disconnect(disconnect);
                    events.extend(disconnect_events);
                }
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_publish(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        let flags = raw_packet.flags();
        match &raw_packet.data {
            PacketData::Publish(arc) => {
                match v3_1_1::GenericPublish::<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::parse(flags, arc.clone()) {
                    Ok((packet, _consumed)) => {
                        match packet.qos() {
                            Qos::AtMostOnce => {
                                events.extend(self.refresh_pingreq_recv());
                                events.push(GenericEvent::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::NotifyPacketReceived(packet.into()));
                            }
                            Qos::AtLeastOnce => {
                                let packet_id = packet.packet_id().unwrap();
                                if self.status == ConnectionStatus::Connected
                                    && self.auto_pub_response
                                {
                                    // Send PUBACK automatically
                                    let puback = v3_1_1::GenericPuback::builder()
                                        .packet_id(packet_id)
                                        .build()
                                        .unwrap();
                                    events.extend(self.process_send_v3_1_1_puback(puback));
                                }
                                events.extend(self.refresh_pingreq_recv());
                                events.push(GenericEvent::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::NotifyPacketReceived(packet.into()));
                            }
                            Qos::ExactlyOnce => {
                                let packet_id = packet.packet_id().unwrap();
                                let already_handled = !self.qos2_publish_handled.insert(packet_id);

                                if self.status == ConnectionStatus::Connected
                                    && (self.auto_pub_response || already_handled)
                                {
                                    let pubrec = v3_1_1::GenericPubrec::builder()
                                        .packet_id(packet_id)
                                        .build()
                                        .unwrap();
                                    events.extend(self.process_send_v3_1_1_pubrec(pubrec));
                                }
                                events.extend(self.refresh_pingreq_recv());
                                if !already_handled {
                                    events.push(GenericEvent::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::NotifyPacketReceived(packet.into()));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        events.push(GenericEvent::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::RequestClose);
                        events.push(GenericEvent::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>::NotifyError(e));
                    }
                }
            }
            PacketData::Normal(_) => {
                unreachable!("PUBLISH packet must use PacketData::Publish variant");
            }
        }

        events
    }

    fn process_recv_v5_0_publish(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        let flags = raw_packet.flags();
        match &raw_packet.data {
            PacketData::Publish(arc) => {
                match v5_0::GenericPublish::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::parse(flags, arc.clone())
                {
                    Ok((packet, _consumed)) => {
                        let mut already_handled = false;
                        let mut puback_send = false;
                        let mut pubrec_send = false;

                        match packet.qos() {
                            Qos::AtLeastOnce => {
                                let packet_id = packet.packet_id().unwrap();
                                if let Some(max) = self.publish_recv_max {
                                    if self.publish_recv.len() >= max as usize {
                                        let disconnect = v5_0::GenericDisconnect::<
                                            STRING_BUFFER_SIZE,
                                            BINARY_BUFFER_SIZE,
                                        >::builder(
                                        )
                                        .reason_code(DisconnectReasonCode::ReceiveMaximumExceeded)
                                        .build()
                                        .unwrap();
                                        events
                                            .extend(self.process_send_v5_0_disconnect(disconnect));
                                        events.push(GenericEvent::<
                                            PacketIdType,
                                            STRING_BUFFER_SIZE,
                                            BINARY_BUFFER_SIZE,
                                            PAYLOAD_BUFFER_SIZE,
                                        >::NotifyError(
                                            MqttError::ReceiveMaximumExceeded
                                        ));
                                        return events;
                                    }
                                }
                                self.publish_recv.insert(packet_id);
                                if self.auto_pub_response
                                    && self.status == ConnectionStatus::Connected
                                {
                                    puback_send = true;
                                }
                            }
                            Qos::ExactlyOnce => {
                                let packet_id = packet.packet_id().unwrap();
                                if let Some(max) = self.publish_recv_max {
                                    if self.publish_recv.len() >= max as usize {
                                        let disconnect = v5_0::GenericDisconnect::<
                                            STRING_BUFFER_SIZE,
                                            BINARY_BUFFER_SIZE,
                                        >::builder(
                                        )
                                        .reason_code(DisconnectReasonCode::ReceiveMaximumExceeded)
                                        .build()
                                        .unwrap();
                                        events
                                            .extend(self.process_send_v5_0_disconnect(disconnect));
                                        events.push(GenericEvent::<
                                            PacketIdType,
                                            STRING_BUFFER_SIZE,
                                            BINARY_BUFFER_SIZE,
                                            PAYLOAD_BUFFER_SIZE,
                                        >::NotifyError(
                                            MqttError::ReceiveMaximumExceeded
                                        ));
                                        return events;
                                    }
                                }
                                self.publish_recv.insert(packet_id);

                                if !self.qos2_publish_handled.insert(packet_id) {
                                    already_handled = true;
                                }
                                if self.status == ConnectionStatus::Connected
                                    && (self.auto_pub_response || already_handled)
                                {
                                    pubrec_send = true;
                                }
                            }
                            Qos::AtMostOnce => {
                                // No packet ID handling for QoS 0
                            }
                        }

                        // Topic Alias handling
                        if packet.topic_name().is_empty() {
                            // Extract topic from topic_alias
                            if let Some(ta) = Self::get_topic_alias_from_props_opt(packet.props()) {
                                if ta == 0
                                    || self.topic_alias_recv.is_none()
                                    || ta > self.topic_alias_recv.as_ref().unwrap().max()
                                {
                                    let disconnect = v5_0::GenericDisconnect::<
                                        STRING_BUFFER_SIZE,
                                        BINARY_BUFFER_SIZE,
                                    >::builder(
                                    )
                                    .reason_code(DisconnectReasonCode::TopicAliasInvalid)
                                    .build()
                                    .unwrap();
                                    events.extend(self.process_send_v5_0_disconnect(disconnect));
                                    events.push(GenericEvent::<
                                        PacketIdType,
                                        STRING_BUFFER_SIZE,
                                        BINARY_BUFFER_SIZE,
                                        PAYLOAD_BUFFER_SIZE,
                                    >::NotifyError(
                                        MqttError::TopicAliasInvalid
                                    ));
                                    return events;
                                }

                                if let Some(ref topic_alias_recv) = self.topic_alias_recv {
                                    if topic_alias_recv.get(ta).is_none() {
                                        let disconnect = v5_0::GenericDisconnect::<
                                            STRING_BUFFER_SIZE,
                                            BINARY_BUFFER_SIZE,
                                        >::builder(
                                        )
                                        .reason_code(DisconnectReasonCode::TopicAliasInvalid)
                                        .build()
                                        .unwrap();
                                        events
                                            .extend(self.process_send_v5_0_disconnect(disconnect));
                                        events.push(GenericEvent::<
                                            PacketIdType,
                                            STRING_BUFFER_SIZE,
                                            BINARY_BUFFER_SIZE,
                                            PAYLOAD_BUFFER_SIZE,
                                        >::NotifyError(
                                            MqttError::TopicAliasInvalid
                                        ));
                                        return events;
                                    }
                                    // Note: In a complete implementation, we would modify the packet
                                    // to add the resolved topic. For now, we'll proceed.
                                }
                            } else {
                                let disconnect = v5_0::GenericDisconnect::<
                                    STRING_BUFFER_SIZE,
                                    BINARY_BUFFER_SIZE,
                                >::builder()
                                .reason_code(DisconnectReasonCode::TopicAliasInvalid)
                                .build()
                                .unwrap();
                                events.extend(self.process_send_v5_0_disconnect(disconnect));
                                events.push(GenericEvent::<
                                    PacketIdType,
                                    STRING_BUFFER_SIZE,
                                    BINARY_BUFFER_SIZE,
                                    PAYLOAD_BUFFER_SIZE,
                                >::NotifyError(
                                    MqttError::TopicAliasInvalid
                                ));
                                return events;
                            }
                        } else {
                            // Topic is not empty, check if topic alias needs to be registered
                            if let Some(ta) = Self::get_topic_alias_from_props_opt(packet.props()) {
                                if ta == 0
                                    || self.topic_alias_recv.is_none()
                                    || ta > self.topic_alias_recv.as_ref().unwrap().max()
                                {
                                    let disconnect = v5_0::GenericDisconnect::<
                                        STRING_BUFFER_SIZE,
                                        BINARY_BUFFER_SIZE,
                                    >::builder(
                                    )
                                    .reason_code(DisconnectReasonCode::TopicAliasInvalid)
                                    .build()
                                    .unwrap();
                                    events.extend(self.process_send_v5_0_disconnect(disconnect));
                                    events.push(GenericEvent::<
                                        PacketIdType,
                                        STRING_BUFFER_SIZE,
                                        BINARY_BUFFER_SIZE,
                                        PAYLOAD_BUFFER_SIZE,
                                    >::NotifyError(
                                        MqttError::TopicAliasInvalid
                                    ));
                                    return events;
                                }
                                if let Some(ref mut topic_alias_recv) = self.topic_alias_recv {
                                    topic_alias_recv.insert_or_update(packet.topic_name(), ta);
                                }
                            }
                        }

                        // Send response packets
                        if puback_send {
                            let puback = v5_0::GenericPuback::builder()
                                .packet_id(packet.packet_id().unwrap())
                                .build()
                                .unwrap();
                            events.extend(self.process_send_v5_0_puback(puback));
                        }
                        if pubrec_send {
                            let pubrec = v5_0::GenericPubrec::builder()
                                .packet_id(packet.packet_id().unwrap())
                                .build()
                                .unwrap();
                            events.extend(self.process_send_v5_0_pubrec(pubrec));
                        }

                        // Refresh PINGREQ receive timer
                        events.extend(self.refresh_pingreq_recv());

                        // Notify packet received (only if not already handled)
                        if !already_handled {
                            events.push(GenericEvent::<
                                PacketIdType,
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                                PAYLOAD_BUFFER_SIZE,
                            >::NotifyPacketReceived(
                                packet.into()
                            ));
                        }
                    }
                    Err(e) => {
                        if self.status == ConnectionStatus::Connected {
                            let disconnect = v5_0::GenericDisconnect::<
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                            >::builder()
                            .reason_code(e.into())
                            .build()
                            .unwrap();
                            let disconnect_events = self.process_send_v5_0_disconnect(disconnect);
                            events.extend(disconnect_events);
                        }
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyError(e));
                    }
                }
            }
            PacketData::Normal(_) => {
                unreachable!("PUBLISH packet must use PacketData::Publish variant");
            }
        }

        events
    }

    fn process_recv_v3_1_1_puback(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericPuback::<PacketIdType>::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_puback.remove(&packet_id) {
                    self.store.erase(ResponsePacket::V3_1_1Puback, packet_id);
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    events.push(
                        GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::RequestClose,
                    );
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_puback(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericPuback::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_puback.remove(&packet_id) {
                    self.store.erase(ResponsePacket::V5_0Puback, packet_id);
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    if self.publish_send_max.is_some() {
                        self.publish_send_count -= 1;
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    let disconnect =
                        v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder(
                        )
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                    events.extend(self.process_send_v5_0_disconnect(disconnect));
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_pubrec(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericPubrec::<PacketIdType>::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_pubrec.remove(&packet_id) {
                    self.store.erase(ResponsePacket::V3_1_1Pubrec, packet_id);
                    if self.auto_pub_response && self.status == ConnectionStatus::Connected {
                        let pubrel = v3_1_1::GenericPubrel::<PacketIdType>::builder()
                            .packet_id(packet_id)
                            .build()
                            .unwrap();
                        events.extend(self.process_send_v3_1_1_pubrel(pubrel));
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    events.push(
                        GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::RequestClose,
                    );
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_pubrec(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericPubrec::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_pubrec.remove(&packet_id) {
                    self.store.erase(ResponsePacket::V5_0Pubrec, packet_id);
                    if let Some(reason_code) = packet.reason_code() {
                        if reason_code != PubrecReasonCode::Success {
                            if self.pid_man.is_used_id(packet_id) {
                                self.pid_man.release_id(packet_id);
                                events.push(GenericEvent::<
                                    PacketIdType,
                                    STRING_BUFFER_SIZE,
                                    BINARY_BUFFER_SIZE,
                                    PAYLOAD_BUFFER_SIZE,
                                >::NotifyPacketIdReleased(
                                    packet_id
                                ));
                            }
                            self.qos2_publish_processing.remove(&packet_id);
                            if self.publish_send_max.is_some() {
                                self.publish_send_count -= 1;
                            }
                        } else if self.auto_pub_response
                            && self.status == ConnectionStatus::Connected
                        {
                            let pubrel = v5_0::GenericPubrel::<
                                PacketIdType,
                                STRING_BUFFER_SIZE,
                                BINARY_BUFFER_SIZE,
                            >::builder()
                            .packet_id(packet_id)
                            .build()
                            .unwrap();
                            events.extend(self.process_send_v5_0_pubrel(pubrel));
                        }
                    } else if self.auto_pub_response && self.status == ConnectionStatus::Connected {
                        let pubrel = v5_0::GenericPubrel::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                        >::builder()
                        .packet_id(packet_id)
                        .build()
                        .unwrap();
                        events.extend(self.process_send_v5_0_pubrel(pubrel));
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    let disconnect =
                        v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder(
                        )
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                    events.extend(self.process_send_v5_0_disconnect(disconnect));
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::from(
                        DisconnectReasonCode::ProtocolError,
                    )));
                }
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_pubrel(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericPubrel::<PacketIdType>::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                self.qos2_publish_handled.remove(&packet_id);
                if self.auto_pub_response && self.status == ConnectionStatus::Connected {
                    let pubcomp = v3_1_1::GenericPubcomp::<PacketIdType>::builder()
                        .packet_id(packet_id)
                        .build()
                        .unwrap();
                    events.extend(self.process_send_v3_1_1_pubcomp(pubcomp));
                }
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_pubrel(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericPubrel::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                self.qos2_publish_handled.remove(&packet_id);
                if self.auto_pub_response && self.status == ConnectionStatus::Connected {
                    let pubcomp = v5_0::GenericPubcomp::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                    >::builder()
                    .packet_id(packet_id)
                    .build()
                    .unwrap();
                    events.extend(self.process_send_v5_0_pubcomp(pubcomp));
                }
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_pubcomp(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericPubcomp::<PacketIdType>::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_pubcomp.remove(&packet_id) {
                    self.store.erase(ResponsePacket::V3_1_1Pubcomp, packet_id);
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    self.qos2_publish_processing.remove(&packet_id);
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    events.push(
                        GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::RequestClose,
                    );
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_pubcomp(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericPubcomp::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_pubcomp.remove(&packet_id) {
                    self.store.erase(ResponsePacket::V5_0Pubcomp, packet_id);
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    self.qos2_publish_processing.remove(&packet_id);
                    if self.publish_send_max.is_some() {
                        self.publish_send_count -= 1;
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    let disconnect =
                        v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder(
                        )
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                    events.extend(self.process_send_v5_0_disconnect(disconnect));
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_subscribe(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericSubscribe::<PacketIdType, STRING_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_subscribe(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericSubscribe::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_suback(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericSuback::<PacketIdType>::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_suback.remove(&packet_id) {
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    events.push(
                        GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::RequestClose,
                    );
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_suback(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericSuback::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_suback.remove(&packet_id) {
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    let disconnect =
                        v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder(
                        )
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                    events.extend(self.process_send_v5_0_disconnect(disconnect));
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_unsubscribe(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericUnsubscribe::<PacketIdType, STRING_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_unsubscribe(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericUnsubscribe::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_unsuback(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::GenericUnsuback::<PacketIdType>::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_unsuback.remove(&packet_id) {
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    events.push(
                        GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::RequestClose,
                    );
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_unsuback(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericUnsuback::<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                let packet_id = packet.packet_id();
                if self.pid_unsuback.remove(&packet_id) {
                    if self.pid_man.is_used_id(packet_id) {
                        self.pid_man.release_id(packet_id);
                        events.push(GenericEvent::<
                            PacketIdType,
                            STRING_BUFFER_SIZE,
                            BINARY_BUFFER_SIZE,
                            PAYLOAD_BUFFER_SIZE,
                        >::NotifyPacketIdReleased(packet_id));
                    }
                    events.extend(self.refresh_pingreq_recv());
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyPacketReceived(packet.into()));
                } else {
                    let disconnect =
                        v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder(
                        )
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                    events.extend(self.process_send_v5_0_disconnect(disconnect));
                    events.push(GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::NotifyError(MqttError::ProtocolError));
                }
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_pingreq(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::Pingreq::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                if (Role::IS_SERVER || Role::IS_ANY)
                    && !self.is_client
                    && self.auto_ping_response
                    && self.status == ConnectionStatus::Connected
                {
                    let pingresp = v3_1_1::Pingresp::new();
                    events.extend(self.process_send_v3_1_1_pingresp(pingresp));
                }
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_pingreq(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::Pingreq::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                if (Role::IS_SERVER || Role::IS_ANY)
                    && !self.is_client
                    && self.auto_ping_response
                    && self.status == ConnectionStatus::Connected
                {
                    let pingresp = v5_0::Pingresp::new();
                    events.extend(self.process_send_v5_0_pingresp(pingresp));
                }
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_pingresp(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::Pingresp::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                self.pingresp_recv_set = false;
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::RequestTimerCancel(
                    TimerKind::PingrespRecv
                ));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_pingresp(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::Pingresp::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                self.pingresp_recv_set = false;
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::RequestTimerCancel(
                    TimerKind::PingrespRecv
                ));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v3_1_1_disconnect(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v3_1_1::Disconnect::parse(raw_packet.data_as_slice()) {
            Ok((packet, _)) => {
                self.cancel_timers(&mut events);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                events.push(
                    GenericEvent::<
                        PacketIdType,
                        STRING_BUFFER_SIZE,
                        BINARY_BUFFER_SIZE,
                        PAYLOAD_BUFFER_SIZE,
                    >::RequestClose,
                );
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_disconnect(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                self.cancel_timers(&mut events);
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn process_recv_v5_0_auth(
        &mut self,
        raw_packet: RawPacket,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();

        match v5_0::GenericAuth::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::parse(
            raw_packet.data_as_slice(),
        ) {
            Ok((packet, _)) => {
                events.extend(self.refresh_pingreq_recv());
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyPacketReceived(packet.into()));
            }
            Err(e) => {
                let disconnect =
                    v5_0::GenericDisconnect::<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>::builder()
                        .reason_code(DisconnectReasonCode::ProtocolError)
                        .build()
                        .unwrap();
                events.extend(self.process_send_v5_0_disconnect(disconnect));
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::NotifyError(e));
            }
        }

        events
    }

    fn get_topic_alias_from_props_opt(
        props: &Option<Vec<GenericProperty<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>>>,
    ) -> Option<u16> {
        if let Some(props) = props {
            Self::get_topic_alias_from_props(props.as_slice())
        } else {
            None
        }
    }

    fn refresh_pingreq_recv(
        &mut self,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        let mut events = Vec::new();
        if let Some(timeout_ms) = self.pingreq_recv_timeout_ms {
            if self.status == ConnectionStatus::Connecting
                || self.status == ConnectionStatus::Connected
            {
                self.pingreq_recv_set = true;
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::RequestTimerReset {
                    kind: TimerKind::PingreqRecv,
                    duration_ms: timeout_ms,
                });
            } else {
                self.pingreq_recv_set = false;
                events.push(GenericEvent::<
                    PacketIdType,
                    STRING_BUFFER_SIZE,
                    BINARY_BUFFER_SIZE,
                    PAYLOAD_BUFFER_SIZE,
                >::RequestTimerCancel(TimerKind::PingreqRecv));
            }
        }

        events
    }

    /// Cancel timers and collect events instead of calling handlers
    fn cancel_timers(
        &mut self,
        events: &mut Vec<
            GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>,
        >,
    ) {
        if self.pingreq_send_set {
            self.pingreq_send_set = false;
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestTimerCancel(TimerKind::PingreqSend));
        }
        if self.pingreq_recv_set {
            self.pingreq_recv_set = false;
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestTimerCancel(TimerKind::PingreqRecv));
        }
        if self.pingresp_recv_set {
            self.pingresp_recv_set = false;
            events.push(GenericEvent::<
                PacketIdType,
                STRING_BUFFER_SIZE,
                BINARY_BUFFER_SIZE,
                PAYLOAD_BUFFER_SIZE,
            >::RequestTimerCancel(TimerKind::PingrespRecv));
        }
    }

    /// Helper function to extract TopicAlias from properties
    fn get_topic_alias_from_props(
        props: &[GenericProperty<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>],
    ) -> Option<u16> {
        for prop in props {
            if let GenericProperty::TopicAlias(ta) = prop {
                return Some(ta.val());
            }
        }
        None
    }

    #[allow(dead_code)]
    fn is_packet_id_used(&self, packet_id: PacketIdType) -> bool {
        self.pid_man.is_used_id(packet_id)
    }
}

// traits

pub trait RecvBehavior<
    Role,
    PacketIdType,
    const STRING_BUFFER_SIZE: usize = 32,
    const BINARY_BUFFER_SIZE: usize = 32,
    const PAYLOAD_BUFFER_SIZE: usize = 32,
> where
    PacketIdType: IsPacketId,
{
    fn recv(
        &mut self,
        data: &mut Cursor<&[u8]>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>;
}

// RecvBehavior implementations
impl<
        PacketIdType,
        const STRING_BUFFER_SIZE: usize,
        const BINARY_BUFFER_SIZE: usize,
        const PAYLOAD_BUFFER_SIZE: usize,
    >
    RecvBehavior<
        role::Client,
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
    for GenericConnection<
        role::Client,
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
where
    PacketIdType: IsPacketId,
{
    fn recv(
        &mut self,
        data: &mut Cursor<&[u8]>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        self.recv(data)
    }
}

impl<
        PacketIdType,
        const STRING_BUFFER_SIZE: usize,
        const BINARY_BUFFER_SIZE: usize,
        const PAYLOAD_BUFFER_SIZE: usize,
    >
    RecvBehavior<
        role::Server,
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
    for GenericConnection<
        role::Server,
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
where
    PacketIdType: IsPacketId,
{
    fn recv(
        &mut self,
        data: &mut Cursor<&[u8]>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        self.recv(data)
    }
}

impl<
        PacketIdType,
        const STRING_BUFFER_SIZE: usize,
        const BINARY_BUFFER_SIZE: usize,
        const PAYLOAD_BUFFER_SIZE: usize,
    >
    RecvBehavior<
        role::Any,
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
    for GenericConnection<
        role::Any,
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
where
    PacketIdType: IsPacketId,
{
    fn recv(
        &mut self,
        data: &mut Cursor<&[u8]>,
    ) -> Vec<GenericEvent<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE>>
    {
        self.recv(data)
    }
}

// tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mqtt_internal::connection::version::Version;
    use crate::mqtt_internal::packet::TopicAliasSend;
    use crate::mqtt_internal::role;

    #[test]
    fn test_initialize_client_mode() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Initialize in client mode
        connection.initialize(true);

        // Verify client mode is set
        assert!(connection.is_client);
        assert_eq!(connection.publish_send_count, 0);
        assert!(connection.publish_send_max.is_none());
        assert!(connection.publish_recv_max.is_none());
        assert!(!connection.need_store);
    }

    #[test]
    fn test_initialize_server_mode() {
        let mut connection = GenericConnection::<role::Server, u32>::new(Version::V3_1_1);

        // Initialize in server mode
        connection.initialize(false);

        // Verify server mode is set
        assert!(!connection.is_client);
        assert_eq!(connection.publish_send_count, 0);
        assert!(connection.publish_send_max.is_none());
        assert!(connection.publish_recv_max.is_none());
        assert!(!connection.need_store);
    }

    #[test]
    fn test_validate_topic_alias_no_topic_alias_send() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Should return None when topic_alias_send is not configured
        let result = connection.validate_topic_alias(Some(1));
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_topic_alias_none_input() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Should return None when no topic alias is provided
        let result = connection.validate_topic_alias(None);
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_topic_alias_range_no_topic_alias_send() {
        let connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Should return false when topic_alias_send is not configured
        let result = connection.validate_topic_alias_range(1);
        assert!(!result);
    }

    #[test]
    fn test_validate_topic_alias_range_zero() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set up topic alias send with max 10
        let topic_alias_send = TopicAliasSend::new(10);
        connection.topic_alias_send = Some(topic_alias_send);

        // Should return false for alias 0
        let result = connection.validate_topic_alias_range(0);
        assert!(!result);
    }

    #[test]
    fn test_validate_topic_alias_range_over_max() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set up topic alias send with max 5
        let topic_alias_send = TopicAliasSend::new(5);
        connection.topic_alias_send = Some(topic_alias_send);

        // Should return false for alias > max
        let result = connection.validate_topic_alias_range(6);
        assert!(!result);
    }

    #[test]
    fn test_validate_topic_alias_range_valid() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set up topic alias send with max 10
        let topic_alias_send = TopicAliasSend::new(10);
        connection.topic_alias_send = Some(topic_alias_send);

        // Should return true for valid aliases
        assert!(connection.validate_topic_alias_range(1));
        assert!(connection.validate_topic_alias_range(5));
        assert!(connection.validate_topic_alias_range(10));
    }

    #[test]
    fn test_validate_topic_alias_with_registered_alias() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set up topic alias send with max 10
        let mut topic_alias_send = TopicAliasSend::new(10);
        topic_alias_send.insert_or_update("test/topic", 5);
        connection.topic_alias_send = Some(topic_alias_send);

        // Should return the topic name for registered alias
        let result = connection.validate_topic_alias(Some(5));
        assert_eq!(result, Some("test/topic".to_string()));
    }

    #[test]
    fn test_validate_topic_alias_unregistered_alias() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set up topic alias send with max 10 but don't register any aliases
        let topic_alias_send = TopicAliasSend::new(10);
        connection.topic_alias_send = Some(topic_alias_send);

        // Should return None for unregistered alias
        let result = connection.validate_topic_alias(Some(5));
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_maximum_packet_size_within_limit() {
        let connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Default maximum_packet_size_send is u32::MAX
        let result = connection.validate_maximum_packet_size_send(1000);
        assert!(result);
    }

    #[test]
    fn test_validate_maximum_packet_size_at_limit() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set a specific limit
        connection.maximum_packet_size_send = 1000;

        // Should return true for size equal to limit
        let result = connection.validate_maximum_packet_size_send(1000);
        assert!(result);
    }

    #[test]
    fn test_validate_maximum_packet_size_over_limit() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set a specific limit
        connection.maximum_packet_size_send = 1000;

        // Should return false for size over limit
        let result = connection.validate_maximum_packet_size_send(1001);
        assert!(!result);
    }

    #[test]
    fn test_validate_maximum_packet_size_zero_limit() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set limit to 0
        connection.maximum_packet_size_send = 0;

        // Should return false for any non-zero size
        let result = connection.validate_maximum_packet_size_send(1);
        assert!(!result);

        // Should return true for zero size
        let result = connection.validate_maximum_packet_size_send(0);
        assert!(result);
    }

    #[test]
    fn test_initialize_clears_state() {
        let mut connection = GenericConnection::<role::Client, u16>::new(Version::V5_0);

        // Set up some state that should be cleared
        connection.publish_send_count = 5;
        connection.need_store = true;
        connection.pid_suback.insert(123);
        connection.pid_unsuback.insert(456);

        // Initialize should clear state
        connection.initialize(true);

        // Verify state is cleared
        assert_eq!(connection.publish_send_count, 0);
        assert!(!connection.need_store);
        assert!(connection.pid_suback.is_empty());
        assert!(connection.pid_unsuback.is_empty());
        assert!(connection.is_client);
    }

    #[test]
    fn test_remaining_length_to_total_size() {
        // Test 1-byte remaining length encoding (0-127)
        assert_eq!(remaining_length_to_total_size(0), 2); // 1 + 1 + 0
        assert_eq!(remaining_length_to_total_size(127), 129); // 1 + 1 + 127

        // Test 2-byte remaining length encoding (128-16383)
        assert_eq!(remaining_length_to_total_size(128), 131); // 1 + 2 + 128
        assert_eq!(remaining_length_to_total_size(16383), 16386); // 1 + 2 + 16383

        // Test 3-byte remaining length encoding (16384-2097151)
        assert_eq!(remaining_length_to_total_size(16384), 16388); // 1 + 3 + 16384
        assert_eq!(remaining_length_to_total_size(2097151), 2097155); // 1 + 3 + 2097151

        // Test 4-byte remaining length encoding (2097152-268435455)
        assert_eq!(remaining_length_to_total_size(2097152), 2097157); // 1 + 4 + 2097152
        assert_eq!(remaining_length_to_total_size(268435455), 268435460); // 1 + 4 + 268435455
    }
}
