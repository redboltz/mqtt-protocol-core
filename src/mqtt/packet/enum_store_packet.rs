use crate::mqtt::packet::PacketType;
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
use crate::mqtt::packet::enum_packet::{GenericPacket, GenericPacketDisplay, GenericPacketTrait};
use crate::mqtt::packet::qos::Qos;
use crate::mqtt::packet::v3_1_1;
use crate::mqtt::packet::v5_0;
use crate::mqtt::result_code::MqttError;
use crate::mqtt::packet::IsPacketId;
use serde::Serialize;
use std::io::IoSlice;

/// ResponsePacket denotes the type of the response matching a stored packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponsePacket {
    V3_1_1Puback,
    V3_1_1Pubrec,
    V3_1_1Pubcomp,
    V5_0Puback,
    V5_0Pubrec,
    V5_0Pubcomp,
}

#[derive(Serialize, Clone, PartialEq, Eq)]
pub enum GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    V3_1_1Publish(v3_1_1::GenericPublish<PacketIdType>),
    V3_1_1Pubrel(v3_1_1::GenericPubrel<PacketIdType>),
    V5_0Publish(v5_0::GenericPublish<PacketIdType>),
    V5_0Pubrel(v5_0::GenericPubrel<PacketIdType>),
}

// Type alias for commonly used u16 PacketIdType
pub type StorePacket = GenericStorePacket<u16>;

impl<PacketIdType> std::fmt::Debug for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_debug(f)
    }
}

impl<PacketIdType> std::fmt::Display for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_display(f)
    }
}

impl<PacketIdType> GenericPacketTrait for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn size(&self) -> usize {
        match self {
            GenericStorePacket::V3_1_1Publish(p) => p.size(),
            GenericStorePacket::V3_1_1Pubrel(p) => p.size(),
            GenericStorePacket::V5_0Publish(p) => p.size(),
            GenericStorePacket::V5_0Pubrel(p) => p.size(),
        }
    }

    fn to_buffers(&self) -> Vec<IoSlice<'_>> {
        match self {
            GenericStorePacket::V3_1_1Publish(p) => p.to_buffers(),
            GenericStorePacket::V3_1_1Pubrel(p) => p.to_buffers(),
            GenericStorePacket::V5_0Publish(p) => p.to_buffers(),
            GenericStorePacket::V5_0Pubrel(p) => p.to_buffers(),
        }
    }
}

impl<PacketIdType> GenericPacketDisplay for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn fmt_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericStorePacket::V3_1_1Publish(p) => p.fmt_debug(f),
            GenericStorePacket::V3_1_1Pubrel(p) => p.fmt_debug(f),
            GenericStorePacket::V5_0Publish(p) => p.fmt_debug(f),
            GenericStorePacket::V5_0Pubrel(p) => p.fmt_debug(f),
        }
    }

    fn fmt_display(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericStorePacket::V3_1_1Publish(p) => p.fmt_display(f),
            GenericStorePacket::V3_1_1Pubrel(p) => p.fmt_display(f),
            GenericStorePacket::V5_0Publish(p) => p.fmt_display(f),
            GenericStorePacket::V5_0Pubrel(p) => p.fmt_display(f),
        }
    }
}

impl<PacketIdType> GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    /// Get the packet type
    pub fn packet_type(&self) -> PacketType {
        match self {
            GenericStorePacket::V3_1_1Publish(_) => PacketType::Publish,
            GenericStorePacket::V3_1_1Pubrel(_) => PacketType::Pubrel,
            GenericStorePacket::V5_0Publish(_) => PacketType::Publish,
            GenericStorePacket::V5_0Pubrel(_) => PacketType::Pubrel,
        }
    }

    /// Get the packet ID of this store packet
    pub fn packet_id(&self) -> PacketIdType {
        match self {
            GenericStorePacket::V3_1_1Publish(p) => p.packet_id().unwrap(),
            GenericStorePacket::V3_1_1Pubrel(p) => p.packet_id(),
            GenericStorePacket::V5_0Publish(p) => p.packet_id().unwrap(),
            GenericStorePacket::V5_0Pubrel(p) => p.packet_id(),
        }
    }

    /// Get the response packet type for this store packet
    pub fn response_packet(&self) -> ResponsePacket {
        match self {
            GenericStorePacket::V3_1_1Publish(p) => match p.qos() {
                Qos::AtLeastOnce => ResponsePacket::V3_1_1Puback,
                Qos::ExactlyOnce => ResponsePacket::V3_1_1Pubrec,
                _ => panic!("QoS 0 packets should not be stored"),
            },
            GenericStorePacket::V3_1_1Pubrel(_) => ResponsePacket::V3_1_1Pubcomp,
            GenericStorePacket::V5_0Publish(p) => match p.qos() {
                Qos::AtLeastOnce => ResponsePacket::V5_0Puback,
                Qos::ExactlyOnce => ResponsePacket::V5_0Pubrec,
                _ => panic!("QoS 0 packets should not be stored"),
            },
            GenericStorePacket::V5_0Pubrel(_) => ResponsePacket::V5_0Pubcomp,
        }
    }
}

// TryFrom implementations for all packet types (unified API)
impl<PacketIdType> TryFrom<v3_1_1::GenericPublish<PacketIdType>>
    for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    type Error = MqttError;

    fn try_from(publish: v3_1_1::GenericPublish<PacketIdType>) -> Result<Self, Self::Error> {
        match publish.qos() {
            Qos::AtMostOnce => Err(MqttError::InvalidQos),
            _ => Ok(GenericStorePacket::V3_1_1Publish(publish)),
        }
    }
}

impl<PacketIdType> TryFrom<v5_0::GenericPublish<PacketIdType>> for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    type Error = MqttError;

    fn try_from(publish: v5_0::GenericPublish<PacketIdType>) -> Result<Self, Self::Error> {
        match publish.qos() {
            Qos::AtMostOnce => Err(MqttError::InvalidQos),
            _ => Ok(GenericStorePacket::V5_0Publish(publish)),
        }
    }
}

impl<PacketIdType> TryFrom<v3_1_1::GenericPubrel<PacketIdType>> for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    type Error = MqttError;

    fn try_from(pubrel: v3_1_1::GenericPubrel<PacketIdType>) -> Result<Self, Self::Error> {
        Ok(GenericStorePacket::V3_1_1Pubrel(pubrel))
    }
}

impl<PacketIdType> TryFrom<v5_0::GenericPubrel<PacketIdType>> for GenericStorePacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    type Error = MqttError;

    fn try_from(pubrel: v5_0::GenericPubrel<PacketIdType>) -> Result<Self, Self::Error> {
        Ok(GenericStorePacket::V5_0Pubrel(pubrel))
    }
}

// From implementations for GenericStorePacket to GenericPacket conversion
impl<PacketIdType> From<GenericStorePacket<PacketIdType>> for GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId + Serialize,
{
    fn from(store_packet: GenericStorePacket<PacketIdType>) -> Self {
        match store_packet {
            GenericStorePacket::V3_1_1Publish(p) => GenericPacket::V3_1_1Publish(p),
            GenericStorePacket::V3_1_1Pubrel(p) => GenericPacket::V3_1_1Pubrel(p),
            GenericStorePacket::V5_0Publish(p) => GenericPacket::V5_0Publish(p),
            GenericStorePacket::V5_0Pubrel(p) => GenericPacket::V5_0Pubrel(p),
        }
    }
}
