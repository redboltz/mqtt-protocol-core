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
use std::thread;
use std::time::Duration;

use mqtt_protocol_core::default_alias;
use mqtt_protocol_core::mqtt;
mod common;

#[test]
fn test_send_basic_functionality() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(5);

    // Test max() method
    assert_eq!(tas.max(), 5);

    // Insert some topics
    tas.insert_or_update("topic1", 1);
    tas.insert_or_update("topic3", 3);

    // Test find by alias
    assert_eq!(tas.get(1), Some("topic1"));
    assert_eq!(tas.get(3), Some("topic3"));
    assert_eq!(tas.get(2), None); // not registered

    // Test get_lru_alias - should return first vacant
    assert_eq!(tas.get_lru_alias(), 2); // first vacant

    tas.insert_or_update("topic2", 2);
    assert_eq!(tas.get_lru_alias(), 4); // first vacant

    tas.insert_or_update("topic4", 4);
    assert_eq!(tas.get_lru_alias(), 5); // first vacant

    tas.insert_or_update("topic5", 5);

    // Map fulfilled - should return least recently used
    assert_eq!(tas.get_lru_alias(), 1); // least recently used

    // Update existing alias
    tas.insert_or_update("topic10", 1);
    assert_eq!(tas.get_lru_alias(), 3); // least recently used
    assert_eq!(tas.get(1), Some("topic10"));

    // Access topic3 to update its timestamp
    assert_eq!(tas.get(3), Some("topic3"));
    assert_eq!(tas.get_lru_alias(), 2); // least recently used

    // Test find_by_topic method
    assert_eq!(tas.find_by_topic("topic2"), Some(2));
    assert_eq!(tas.get_lru_alias(), 2); // LRU doesn't update on find_by_topic
    assert_eq!(tas.find_by_topic("non exist"), None);

    // Test clear
    tas.clear();
    assert_eq!(tas.get_lru_alias(), 1);
    assert_eq!(tas.get(1), None);
    assert_eq!(tas.get(2), None);
    assert_eq!(tas.get(3), None);
    assert_eq!(tas.get(4), None);
    assert_eq!(tas.get(5), None);

    // Insert after clear
    tas.insert_or_update("topic1", 1);
    assert_eq!(tas.get(1), Some("topic1"));
}

#[test]
fn test_recv_functionality() {
    common::init_tracing();
    let mut tar = mqtt::packet::TopicAliasSend::new(5);

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
fn test_peek() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(3);

    tas.insert_or_update("topic1", 1);
    tas.insert_or_update("topic2", 2);

    // Test peek doesn't affect LRU
    assert_eq!(tas.peek(1), Some("topic1"));
    assert_eq!(tas.peek(2), Some("topic2"));
    assert_eq!(tas.peek(3), None); // not found
    assert_eq!(tas.peek(0), None); // out of range
    assert_eq!(tas.peek(999), None); // out of range

    // LRU should still be topic1 (oldest)
    tas.insert_or_update("topic3", 3);
    let lru = tas.get_lru_alias();
    assert_eq!(lru, 1); // topic1 should be LRU
}

#[test]
fn test_alias_update_behavior() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(3);

    tas.insert_or_update("topic1", 1);
    tas.insert_or_update("topic2", 2);

    // Verify initial state
    assert_eq!(tas.find_by_topic("topic1"), Some(1));
    assert_eq!(tas.find_by_topic("topic2"), Some(2));

    // Update topic1 to use alias 2 (should add new mapping, topic2 replaced)
    tas.insert_or_update("topic1", 2);

    // topic1 should now have both aliases 1 and 2, topic2 should be removed
    let found_alias = tas.find_by_topic("topic1");
    assert!(found_alias == Some(1) || found_alias == Some(2)); // either is valid as first
    assert_eq!(tas.find_by_topic("topic2"), None); // topic2 should be removed
    assert_eq!(tas.get(1), Some("topic1")); // alias 1 should still have topic1
    assert_eq!(tas.get(2), Some("topic1")); // alias 2 should now have topic1
}

#[test]
fn test_topic_update_behavior() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(3);

    tas.insert_or_update("topic1", 1);
    tas.insert_or_update("topic2", 2);

    // Update alias 1 to point to topic2 (topic2 takes over alias 1, topic1 removed from alias 1)
    tas.insert_or_update("topic2", 1);

    // alias 1 should now have topic2, topic2 should have both aliases 1 and 2
    assert_eq!(tas.get(1), Some("topic2"));
    assert_eq!(tas.get(2), Some("topic2")); // alias 2 should still have topic2
    let found_alias = tas.find_by_topic("topic2");
    assert!(found_alias == Some(1) || found_alias == Some(2)); // either is valid
    assert_eq!(tas.find_by_topic("topic1"), None); // topic1 should be removed
}

