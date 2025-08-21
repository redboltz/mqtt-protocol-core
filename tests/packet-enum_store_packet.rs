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
use mqtt_protocol_core::mqtt::packet::GenericPacketTrait;

// ResponsePacket enum tests
#[test]
fn response_packet_debug() {
    common::init_tracing();
    let response_packets = [
        mqtt::packet::ResponsePacket::V3_1_1Puback,
        mqtt::packet::ResponsePacket::V3_1_1Pubrec,
        mqtt::packet::ResponsePacket::V3_1_1Pubcomp,
        mqtt::packet::ResponsePacket::V5_0Puback,
        mqtt::packet::ResponsePacket::V5_0Pubrec,
        mqtt::packet::ResponsePacket::V5_0Pubcomp,
    ];

    for packet in response_packets {
        let debug_str = format!("{packet:?}");
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn response_packet_clone_copy_eq() {
    common::init_tracing();
    let packet = mqtt::packet::ResponsePacket::V3_1_1Puback;
    let copied = packet;

    assert_eq!(packet, copied);
}

#[test]
fn response_packet_hash() {
    common::init_tracing();
    use std::collections::HashMap;
    let mut map = HashMap::new();

    map.insert(mqtt::packet::ResponsePacket::V3_1_1Puback, "puback");
    map.insert(mqtt::packet::ResponsePacket::V3_1_1Pubrec, "pubrec");
    map.insert(mqtt::packet::ResponsePacket::V3_1_1Pubcomp, "pubcomp");
    map.insert(mqtt::packet::ResponsePacket::V5_0Puback, "v5_puback");
    map.insert(mqtt::packet::ResponsePacket::V5_0Pubrec, "v5_pubrec");
    map.insert(mqtt::packet::ResponsePacket::V5_0Pubcomp, "v5_pubcomp");

    assert_eq!(map.len(), 6);
    assert_eq!(map[&mqtt::packet::ResponsePacket::V3_1_1Puback], "puback");
}

// V3.1.1 Publish tests
#[test]
fn v3_1_1_publish_qos1_store_packet() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(123u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();

    match store_packet {
        mqtt::packet::StorePacket::V3_1_1Publish(p) => {
            assert_eq!(p.packet_id().unwrap(), 123u16);
            assert_eq!(p.qos(), mqtt::packet::Qos::AtLeastOnce);
        }
        _ => panic!("Expected V3_1_1Publish"),
    }
}

#[test]
fn v3_1_1_publish_qos2_store_packet() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(456u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();

    match store_packet {
        mqtt::packet::StorePacket::V3_1_1Publish(p) => {
            assert_eq!(p.packet_id().unwrap(), 456u16);
            assert_eq!(p.qos(), mqtt::packet::Qos::ExactlyOnce);
        }
        _ => panic!("Expected V3_1_1Publish"),
    }
}

#[test]
fn v3_1_1_publish_qos0_store_packet_fail() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"test payload")
        .build()
        .unwrap();

    let result: Result<mqtt::packet::StorePacket, _> = publish.try_into();
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::InvalidQos
    );
}

// V5.0 Publish tests
#[test]
fn v5_0_publish_qos1_store_packet() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(789u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();

    match store_packet {
        mqtt::packet::StorePacket::V5_0Publish(p) => {
            assert_eq!(p.packet_id().unwrap(), 789u16);
            assert_eq!(p.qos(), mqtt::packet::Qos::AtLeastOnce);
        }
        _ => panic!("Expected V5_0Publish"),
    }
}

#[test]
fn v5_0_publish_qos2_store_packet() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(999u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();

    match store_packet {
        mqtt::packet::StorePacket::V5_0Publish(p) => {
            assert_eq!(p.packet_id().unwrap(), 999u16);
            assert_eq!(p.qos(), mqtt::packet::Qos::ExactlyOnce);
        }
        _ => panic!("Expected V5_0Publish"),
    }
}

#[test]
fn v5_0_publish_qos0_store_packet_fail() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtMostOnce)
        .payload(b"test payload")
        .build()
        .unwrap();

    let result: Result<mqtt::packet::StorePacket, _> = publish.try_into();
    assert_eq!(
        result.unwrap_err(),
        mqtt::result_code::MqttError::InvalidQos
    );
}

// V3.1.1 Pubrel tests
#[test]
fn v3_1_1_pubrel_store_packet() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(111u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();

    match store_packet {
        mqtt::packet::StorePacket::V3_1_1Pubrel(p) => {
            assert_eq!(p.packet_id(), 111u16);
        }
        _ => panic!("Expected V3_1_1Pubrel"),
    }
}

