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
use crate::mqtt::packet::v3_1_1;
use crate::mqtt::packet::v5_0;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::packet::PacketType;
use crate::mqtt::Version;
use alloc::vec::Vec;
use enum_dispatch::enum_dispatch;
use serde::Serialize;
#[cfg(feature = "std")]
use std::io::IoSlice;

#[enum_dispatch]
pub trait GenericPacketTrait {
    fn size(&self) -> usize;

    /// Create a continuous buffer containing the complete packet data
    ///
    /// Returns a vector containing all packet bytes in a single continuous buffer.
    /// This method is compatible with no-std environments.
    ///
    /// The returned buffer contains the complete packet serialized according
    /// to the MQTT protocol specification.
    ///
    /// # Returns
    ///
    /// A vector containing the complete packet data
    fn to_continuous_buffer(&self) -> Vec<u8>;

    #[cfg(feature = "std")]
    fn to_buffers(&self) -> Vec<IoSlice<'_>>;
}

#[enum_dispatch]
pub trait GenericPacketDisplay {
    fn fmt_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
    fn fmt_display(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result;
}

#[enum_dispatch(GenericPacketTrait, GenericPacketDisplay)]
#[derive(Serialize, Clone, PartialEq, Eq)]
pub enum GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    V3_1_1Connect(v3_1_1::Connect),
    V3_1_1Connack(v3_1_1::Connack),
    V3_1_1Subscribe(v3_1_1::GenericSubscribe<PacketIdType>),
    V3_1_1Suback(v3_1_1::GenericSuback<PacketIdType>),
    V3_1_1Unsubscribe(v3_1_1::GenericUnsubscribe<PacketIdType>),
    V3_1_1Unsuback(v3_1_1::GenericUnsuback<PacketIdType>),
    V3_1_1Publish(v3_1_1::GenericPublish<PacketIdType>),
    V3_1_1Puback(v3_1_1::GenericPuback<PacketIdType>),
    V3_1_1Pubrec(v3_1_1::GenericPubrec<PacketIdType>),
    V3_1_1Pubrel(v3_1_1::GenericPubrel<PacketIdType>),
    V3_1_1Pubcomp(v3_1_1::GenericPubcomp<PacketIdType>),
    V3_1_1Disconnect(v3_1_1::Disconnect),
    V3_1_1Pingreq(v3_1_1::Pingreq),
    V3_1_1Pingresp(v3_1_1::Pingresp),

    V5_0Connect(v5_0::Connect),
    V5_0Connack(v5_0::Connack),
    V5_0Subscribe(v5_0::GenericSubscribe<PacketIdType>),
    V5_0Suback(v5_0::GenericSuback<PacketIdType>),
    V5_0Unsubscribe(v5_0::GenericUnsubscribe<PacketIdType>),
    V5_0Unsuback(v5_0::GenericUnsuback<PacketIdType>),
    V5_0Publish(v5_0::GenericPublish<PacketIdType>),
    V5_0Puback(v5_0::GenericPuback<PacketIdType>),
    V5_0Pubrec(v5_0::GenericPubrec<PacketIdType>),
    V5_0Pubrel(v5_0::GenericPubrel<PacketIdType>),
    V5_0Pubcomp(v5_0::GenericPubcomp<PacketIdType>),
    V5_0Disconnect(v5_0::Disconnect),
    V5_0Pingreq(v5_0::Pingreq),
    V5_0Pingresp(v5_0::Pingresp),
    V5_0Auth(v5_0::Auth),
}

// Type alias for commonly used u16 PacketIdType
pub type Packet = GenericPacket<u16>;

impl<PacketIdType> core::fmt::Debug for GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_debug(f)
    }
}

impl<PacketIdType> core::fmt::Display for GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.fmt_display(f)
    }
}