#[test]
fn test_lru_ordering_with_timestamps() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(3);

    // Insert topics with small delays to ensure different timestamps
    tas.insert_or_update("topic1", 1);
    thread::sleep(Duration::from_millis(1));

    tas.insert_or_update("topic2", 2);
    thread::sleep(Duration::from_millis(1));

    tas.insert_or_update("topic3", 3);

    // All aliases used, should return LRU (topic1)
    assert_eq!(tas.get_lru_alias(), 1);

    // Access topic1 to update its timestamp
    tas.get(1);

    // Now topic2 should be LRU
    assert_eq!(tas.get_lru_alias(), 2);
}

#[test]
fn test_edge_cases() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(1);

    // Test with max_alias = 1
    assert_eq!(tas.max(), 1);
    assert_eq!(tas.get_lru_alias(), 1);

    tas.insert_or_update("topic1", 1);
    assert_eq!(tas.get(1), Some("topic1"));
    assert_eq!(tas.get_lru_alias(), 1); // only option

    // Update with same alias
    tas.insert_or_update("topic2", 1);
    assert_eq!(tas.get(1), Some("topic2"));
    assert_eq!(tas.find_by_topic("topic1"), None);
    assert_eq!(tas.find_by_topic("topic2"), Some(1));
}

#[test]
fn test_boundary_conditions() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(2);

    // Test MIN_ALIAS boundary (should be 1)
    tas.insert_or_update("topic1", 1);
    assert_eq!(tas.get(1), Some("topic1"));

    // Test max_alias boundary
    tas.insert_or_update("topic2", 2);
    assert_eq!(tas.get(2), Some("topic2"));

    // Test out of range queries
    assert_eq!(tas.get(0), None); // below MIN_ALIAS
    assert_eq!(tas.get(3), None); // above max_alias
    assert_eq!(tas.peek(0), None);
    assert_eq!(tas.peek(3), None);
}

#[test]
fn test_empty_container() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(5);

    // Test operations on empty container
    assert_eq!(tas.get(1), None);
    assert_eq!(tas.peek(1), None);
    assert_eq!(tas.find_by_topic("nonexistent"), None);
    assert_eq!(tas.get_lru_alias(), 1); // should return first available

    // Clear empty container
    tas.clear();
    assert_eq!(tas.get_lru_alias(), 1);
}

#[test]
fn test_duplicate_operations() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(3);

    // Insert same topic-alias pair multiple times
    tas.insert_or_update("topic1", 1);
    tas.insert_or_update("topic1", 1);
    tas.insert_or_update("topic1", 1);

    assert_eq!(tas.get(1), Some("topic1"));
    assert_eq!(tas.find_by_topic("topic1"), Some(1));

    // Should still have only one entry
    assert_eq!(tas.get_lru_alias(), 2); // next available should be 2
}

#[test]
fn test_large_max_alias() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(1000);

    assert_eq!(tas.max(), 1000);
    assert_eq!(tas.get_lru_alias(), 1);

    // Insert some entries
    tas.insert_or_update("topic1", 1);
    tas.insert_or_update("topic500", 500);
    tas.insert_or_update("topic1000", 1000);

    assert_eq!(tas.get(1), Some("topic1"));
    assert_eq!(tas.get(500), Some("topic500"));
    assert_eq!(tas.get(1000), Some("topic1000"));

    assert_eq!(tas.get_lru_alias(), 2); // next vacant
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_insert_empty_topic_panic() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(10);
    tas.insert_or_update("", 1); // Should panic
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_insert_invalid_alias_zero_panic() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(10);
    tas.insert_or_update("test", 0); // Should panic
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_insert_invalid_alias_too_high_panic() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(10);
    tas.insert_or_update("test", 11); // Should panic
}

#[test]
#[should_panic(expected = "assertion failed")]
fn test_get_lru_alias_zero_max_panic() {
    common::init_tracing();
    let tas = mqtt::packet::TopicAliasSend::new(0);
    tas.get_lru_alias(); // Should panic
}

