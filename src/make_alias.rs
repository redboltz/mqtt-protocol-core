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

//! Type alias generation macros for MQTT packet types
//!
//! This module provides macros to generate convenient type aliases for Generic MQTT packet types
//! with customizable buffer sizes. This allows library users to easily create packet types
//! with different buffer sizes without having to specify all the generic parameters manually.

/// Generate type aliases for all MQTT packet types with custom buffer sizes and packet ID type
///
/// This macro creates type aliases for all Generic* structs in both v3.1.1 and v5.0 MQTT versions,
/// allowing users to specify custom buffer sizes for string, binary, and payload data.
///
/// # Parameters
///
/// * `$packet_id_type` - The packet identifier type (typically `u16` or `u32`)
/// * `$string_buffer_size` - Buffer size for string data (typically 32, 64, 128, etc.)
/// * `$binary_buffer_size` - Buffer size for binary data (typically 32, 64, 128, etc.)
/// * `$payload_buffer_size` - Buffer size for payload data (typically 32, 64, 128, etc.)
///
/// # Generated Type Aliases
///
/// For v3.1.1 packets:
/// * `Connect`, `Publish`, `Subscribe`, `Unsubscribe` (with buffer parameters where applicable)
/// * `Puback`, `Pubrec`, `Pubrel`, `Pubcomp`, `Suback`, `Unsuback` (with packet ID type)
///
/// For v5.0 packets:
/// * All packets with their respective buffer size parameters
/// * Packets with PacketIdType parameter use the specified packet ID type
/// * Packets without PacketIdType (like Auth, Connack, Disconnect) use buffer sizes only
///
/// For connections:
/// * `Connection<Role>` - Generic connection type that accepts any role type parameter
///   - Use `Connection<mqtt::connection::role::Client>` for client connections
///   - Use `Connection<mqtt::connection::role::Server>` for server connections
///   - Use `Connection<mqtt::connection::role::Any>` for flexible connections
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Generate aliases with u16 packet IDs and 64-byte buffers
/// mqtt::make_type_size_aliases!(u16, 64, 64, 64);
///
/// // Now you can use the convenient aliases:
/// let connect = mqtt::packet::v5_0::Connect::builder()
///     .client_identifier("my_client")
///     .build()
///     .unwrap();
///
/// let publish = mqtt::packet::v5_0::Publish::builder()
///     .topic_name("my/topic")
///     .payload(b"hello world")
///     .build()
///     .unwrap();
///
/// // Use connection types with specific roles:
/// type ClientConn = Connection<mqtt::connection::role::Client>;
/// type ServerConn = Connection<mqtt::connection::role::Server>;
/// type AnyConn = Connection<mqtt::connection::role::Any>;
/// ```
#[macro_export]
macro_rules! make_type_size_aliases {
    ($packet_id_type:ty, $string_buffer_size:expr, $binary_buffer_size:expr, $payload_buffer_size:expr) => {
        // Create a full module hierarchy that shadows the existing mqtt module structure
        mod generated_aliases {
            pub mod mqtt {
                pub mod packet {
                    // v3.1.1 packet type aliases
                    pub mod v3_1_1 {
                        // Packets with buffer size parameters
                        pub type Connect =
                            $crate::mqtt_internal::packet::v3_1_1::GenericConnect<$string_buffer_size>;
                        pub type Publish = $crate::mqtt_internal::packet::v3_1_1::GenericPublish<
                            $packet_id_type,
                            $string_buffer_size,
                            $payload_buffer_size,
                        >;
                        pub type Subscribe = $crate::mqtt_internal::packet::v3_1_1::GenericSubscribe<
                            $packet_id_type,
                            $string_buffer_size,
                        >;
                        pub type Unsubscribe = $crate::mqtt_internal::packet::v3_1_1::GenericUnsubscribe<
                            $packet_id_type,
                            $string_buffer_size,
                        >;

                        // Packets with only packet ID type
                        pub type Puback =
                            $crate::mqtt_internal::packet::v3_1_1::GenericPuback<$packet_id_type>;
                        pub type Pubrec =
                            $crate::mqtt_internal::packet::v3_1_1::GenericPubrec<$packet_id_type>;
                        pub type Pubrel =
                            $crate::mqtt_internal::packet::v3_1_1::GenericPubrel<$packet_id_type>;
                        pub type Pubcomp =
                            $crate::mqtt_internal::packet::v3_1_1::GenericPubcomp<$packet_id_type>;
                        pub type Suback =
                            $crate::mqtt_internal::packet::v3_1_1::GenericSuback<$packet_id_type>;
                        pub type Unsuback =
                            $crate::mqtt_internal::packet::v3_1_1::GenericUnsuback<$packet_id_type>;
                    }

                    // v5.0 packet type aliases
                    pub mod v5_0 {
                        // Packets with buffer size parameters only (no PacketIdType)
                        pub type Auth = $crate::mqtt_internal::packet::v5_0::GenericAuth<
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Connack = $crate::mqtt_internal::packet::v5_0::GenericConnack<
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Connect = $crate::mqtt_internal::packet::v5_0::GenericConnect<
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Disconnect = $crate::mqtt_internal::packet::v5_0::GenericDisconnect<
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;

                        // Packets with PacketIdType and buffer size parameters
                        pub type Puback = $crate::mqtt_internal::packet::v5_0::GenericPuback<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Pubcomp = $crate::mqtt_internal::packet::v5_0::GenericPubcomp<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Publish = $crate::mqtt_internal::packet::v5_0::GenericPublish<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                            $payload_buffer_size,
                        >;
                        pub type Pubrec = $crate::mqtt_internal::packet::v5_0::GenericPubrec<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Pubrel = $crate::mqtt_internal::packet::v5_0::GenericPubrel<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Suback = $crate::mqtt_internal::packet::v5_0::GenericSuback<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Subscribe = $crate::mqtt_internal::packet::v5_0::GenericSubscribe<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Unsuback = $crate::mqtt_internal::packet::v5_0::GenericUnsuback<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                        pub type Unsubscribe = $crate::mqtt_internal::packet::v5_0::GenericUnsubscribe<
                            $packet_id_type,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                    }

                    // Packet base types with custom buffer sizes
                    pub type Packet = $crate::mqtt_internal::packet::GenericPacket<
                        $packet_id_type,
                        $string_buffer_size,
                        $binary_buffer_size,
                        $payload_buffer_size,
                    >;
                    pub type StorePacket = $crate::mqtt_internal::packet::GenericStorePacket<
                        $packet_id_type,
                        $string_buffer_size,
                        $binary_buffer_size,
                        $payload_buffer_size,
                    >;
                    pub type Property = $crate::mqtt_internal::packet::GenericProperty<
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Properties = $crate::mqtt_internal::packet::GenericProperties<
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type MqttString =
                        $crate::mqtt_internal::packet::GenericMqttString<$string_buffer_size>;
                    pub type MqttBinary =
                        $crate::mqtt_internal::packet::GenericMqttBinary<$binary_buffer_size>;

                    // Property type aliases with custom buffer sizes
                    pub type ContentType =
                        $crate::mqtt_internal::packet::GenericContentType<$string_buffer_size>;
                    pub type ResponseTopic =
                        $crate::mqtt_internal::packet::GenericResponseTopic<$string_buffer_size>;
                    pub type CorrelationData =
                        $crate::mqtt_internal::packet::GenericCorrelationData<$binary_buffer_size>;
                    pub type AssignedClientIdentifier =
                        $crate::mqtt_internal::packet::GenericAssignedClientIdentifier<$string_buffer_size>;
                    pub type AuthenticationMethod =
                        $crate::mqtt_internal::packet::GenericAuthenticationMethod<$string_buffer_size>;
                    pub type AuthenticationData =
                        $crate::mqtt_internal::packet::GenericAuthenticationData<$binary_buffer_size>;
                    pub type ResponseInformation =
                        $crate::mqtt_internal::packet::GenericResponseInformation<$string_buffer_size>;
                    pub type ServerReference =
                        $crate::mqtt_internal::packet::GenericServerReference<$string_buffer_size>;
                    pub type ReasonString =
                        $crate::mqtt_internal::packet::GenericReasonString<$string_buffer_size>;
                    pub type UserProperty =
                        $crate::mqtt_internal::packet::GenericUserProperty<$string_buffer_size>;
                }

                // Connection types with custom buffer sizes
                pub type Connection<Role> = $crate::mqtt_internal::connection::GenericConnection<
                    Role,
                    $packet_id_type,
                    $string_buffer_size,
                    $binary_buffer_size,
                    $payload_buffer_size,
                >;

                pub type Store = $crate::mqtt_internal::connection::GenericStore<
                    $packet_id_type,
                    $string_buffer_size,
                    $binary_buffer_size,
                    $payload_buffer_size,
                >;

                pub type Event = $crate::mqtt_internal::connection::GenericEvent<
                    $packet_id_type,
                    $string_buffer_size,
                    $binary_buffer_size,
                    $payload_buffer_size,
                >;

                // Common types with custom buffer sizes
                pub type ArcPayload = $crate::mqtt_internal::common::GenericArcPayload<$payload_buffer_size>;
            }
        }

        // Re-export the generated aliases to shadow existing mqtt module
        pub use generated_aliases::*;
    };
}

