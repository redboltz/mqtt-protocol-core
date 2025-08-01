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

use tracing::trace;

/// Type alias for topic alias values
pub type TopicAliasType = u16;

/// Topic alias manager for receiving MQTT packets
///
/// This manages the mapping between numeric aliases and topic names for incoming
/// MQTT PUBLISH packets to reduce packet size for frequently used topics.
pub struct TopicAliasRecv {
    max_alias: TopicAliasType,
    aliases: HashMap<TopicAliasType, String>,
}

impl TopicAliasRecv {
    const MIN_ALIAS: TopicAliasType = 1;

    /// Create a new TopicAliasRecv with the specified maximum alias value
    pub fn new(max_alias: TopicAliasType) -> Self {
        trace!("Creating TopicAliasRecv with max_alias: {}", max_alias);

        Self {
            max_alias,
            aliases: HashMap::new(),
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
        trace!("TopicAliasRecv insert topic: '{}', alias: {}", topic, alias);

        assert!(!topic.is_empty() && alias >= Self::MIN_ALIAS && alias <= self.max_alias);

        self.aliases.insert(alias, topic.to_string());
    }

    /// Get topic by alias
    ///
    /// # Arguments
    /// * `alias` - The alias to look up
    ///
    /// # Returns
    /// The topic name if found, None otherwise
    pub fn get(&self, alias: TopicAliasType) -> Option<&str> {
        trace!("Getting topic by alias: {}", alias);

        if alias >= Self::MIN_ALIAS && alias <= self.max_alias {
            self.aliases.get(&alias).map(|s| s.as_str())
        } else {
            None
        }
    }

    /// Peek topic by alias (alias for get, for consistency with TopicAliasSend)
    ///
    /// # Arguments
    /// * `alias` - The alias to look up
    ///
    /// # Returns
    /// The topic name if found, None otherwise
    pub fn peek(&self, alias: TopicAliasType) -> Option<&str> {
        self.get(alias)
    }

    /// Clear all topic-alias mappings
    pub fn clear(&mut self) {
        trace!("Clearing all topic aliases");
        self.aliases.clear();
    }

    /// Get the maximum alias value
    pub fn max(&self) -> TopicAliasType {
        self.max_alias
    }
}
