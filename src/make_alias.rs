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

// Allow unused type aliases since not all generated aliases may be used in every context
#![allow(dead_code)]
// Allow unused imports since not all imports may be used in every generated module
#![allow(unused_imports)]

/// Generate type aliases for all MQTT packet types with custom buffer sizes and packet ID type
///
/// This macro creates type aliases for all Generic* structs in both v3.1.1 and v5.0 MQTT versions,
/// allowing users to specify custom buffer sizes for string, binary, and payload data.
///
/// # Parameters
///
/// * `$mod_name` - The module name for the generated aliases (e.g., `mqtt`, `mqtt_128`, `mqtt_big`)
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
///   - Use `Connection<$mod_name::connection::role::Client>` for client connections
///   - Use `Connection<$mod_name::connection::role::Server>` for server connections
///   - Use `Connection<$mod_name::connection::role::Any>` for flexible connections
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::*;
///
/// // Generate aliases with u16 packet IDs and 128-byte buffers in mqtt_128 module
/// make_type_size_aliases!(mqtt_128, u16, 128, 128, 128);
///
/// // Generate aliases with u32 packet IDs and large buffers in mqtt_big module
/// make_type_size_aliases!(mqtt_big, u32, 512, 512, 1024);
///
/// // Now you can use the convenient aliases:
/// let connect = mqtt_128::packet::v5_0::Connect::builder()
///     .client_identifier("my_client")
///     .build()
///     .unwrap();
///
/// let publish = mqtt_big::packet::v5_0::Publish::builder()
///     .packet_id(1u32)
///     .topic_name("my/topic")
///     .payload(b"hello world")
///     .build()
///     .unwrap();
///
/// // Use connection types with specific roles:
/// type ClientConn = mqtt_128::connection::Connection<mqtt_128::connection::role::Client>;
/// type ServerConn = mqtt_big::connection::Connection<mqtt_big::connection::role::Server>;
/// ```
#[macro_export]
macro_rules! make_type_size_aliases {
    ($mod_name:ident, $packet_id_type:ty, $string_buffer_size:expr, $binary_buffer_size:expr, $payload_buffer_size:expr) => {
        pub mod $mod_name {
            pub mod packet {
                pub mod v3_1_1 {
                    // Generic* → * aliases only
                    pub type Connect = $crate::mqtt_internal::packet::v3_1_1::GenericConnect<
                        $string_buffer_size,
                        $binary_buffer_size,
                    >;
                    pub type Publish = $crate::mqtt_internal::packet::v3_1_1::GenericPublish<
                        $packet_id_type,
                        $string_buffer_size,
                        $payload_buffer_size,
                    >;
                    pub type Subscribe = $crate::mqtt_internal::packet::v3_1_1::GenericSubscribe<
                        $packet_id_type,
                        $string_buffer_size,
                    >;
                    pub type Unsubscribe =
                        $crate::mqtt_internal::packet::v3_1_1::GenericUnsubscribe<
                            $packet_id_type,
                            $string_buffer_size,
                        >;
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

                    // Re-export everything else from mqtt_internal
                    #[allow(unused_imports)]
                    pub use $crate::mqtt_internal::packet::v3_1_1::*;
                }

                pub mod v5_0 {
                    // Generic* → * aliases only
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

                    // Re-export everything else from mqtt_internal
                    #[allow(unused_imports)]
                    pub use $crate::mqtt_internal::packet::v5_0::*;
                }

                // Generic* → * aliases for packet-level types
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

                // Generic* → * aliases for property types
                pub type ContentType =
                    $crate::mqtt_internal::packet::GenericContentType<$string_buffer_size>;
                pub type ResponseTopic =
                    $crate::mqtt_internal::packet::GenericResponseTopic<$string_buffer_size>;
                pub type CorrelationData =
                    $crate::mqtt_internal::packet::GenericCorrelationData<$binary_buffer_size>;
                pub type AssignedClientIdentifier =
                    $crate::mqtt_internal::packet::GenericAssignedClientIdentifier<
                        $string_buffer_size,
                    >;
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

                // Re-export everything else from mqtt_internal
                #[allow(unused_imports)]
                pub use $crate::mqtt_internal::packet::*;
            }

            pub mod connection {
                // Generic* → * aliases only
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

                // Re-export everything else from mqtt_internal
                #[allow(unused_imports)]
                pub use $crate::mqtt_internal::connection::*;
            }

            pub mod common {
                // Generic* → * aliases only
                pub type ArcPayload =
                    $crate::mqtt_internal::common::GenericArcPayload<$payload_buffer_size>;

                // Re-export everything else from mqtt_internal
                #[allow(unused_imports)]
                pub use $crate::mqtt_internal::common::*;
            }

            // Top-level convenience aliases
            pub type Connection<Role> = $crate::mqtt_internal::connection::GenericConnection<
                Role,
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

            // Re-export everything else from mqtt_internal at the top level
            #[allow(unused_imports)]
            pub use $crate::mqtt_internal::*;
            // Ensure trait is available for packet operations
            #[allow(unused_imports)]
            pub use $crate::mqtt_internal::packet::GenericPacketTrait;

            // Re-export prelude for convenience
            pub mod prelude {
                #[allow(unused_imports)]
                pub use $crate::mqtt_internal::connection::prelude::*;
                #[allow(unused_imports)]
                pub use $crate::mqtt_internal::packet::prelude::*;
            }
        }
    };
}