/// Generate type aliases for all MQTT packet types with custom buffer sizes using u16 packet IDs
///
/// This macro is a convenience wrapper around `make_type_size_aliases!` that uses the standard
/// u16 packet identifier type as specified in the MQTT specification.
///
/// # Parameters
///
/// * `$string_buffer_size` - Buffer size for string data (typically 32, 64, 128, etc.)
/// * `$binary_buffer_size` - Buffer size for binary data (typically 32, 64, 128, etc.)
/// * `$payload_buffer_size` - Buffer size for payload data (typically 32, 64, 128, etc.)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt;
///
/// // Generate aliases with standard u16 packet IDs and custom buffer sizes
/// mqtt::make_size_aliases!(128, 128, 256);
///
/// // Use the generated aliases:
/// let publish = mqtt::packet::v5_0::Publish::builder()
///     .packet_id(1u16)
///     .topic_name("sensors/temperature")
///     .payload(b"22.5")
///     .build()
///     .unwrap();
///
/// let connect = mqtt::packet::v3_1_1::Connect::builder()
///     .client_identifier("temperature_sensor")
///     .build()
///     .unwrap();
/// ```
#[macro_export]
macro_rules! make_size_aliases {
    ($string_buffer_size:expr, $binary_buffer_size:expr, $payload_buffer_size:expr) => {
        // Define public mqtt module with type aliases in the calling scope
        pub mod mqtt {
            pub mod packet {
                pub mod v3_1_1 {
                    pub type Connect = $crate::mqtt_internal::packet::v3_1_1::GenericConnect<
                        $string_buffer_size,
                    >;
                    pub type Publish = $crate::mqtt_internal::packet::v3_1_1::GenericPublish<
                        u16,
                        $string_buffer_size,
                        $payload_buffer_size,
                    >;
                    pub type Subscribe =
                        $crate::mqtt_internal::packet::v3_1_1::GenericSubscribe<
                            u16,
                            $string_buffer_size,
                        >;
                    pub type Unsubscribe =
                        $crate::mqtt_internal::packet::v3_1_1::GenericUnsubscribe<
                            u16,
                            $string_buffer_size,
                        >;
                    pub type Puback =
                        $crate::mqtt_internal::packet::v3_1_1::GenericPuback<u16>;
                    pub type Pubrec =
                        $crate::mqtt_internal::packet::v3_1_1::GenericPubrec<u16>;
                    pub type Pubrel =
                        $crate::mqtt_internal::packet::v3_1_1::GenericPubrel<u16>;
                    pub type Pubcomp =
                        $crate::mqtt_internal::packet::v3_1_1::GenericPubcomp<u16>;
                    pub type Suback =
                        $crate::mqtt_internal::packet::v3_1_1::GenericSuback<u16>;
                    pub type Unsuback =
                        $crate::mqtt_internal::packet::v3_1_1::GenericUnsuback<u16>;

                    // Re-export non-alias types from mqtt_internal
                    pub use $crate::mqtt_internal::packet::v3_1_1::{
                        Connack, Disconnect, Pingreq, Pingresp,
                    };
                }

                pub mod v5_0 {
                    pub type Auth = $crate::mqtt_internal::packet::v5_0::GenericAuth<
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Connack = $crate::mqtt_internal::packet::v5_0::GenericConnack<
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Connect = $crate::mqtt_internal::packet::v5_0::GenericConnect<
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Disconnect =
                        $crate::mqtt_internal::packet::v5_0::GenericDisconnect<
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                    pub type Puback = $crate::mqtt_internal::packet::v5_0::GenericPuback<
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Pubcomp = $crate::mqtt_internal::packet::v5_0::GenericPubcomp<
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Publish = $crate::mqtt_internal::packet::v5_0::GenericPublish<
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                        $payload_buffer_size,
                    >;
                    pub type Pubrec = $crate::mqtt_internal::packet::v5_0::GenericPubrec<
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Pubrel = $crate::mqtt_internal::packet::v5_0::GenericPubrel<
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Suback = $crate::mqtt_internal::packet::v5_0::GenericSuback<
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Subscribe =
                        $crate::mqtt_internal::packet::v5_0::GenericSubscribe<
                            u16,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;
                    pub type Unsuback = $crate::mqtt_internal::packet::v5_0::GenericUnsuback<
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Unsubscribe =
                        $crate::mqtt_internal::packet::v5_0::GenericUnsubscribe<
                            u16,
                            $string_buffer_size,
                            $binary_buffer_size,
                        >;

                    // Re-export non-alias types from mqtt_internal
                    pub use $crate::mqtt_internal::packet::v5_0::{Pingreq, Pingresp};
                }

                // Property type aliases
                pub type ContentType =
                    $crate::mqtt_internal::packet::GenericContentType<$string_buffer_size>;
                pub type ResponseTopic =
                    $crate::mqtt_internal::packet::GenericResponseTopic<$string_buffer_size>;
                pub type CorrelationData =
                    $crate::mqtt_internal::packet::GenericCorrelationData<
                        $binary_buffer_size,
                    >;
                pub type AssignedClientIdentifier =
                    $crate::mqtt_internal::packet::GenericAssignedClientIdentifier<
                        $string_buffer_size,
                    >;
                pub type AuthenticationMethod =
                    $crate::mqtt_internal::packet::GenericAuthenticationMethod<
                        $string_buffer_size,
                    >;
                pub type AuthenticationData =
                    $crate::mqtt_internal::packet::GenericAuthenticationData<
                        $binary_buffer_size,
                    >;
                pub type ResponseInformation =
                    $crate::mqtt_internal::packet::GenericResponseInformation<
                        $string_buffer_size,
                    >;
                pub type ServerReference =
                    $crate::mqtt_internal::packet::GenericServerReference<
                        $string_buffer_size,
                    >;
                pub type ReasonString =
                    $crate::mqtt_internal::packet::GenericReasonString<$string_buffer_size>;
                pub type UserProperty =
                    $crate::mqtt_internal::packet::GenericUserProperty<$string_buffer_size>;

                // Base packet types
                pub type Packet = $crate::mqtt_internal::packet::GenericPacket<
                    u16,
                    $string_buffer_size,
                    $binary_buffer_size,
                    $payload_buffer_size,
                >;
                pub type StorePacket = $crate::mqtt_internal::packet::GenericStorePacket<
                    u16,
                    $string_buffer_size,
                    $binary_buffer_size,
                    $payload_buffer_size,
                >;
                pub type Property = $crate::mqtt_internal::packet::GenericProperty<
                    $string_buffer_size,
                    $binary_buffer_size,
                >;
                pub type Properties = $crate::mqtt_internal::packet::GenericProperties<
                    $string_buffer_size,
                    $binary_buffer_size,
                >;
                pub type MqttString =
                    $crate::mqtt_internal::packet::GenericMqttString<$string_buffer_size>;
                pub type MqttBinary =
                    $crate::mqtt_internal::packet::GenericMqttBinary<$binary_buffer_size>;

                // Re-export other important packet types
                pub use $crate::mqtt_internal::packet::{
                    MaximumPacketSize, MessageExpiryInterval, PacketType, PayloadFormat,
                    PayloadFormatIndicator, Qos, ReceiveMaximum, RequestProblemInformation,
                    RetainHandling, SubEntry, SubOpts, SubscriptionIdentifier, TopicAlias,
                    TopicAliasMaximum, WillDelayInterval,
                };

                // Re-export all Generic types for direct access
                pub use $crate::mqtt_internal::packet::{
                    GenericMqttBinary, GenericMqttString, GenericPacket, GenericProperties,
                    GenericProperty, GenericStorePacket,
                };
            }

            pub mod connection {
                pub type Connection<Role> =
                    $crate::mqtt_internal::connection::GenericConnection<
                        Role,
                        u16,
                        $string_buffer_size,
                        $binary_buffer_size,
                        $payload_buffer_size,
                    >;
                pub type Store = $crate::mqtt_internal::connection::GenericStore<
                    u16,
                    $string_buffer_size,
                    $binary_buffer_size,
                    $payload_buffer_size,
                >;
                pub type Event = $crate::mqtt_internal::connection::GenericEvent<
                    u16,
                    $string_buffer_size,
                    $binary_buffer_size,
                    $payload_buffer_size,
                >;

                // Re-export important connection types
                pub use $crate::mqtt_internal::connection::{role, SendBehavior};
            }

            pub mod common {
                pub type ArcPayload =
                    $crate::mqtt_internal::common::GenericArcPayload<$payload_buffer_size>;

                // Re-export other common types
                pub use $crate::mqtt_internal::common::{
                    Cursor, IntoPayload, ValueAllocator,
                };
            }

            // Re-export top-level important types
            pub use $crate::mqtt_internal::{result_code, Version};
        }
    };
}

