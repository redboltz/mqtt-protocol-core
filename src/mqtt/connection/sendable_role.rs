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
use crate::mqtt::connection::role::*;
use crate::mqtt::packet::*;

/// Role-specific trait to validate that a packet can be sent under a given Role.
pub trait SendableRole<Role> {}

// --- Role-specific implementations ---

impl SendableRole<Client> for v5_0::Connect {}
impl SendableRole<Client> for v3_1_1::Connect {}

impl SendableRole<Server> for v5_0::Connack {}
impl SendableRole<Server> for v3_1_1::Connack {}

impl SendableRole<Any> for v5_0::Connect {}
impl SendableRole<Any> for v5_0::Connack {}
impl SendableRole<Any> for v3_1_1::Connect {}
impl SendableRole<Any> for v3_1_1::Connack {}

// Client sendable packets
impl SendableRole<Client> for v3_1_1::Pingreq {}
impl SendableRole<Client> for v3_1_1::Disconnect {}

impl SendableRole<Client> for v5_0::Pingreq {}
impl SendableRole<Client> for v5_0::Disconnect {}
impl SendableRole<Client> for v5_0::Auth {}

// Server sendable packets
impl SendableRole<Server> for v3_1_1::Pingresp {}

impl SendableRole<Server> for v5_0::Pingresp {}
impl SendableRole<Server> for v5_0::Disconnect {}
impl SendableRole<Server> for v5_0::Auth {}

// Any role sendable packets (both client and server)
impl SendableRole<Any> for v3_1_1::Pingreq {}
impl SendableRole<Any> for v3_1_1::Pingresp {}
impl SendableRole<Any> for v3_1_1::Disconnect {}

impl SendableRole<Any> for v5_0::Pingreq {}
impl SendableRole<Any> for v5_0::Pingresp {}
impl SendableRole<Any> for v5_0::Disconnect {}
impl SendableRole<Any> for v5_0::Auth {}

// Generic packet implementations for roles
// Client sendable generic packets
impl<PacketIdType> SendableRole<Client> for v3_1_1::GenericPublish<PacketIdType> where
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
impl<PacketIdType> SendableRole<Client> for v3_1_1::GenericSubscribe<PacketIdType> where
    PacketIdType: crate::mqtt::packet::IsPacketId
{
}
impl<PacketIdType> SendableRole<Client> for v3_1_1::GenericUnsubscribe<PacketIdType> where
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
