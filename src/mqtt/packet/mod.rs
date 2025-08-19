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
mod mqtt_string;
pub use self::mqtt_string::MqttString;
mod mqtt_binary;
pub use self::mqtt_binary::MqttBinary;

mod enum_packet;
mod enum_store_packet;
mod property;
mod qos;
pub use self::qos::Qos;
mod retain_handling;
pub use self::retain_handling::RetainHandling;
mod sub_entry;
pub use self::sub_entry::{SubEntry, SubOpts};
mod variable_byte_integer;
pub use self::variable_byte_integer::{DecodeResult, VariableByteInteger};
mod packet_type;
pub use self::packet_type::{FixedHeader, PacketType};
mod packet_id;
pub use self::packet_id::IsPacketId;
pub mod v3_1_1;
pub mod v5_0;
pub use self::enum_packet::{GenericPacket, GenericPacketDisplay, GenericPacketTrait, Packet};
pub use self::enum_store_packet::{GenericStorePacket, ResponsePacket, StorePacket};
pub use self::property::PayloadFormat;
mod json_bin_encode;
#[cfg(feature = "std")]
pub use self::property::PropertiesToBuffers;
pub use self::property::{
    AssignedClientIdentifier, AuthenticationData, AuthenticationMethod, ContentType,
    CorrelationData, MaximumPacketSize, MaximumQos, MessageExpiryInterval, PayloadFormatIndicator,
    Properties, PropertiesParse, PropertiesSize, Property, PropertyId, ReasonString,
    ReceiveMaximum, RequestProblemInformation, RequestResponseInformation, ResponseInformation,
    ResponseTopic, RetainAvailable, ServerKeepAlive, ServerReference, SessionExpiryInterval,
    SharedSubscriptionAvailable, SubscriptionIdentifier, SubscriptionIdentifierAvailable,
    TopicAlias, TopicAliasMaximum, UserProperty, WildcardSubscriptionAvailable, WillDelayInterval,
};
pub use json_bin_encode::escape_binary_json_string;

mod topic_alias_send;
pub use self::topic_alias_send::TopicAliasSend;
mod topic_alias_recv;
pub use self::topic_alias_recv::TopicAliasRecv;

pub mod kind;
pub mod prelude;