#[test]
fn test_special_characters_in_topics() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(5);

    // Test topics with special characters
    tas.insert_or_update("topic/with/slashes", 1);
    tas.insert_or_update("topic with spaces", 2);
    tas.insert_or_update("topic_with_underscores", 3);
    tas.insert_or_update("topic-with-dashes", 4);
    tas.insert_or_update("topic.with.dots", 5);

    assert_eq!(tas.get(1), Some("topic/with/slashes"));
    assert_eq!(tas.get(2), Some("topic with spaces"));
    assert_eq!(tas.get(3), Some("topic_with_underscores"));
    assert_eq!(tas.get(4), Some("topic-with-dashes"));
    assert_eq!(tas.get(5), Some("topic.with.dots"));

    assert_eq!(tas.find_by_topic("topic/with/slashes"), Some(1));
    assert_eq!(tas.find_by_topic("topic with spaces"), Some(2));
    assert_eq!(tas.find_by_topic("topic_with_underscores"), Some(3));
    assert_eq!(tas.find_by_topic("topic-with-dashes"), Some(4));
    assert_eq!(tas.find_by_topic("topic.with.dots"), Some(5));
}

#[test]
fn test_long_topic_names() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(3);

    let long_topic = "a".repeat(1000);
    let very_long_topic = "b".repeat(10000);

    tas.insert_or_update(&long_topic, 1);
    tas.insert_or_update(&very_long_topic, 2);

    assert_eq!(tas.get(1), Some(long_topic.as_str()));
    assert_eq!(tas.get(2), Some(very_long_topic.as_str()));
    assert_eq!(tas.find_by_topic(&long_topic), Some(1));
    assert_eq!(tas.find_by_topic(&very_long_topic), Some(2));
}

#[test]
fn test_unicode_topics() {
    common::init_tracing();
    let mut tas = mqtt::packet::TopicAliasSend::new(5);

    tas.insert_or_update("トピック1", 1);
    tas.insert_or_update("主题2", 2);
    tas.insert_or_update("موضوع3", 3);
    tas.insert_or_update("тема4", 4);
    tas.insert_or_update("🎉emoji🎊", 5);

    assert_eq!(tas.get(1), Some("トピック1"));
    assert_eq!(tas.get(2), Some("主题2"));
    assert_eq!(tas.get(3), Some("موضوع3"));
    assert_eq!(tas.get(4), Some("тема4"));
    assert_eq!(tas.get(5), Some("🎉emoji🎊"));

    assert_eq!(tas.find_by_topic("トピック1"), Some(1));
    assert_eq!(tas.find_by_topic("主题2"), Some(2));
    assert_eq!(tas.find_by_topic("موضوع3"), Some(3));
    assert_eq!(tas.find_by_topic("тема4"), Some(4));
    assert_eq!(tas.find_by_topic("🎉emoji🎊"), Some(5));
}

#[test]
fn test_same_topic_different_alias_update() {
    common::init_tracing();
    // Test: topic1 1 -> topic1 2 (same topic, different alias)
    let mut tas = mqtt::packet::TopicAliasSend::new(5);

    // Initial mapping
    tas.insert_or_update("topic1", 1);
    assert_eq!(tas.find_by_topic("topic1"), Some(1));
    assert_eq!(tas.get(1), Some("topic1"));

    // Update same topic to different alias (should add new mapping)
    tas.insert_or_update("topic1", 2);

    // Verify topic1 now has both aliases
    let found_alias = tas.find_by_topic("topic1");
    assert!(found_alias == Some(1) || found_alias == Some(2)); // either is valid
    assert_eq!(tas.get(1), Some("topic1")); // original alias should still exist
    assert_eq!(tas.get(2), Some("topic1")); // new alias should also exist

    // Verify next alias for reuse
    assert_eq!(tas.get_lru_alias(), 3); // should return first vacant alias
}

#[test]
fn test_same_alias_different_topic_update() {
    common::init_tracing();
    // Test: topic1 1 -> topic2 1 (same alias, different topic)
    let mut tas = mqtt::packet::TopicAliasSend::new(5);

    // Initial mapping
    tas.insert_or_update("topic1", 1);
    assert_eq!(tas.find_by_topic("topic1"), Some(1));
    assert_eq!(tas.get(1), Some("topic1"));

    // Update same alias to different topic
    tas.insert_or_update("topic2", 1);

    // Verify new mapping exists
    assert_eq!(tas.find_by_topic("topic2"), Some(1));
    assert_eq!(tas.get(1), Some("topic2"));

    // Verify old mapping is removed
    assert_eq!(tas.find_by_topic("topic1"), None); // old topic should not be found
}

