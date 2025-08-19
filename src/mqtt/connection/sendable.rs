use crate::mqtt::common::tracing::trace;
use crate::mqtt::connection::role;
use crate::mqtt::connection::role::RoleType;
use crate::mqtt::connection::sendable_role::SendableRole;
use crate::mqtt::connection::sendable_version::SendableVersion;
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
use crate::mqtt::connection::GenericConnection;
use crate::mqtt::connection::GenericEvent;
use crate::mqtt::packet::kind::PacketKind;
use crate::mqtt::packet::GenericPacket;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::result_code::MqttError;
use alloc::vec::Vec;
use core::fmt::Debug;

/// Core trait for sendable packets
pub trait Sendable<Role, PacketIdType>: PacketKind
where
    Role: RoleType,
    PacketIdType: IsPacketId,
{
    fn dispatch_send(
        self,
        connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>>;
}

/// Trait for connection send behavior
pub trait SendBehavior<Role, PacketIdType>
where
    Role: RoleType,
    PacketIdType: IsPacketId,
{
    fn send<T>(&mut self, packet: T) -> Vec<GenericEvent<PacketIdType>>
    where
        T: Sendable<Role, PacketIdType>;
}

/// Helper trait for type-specific send operations
pub trait SendableHelper<Role, PacketIdType>: PacketKind + Sized
where
    Role: RoleType,
    PacketIdType: IsPacketId,
{
    // v3.1.1 methods
    fn send_connect_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_connect_v3_1_1 not implemented for this type")
    }

    fn send_connack_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_connack_v3_1_1 not implemented for this type")
    }

    fn send_publish_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_publish_v3_1_1 not implemented for this type")
    }

    fn send_puback_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_puback_v3_1_1 not implemented for this type")
    }

    fn send_pubrec_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pubrec_v3_1_1 not implemented for this type")
    }

    fn send_pubrel_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pubrel_v3_1_1 not implemented for this type")
    }

    fn send_pubcomp_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pubcomp_v3_1_1 not implemented for this type")
    }

    fn send_subscribe_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_subscribe_v3_1_1 not implemented for this type")
    }

    fn send_suback_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_suback_v3_1_1 not implemented for this type")
    }

    fn send_unsubscribe_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_unsubscribe_v3_1_1 not implemented for this type")
    }

    fn send_unsuback_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_unsuback_v3_1_1 not implemented for this type")
    }

    fn send_pingreq_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pingreq_v3_1_1 not implemented for this type")
    }

    fn send_pingresp_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pingresp_v3_1_1 not implemented for this type")
    }

    fn send_disconnect_v3_1_1(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_disconnect_v3_1_1 not implemented for this type")
    }

    // v5.0 methods
    fn send_connect_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_connect_v5_0 not implemented for this type")
    }

    fn send_connack_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_connack_v5_0 not implemented for this type")
    }

    fn send_publish_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_publish_v5_0 not implemented for this type")
    }

    fn send_puback_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_puback_v5_0 not implemented for this type")
    }

    fn send_pubrec_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pubrec_v5_0 not implemented for this type")
    }

    fn send_pubrel_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pubrel_v5_0 not implemented for this type")
    }

    fn send_pubcomp_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pubcomp_v5_0 not implemented for this type")
    }

    fn send_subscribe_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_subscribe_v5_0 not implemented for this type")
    }

    fn send_suback_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_suback_v5_0 not implemented for this type")
    }

    fn send_unsubscribe_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_unsubscribe_v5_0 not implemented for this type")
    }

    fn send_unsuback_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_unsuback_v5_0 not implemented for this type")
    }

    fn send_pingreq_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pingreq_v5_0 not implemented for this type")
    }

    fn send_pingresp_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_pingresp_v5_0 not implemented for this type")
    }

    fn send_disconnect_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_disconnect_v5_0 not implemented for this type")
    }

    fn send_auth_v5_0(
        self,
        _connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        unreachable!("send_auth_v5_0 not implemented for this type")
    }
}