/// Generate type aliases for all MQTT packet types with custom buffer sizes using u16 packet IDs
///
/// This macro is a convenience wrapper around `make_type_size_aliases!` that uses the standard
/// u16 packet identifier type as specified in the MQTT specification.
///
/// # Parameters
///
/// * `$mod_name` - The module name for the generated aliases (e.g., `mqtt`, `mqtt_128`, `mqtt_big`)
/// * `$string_buffer_size` - Buffer size for string data (typically 32, 64, 128, etc.)
/// * `$binary_buffer_size` - Buffer size for binary data (typically 32, 64, 128, etc.)
/// * `$payload_buffer_size` - Buffer size for payload data (typically 32, 64, 128, etc.)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::*;
///
/// // Generate aliases with standard u16 packet IDs and custom buffer sizes
/// make_size_aliases!(mqtt_128, 128, 128, 256);
///
/// // Use the generated aliases:
/// let publish = mqtt_128::packet::v5_0::Publish::builder()
///     .packet_id(1u16)
///     .topic_name("sensors/temperature")
///     .payload(b"22.5")
///     .build()
///     .unwrap();
///
/// let connect = mqtt_128::packet::v3_1_1::Connect::builder()
///     .client_identifier("temperature_sensor")
///     .build()
///     .unwrap();
/// ```
#[macro_export]
macro_rules! make_size_aliases {
    ($mod_name:ident, $string_buffer_size:expr, $binary_buffer_size:expr, $payload_buffer_size:expr) => {
        $crate::make_type_size_aliases!(
            $mod_name,
            u16,
            $string_buffer_size,
            $binary_buffer_size,
            $payload_buffer_size
        );
    };
}

// Re-export the macros at the crate level for easier access
pub use make_size_aliases;
pub use make_type_size_aliases;

/// Generate default type aliases with standard buffer sizes (32, 32, 128) in the `mqtt` module.
/// This is a convenience macro that calls `make_size_aliases!` with default values.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::*;
///
/// // Generate default aliases in the mqtt module
/// make_default_aliases!();
///
/// // Use the generated aliases:
/// let publish = mqtt::packet::v5_0::Publish::builder()
///     .packet_id(1u16)
///     .topic_name("test/topic")
///     .payload(b"test payload")
///     .build()
///     .unwrap();
///
/// let connect = mqtt::packet::v3_1_1::Connect::builder()
///     .client_identifier("test_client")
///     .build()
///     .unwrap();
/// ```
#[macro_export]
macro_rules! make_default_aliases {
    () => {
        $crate::make_size_aliases!(mqtt, 32, 32, 128);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_expansion() {
        // Test that the macros expand without compilation errors
        make_size_aliases!(mqtt_test, 64, 64, 128);

        // This should create the type aliases in the module scope
        let _ = || {
            let _auth = mqtt_test::packet::v5_0::Auth::builder().build();
            let _connect = mqtt_test::packet::v3_1_1::Connect::builder()
                .client_id("test")
                .unwrap()
                .build()
                .unwrap();
        };
    }

    #[test]
    fn test_custom_packet_id_type() {
        // Test with u32 packet ID type for extended packet IDs
        make_type_size_aliases!(mqtt_u32_test, u32, 32, 32, 64);

        let _ = || {
            // These type aliases should work with u32 packet IDs
            let _puback = mqtt_u32_test::packet::v5_0::Puback::builder()
                .packet_id(1u32)
                .build();
            let _publish = mqtt_u32_test::packet::v5_0::Publish::builder()
                .packet_id(2u32)
                .topic_name("test")
                .unwrap()
                .build()
                .unwrap();

            // Test connection types with different roles
            let _ = core::marker::PhantomData::<
                mqtt_u32_test::connection::Connection<mqtt_u32_test::connection::role::Client>,
            >;
            let _ = core::marker::PhantomData::<
                mqtt_u32_test::connection::Connection<mqtt_u32_test::connection::role::Server>,
            >;
            let _ = core::marker::PhantomData::<
                mqtt_u32_test::connection::Connection<mqtt_u32_test::connection::role::Any>,
            >;
        };
    }

    #[test]
    fn test_default_aliases() {
        // Test the default aliases macro
        make_default_aliases!();

        let _ = || {
            // These should use the default mqtt module
            let _connect = mqtt::packet::v3_1_1::Connect::builder()
                .client_id("default_test")
                .unwrap()
                .build()
                .unwrap();

            let _publish = mqtt::packet::v5_0::Publish::builder()
                .packet_id(1u16)
                .topic_name("default/test")
                .unwrap()
                .build()
                .unwrap();
        };
    }

    #[test]
    fn test_multiple_size_aliases() {
        // Test that multiple different sized aliases can coexist
        make_size_aliases!(mqtt_small, 16, 16, 32);
        make_size_aliases!(mqtt_large, 256, 256, 512);
        make_type_size_aliases!(mqtt_u32_large, u32, 128, 128, 256);

        let _ = || {
            // Each should work independently
            let _small_connect = mqtt_small::packet::v3_1_1::Connect::builder()
                .client_id("s")
                .unwrap()
                .build()
                .unwrap();

            let _large_connect = mqtt_large::packet::v3_1_1::Connect::builder()
                .client_id("large_client_identifier_with_long_name")
                .unwrap()
                .build()
                .unwrap();

            let _u32_publish = mqtt_u32_large::packet::v5_0::Publish::builder()
                .packet_id(0x12345678u32)
                .topic_name("test")
                .unwrap()
                .build()
                .unwrap();
        };
    }
}
