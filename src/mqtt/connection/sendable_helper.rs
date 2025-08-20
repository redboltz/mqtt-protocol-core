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

use crate::mqtt::connection::role;
use crate::mqtt::connection::sendable::SendableHelper;
use crate::mqtt::connection::GenericConnection;
use crate::mqtt::connection::GenericEvent;
use crate::mqtt::packet::IsPacketId;
use crate::mqtt::packet::{v3_1_1, v5_0};
use alloc::vec::Vec;

macro_rules! impl_sendable_helper {
    ($role:ty, $packet_type:ty, $method:ident, $process_method:ident) => {
        impl<PacketIdType> SendableHelper<$role, PacketIdType> for $packet_type
        where
            PacketIdType: IsPacketId,
        {
            fn $method(
                self,
                connection: &mut GenericConnection<$role, PacketIdType>,
            ) -> Vec<GenericEvent<PacketIdType>> {
                connection.$process_method(self)
            }
        }
    };
}

#[rustfmt::skip]
mod unformatted {
    use super::*;
    // Client
    impl_sendable_helper!(role::Client, v3_1_1::GenericConnect<32>,               send_connect_v3_1_1,         process_send_v3_1_1_connect);
    impl_sendable_helper!(role::Client, v5_0::GenericConnect<32, 32>,             send_connect_v5_0,           process_send_v5_0_connect);
    impl_sendable_helper!(role::Client, v3_1_1::GenericPublish<PacketIdType>,      send_publish_v3_1_1,         process_send_v3_1_1_publish);
    impl_sendable_helper!(role::Client, v5_0::GenericPublish<PacketIdType>,        send_publish_v5_0,           process_send_v5_0_publish);
    impl_sendable_helper!(role::Client, v3_1_1::GenericPuback<PacketIdType>,       send_puback_v3_1_1,          process_send_v3_1_1_puback);
    impl_sendable_helper!(role::Client, v5_0::GenericPuback<PacketIdType>,         send_puback_v5_0,            process_send_v5_0_puback);
    impl_sendable_helper!(role::Client, v3_1_1::GenericPubrec<PacketIdType>,       send_pubrec_v3_1_1,          process_send_v3_1_1_pubrec);
    impl_sendable_helper!(role::Client, v5_0::GenericPubrec<PacketIdType>,         send_pubrec_v5_0,            process_send_v5_0_pubrec);
    impl_sendable_helper!(role::Client, v3_1_1::GenericPubrel<PacketIdType>,       send_pubrel_v3_1_1,          process_send_v3_1_1_pubrel);
    impl_sendable_helper!(role::Client, v5_0::GenericPubrel<PacketIdType>,         send_pubrel_v5_0,            process_send_v5_0_pubrel);
    impl_sendable_helper!(role::Client, v3_1_1::GenericPubcomp<PacketIdType>,      send_pubcomp_v3_1_1,         process_send_v3_1_1_pubcomp);
    impl_sendable_helper!(role::Client, v5_0::GenericPubcomp<PacketIdType>,        send_pubcomp_v5_0,           process_send_v5_0_pubcomp);
    impl_sendable_helper!(role::Client, v3_1_1::GenericSubscribe<PacketIdType>,    send_subscribe_v3_1_1,       process_send_v3_1_1_subscribe);
    impl_sendable_helper!(role::Client, v5_0::GenericSubscribe<PacketIdType>,      send_subscribe_v5_0,         process_send_v5_0_subscribe);
    impl_sendable_helper!(role::Client, v3_1_1::GenericUnsubscribe<PacketIdType>,  send_unsubscribe_v3_1_1,     process_send_v3_1_1_unsubscribe);
    impl_sendable_helper!(role::Client, v5_0::GenericUnsubscribe<PacketIdType>,    send_unsubscribe_v5_0,       process_send_v5_0_unsubscribe);
    impl_sendable_helper!(role::Client, v3_1_1::Pingreq,                           send_pingreq_v3_1_1,         process_send_v3_1_1_pingreq);
    impl_sendable_helper!(role::Client, v5_0::Pingreq,                             send_pingreq_v5_0,           process_send_v5_0_pingreq);
    impl_sendable_helper!(role::Client, v3_1_1::Disconnect,                        send_disconnect_v3_1_1,      process_send_v3_1_1_disconnect);
    impl_sendable_helper!(role::Client, v5_0::GenericDisconnect<32, 32>,          send_disconnect_v5_0,        process_send_v5_0_disconnect);
    impl_sendable_helper!(role::Client, v5_0::GenericAuth<32, 32>,                send_auth_v5_0,              process_send_v5_0_auth);