// V5.0 Pubrel tests
#[test]
fn v5_0_pubrel_store_packet() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(222u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();

    match store_packet {
        mqtt::packet::StorePacket::V5_0Pubrel(p) => {
            assert_eq!(p.packet_id(), 222u16);
        }
        _ => panic!("Expected V5_0Pubrel"),
    }
}

// packet_type() tests
#[test]
fn packet_type_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(
        store_packet.packet_type(),
        mqtt::packet::PacketType::Publish
    );
}

#[test]
fn packet_type_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    assert_eq!(store_packet.packet_type(), mqtt::packet::PacketType::Pubrel);
}

#[test]
fn packet_type_v5_0_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(
        store_packet.packet_type(),
        mqtt::packet::PacketType::Publish
    );
}

#[test]
fn packet_type_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    assert_eq!(store_packet.packet_type(), mqtt::packet::PacketType::Pubrel);
}

// packet_id() tests
#[test]
fn packet_id_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(333u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(store_packet.packet_id(), 333u16);
}

#[test]
fn packet_id_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(444u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    assert_eq!(store_packet.packet_id(), 444u16);
}

#[test]
fn packet_id_v5_0_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(555u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(store_packet.packet_id(), 555u16);
}

#[test]
fn packet_id_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(666u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    assert_eq!(store_packet.packet_id(), 666u16);
}

// response_packet() tests
#[test]
fn response_packet_v3_1_1_publish_qos1() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(
        store_packet.response_packet(),
        mqtt::packet::ResponsePacket::V3_1_1Puback
    );
}

#[test]
fn response_packet_v3_1_1_publish_qos2() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(
        store_packet.response_packet(),
        mqtt::packet::ResponsePacket::V3_1_1Pubrec
    );
}

#[test]
fn response_packet_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    assert_eq!(
        store_packet.response_packet(),
        mqtt::packet::ResponsePacket::V3_1_1Pubcomp
    );
}

#[test]
fn response_packet_v5_0_publish_qos1() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(
        store_packet.response_packet(),
        mqtt::packet::ResponsePacket::V5_0Puback
    );
}

#[test]
fn response_packet_v5_0_publish_qos2() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::ExactlyOnce)
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    assert_eq!(
        store_packet.response_packet(),
        mqtt::packet::ResponsePacket::V5_0Pubrec
    );
}

#[test]
fn response_packet_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    assert_eq!(
        store_packet.response_packet(),
        mqtt::packet::ResponsePacket::V5_0Pubcomp
    );
}

// size() tests (GenericPacketTrait)
#[test]
fn size_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let size = store_packet.size();
    assert!(size > 0);

    #[cfg(feature = "std")]
    {
        let buffers = store_packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

#[test]
fn size_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let size = store_packet.size();
    assert!(size > 0);

    #[cfg(feature = "std")]
    {
        let buffers = store_packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

#[test]
fn size_v5_0_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let size = store_packet.size();
    assert!(size > 0);

    #[cfg(feature = "std")]
    {
        let buffers = store_packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

#[test]
fn size_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let size = store_packet.size();
    assert!(size > 0);

    #[cfg(feature = "std")]
    {
        let buffers = store_packet.to_buffers();
        let actual_size: usize = buffers.iter().map(|buf| buf.len()).sum();
        assert_eq!(size, actual_size);
    }
}

// to_buffers() tests (GenericPacketTrait)
#[cfg(feature = "std")]
#[test]
fn to_buffers_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let buffers = store_packet.to_buffers();
    assert!(!buffers.is_empty());

    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(buf);
    }

    assert_eq!(all_bytes[0], 0x32); // PUBLISH packet type with QoS 1
}

#[cfg(feature = "std")]
#[test]
fn to_buffers_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let buffers = store_packet.to_buffers();
    assert!(!buffers.is_empty());

    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(buf);
    }

    assert_eq!(all_bytes[0], 0x62); // PUBREL packet type
}

#[cfg(feature = "std")]
#[test]
fn to_buffers_v5_0_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let buffers = store_packet.to_buffers();
    assert!(!buffers.is_empty());

    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(buf);
    }

    assert_eq!(all_bytes[0], 0x32); // PUBLISH packet type with QoS 1
}

#[cfg(feature = "std")]
#[test]
fn to_buffers_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let buffers = store_packet.to_buffers();
    assert!(!buffers.is_empty());

    let mut all_bytes = Vec::new();
    for buf in &buffers {
        all_bytes.extend_from_slice(buf);
    }

    assert_eq!(all_bytes[0], 0x62); // PUBREL packet type
}

