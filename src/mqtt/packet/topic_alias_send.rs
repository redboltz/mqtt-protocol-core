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
use std::collections::HashMap;

use indexmap::IndexMap;
use tracing::trace;

use crate::mqtt::ValueAllocator;

/// Type alias for topic alias values
pub type TopicAliasType = u16;

/// Topic alias manager for sending MQTT packets
///
/// This manages the mapping between topic names and numeric aliases for outgoing
/// MQTT PUBLISH packets to reduce packet size for frequently used topics.
///
/// According to MQTT v5.0 specification, one topic can have multiple aliases.
pub struct TopicAliasSend {
    max_alias: TopicAliasType,
    // alias -> topic mapping with insertion order preserved
    alias_to_topic: IndexMap<TopicAliasType, String>,
    // topic -> aliases mapping for fast topic lookups (supports multiple aliases per topic)
    topic_to_aliases: HashMap<String, Vec<TopicAliasType>>,
    value_allocator: ValueAllocator<TopicAliasType>,
}

impl TopicAliasSend {
    const MIN_ALIAS: TopicAliasType = 1;

    /// Create a new TopicAliasSend with the specified maximum alias value
    pub fn new(max_alias: TopicAliasType) -> Self {
        trace!("Creating TopicAliasSend with max_alias: {}", max_alias);

        Self {
            max_alias,
            alias_to_topic: IndexMap::new(),
            topic_to_aliases: HashMap::new(),
            value_allocator: ValueAllocator::new(Self::MIN_ALIAS, max_alias),
        }
    }

    /// Insert or update a topic-alias mapping
    ///
    /// # Arguments
    /// * `topic` - The topic name (must not be empty)
    /// * `alias` - The alias value (must be between MIN_ALIAS and max_alias)
    ///
    /// # Panics
    /// Panics if topic is empty or alias is out of valid range
    pub fn insert_or_update(&mut self, topic: &str, alias: TopicAliasType) {
        trace!("TopicAliasSend insert topic: '{}', alias: {}", topic, alias);

        assert!(!topic.is_empty() && alias >= Self::MIN_ALIAS && alias <= self.max_alias);

        let topic_string = topic.to_string();

        // Check if this is a new alias allocation or update of existing alias
        let is_new_alias = self.value_allocator.use_value(alias);

        if !is_new_alias {
            // Alias already in use: need to remove old alias->topic mapping
            if let Some(old_topic) = self.alias_to_topic.shift_remove(&alias) {
                // Remove this alias from the old topic's aliases list
                if let Some(aliases) = self.topic_to_aliases.get_mut(&old_topic) {
                    aliases.retain(|&a| a != alias);
                    if aliases.is_empty() {
                        self.topic_to_aliases.remove(&old_topic);
                    }
                }
            }
        }

        // Insert new alias -> topic mapping
        self.alias_to_topic.insert(alias, topic_string.clone());

        // Add alias to topic's aliases list (or create new list)
        self.topic_to_aliases
            .entry(topic_string)
            .or_insert_with(Vec::new)
            .push(alias);
    }

    /// Get topic by alias and update access timestamp (affects LRU)
    ///
    /// # Arguments
    /// * `alias` - The alias to look up
    ///
    /// # Returns
    /// The topic name if found, None otherwise
    pub fn get(&mut self, alias: TopicAliasType) -> Option<&str> {
        trace!("Getting topic by alias: {}", alias);

        if alias >= Self::MIN_ALIAS && alias <= self.max_alias {
            if let Some(topic) = self.alias_to_topic.get(&alias).cloned() {
                // Move to end for LRU tracking (remove and re-insert)
                self.alias_to_topic.shift_remove(&alias);
                self.alias_to_topic.insert(alias, topic);
                return Some(self.alias_to_topic.get(&alias).unwrap());
            }
        }
        None
    }

    /// Peek topic by alias without updating access timestamp (does not affect LRU)
    ///
    /// # Arguments
    /// * `alias` - The alias to look up
    ///
    /// # Returns
    /// The topic name if found, None otherwise
    pub fn peek(&self, alias: TopicAliasType) -> Option<&str> {
        trace!("Peeking topic by alias (no touch): {}", alias);

        if alias >= Self::MIN_ALIAS && alias <= self.max_alias {
            if let Some(topic) = self.alias_to_topic.get(&alias) {
                return Some(topic);
            }
        }
        None
    }

    /// Find alias by topic name
    ///
    /// # Arguments
    /// * `topic` - The topic name to look up
    ///
    /// # Returns
    /// The first alias if found, None otherwise
    pub fn find_by_topic(&self, topic: &str) -> Option<TopicAliasType> {
        trace!("Finding alias by topic: '{}'", topic);

        self.topic_to_aliases
            .get(topic)
            .and_then(|aliases| aliases.first().copied())
    }

    /// Clear all topic-alias mappings
    pub fn clear(&mut self) {
        trace!("Clearing all topic aliases");
        self.alias_to_topic.clear();
        self.topic_to_aliases.clear();
        self.value_allocator.clear();
    }

    /// Get the least recently used (LRU) alias
    ///
    /// Returns either the first vacant alias or the oldest used alias
    ///
    /// # Returns
    /// An alias value that can be reused
    ///
    /// # Panics
    /// Panics if max_alias is 0
    pub fn get_lru_alias(&self) -> TopicAliasType {
        assert!(self.max_alias > 0);

        // First try to get a vacant alias
        if let Some(alias) = self.value_allocator.first_vacant() {
            return alias;
        }

        // If no vacant aliases, return the least recently used one (first in IndexMap)
        self.alias_to_topic
            .keys()
            .next()
            .copied()
            .unwrap_or(Self::MIN_ALIAS)
    }

    /// Get the maximum alias value
    pub fn max(&self) -> TopicAliasType {
        self.max_alias
    }
}