// Re-export the macros at the crate level for easier access
pub use make_size_aliases;
pub use make_type_size_aliases;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_expansion() {
        // Test that the macros expand without compilation errors
        make_size_aliases!(64, 64, 128);

        // This should create the type aliases in the local scope
        // In a real usage scenario, these would be available at the module level
        let _ = || {
            let _auth = mqtt::packet::v5_0::Auth::builder().build();
            let _connect = mqtt::packet::v3_1_1::Connect::builder()
                .client_id("test")
                .unwrap()
                .build()
                .unwrap();
        };
    }

    #[test]
    fn test_custom_packet_id_type() {
        // Test with u32 packet ID type for extended packet IDs
        make_type_size_aliases!(u32, 32, 32, 64);

        let _ = || {
            // These type aliases should work with u32 packet IDs
            let _puback = mqtt::packet::v5_0::Puback::builder()
                .packet_id(1u32)
                .build();
            let _publish = mqtt::packet::v5_0::Publish::builder()
                .packet_id(2u32)
                .topic_name("test")
                .unwrap()
                .build()
                .unwrap();

            // Test connection types with different roles
            let _ = core::marker::PhantomData::<
                mqtt::Connection<crate::mqtt::connection::role::Client>,
            >;
            let _ = core::marker::PhantomData::<
                mqtt::Connection<crate::mqtt::connection::role::Server>,
            >;
            let _ =
                core::marker::PhantomData::<mqtt::Connection<crate::mqtt::connection::role::Any>>;
        };
    }
}

/// Generate default type aliases with standard buffer sizes (32, 32, 128).
/// This is a convenience macro that calls `make_size_aliases!` with default values.
#[macro_export]
macro_rules! make_default_aliases {
    () => {
        $crate::make_size_aliases!(32, 32, 128);
    };
}