impl<PacketIdType> GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    /// Get the packet type
    pub fn packet_type(&self) -> PacketType {
        match self {
            GenericPacket::V3_1_1Connect(_) => PacketType::Connect,
            GenericPacket::V3_1_1Connack(_) => PacketType::Connack,
            GenericPacket::V3_1_1Subscribe(_) => PacketType::Subscribe,
            GenericPacket::V3_1_1Suback(_) => PacketType::Suback,
            GenericPacket::V3_1_1Unsubscribe(_) => PacketType::Unsubscribe,
            GenericPacket::V3_1_1Unsuback(_) => PacketType::Unsuback,
            GenericPacket::V3_1_1Publish(_) => PacketType::Publish,
            GenericPacket::V3_1_1Puback(_) => PacketType::Puback,
            GenericPacket::V3_1_1Pubrec(_) => PacketType::Pubrec,
            GenericPacket::V3_1_1Pubrel(_) => PacketType::Pubrel,
            GenericPacket::V3_1_1Pubcomp(_) => PacketType::Pubcomp,
            GenericPacket::V3_1_1Disconnect(_) => PacketType::Disconnect,
            GenericPacket::V3_1_1Pingreq(_) => PacketType::Pingreq,
            GenericPacket::V3_1_1Pingresp(_) => PacketType::Pingresp,

            GenericPacket::V5_0Connect(_) => PacketType::Connect,
            GenericPacket::V5_0Connack(_) => PacketType::Connack,
            GenericPacket::V5_0Subscribe(_) => PacketType::Subscribe,
            GenericPacket::V5_0Suback(_) => PacketType::Suback,
            GenericPacket::V5_0Unsubscribe(_) => PacketType::Unsubscribe,
            GenericPacket::V5_0Unsuback(_) => PacketType::Unsuback,
            GenericPacket::V5_0Publish(_) => PacketType::Publish,
            GenericPacket::V5_0Puback(_) => PacketType::Puback,
            GenericPacket::V5_0Pubrec(_) => PacketType::Pubrec,
            GenericPacket::V5_0Pubrel(_) => PacketType::Pubrel,
            GenericPacket::V5_0Pubcomp(_) => PacketType::Pubcomp,
            GenericPacket::V5_0Disconnect(_) => PacketType::Disconnect,
            GenericPacket::V5_0Pingreq(_) => PacketType::Pingreq,
            GenericPacket::V5_0Pingresp(_) => PacketType::Pingresp,
            GenericPacket::V5_0Auth(_) => PacketType::Auth,
        }
    }

    /// Get the MQTT protocol version of this packet
    pub fn protocol_version(&self) -> Version {
        match self {
            GenericPacket::V3_1_1Connect(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Connack(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Subscribe(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Suback(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Unsubscribe(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Unsuback(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Publish(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Puback(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Pubrec(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Pubrel(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Pubcomp(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Disconnect(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Pingreq(_) => Version::V3_1_1,
            GenericPacket::V3_1_1Pingresp(_) => Version::V3_1_1,

            GenericPacket::V5_0Connect(_) => Version::V5_0,
            GenericPacket::V5_0Connack(_) => Version::V5_0,
            GenericPacket::V5_0Subscribe(_) => Version::V5_0,
            GenericPacket::V5_0Suback(_) => Version::V5_0,
            GenericPacket::V5_0Unsubscribe(_) => Version::V5_0,
            GenericPacket::V5_0Unsuback(_) => Version::V5_0,
            GenericPacket::V5_0Publish(_) => Version::V5_0,
            GenericPacket::V5_0Puback(_) => Version::V5_0,
            GenericPacket::V5_0Pubrec(_) => Version::V5_0,
            GenericPacket::V5_0Pubrel(_) => Version::V5_0,
            GenericPacket::V5_0Pubcomp(_) => Version::V5_0,
            GenericPacket::V5_0Disconnect(_) => Version::V5_0,
            GenericPacket::V5_0Pingreq(_) => Version::V5_0,
            GenericPacket::V5_0Pingresp(_) => Version::V5_0,
            GenericPacket::V5_0Auth(_) => Version::V5_0,
        }
    }
}
