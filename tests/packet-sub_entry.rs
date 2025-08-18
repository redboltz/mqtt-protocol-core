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
use mqtt_protocol_core::mqtt;
mod common;

#[test]
fn test_sub_opts_new() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new();
    assert_eq!(opts.qos(), mqtt::packet::Qos::AtMostOnce);
    assert_eq!(opts.nl(), false);
    assert_eq!(opts.rap(), false);
    assert_eq!(opts.rh(), mqtt::packet::RetainHandling::SendRetained);
}

#[test]
fn test_sub_opts_default() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::default();
    assert_eq!(opts.qos(), mqtt::packet::Qos::AtMostOnce);
    assert_eq!(opts.nl(), false);
    assert_eq!(opts.rap(), false);
    assert_eq!(opts.rh(), mqtt::packet::RetainHandling::SendRetained);
}

#[test]
fn test_sub_opts_from_u8() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::from_u8(0x00).unwrap();
    assert_eq!(opts.qos(), mqtt::packet::Qos::AtMostOnce);
    assert_eq!(opts.nl(), false);
    assert_eq!(opts.rap(), false);
    assert_eq!(opts.rh(), mqtt::packet::RetainHandling::SendRetained);

    let opts = mqtt::packet::SubOpts::from_u8(0x01).unwrap();
    assert_eq!(opts.qos(), mqtt::packet::Qos::AtLeastOnce);

    let opts = mqtt::packet::SubOpts::from_u8(0x02).unwrap();
    assert_eq!(opts.qos(), mqtt::packet::Qos::ExactlyOnce);

    let opts = mqtt::packet::SubOpts::from_u8(0x04).unwrap();
    assert_eq!(opts.nl(), true);

    let opts = mqtt::packet::SubOpts::from_u8(0x08).unwrap();
    assert_eq!(opts.rap(), true);

    let opts = mqtt::packet::SubOpts::from_u8(0x10).unwrap();
    assert_eq!(
        opts.rh(),
        mqtt::packet::RetainHandling::SendRetainedIfNotExists
    );

    let opts = mqtt::packet::SubOpts::from_u8(0x20).unwrap();
    assert_eq!(opts.rh(), mqtt::packet::RetainHandling::DoNotSendRetained);

    assert!(mqtt::packet::SubOpts::from_u8(0x03).is_err());
    assert!(mqtt::packet::SubOpts::from_u8(0x30).is_err());
    assert!(mqtt::packet::SubOpts::from_u8(0x40).is_err());
    assert!(mqtt::packet::SubOpts::from_u8(0x80).is_err());
}

#[test]
fn test_sub_opts_set_qos() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    assert_eq!(opts.qos(), mqtt::packet::Qos::AtLeastOnce);

    let opts = opts.set_qos(mqtt::packet::Qos::ExactlyOnce);
    assert_eq!(opts.qos(), mqtt::packet::Qos::ExactlyOnce);
}

#[test]
fn test_sub_opts_set_nl() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new().set_nl(true);
    assert_eq!(opts.nl(), true);

    let opts = opts.set_nl(false);
    assert_eq!(opts.nl(), false);
}

#[test]
fn test_sub_opts_set_rap() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new().set_rap(true);
    assert_eq!(opts.rap(), true);

    let opts = opts.set_rap(false);
    assert_eq!(opts.rap(), false);
}

#[test]
fn test_sub_opts_set_rh() {
    common::init_tracing();
    let opts =
        mqtt::packet::SubOpts::new().set_rh(mqtt::packet::RetainHandling::SendRetainedIfNotExists);
    assert_eq!(
        opts.rh(),
        mqtt::packet::RetainHandling::SendRetainedIfNotExists
    );

    let opts = opts.set_rh(mqtt::packet::RetainHandling::DoNotSendRetained);
    assert_eq!(opts.rh(), mqtt::packet::RetainHandling::DoNotSendRetained);
}

#[test]
fn test_sub_opts_chaining() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new()
        .set_qos(mqtt::packet::Qos::AtLeastOnce)
        .set_nl(true)
        .set_rap(true)
        .set_rh(mqtt::packet::RetainHandling::DoNotSendRetained);

    assert_eq!(opts.qos(), mqtt::packet::Qos::AtLeastOnce);
    assert_eq!(opts.nl(), true);
    assert_eq!(opts.rap(), true);
    assert_eq!(opts.rh(), mqtt::packet::RetainHandling::DoNotSendRetained);
}

#[test]
fn test_sub_opts_to_buffer() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new()
        .set_qos(mqtt::packet::Qos::AtLeastOnce)
        .set_nl(true)
        .set_rap(true)
        .set_rh(mqtt::packet::RetainHandling::DoNotSendRetained);

    let buffer = opts.to_buffer();
    assert_eq!(buffer[0], 0x01 | 0x04 | 0x08 | 0x20);
}

