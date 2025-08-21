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

/// Trait for compile-time packet classification and type introspection
///
/// `PacketKind` provides a way to classify MQTT packet types at compile time using
/// constant boolean flags. This enables efficient type-based dispatch and filtering
/// without runtime overhead. Each MQTT packet type implements this trait with
/// appropriate flags set to `true`.
///
/// The trait includes flags for:
/// - **Packet types**: CONNECT, CONNACK, PUBLISH, PUBACK, PUBREC, PUBREL, PUBCOMP,
///   SUBSCRIBE, SUBACK, UNSUBSCRIBE, UNSUBACK, PINGREQ, PINGRESP, DISCONNECT, AUTH
/// - **Protocol versions**: V3_1_1, V5_0
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::packet::PacketKind;
/// use mqtt_protocol_core::mqtt::packet::v5_0::Connect;
///
/// // Check packet type at compile time
/// if Connect::IS_CONNECT {
///     println!("This is a CONNECT packet");
/// }
///
/// // Check protocol version
/// if Connect::IS_V5_0 {
///     println!("This is a v5.0 packet");
/// }
/// ```
pub trait PacketKind {
    /// `true` if this is a CONNECT packet
    const IS_CONNECT: bool = false;
    /// `true` if this is a CONNACK packet
    const IS_CONNACK: bool = false;
    /// `true` if this is a PUBLISH packet
    const IS_PUBLISH: bool = false;
    /// `true` if this is a PUBACK packet
    const IS_PUBACK: bool = false;
    /// `true` if this is a PUBREC packet
    const IS_PUBREC: bool = false;
    /// `true` if this is a PUBREL packet
    const IS_PUBREL: bool = false;
    /// `true` if this is a PUBCOMP packet
    const IS_PUBCOMP: bool = false;
    /// `true` if this is a SUBSCRIBE packet
    const IS_SUBSCRIBE: bool = false;
    /// `true` if this is a SUBACK packet
    const IS_SUBACK: bool = false;
    /// `true` if this is an UNSUBSCRIBE packet
    const IS_UNSUBSCRIBE: bool = false;
    /// `true` if this is an UNSUBACK packet
    const IS_UNSUBACK: bool = false;
    /// `true` if this is a PINGREQ packet
    const IS_PINGREQ: bool = false;
    /// `true` if this is a PINGRESP packet
    const IS_PINGRESP: bool = false;
    /// `true` if this is a DISCONNECT packet
    const IS_DISCONNECT: bool = false;
    /// `true` if this is an AUTH packet (v5.0 only)
    const IS_AUTH: bool = false;
    /// `true` if this is an MQTT v3.1.1 packet
    const IS_V3_1_1: bool = false;
    /// `true` if this is an MQTT v5.0 packet
    const IS_V5_0: bool = false;
}

// MQTT v3.1.1 and v5.0 packet implementations

/// `PacketKind` implementation for v3.1.1 CONNECT packet
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind
    for crate::mqtt::packet::v3_1_1::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    const IS_CONNECT: bool = true;
    const IS_V3_1_1: bool = true;
}

/// `PacketKind` implementation for v3.1.1 CONNACK packet
impl PacketKind for crate::mqtt::packet::v3_1_1::Connack {
    const IS_CONNACK: bool = true;
    const IS_V3_1_1: bool = true;
}

/// `PacketKind` implementation for v5.0 CONNECT packet
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind
    for crate::mqtt::packet::v5_0::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    const IS_CONNECT: bool = true;
    const IS_V5_0: bool = true;
}

/// `PacketKind` implementation for v5.0 CONNACK packet
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind
    for crate::mqtt::packet::v5_0::GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    const IS_CONNACK: bool = true;
    const IS_V5_0: bool = true;
}

// Additional v3.1.1 packet implementations

/// `PacketKind` implementation for v3.1.1 PINGREQ packet
impl PacketKind for crate::mqtt::packet::v3_1_1::Pingreq {
    const IS_PINGREQ: bool = true;
    const IS_V3_1_1: bool = true;
}

/// `PacketKind` implementation for v3.1.1 PINGRESP packet
impl PacketKind for crate::mqtt::packet::v3_1_1::Pingresp {
    const IS_PINGRESP: bool = true;
    const IS_V3_1_1: bool = true;
}

/// `PacketKind` implementation for v3.1.1 DISCONNECT packet
impl PacketKind for crate::mqtt::packet::v3_1_1::Disconnect {
    const IS_DISCONNECT: bool = true;
    const IS_V3_1_1: bool = true;
}

// Additional v5.0 packet implementations

/// `PacketKind` implementation for v5.0 PINGREQ packet
impl PacketKind for crate::mqtt::packet::v5_0::Pingreq {
    const IS_PINGREQ: bool = true;
    const IS_V5_0: bool = true;
}

