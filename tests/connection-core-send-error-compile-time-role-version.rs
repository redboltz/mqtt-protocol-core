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
mod common;
use common::mqtt;

/// Compile-time constraint verification tests
///
/// These tests verify that the type system correctly enforces MQTT role constraints.
/// We use static_assertions for reliable compile-time checking that is not affected
/// by changes in error messages, line numbers, or compiler versions.
#[test]
fn verify_sendable_constraints() {
    common::init_tracing();

    use static_assertions::{assert_impl_all, assert_not_impl_any};

    // Type alias for convenience
    #[allow(dead_code)]
    type PacketIdType = u16;

    // ===== CONNECTION PACKET CONSTRAINTS =====

    // ===== CONNECT packets =====
    // MQTT v3.1.1 Connect
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Connect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::Connect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Connect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 Connect
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::Connect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::Connect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::Connect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== CONNACK packets =====
    // MQTT v3.1.1 Connack
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::Connack:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Connack:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Connack:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 Connack
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::Connack:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::Connack:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::Connack:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== CONTROL PACKET CONSTRAINTS =====

    // ===== PINGREQ packets =====
    // MQTT v3.1.1 Pingreq
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Pingreq:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::Pingreq:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Pingreq:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 Pingreq
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::Pingreq:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::Pingreq:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::Pingreq:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== PINGRESP packets =====
    // MQTT v3.1.1 Pingresp
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::Pingresp:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Pingresp:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Pingresp:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 Pingresp
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::Pingresp:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::Pingresp:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::Pingresp:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== DISCONNECT packets =====
    // MQTT v3.1.1 Disconnect
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Disconnect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::Disconnect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::Disconnect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 Disconnect
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::Disconnect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::Disconnect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::Disconnect:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== AUTH packets (MQTT v5.0 only) =====
    // MQTT v5.0 Auth
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::Auth:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::Auth:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::Auth:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== PUBLISH-RELATED PACKET CONSTRAINTS =====

    // ===== PUBLISH packets =====
    // MQTT v3.1.1 GenericPublish
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPublish<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPublish<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPublish<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericPublish
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPublish<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPublish<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPublish<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== PUBACK packets =====
    // MQTT v3.1.1 GenericPuback
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericPuback
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== PUBREC packets =====
    // MQTT v3.1.1 GenericPubrec
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubrec<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubrec<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubrec<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericPubrec
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubrec<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubrec<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubrec<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== PUBREL packets =====
    // MQTT v3.1.1 GenericPubrel
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubrel<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubrel<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubrel<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericPubrel
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubrel<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubrel<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubrel<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== PUBCOMP packets =====
    // MQTT v3.1.1 GenericPubcomp
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubcomp<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubcomp<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericPubcomp<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericPubcomp
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubcomp<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubcomp<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericPubcomp<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== SUBSCRIBE-RELATED PACKET CONSTRAINTS =====

    // ===== SUBSCRIBE packets =====
    // MQTT v3.1.1 GenericSubscribe
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericSubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::GenericSubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericSubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericSubscribe
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericSubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::GenericSubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericSubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== SUBACK packets =====
    // MQTT v3.1.1 GenericSuback
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::GenericSuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericSuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericSuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericSuback
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::GenericSuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericSuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericSuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== UNSUBSCRIBE packets =====
    // MQTT v3.1.1 GenericUnsubscribe
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericUnsubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::GenericUnsubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericUnsubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericUnsubscribe
    // OK Client can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericUnsubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // NG Server cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::GenericUnsubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericUnsubscribe<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // ===== UNSUBACK packets =====
    // MQTT v3.1.1 GenericUnsuback
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v3_1_1::GenericUnsuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericUnsuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v3_1_1::GenericUnsuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // MQTT v5.0 GenericUnsuback
    // NG Client cannot send
    assert_not_impl_any!(
        mqtt::packet::v5_0::GenericUnsuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Client>
    );
    // OK Server can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericUnsuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Server>
    );
    // OK Any role can send
    assert_impl_all!(
        mqtt::packet::v5_0::GenericUnsuback<PacketIdType>:
        mqtt::connection::sendable_role::SendableRole<mqtt::role::Any>
    );

    // If we reach this point, the compile-time constraints are working correctly
    assert!(true, "Type constraints are enforced at compile time");
}