    // Server
    impl_sendable_helper!(role::Server, v3_1_1::Connack,                           send_connack_v3_1_1,         process_send_v3_1_1_connack);
    impl_sendable_helper!(role::Server, v5_0::GenericConnack<32, 32>,             send_connack_v5_0,           process_send_v5_0_connack);
    impl_sendable_helper!(role::Server, v3_1_1::GenericPublish<PacketIdType>,      send_publish_v3_1_1,         process_send_v3_1_1_publish);
    impl_sendable_helper!(role::Server, v5_0::GenericPublish<PacketIdType>,        send_publish_v5_0,           process_send_v5_0_publish);
    impl_sendable_helper!(role::Server, v3_1_1::GenericPuback<PacketIdType>,       send_puback_v3_1_1,          process_send_v3_1_1_puback);
    impl_sendable_helper!(role::Server, v5_0::GenericPuback<PacketIdType>,         send_puback_v5_0,            process_send_v5_0_puback);
    impl_sendable_helper!(role::Server, v3_1_1::GenericPubrec<PacketIdType>,       send_pubrec_v3_1_1,          process_send_v3_1_1_pubrec);
    impl_sendable_helper!(role::Server, v5_0::GenericPubrec<PacketIdType>,         send_pubrec_v5_0,            process_send_v5_0_pubrec);
    impl_sendable_helper!(role::Server, v3_1_1::GenericPubrel<PacketIdType>,       send_pubrel_v3_1_1,          process_send_v3_1_1_pubrel);
    impl_sendable_helper!(role::Server, v5_0::GenericPubrel<PacketIdType>,         send_pubrel_v5_0,            process_send_v5_0_pubrel);
    impl_sendable_helper!(role::Server, v3_1_1::GenericPubcomp<PacketIdType>,      send_pubcomp_v3_1_1,         process_send_v3_1_1_pubcomp);
    impl_sendable_helper!(role::Server, v5_0::GenericPubcomp<PacketIdType>,        send_pubcomp_v5_0,           process_send_v5_0_pubcomp);
    impl_sendable_helper!(role::Server, v3_1_1::GenericSuback<PacketIdType>,       send_suback_v3_1_1,          process_send_v3_1_1_suback);
    impl_sendable_helper!(role::Server, v5_0::GenericSuback<PacketIdType>,         send_suback_v5_0,            process_send_v5_0_suback);
    impl_sendable_helper!(role::Server, v3_1_1::GenericUnsuback<PacketIdType>,     send_unsuback_v3_1_1,        process_send_v3_1_1_unsuback);
    impl_sendable_helper!(role::Server, v5_0::GenericUnsuback<PacketIdType>,       send_unsuback_v5_0,          process_send_v5_0_unsuback);
    impl_sendable_helper!(role::Server, v3_1_1::Pingresp,                          send_pingresp_v3_1_1,        process_send_v3_1_1_pingresp);
    impl_sendable_helper!(role::Server, v5_0::Pingresp,                            send_pingresp_v5_0,          process_send_v5_0_pingresp);
    impl_sendable_helper!(role::Server, v3_1_1::Disconnect,                        send_disconnect_v3_1_1,      process_send_v3_1_1_disconnect);
    impl_sendable_helper!(role::Server, v5_0::GenericDisconnect<32, 32>,          send_disconnect_v5_0,        process_send_v5_0_disconnect);
    impl_sendable_helper!(role::Server, v5_0::GenericAuth<32, 32>,                send_auth_v5_0,              process_send_v5_0_auth);

