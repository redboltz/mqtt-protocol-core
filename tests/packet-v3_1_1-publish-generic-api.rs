use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::packet::Qos;

#[test]
fn test_publish_default_sizes() {
    // Default usage without specifying buffer sizes
    let publish: mqtt::packet::v3_1_1::Publish = mqtt::packet::v3_1_1::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(Qos::AtMostOnce)
        .payload(b"Hello")
        .build()
        .unwrap();

    assert_eq!(publish.topic_name(), "test/topic");
    assert_eq!(publish.payload().as_slice(), b"Hello");
}

#[test]
fn test_custom_publish_types() {
    // Define custom types with different buffer sizes
    type LargePublish = mqtt::packet::v3_1_1::GenericPublish<u16, 128, 256>;
    type SmallPublish = mqtt::packet::v3_1_1::GenericPublish<u16, 16, 16>;

    // Large buffer publish
    let large_publish: LargePublish = LargePublish::builder()
        .topic_name("sensors/temperature/zone1/outdoor/detailed")
        .unwrap()
        .qos(Qos::AtMostOnce)
        .payload(&[42u8; 200])
        .build()
        .unwrap();

    assert_eq!(
        large_publish.topic_name(),
        "sensors/temperature/zone1/outdoor/detailed"
    );
    assert_eq!(large_publish.payload().as_slice(), &[42u8; 200]);

    // Small buffer publish
    let small_publish: SmallPublish = SmallPublish::builder()
        .topic_name("temp")
        .unwrap()
        .qos(Qos::AtMostOnce)
        .payload(b"25")
        .build()
        .unwrap();

    assert_eq!(small_publish.topic_name(), "temp");
    assert_eq!(small_publish.payload().as_slice(), b"25");
}

#[test]
fn test_generic_publish_u32_default_buffers() {
    // Custom packet ID type with default buffer sizes
    let u32_publish: mqtt::packet::v3_1_1::GenericPublish<u32> =
        mqtt::packet::v3_1_1::GenericPublish::builder()
            .topic_name("cluster/data")
            .unwrap()
            .qos(Qos::AtLeastOnce)
            .packet_id(0x12345678u32)
            .payload(b"cluster message")
            .build()
            .unwrap();

    assert_eq!(u32_publish.topic_name(), "cluster/data");
    assert_eq!(u32_publish.packet_id(), Some(0x12345678u32));
    assert_eq!(u32_publish.payload().as_slice(), b"cluster message");
}

#[test]
fn test_generic_publish_full_customization() {
    // Full customization: custom packet ID type and custom buffer sizes
    let full_custom: mqtt::packet::v3_1_1::GenericPublish<u32, 64, 128> =
        mqtt::packet::v3_1_1::GenericPublish::builder()
            .topic_name("enterprise/high-throughput/data")
            .unwrap()
            .qos(Qos::ExactlyOnce)
            .packet_id(0xABCDEF01u32)
            .payload(&[0xFFu8; 100])
            .build()
            .unwrap();

    assert_eq!(full_custom.topic_name(), "enterprise/high-throughput/data");
    assert_eq!(full_custom.packet_id(), Some(0xABCDEF01u32));
    assert_eq!(full_custom.payload().as_slice(), &[0xFFu8; 100]);
}
