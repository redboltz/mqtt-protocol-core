/*!
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

/// Trait defining MQTT connection role types
///
/// This trait provides a type-level mechanism to distinguish between different
/// MQTT connection roles (Client, Server, Any) at compile time. It enables
/// role-specific behavior and validation throughout the MQTT protocol implementation
/// without runtime overhead.
///
/// Each role type implements this trait with specific constant boolean flags
/// that indicate which role it represents. This allows for compile-time role
/// detection and role-specific code paths.
///
/// # Role Types
///
/// - **Client**: Represents an MQTT client that connects to brokers
/// - **Server**: Represents an MQTT broker/server that accepts client connections  
/// - **Any**: Represents a generic role that can behave as either client or server
///
/// # Design Pattern
///
/// This trait uses the "phantom type" pattern where the type itself carries
/// semantic meaning without runtime data. The constant boolean flags allow
/// for efficient compile-time branching and role validation.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Check role type at compile time
/// fn process_connection<R: mqtt::connection::RoleType>() {
///     if R::IS_CLIENT {
///         // Client-specific processing
///     } else if R::IS_SERVER {
///         // Server-specific processing
///     }
/// }
///
/// // Use with specific role types
/// process_connection::<mqtt::connection::Client>();
/// process_connection::<mqtt::connection::Server>();
/// ```
#[rustfmt::skip]
pub trait RoleType: 'static {
    /// Indicates if this role type represents an MQTT client
    ///
    /// Set to `true` for client roles, `false` for all other roles.
    /// Clients initiate connections to MQTT brokers and can publish
    /// messages, subscribe to topics, and receive messages.
    const IS_CLIENT: bool = false;
    
    /// Indicates if this role type represents an MQTT server/broker
    ///
    /// Set to `true` for server roles, `false` for all other roles.
    /// Servers accept client connections, route messages between clients,
    /// manage subscriptions, and handle retained messages.
    const IS_SERVER: bool = false;
    
    /// Indicates if this role type can represent any MQTT role
    ///
    /// Set to `true` for generic roles that can behave as either client
    /// or server, `false` for specific role types. This is useful for
    /// testing and flexible implementations.
    const IS_ANY:    bool = false;
}

/// MQTT Client role type
///
/// Represents an MQTT client that connects to MQTT brokers. Clients can:
/// - Establish connections to brokers using CONNECT packets
/// - Publish messages to topics
/// - Subscribe to topic filters to receive messages
/// - Send heartbeat PINGREQ packets
/// - Gracefully disconnect using DISCONNECT packets
///
/// This is a zero-sized type used purely for compile-time role identification.
/// The actual client behavior is implemented in the connection logic that
/// uses this role type as a generic parameter.
///
/// # Protocol Restrictions
///
/// When using the Client role, certain MQTT packets are restricted:
/// - Cannot send CONNACK (connection acknowledgment)
/// - Cannot send SUBACK (subscription acknowledgment)
/// - Cannot send UNSUBACK (unsubscription acknowledgment)
/// - Cannot send PINGRESP (ping response)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Client role is typically used as a generic parameter
/// type ClientConnection = mqtt::connection::GenericConnection<mqtt::connection::Client, u16>;
///
/// // Role constants can be checked at compile time
/// assert_eq!(mqtt::connection::Client::IS_CLIENT, true);
/// assert_eq!(mqtt::connection::Client::IS_SERVER, false);
/// ```
pub struct Client;

/// MQTT Server/Broker role type
///
/// Represents an MQTT broker/server that accepts client connections. Servers can:
/// - Accept CONNECT packets from clients and respond with CONNACK
/// - Receive published messages and route them to subscribers
/// - Handle SUBSCRIBE packets and respond with SUBACK
/// - Handle UNSUBSCRIBE packets and respond with UNSUBACK
/// - Respond to PINGREQ packets with PINGRESP
/// - Manage client sessions and retained messages
///
/// This is a zero-sized type used purely for compile-time role identification.
/// The actual server behavior is implemented in the connection logic that
/// uses this role type as a generic parameter.
///
/// # Protocol Restrictions
///
/// When using the Server role, certain MQTT packets are restricted:
/// - Cannot send CONNECT (connection request)
/// - Cannot send SUBSCRIBE (subscription request)
/// - Cannot send UNSUBSCRIBE (unsubscription request)
/// - Cannot send PINGREQ (ping request)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Server role is typically used as a generic parameter
/// type ServerConnection = mqtt::connection::GenericConnection<mqtt::connection::Server, u16>;
///
/// // Role constants can be checked at compile time
/// assert_eq!(mqtt::connection::Server::IS_CLIENT, false);
/// assert_eq!(mqtt::connection::Server::IS_SERVER, true);
/// ```
pub struct Server;

/// Generic MQTT role type
///
/// Represents a flexible MQTT role that can behave as either a client or server.
/// This role type is useful for:
/// - Testing scenarios where both client and server behavior is needed
/// - Bridge implementations that act as both client and server
/// - Development tools that need to simulate both roles
/// - Generic code that works with any MQTT role
///
/// This is a zero-sized type used purely for compile-time role identification.
/// When using the Any role, the implementation typically allows all MQTT
/// packet types without role-based restrictions.
///
/// # Protocol Behavior
///
/// The Any role typically allows all MQTT packet types and behaviors,
/// making it the most permissive role type. This flexibility comes at
/// the cost of losing compile-time role-specific validations.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Any role can be used for flexible implementations
/// type FlexibleConnection = mqtt::connection::GenericConnection<mqtt::connection::Any, u16>;
///
/// // Role constants can be checked at compile time
/// assert_eq!(mqtt::connection::Any::IS_CLIENT, false);
/// assert_eq!(mqtt::connection::Any::IS_SERVER, false);
/// assert_eq!(mqtt::connection::Any::IS_ANY, true);
/// ```
pub struct Any;

/// Implementation of `RoleType` for `Client`
///
/// Configures the Client role type with the appropriate role flags.
/// Only the `IS_CLIENT` flag is set to `true`, clearly identifying
/// this type as representing an MQTT client role.
impl RoleType for Client {
    const IS_CLIENT: bool = true;
}

/// Implementation of `RoleType` for `Server`
///
/// Configures the Server role type with the appropriate role flags.
/// Only the `IS_SERVER` flag is set to `true`, clearly identifying
/// this type as representing an MQTT server/broker role.
impl RoleType for Server {
    const IS_SERVER: bool = true;
}

/// Implementation of `RoleType` for `Any`
///
/// Configures the Any role type with the appropriate role flags.
/// Only the `IS_ANY` flag is set to `true`, indicating this type
/// can represent any MQTT role (client, server, or both).
impl RoleType for Any {
    const IS_ANY: bool = true;
}