// Debug and Display tests
#[test]
fn debug_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let debug_str = format!("{store_packet:?}");
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("packet_id"));
}

#[test]
fn display_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let display_str = format!("{store_packet}");
    assert!(!display_str.is_empty());
    assert!(display_str.contains("packet_id"));
}

#[test]
fn debug_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let debug_str = format!("{store_packet:?}");
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("packet_id"));
}

#[test]
fn debug_v5_0_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let debug_str = format!("{store_packet:?}");
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("packet_id"));
}

#[test]
fn debug_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let debug_str = format!("{store_packet:?}");
    assert!(!debug_str.is_empty());
    assert!(debug_str.contains("packet_id"));
}

#[test]
fn display_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let display_str = format!("{store_packet}");
    assert!(!display_str.is_empty());
    assert!(display_str.contains("packet_id"));
}

#[test]
fn display_v5_0_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let display_str = format!("{store_packet}");
    assert!(!display_str.is_empty());
    assert!(display_str.contains("packet_id"));
}

#[test]
fn display_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(1u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let display_str = format!("{store_packet}");
    assert!(!display_str.is_empty());
    assert!(display_str.contains("packet_id"));
}

// Clone and PartialEq tests
#[test]
fn clone_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let cloned = store_packet.clone();
    assert_eq!(store_packet, cloned);
}

#[test]
fn clone_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(123u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let cloned = store_packet.clone();
    assert_eq!(store_packet, cloned);
}

// From GenericStorePacket to GenericPacket conversion tests
#[test]
fn convert_to_generic_packet_v3_1_1_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(1u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let generic_packet: mqtt::packet::GenericPacket<u16> = store_packet.into();

    match generic_packet {
        mqtt::packet::GenericPacket::V3_1_1Publish(p) => {
            assert_eq!(p.packet_id().unwrap(), 1u16);
        }
        _ => panic!("Expected V3_1_1Publish in GenericPacket"),
    }
}

#[test]
fn convert_to_generic_packet_v3_1_1_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(456u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let generic_packet: mqtt::packet::GenericPacket<u16> = store_packet.into();

    match generic_packet {
        mqtt::packet::GenericPacket::V3_1_1Pubrel(p) => {
            assert_eq!(p.packet_id(), 456u16);
        }
        _ => panic!("Expected V3_1_1Pubrel in GenericPacket"),
    }
}

#[test]
fn convert_to_generic_packet_v5_0_publish() {
    common::init_tracing();
    let publish = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .packet_id(789u16)
        .payload(b"test payload")
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = publish.try_into().unwrap();
    let generic_packet: mqtt::packet::GenericPacket<u16> = store_packet.into();

    match generic_packet {
        mqtt::packet::GenericPacket::V5_0Publish(p) => {
            assert_eq!(p.packet_id().unwrap(), 789u16);
            assert_eq!(p.qos(), mqtt::packet::Qos::AtLeastOnce);
        }
        _ => panic!("Expected V5_0Publish in GenericPacket"),
    }
}

#[test]
fn convert_to_generic_packet_v5_0_pubrel() {
    common::init_tracing();
    let pubrel = mqtt::packet::v5_0::Pubrel::builder()
        .packet_id(789u16)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    let generic_packet: mqtt::packet::GenericPacket<u16> = store_packet.into();

    match generic_packet {
        mqtt::packet::GenericPacket::V5_0Pubrel(p) => {
            assert_eq!(p.packet_id(), 789u16);
        }
        _ => panic!("Expected V5_0Pubrel in GenericPacket"),
    }
}

// Type alias test
#[test]
fn store_packet_type_alias() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::Pubrel::builder()
        .packet_id(42u16)
        .build()
        .unwrap();

    let _store_packet: mqtt::packet::StorePacket = pubrel.try_into().unwrap();
    // Test that StorePacket is indeed GenericStorePacket<u16>
}

// Generic PacketIdType test (u32)
#[test]
fn generic_packet_id_type_u32() {
    common::init_tracing();
    let pubrel = mqtt::packet::v3_1_1::GenericPubrel::<u32>::builder()
        .packet_id(0x12345678u32)
        .build()
        .unwrap();

    let store_packet: mqtt::packet::GenericStorePacket<u32> = pubrel.try_into().unwrap();
    assert_eq!(store_packet.packet_id(), 0x12345678u32);
    assert_eq!(store_packet.packet_type(), mqtt::packet::PacketType::Pubrel);
}
