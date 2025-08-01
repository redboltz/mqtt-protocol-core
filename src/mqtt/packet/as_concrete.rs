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
use crate::mqtt::packet::enum_packet::Packet;
use crate::mqtt::packet::v3_1_1;
use crate::mqtt::packet::v5_0;

/// Trait to extract a reference to a concrete type from an enum Packet, if applicable.
pub trait AsConcrete<T> {
    fn as_concrete(&self) -> Option<&T>;
}

/// Trait to extract an owned concrete type from an enum Packet by consuming it, if applicable.
pub trait IntoConcreteOwned<T> {
    fn into_concrete_owned(self) -> Option<T>;
}

// Identity implementation: concrete type is already T
impl<T> AsConcrete<T> for T {
    fn as_concrete(&self) -> Option<&T> {
        Some(self)
    }
}

// Identity implementation for owned version: concrete type is already T
impl<T> IntoConcreteOwned<T> for T {
    fn into_concrete_owned(self) -> Option<T> {
        Some(self)
    }
}

// v3.1.1 packets
impl AsConcrete<v3_1_1::Connect> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Connect> {
        match self {
            Packet::V3_1_1Connect(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Connack> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Connack> {
        match self {
            Packet::V3_1_1Connack(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Subscribe> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Subscribe> {
        match self {
            Packet::V3_1_1Subscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Suback> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Suback> {
        match self {
            Packet::V3_1_1Suback(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Unsubscribe> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Unsubscribe> {
        match self {
            Packet::V3_1_1Unsubscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Unsuback> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Unsuback> {
        match self {
            Packet::V3_1_1Unsuback(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Publish> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Publish> {
        match self {
            Packet::V3_1_1Publish(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Puback> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Puback> {
        match self {
            Packet::V3_1_1Puback(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Pubrec> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Pubrec> {
        match self {
            Packet::V3_1_1Pubrec(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Pubrel> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Pubrel> {
        match self {
            Packet::V3_1_1Pubrel(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Pubcomp> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Pubcomp> {
        match self {
            Packet::V3_1_1Pubcomp(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Disconnect> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Disconnect> {
        match self {
            Packet::V3_1_1Disconnect(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Pingreq> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Pingreq> {
        match self {
            Packet::V3_1_1Pingreq(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v3_1_1::Pingresp> for Packet {
    fn as_concrete(&self) -> Option<&v3_1_1::Pingresp> {
        match self {
            Packet::V3_1_1Pingresp(v) => Some(v),
            _ => None,
        }
    }
}

// v5.0 packets
impl AsConcrete<v5_0::Connect> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Connect> {
        match self {
            Packet::V5_0Connect(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Connack> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Connack> {
        match self {
            Packet::V5_0Connack(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Subscribe> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Subscribe> {
        match self {
            Packet::V5_0Subscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Suback> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Suback> {
        match self {
            Packet::V5_0Suback(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Unsubscribe> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Unsubscribe> {
        match self {
            Packet::V5_0Unsubscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Unsuback> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Unsuback> {
        match self {
            Packet::V5_0Unsuback(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Publish> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Publish> {
        match self {
            Packet::V5_0Publish(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Puback> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Puback> {
        match self {
            Packet::V5_0Puback(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Pubrec> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Pubrec> {
        match self {
            Packet::V5_0Pubrec(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Pubrel> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Pubrel> {
        match self {
            Packet::V5_0Pubrel(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Pubcomp> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Pubcomp> {
        match self {
            Packet::V5_0Pubcomp(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Disconnect> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Disconnect> {
        match self {
            Packet::V5_0Disconnect(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Pingreq> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Pingreq> {
        match self {
            Packet::V5_0Pingreq(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Pingresp> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Pingresp> {
        match self {
            Packet::V5_0Pingresp(v) => Some(v),
            _ => None,
        }
    }
}

impl AsConcrete<v5_0::Auth> for Packet {
    fn as_concrete(&self) -> Option<&v5_0::Auth> {
        match self {
            Packet::V5_0Auth(v) => Some(v),
            _ => None,
        }
    }
}

// v3.1.1 packets - move versions
impl IntoConcreteOwned<v3_1_1::Connect> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Connect> {
        match self {
            Packet::V3_1_1Connect(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Connack> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Connack> {
        match self {
            Packet::V3_1_1Connack(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Subscribe> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Subscribe> {
        match self {
            Packet::V3_1_1Subscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Suback> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Suback> {
        match self {
            Packet::V3_1_1Suback(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Unsubscribe> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Unsubscribe> {
        match self {
            Packet::V3_1_1Unsubscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Unsuback> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Unsuback> {
        match self {
            Packet::V3_1_1Unsuback(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Publish> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Publish> {
        match self {
            Packet::V3_1_1Publish(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Puback> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Puback> {
        match self {
            Packet::V3_1_1Puback(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Pubrec> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Pubrec> {
        match self {
            Packet::V3_1_1Pubrec(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Pubrel> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Pubrel> {
        match self {
            Packet::V3_1_1Pubrel(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Pubcomp> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Pubcomp> {
        match self {
            Packet::V3_1_1Pubcomp(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Disconnect> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Disconnect> {
        match self {
            Packet::V3_1_1Disconnect(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Pingreq> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Pingreq> {
        match self {
            Packet::V3_1_1Pingreq(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v3_1_1::Pingresp> for Packet {
    fn into_concrete_owned(self) -> Option<v3_1_1::Pingresp> {
        match self {
            Packet::V3_1_1Pingresp(v) => Some(v),
            _ => None,
        }
    }
}

// v5.0 packets - move versions
impl IntoConcreteOwned<v5_0::Connect> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Connect> {
        match self {
            Packet::V5_0Connect(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Connack> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Connack> {
        match self {
            Packet::V5_0Connack(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Subscribe> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Subscribe> {
        match self {
            Packet::V5_0Subscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Suback> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Suback> {
        match self {
            Packet::V5_0Suback(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Unsubscribe> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Unsubscribe> {
        match self {
            Packet::V5_0Unsubscribe(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Unsuback> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Unsuback> {
        match self {
            Packet::V5_0Unsuback(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Publish> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Publish> {
        match self {
            Packet::V5_0Publish(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Puback> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Puback> {
        match self {
            Packet::V5_0Puback(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Pubrec> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Pubrec> {
        match self {
            Packet::V5_0Pubrec(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Pubrel> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Pubrel> {
        match self {
            Packet::V5_0Pubrel(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Pubcomp> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Pubcomp> {
        match self {
            Packet::V5_0Pubcomp(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Disconnect> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Disconnect> {
        match self {
            Packet::V5_0Disconnect(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Pingreq> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Pingreq> {
        match self {
            Packet::V5_0Pingreq(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Pingresp> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Pingresp> {
        match self {
            Packet::V5_0Pingresp(v) => Some(v),
            _ => None,
        }
    }
}

impl IntoConcreteOwned<v5_0::Auth> for Packet {
    fn into_concrete_owned(self) -> Option<v5_0::Auth> {
        match self {
            Packet::V5_0Auth(v) => Some(v),
            _ => None,
        }
    }
}
