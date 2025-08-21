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
use crate::mqtt_internal::connection::role::*;
use crate::mqtt_internal::packet::*;

/// Role-specific trait to validate that a packet can be sent under a given Role.
pub trait SendableRole<Role> {}

// --- Role-specific implementations ---

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Client>
    for v5_0::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Client>
    for v3_1_1::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Server>
    for v5_0::GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl SendableRole<Server> for v3_1_1::Connack {}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Any>
    for v5_0::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Any>
    for v5_0::GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Any>
    for v3_1_1::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl SendableRole<Any> for v3_1_1::Connack {}

// Client sendable packets
impl SendableRole<Client> for v3_1_1::Pingreq {}
impl SendableRole<Client> for v3_1_1::Disconnect {}

impl SendableRole<Client> for v5_0::Pingreq {}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Client>
    for v5_0::GenericDisconnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Client>
    for v5_0::GenericAuth<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}

// Server sendable packets
impl SendableRole<Server> for v3_1_1::Pingresp {}

impl SendableRole<Server> for v5_0::Pingresp {}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Server>
    for v5_0::GenericDisconnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Server>
    for v5_0::GenericAuth<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}

// Any role sendable packets (both client and server)
impl SendableRole<Any> for v3_1_1::Pingreq {}
impl SendableRole<Any> for v3_1_1::Pingresp {}
impl SendableRole<Any> for v3_1_1::Disconnect {}

impl SendableRole<Any> for v5_0::Pingreq {}
impl SendableRole<Any> for v5_0::Pingresp {}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Any>
    for v5_0::GenericDisconnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}
impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableRole<Any>
    for v5_0::GenericAuth<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
}

// Generic packet implementations for roles
// Client sendable generic packets
impl<PacketIdType, const STRING_BUFFER_SIZE: usize, const PAYLOAD_BUFFER_SIZE: usize> SendableRole<Client> for v3_1_1::GenericPublish<PacketIdType, STRING_BUFFER_SIZE, PAYLOAD_BUFFER_SIZE> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v3_1_1::GenericPuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v3_1_1::GenericPubrec<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v3_1_1::GenericPubrel<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v3_1_1::GenericPubcomp<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType, const STRING_BUFFER_SIZE: usize> SendableRole<Client> for v3_1_1::GenericSubscribe<PacketIdType, STRING_BUFFER_SIZE> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType, const STRING_BUFFER_SIZE: usize> SendableRole<Client> for v3_1_1::GenericUnsubscribe<PacketIdType, STRING_BUFFER_SIZE> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}

impl<PacketIdType> SendableRole<Client> for v5_0::GenericPublish<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v5_0::GenericPuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v5_0::GenericPubrec<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v5_0::GenericPubrel<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v5_0::GenericPubcomp<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v5_0::GenericSubscribe<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v5_0::GenericUnsubscribe<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}

// Server sendable generic packets
impl<PacketIdType> SendableRole<Server> for v3_1_1::GenericPublish<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v3_1_1::GenericPuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v3_1_1::GenericPubrec<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v3_1_1::GenericPubrel<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v3_1_1::GenericPubcomp<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v3_1_1::GenericSuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v3_1_1::GenericUnsuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}

impl<PacketIdType> SendableRole<Server> for v5_0::GenericPublish<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v5_0::GenericPuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v5_0::GenericPubrec<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v5_0::GenericPubrel<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v5_0::GenericPubcomp<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v5_0::GenericSuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Server> for v5_0::GenericUnsuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}

// Any role sendable generic packets
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericPublish<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericPuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericPubrec<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericPubrel<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericPubcomp<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericSubscribe<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericSuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericUnsubscribe<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v3_1_1::GenericUnsuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}

impl<PacketIdType> SendableRole<Any> for v5_0::GenericPublish<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericPuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericPubrec<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericPubrel<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericPubcomp<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericSubscribe<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericSuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericUnsubscribe<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Any> for v5_0::GenericUnsuback<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}

// --- Blanket impl to support &T: SendableRole<Role> when T: SendableRole<Role> ---

impl<T, R> SendableRole<R> for &T where T: SendableRole<R> {}