#[test]
fn test_sub_opts_display() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    let display_str = format!("{opts}");
    assert!(display_str.contains("qos"));
    assert!(display_str.contains("AtLeastOnce"));
}

#[test]
fn test_sub_entry_new() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new();
    let entry = mqtt::packet::SubEntry::new("test/topic", opts).unwrap();

    assert_eq!(entry.topic_filter(), "test/topic");
    assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::AtMostOnce);
}

#[test]
fn test_sub_entry_default() {
    common::init_tracing();
    let entry = mqtt::packet::SubEntry::default();
    assert_eq!(entry.topic_filter(), "");
    assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::AtMostOnce);
}

#[test]
fn test_sub_entry_with_wildcards() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    let entry = mqtt::packet::SubEntry::new("sensors/+/temperature", opts).unwrap();

    assert_eq!(entry.topic_filter(), "sensors/+/temperature");
    assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::AtLeastOnce);

    let entry = mqtt::packet::SubEntry::new("home/#", opts).unwrap();
    assert_eq!(entry.topic_filter(), "home/#");
}

#[test]
fn test_sub_entry_set_topic_filter() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new();
    let mut entry = mqtt::packet::SubEntry::new("old/topic", opts).unwrap();

    entry.set_topic_filter("new/topic".to_string()).unwrap();
    assert_eq!(entry.topic_filter(), "new/topic");
}

#[test]
fn test_sub_entry_set_sub_opts() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new();
    let mut entry = mqtt::packet::SubEntry::new("test/topic", opts).unwrap();

    let new_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::ExactlyOnce);
    entry.set_sub_opts(new_opts);
    assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::ExactlyOnce);
}

#[test]
fn test_sub_entry_size() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new();
    let entry = mqtt::packet::SubEntry::new("test", opts).unwrap();
    assert_eq!(entry.size(), 7);

    let entry = mqtt::packet::SubEntry::new("longer/topic/name", opts).unwrap();
    assert_eq!(entry.size(), 2 + "longer/topic/name".len() + 1);
}

#[test]
#[cfg(feature = "std")]
fn test_sub_entry_to_buffers() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new();
    let entry = mqtt::packet::SubEntry::new("test/topic", opts).unwrap();
    let buffers = entry.to_buffers();

    assert!(!buffers.is_empty());
}

#[test]
fn test_sub_entry_parse() {
    common::init_tracing();
    let data = [0x00, 0x04, b't', b'e', b's', b't', 0x01];
    let (entry, consumed) = mqtt::packet::SubEntry::parse(&data).unwrap();

    assert_eq!(entry.topic_filter(), "test");
    assert_eq!(entry.sub_opts().qos(), mqtt::packet::Qos::AtLeastOnce);
    assert_eq!(consumed, 7);
}

#[test]
fn test_sub_entry_parse_insufficient_data() {
    common::init_tracing();
    let data = [0x00, 0x04, b't', b'e', b's', b't'];
    let result = mqtt::packet::SubEntry::parse(&data);
    assert!(result.is_err());

    let data = [0x00, 0x04, b't', b'e'];
    let result = mqtt::packet::SubEntry::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_sub_entry_parse_invalid_sub_opts() {
    common::init_tracing();
    let data = [0x00, 0x04, b't', b'e', b's', b't', 0x03];
    let result = mqtt::packet::SubEntry::parse(&data);
    assert!(result.is_err());
}

#[test]
fn test_sub_entry_display() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    let entry = mqtt::packet::SubEntry::new("test/topic", opts).unwrap();
    let display_str = format!("{entry}");
    assert!(display_str.contains("test/topic"));
    assert!(display_str.contains("AtLeastOnce"));
}

#[test]
fn test_sub_entry_equality() {
    common::init_tracing();
    let opts1 = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    let opts2 = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    let opts3 = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::ExactlyOnce);

    let entry1 = mqtt::packet::SubEntry::new("test/topic", opts1).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("test/topic", opts2).unwrap();
    let entry3 = mqtt::packet::SubEntry::new("test/topic", opts3).unwrap();
    let entry4 = mqtt::packet::SubEntry::new("other/topic", opts1).unwrap();

    assert_eq!(entry1, entry2);
    assert_ne!(entry1, entry3);
    assert_ne!(entry1, entry4);
}

#[test]
fn test_sub_entry_ordering() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new();
    let entry1 = mqtt::packet::SubEntry::new("a/topic", opts).unwrap();
    let entry2 = mqtt::packet::SubEntry::new("b/topic", opts).unwrap();

    assert!(entry1 < entry2);
}

#[test]
fn test_sub_entry_clone() {
    common::init_tracing();
    let opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
    let entry1 = mqtt::packet::SubEntry::new("test/topic", opts).unwrap();
    let entry2 = entry1.clone();

    assert_eq!(entry1, entry2);
    assert_eq!(entry1.topic_filter(), entry2.topic_filter());
    assert_eq!(entry1.sub_opts().qos(), entry2.sub_opts().qos());
}
