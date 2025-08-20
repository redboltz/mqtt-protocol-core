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

use crate::mqtt::connection::version::*;
use crate::mqtt::packet::*;

/// Trait to check if a packet is valid for a specific MQTT version.
pub trait SendableVersion {
    fn check(version: &Version) -> bool;
}

// ---- Implementation for concrete packet types ----

impl<const STRING_BUFFER_SIZE: usize> SendableVersion
    for v3_1_1::GenericConnect<STRING_BUFFER_SIZE>
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl SendableVersion for v3_1_1::Connack {
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableVersion
    for v5_0::GenericConnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableVersion
    for v5_0::GenericConnack<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

// v3.1.1 packet implementations

impl SendableVersion for v3_1_1::Pingreq {
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl SendableVersion for v3_1_1::Pingresp {
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl SendableVersion for v3_1_1::Disconnect {
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

// v5.0 packet implementations

impl SendableVersion for v5_0::Pingreq {
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl SendableVersion for v5_0::Pingresp {
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableVersion
    for v5_0::GenericDisconnect<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<const STRING_BUFFER_SIZE: usize, const BINARY_BUFFER_SIZE: usize> SendableVersion
    for v5_0::GenericAuth<STRING_BUFFER_SIZE, BINARY_BUFFER_SIZE>
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

// Generic packet implementations for versions
// v3.1.1 Generic packet implementations
impl<PacketIdType> SendableVersion for v3_1_1::GenericPublish<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericPuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericPubrec<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericPubrel<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericPubcomp<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericSubscribe<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericSuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericUnsubscribe<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

impl<PacketIdType> SendableVersion for v3_1_1::GenericUnsuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V3_1_1
    }
}

// v5.0 Generic packet implementations
impl<PacketIdType> SendableVersion for v5_0::GenericPublish<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericPuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericPubrec<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericPubrel<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericPubcomp<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericSubscribe<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericSuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericUnsubscribe<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

impl<PacketIdType> SendableVersion for v5_0::GenericUnsuback<PacketIdType>
where
    PacketIdType: crate::mqtt::packet::IsPacketId,
{
    fn check(version: &Version) -> bool {
        *version == Version::V5_0
    }
}

// ---- Blanket impl to allow &T: SendableVersion when T: SendableVersion ----

impl<T: SendableVersion> SendableVersion for &T {
    fn check(version: &Version) -> bool {
        T::check(version)
    }
}