#[test]
fn test_overwrite_verification_comprehensive() {
    common::init_tracing();
    // Comprehensive test for overwrite scenarios with detailed verification
    let mut tas = mqtt::packet::TopicAliasSend::new(10);

    // Setup initial mappings
    tas.insert_or_update("topicA", 1);
    tas.insert_or_update("topicB", 2);
    tas.insert_or_update("topicC", 3);

    // Verify initial state
    assert_eq!(tas.find_by_topic("topicA"), Some(1));
    assert_eq!(tas.find_by_topic("topicB"), Some(2));
    assert_eq!(tas.find_by_topic("topicC"), Some(3));
    assert_eq!(tas.get(1), Some("topicA"));
    assert_eq!(tas.get(2), Some("topicB"));
    assert_eq!(tas.get(3), Some("topicC"));

    // Case 1: Same topic, different alias (topicA 1 -> topicA 5)
    tas.insert_or_update("topicA", 5);

    // Verify topicA now has both aliases 1 and 5
    let found_alias = tas.find_by_topic("topicA");
    assert!(found_alias == Some(1) || found_alias == Some(5)); // either is valid
    assert_eq!(tas.get(1), Some("topicA")); // original alias should still exist
    assert_eq!(tas.get(5), Some("topicA")); // new alias should also exist

    // Verify other mappings unchanged
    assert_eq!(tas.find_by_topic("topicB"), Some(2));
    assert_eq!(tas.find_by_topic("topicC"), Some(3));

    // Case 2: Same alias, different topic (topicD -> alias 2, overwriting topicB)
    tas.insert_or_update("topicD", 2);

    // Verify topicD took over alias 2
    assert_eq!(tas.find_by_topic("topicD"), Some(2));
    assert_eq!(tas.get(2), Some("topicD"));

    // Verify topicB is no longer accessible
    assert_eq!(tas.find_by_topic("topicB"), None);

    // Verify other mappings unchanged
    let found_alias_a = tas.find_by_topic("topicA");
    assert!(found_alias_a == Some(1) || found_alias_a == Some(5)); // either is valid
    assert_eq!(tas.find_by_topic("topicC"), Some(3));

    // Case 3: Cross update (topicC to alias that was used by topicA)
    tas.insert_or_update("topicC", 1); // Use alias 1 (shared with topicA)

    // Verify topicC now also has alias 1
    let found_alias_c = tas.find_by_topic("topicC");
    assert!(found_alias_c == Some(1) || found_alias_c == Some(3)); // either is valid
    assert_eq!(tas.get(1), Some("topicC")); // alias 1 should now have topicC

    // Verify old alias 3 still exists with topicC
    assert_eq!(tas.get(3), Some("topicC"));

    // Final state verification - adjust expectations for multiple aliases
    let found_alias_a_final = tas.find_by_topic("topicA");
    assert!(found_alias_a_final == Some(5)); // topicA should still be findable
    assert_eq!(tas.find_by_topic("topicD"), Some(2));
    let found_alias_c_final = tas.find_by_topic("topicC");
    assert!(found_alias_c_final == Some(1) || found_alias_c_final == Some(3));
    assert_eq!(tas.find_by_topic("topicB"), None); // Should be gone

    assert_eq!(tas.get(1), Some("topicC"));
    assert_eq!(tas.get(2), Some("topicD"));
    assert_eq!(tas.get(3), Some("topicC")); // Should have topicC
    assert_eq!(tas.get(5), Some("topicA"));
}

#[test]
fn test_mqtt_v5_spec_multiple_aliases_same_topic() {
    common::init_tracing();
    // Test MQTT v5.0 spec: same topic can be mapped to multiple aliases
    let mut tas = mqtt::packet::TopicAliasSend::new(10);

    // Map alias 1 to "topic1"
    tas.insert_or_update("topic1", 1);
    assert_eq!(tas.find_by_topic("topic1"), Some(1));
    assert_eq!(tas.get(1), Some("topic1"));

    // Map alias 2 to "topic1" - both should be maintained
    tas.insert_or_update("topic1", 2);
    assert_eq!(tas.get(1), Some("topic1"));
    assert_eq!(tas.get(2), Some("topic1"));

    // find_by_topic should return the first alias found
    let found_alias = tas.find_by_topic("topic1");
    assert!(found_alias == Some(1) || found_alias == Some(2));

    // Map alias 3 to "topic1" - all three should be maintained
    tas.insert_or_update("topic1", 3);
    assert_eq!(tas.get(1), Some("topic1"));
    assert_eq!(tas.get(2), Some("topic1"));
    assert_eq!(tas.get(3), Some("topic1"));

    // Update alias 1 to different topic - only alias 1 mapping should change
    tas.insert_or_update("topic2", 1);
    assert_eq!(tas.get(1), Some("topic2"));
    assert_eq!(tas.get(2), Some("topic1"));
    assert_eq!(tas.get(3), Some("topic1"));

    // topic1 should still be findable via remaining aliases
    let found_alias = tas.find_by_topic("topic1");
    assert!(found_alias == Some(2) || found_alias == Some(3));

    // topic2 should be findable
    assert_eq!(tas.find_by_topic("topic2"), Some(1));
}