    // Any
    impl_sendable_helper!(role::Any,    v3_1_1::GenericConnect<32>,               send_connect_v3_1_1,         process_send_v3_1_1_connect);
    impl_sendable_helper!(role::Any,    v5_0::GenericConnect<32, 32>,             send_connect_v5_0,           process_send_v5_0_connect);
    impl_sendable_helper!(role::Any,    v3_1_1::Connack,                           send_connack_v3_1_1,         process_send_v3_1_1_connack);
    impl_sendable_helper!(role::Any,    v5_0::GenericConnack<32, 32>,             send_connack_v5_0,           process_send_v5_0_connack);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericPublish<PacketIdType>,      send_publish_v3_1_1,         process_send_v3_1_1_publish);
    impl_sendable_helper!(role::Any,    v5_0::GenericPublish<PacketIdType>,        send_publish_v5_0,           process_send_v5_0_publish);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericPuback<PacketIdType>,       send_puback_v3_1_1,          process_send_v3_1_1_puback);
    impl_sendable_helper!(role::Any,    v5_0::GenericPuback<PacketIdType>,         send_puback_v5_0,            process_send_v5_0_puback);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericPubrec<PacketIdType>,       send_pubrec_v3_1_1,          process_send_v3_1_1_pubrec);
    impl_sendable_helper!(role::Any,    v5_0::GenericPubrec<PacketIdType>,         send_pubrec_v5_0,            process_send_v5_0_pubrec);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericPubrel<PacketIdType>,       send_pubrel_v3_1_1,          process_send_v3_1_1_pubrel);
    impl_sendable_helper!(role::Any,    v5_0::GenericPubrel<PacketIdType>,         send_pubrel_v5_0,            process_send_v5_0_pubrel);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericPubcomp<PacketIdType>,      send_pubcomp_v3_1_1,         process_send_v3_1_1_pubcomp);
    impl_sendable_helper!(role::Any,    v5_0::GenericPubcomp<PacketIdType>,        send_pubcomp_v5_0,           process_send_v5_0_pubcomp);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericSubscribe<PacketIdType>,    send_subscribe_v3_1_1,       process_send_v3_1_1_subscribe);
    impl_sendable_helper!(role::Any,    v5_0::GenericSubscribe<PacketIdType>,      send_subscribe_v5_0,         process_send_v5_0_subscribe);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericSuback<PacketIdType>,       send_suback_v3_1_1,          process_send_v3_1_1_suback);
    impl_sendable_helper!(role::Any,    v5_0::GenericSuback<PacketIdType>,         send_suback_v5_0,            process_send_v5_0_suback);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericUnsubscribe<PacketIdType>,  send_unsubscribe_v3_1_1,     process_send_v3_1_1_unsubscribe);
    impl_sendable_helper!(role::Any,    v5_0::GenericUnsubscribe<PacketIdType>,    send_unsubscribe_v5_0,       process_send_v5_0_unsubscribe);
    impl_sendable_helper!(role::Any,    v3_1_1::GenericUnsuback<PacketIdType>,     send_unsuback_v3_1_1,        process_send_v3_1_1_unsuback);
    impl_sendable_helper!(role::Any,    v5_0::GenericUnsuback<PacketIdType>,       send_unsuback_v5_0,          process_send_v5_0_unsuback);
    impl_sendable_helper!(role::Any,    v3_1_1::Pingreq,                           send_pingreq_v3_1_1,         process_send_v3_1_1_pingreq);
    impl_sendable_helper!(role::Any,    v5_0::Pingreq,                             send_pingreq_v5_0,           process_send_v5_0_pingreq);
    impl_sendable_helper!(role::Any,    v3_1_1::Pingresp,                          send_pingresp_v3_1_1,        process_send_v3_1_1_pingresp);
    impl_sendable_helper!(role::Any,    v5_0::Pingresp,                            send_pingresp_v5_0,          process_send_v5_0_pingresp);
    impl_sendable_helper!(role::Any,    v3_1_1::Disconnect,                        send_disconnect_v3_1_1,      process_send_v3_1_1_disconnect);
    impl_sendable_helper!(role::Any,    v5_0::GenericDisconnect<32, 32>,          send_disconnect_v5_0,        process_send_v5_0_disconnect);
    impl_sendable_helper!(role::Any,    v5_0::GenericAuth<32, 32>,                send_auth_v5_0,              process_send_v5_0_auth);
}
