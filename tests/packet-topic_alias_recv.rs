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
use mqtt_protocol_core::mqtt;
mod common;

#[test]
fn test_recv_basic_functionality() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(5);

    // Test max() method
    assert_eq!(tar.max(), 5);

    // Insert some topics
    tar.insert_or_update("topic1", 1);
    tar.insert_or_update("topic3", 3);

    // Test get by alias
    assert_eq!(tar.get(1), Some("topic1"));
    assert_eq!(tar.get(3), Some("topic3"));
    assert_eq!(tar.get(2), None); // not registered

    // Test peek (should behave the same as get for recv)
    assert_eq!(tar.peek(1), Some("topic1"));
    assert_eq!(tar.peek(3), Some("topic3"));
    assert_eq!(tar.peek(2), None); // not registered

    // Update existing alias
    tar.insert_or_update("topic10", 1);
    assert_eq!(tar.get(1), Some("topic10"));

    // Test clear
    tar.clear();
    assert_eq!(tar.get(1), None);
    assert_eq!(tar.get(2), None);
    assert_eq!(tar.get(3), None);

    // Insert after clear
    tar.insert_or_update("topic1", 1);
    assert_eq!(tar.get(1), Some("topic1"));
}

#[test]
fn test_recv_functionality() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(5);

    tar.insert_or_update("topic1", 1);
    tar.insert_or_update("topic3", 3);

    assert_eq!(tar.get(1), Some("topic1"));
    assert_eq!(tar.get(3), Some("topic3"));
    assert_eq!(tar.get(2), None); // not registered

    // Update existing alias
    tar.insert_or_update("topic10", 1);
    assert_eq!(tar.get(1), Some("topic10"));

    tar.clear();

    assert_eq!(tar.get(1), None); // not registered
    assert_eq!(tar.get(3), None); // not registered

    tar.insert_or_update("topic1", 1);
    assert_eq!(tar.get(1), Some("topic1"));
}

#[test]
fn test_boundary_conditions() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(2);

    // Test MIN_ALIAS boundary (should be 1)
    tar.insert_or_update("topic1", 1);
    assert_eq!(tar.get(1), Some("topic1"));

    // Test max_alias boundary
    tar.insert_or_update("topic2", 2);
    assert_eq!(tar.get(2), Some("topic2"));

    // Test out of range queries
    assert_eq!(tar.get(0), None); // below MIN_ALIAS
    assert_eq!(tar.get(3), None); // above max_alias
    assert_eq!(tar.peek(0), None);
    assert_eq!(tar.peek(3), None);
}

#[test]
fn test_empty_container() {
    common::init_tracing();
    let tar = mqtt::packet::TopicAliasRecv::new(5);

    // Test operations on empty container
    assert_eq!(tar.get(1), None);
    assert_eq!(tar.peek(1), None);
}

#[test]
fn test_duplicate_operations() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(3);

    // Insert same topic-alias pair multiple times
    tar.insert_or_update("topic1", 1);
    tar.insert_or_update("topic1", 1);
    tar.insert_or_update("topic1", 1);

    assert_eq!(tar.get(1), Some("topic1"));
}

#[test]
fn test_large_max_alias() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(1000);

    assert_eq!(tar.max(), 1000);

    // Insert some entries
    tar.insert_or_update("topic1", 1);
    tar.insert_or_update("topic500", 500);
    tar.insert_or_update("topic1000", 1000);

    assert_eq!(tar.get(1), Some("topic1"));
    assert_eq!(tar.get(500), Some("topic500"));
    assert_eq!(tar.get(1000), Some("topic1000"));
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_insert_empty_topic_panic() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(10);
    tar.insert_or_update("", 1); // Should panic
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_insert_invalid_alias_zero_panic() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(10);
    tar.insert_or_update("test", 0); // Should panic
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_insert_invalid_alias_too_high_panic() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(10);
    tar.insert_or_update("test", 11); // Should panic
}

#[test]
fn test_special_characters_in_topics() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(5);

    // Test topics with special characters
    tar.insert_or_update("topic/with/slashes", 1);
    tar.insert_or_update("topic with spaces", 2);
    tar.insert_or_update("topic_with_underscores", 3);
    tar.insert_or_update("topic-with-dashes", 4);
    tar.insert_or_update("topic.with.dots", 5);

    assert_eq!(tar.get(1), Some("topic/with/slashes"));
    assert_eq!(tar.get(2), Some("topic with spaces"));
    assert_eq!(tar.get(3), Some("topic_with_underscores"));
    assert_eq!(tar.get(4), Some("topic-with-dashes"));
    assert_eq!(tar.get(5), Some("topic.with.dots"));
}

#[test]
fn test_long_topic_names() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(3);

    let long_topic = "a".repeat(1000);
    let very_long_topic = "b".repeat(10000);

    tar.insert_or_update(&long_topic, 1);
    tar.insert_or_update(&very_long_topic, 2);

    assert_eq!(tar.get(1), Some(long_topic.as_str()));
    assert_eq!(tar.get(2), Some(very_long_topic.as_str()));
}

#[test]
fn test_unicode_topics() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(5);

    tar.insert_or_update("トピック1", 1);
    tar.insert_or_update("主题2", 2);
    tar.insert_or_update("موضوع3", 3);
    tar.insert_or_update("тема4", 4);
    tar.insert_or_update("🎉emoji🎊", 5);

    assert_eq!(tar.get(1), Some("トピック1"));
    assert_eq!(tar.get(2), Some("主题2"));
    assert_eq!(tar.get(3), Some("موضوع3"));
    assert_eq!(tar.get(4), Some("тема4"));
    assert_eq!(tar.get(5), Some("🎉emoji🎊"));
}

#[test]
fn test_topic_update_behavior() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(3);

    tar.insert_or_update("topic1", 1);
    tar.insert_or_update("topic2", 2);

    // Verify initial state
    assert_eq!(tar.get(1), Some("topic1"));
    assert_eq!(tar.get(2), Some("topic2"));

    // Update alias 1 to point to topic2
    tar.insert_or_update("topic2", 1);

    // alias 1 should now have topic2
    assert_eq!(tar.get(1), Some("topic2"));
    assert_eq!(tar.get(2), Some("topic2")); // alias 2 should still have topic2
}

#[test]
fn test_alias_overwrite_behavior() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasRecv::new(3);

    tar.insert_or_update("topic1", 1);
    tar.insert_or_update("topic2", 2);

    // Update topic1 to use alias 2 (should overwrite)
    tar.insert_or_update("topic1", 2);

    // alias 2 should now have topic1, and alias 1 should still have topic1
    assert_eq!(tar.get(1), Some("topic1"));
    assert_eq!(tar.get(2), Some("topic1")); // overwritten
}