// Generic implementation for specific packet types with compile-time dispatch
impl<Role, PacketIdType, T> Sendable<Role, PacketIdType> for T
where
    Role: role::RoleType,
    PacketIdType: IsPacketId,
    T: SendableRole<Role>
        + SendableVersion
        + core::fmt::Display
        + Debug
        + PacketKind
        + SendableHelper<Role, PacketIdType>,
{
    fn dispatch_send(
        self,
        connection: &mut GenericConnection<Role, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        // Version check first
        if !T::check(&connection.get_protocol_version()) {
            return vec![GenericEvent::NotifyError(MqttError::VersionMismatch)];
        }

        trace!("Static dispatch sent: {}", self);
        // Compile-time dispatch based on packet type and version
        // The compiler will eliminate unused branches for specific types
        if T::IS_CONNECT {
            if T::IS_V3_1_1 {
                self.send_connect_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_connect_v5_0(connection)
            } else {
                unreachable!("Invalid version for CONNECT packet")
            }
        } else if T::IS_CONNACK {
            if T::IS_V3_1_1 {
                self.send_connack_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_connack_v5_0(connection)
            } else {
                unreachable!("Invalid version for CONNACK packet")
            }
        } else if T::IS_PUBLISH {
            if T::IS_V3_1_1 {
                self.send_publish_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_publish_v5_0(connection)
            } else {
                unreachable!("Invalid version for PUBLISH packet")
            }
        } else if T::IS_PUBACK {
            if T::IS_V3_1_1 {
                self.send_puback_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_puback_v5_0(connection)
            } else {
                unreachable!("Invalid version for PUBACK packet")
            }
        } else if T::IS_PUBREC {
            if T::IS_V3_1_1 {
                self.send_pubrec_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_pubrec_v5_0(connection)
            } else {
                unreachable!("Invalid version for PUBREC packet")
            }
        } else if T::IS_PUBREL {
            if T::IS_V3_1_1 {
                self.send_pubrel_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_pubrel_v5_0(connection)
            } else {
                unreachable!("Invalid version for PUBREL packet")
            }
        } else if T::IS_PUBCOMP {
            if T::IS_V3_1_1 {
                self.send_pubcomp_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_pubcomp_v5_0(connection)
            } else {
                unreachable!("Invalid version for PUBCOMP packet")
            }
        } else if T::IS_SUBSCRIBE {
            if T::IS_V3_1_1 {
                self.send_subscribe_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_subscribe_v5_0(connection)
            } else {
                unreachable!("Invalid version for SUBSCRIBE packet")
            }
        } else if T::IS_SUBACK {
            if T::IS_V3_1_1 {
                self.send_suback_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_suback_v5_0(connection)
            } else {
                unreachable!("Invalid version for SUBACK packet")
            }
        } else if T::IS_UNSUBSCRIBE {
            if T::IS_V3_1_1 {
                self.send_unsubscribe_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_unsubscribe_v5_0(connection)
            } else {
                unreachable!("Invalid version for UNSUBSCRIBE packet")
            }
        } else if T::IS_UNSUBACK {
            if T::IS_V3_1_1 {
                self.send_unsuback_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_unsuback_v5_0(connection)
            } else {
                unreachable!("Invalid version for UNSUBACK packet")
            }
        } else if T::IS_PINGREQ {
            if T::IS_V3_1_1 {
                self.send_pingreq_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_pingreq_v5_0(connection)
            } else {
                unreachable!("Invalid version for PINGREQ packet")
            }
        } else if T::IS_PINGRESP {
            if T::IS_V3_1_1 {
                self.send_pingresp_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_pingresp_v5_0(connection)
            } else {
                unreachable!("Invalid version for PINGRESP packet")
            }
        } else if T::IS_DISCONNECT {
            if T::IS_V3_1_1 {
                self.send_disconnect_v3_1_1(connection)
            } else if T::IS_V5_0 {
                self.send_disconnect_v5_0(connection)
            } else {
                unreachable!("Invalid version for DISCONNECT packet")
            }
        } else if T::IS_AUTH {
            if T::IS_V5_0 {
                self.send_auth_v5_0(connection)
            } else {
                unreachable!("AUTH packet is only valid for v5.0")
            }
        } else {
            unreachable!("Unknown packet type")
        }
    }
}

// Sendable implementation for GenericPacket (runtime dispatch)
impl<PacketIdType> Sendable<role::Client, PacketIdType> for GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    fn dispatch_send(
        self,
        connection: &mut GenericConnection<role::Client, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        connection.send(self)
    }
}

impl<PacketIdType> Sendable<role::Server, PacketIdType> for GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    fn dispatch_send(
        self,
        connection: &mut GenericConnection<role::Server, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        connection.send(self)
    }
}

impl<PacketIdType> Sendable<role::Any, PacketIdType> for GenericPacket<PacketIdType>
where
    PacketIdType: IsPacketId,
{
    fn dispatch_send(
        self,
        connection: &mut GenericConnection<role::Any, PacketIdType>,
    ) -> Vec<GenericEvent<PacketIdType>> {
        connection.send(self)
    }
}