impl PacketKind for crate::mqtt::packet::v5_0::Pingresp {
    const IS_PINGRESP: bool = true;
    const IS_V5_0: bool = true;
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind
    for crate::mqtt::packet::v5_0::GenericDisconnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    const IS_DISCONNECT: bool = true;
    const IS_V5_0: bool = true;
}

/// `PacketKind` implementation for v5.0 AUTH packet (v5.0 exclusive)
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind
    for crate::mqtt::packet::v5_0::GenericAuth<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    const IS_AUTH: bool = true;
    const IS_V5_0: bool = true;
}

/// `PacketKind` implementation for references to packet types
///
/// This implementation forwards all flags from the referenced type,
/// allowing packet references to be used with the same type checking.
impl<T: PacketKind> PacketKind for &T {
    const IS_CONNECT: bool = T::IS_CONNECT;
    const IS_CONNACK: bool = T::IS_CONNACK;
    const IS_PUBLISH: bool = T::IS_PUBLISH;
    const IS_PUBACK: bool = T::IS_PUBACK;
    const IS_PUBREC: bool = T::IS_PUBREC;
    const IS_PUBREL: bool = T::IS_PUBREL;
    const IS_PUBCOMP: bool = T::IS_PUBCOMP;
    const IS_SUBSCRIBE: bool = T::IS_SUBSCRIBE;
    const IS_SUBACK: bool = T::IS_SUBACK;
    const IS_UNSUBSCRIBE: bool = T::IS_UNSUBSCRIBE;
    const IS_UNSUBACK: bool = T::IS_UNSUBACK;
    const IS_PINGREQ: bool = T::IS_PINGREQ;
    const IS_PINGRESP: bool = T::IS_PINGRESP;
    const IS_DISCONNECT: bool = T::IS_DISCONNECT;
    const IS_AUTH: bool = T::IS_AUTH;
    const IS_V3_1_1: bool = T::IS_V3_1_1;
    const IS_V5_0: bool = T::IS_V5_0;
}

/// `PacketKind` implementation for `GenericPacket` enum
///
/// For enum types, all flags are `false` at compile time since the actual
/// packet type is determined at runtime. This implementation provides a
/// fallback for generic packet handling where compile-time type information
/// is not available.
impl<
        PacketIdType,
        const STRING_BUFFER_SIZE: usize,
        const BINARY_BUFFER_SIZE: usize,
        const PAYLOAD_BUFFER_SIZE: usize,
    > PacketKind
    for crate::mqtt::packet::GenericPacket<
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
where
    PacketIdType: crate::mqtt::packet::IsPacketId + serde::Serialize,
{
    // All flags use default (false) values
}

// Generic packet implementations for v3.1.1

/// `PacketKind` implementation for generic v3.1.1 PUBLISH packet
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize> PacketKind
    for crate::mqtt::packet::v3_1_1::GenericPublish<
        PacketIdType,
        STRING_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
    PacketIdType: 'static, // Ensure PacketIdType is not u16 implicitly
{
    const IS_PUBLISH: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType> PacketKind for crate::mqtt::packet::v3_1_1::GenericPuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBACK: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType> PacketKind for crate::mqtt::packet::v3_1_1::GenericPubrec<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBREC: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType> PacketKind for crate::mqtt::packet::v3_1_1::GenericPubrel<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBREL: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType> PacketKind for crate::mqtt::packet::v3_1_1::GenericPubcomp<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBCOMP: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v3_1_1::GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_SUBSCRIBE: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType> PacketKind for crate::mqtt::packet::v3_1_1::GenericSuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_SUBACK: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v3_1_1::GenericUnsubscribe<PacketIdType, STRING_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_UNSUBSCRIBE: bool = true;
    const IS_V3_1_1: bool = true;
}

impl<PacketIdType> PacketKind for crate::mqtt::packet::v3_1_1::GenericUnsuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_UNSUBACK: bool = true;
    const IS_V3_1_1: bool = true;
}

// Generic packet implementations for v5.0
impl<
        PacketIdType,
        const STRING_BUFFER_SIZE: usize,
        const BINARY_BUFFER_SIZE: usize,
        const PAYLOAD_BUFFER_SIZE: usize,
    > PacketKind
    for crate::mqtt::packet::v5_0::GenericPublish<
        PacketIdType,
        STRING_BUFFER_SIZE,
        BINARY_BUFFER_SIZE,
        PAYLOAD_BUFFER_SIZE,
    >
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBLISH: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericPuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBACK: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericPubrec<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBREC: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericPubrel<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBREL: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericPubcomp<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_PUBCOMP: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_SUBSCRIBE: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericSuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_SUBACK: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericUnsubscribe<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_UNSUBSCRIBE: bool = true;
    const IS_V5_0: bool = true;
}

impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> PacketKind for crate::mqtt::packet::v5_0::GenericUnsuback<PacketIdType, STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    const IS_UNSUBACK: bool = true;
    const IS_V5_0: bool = true;
}
