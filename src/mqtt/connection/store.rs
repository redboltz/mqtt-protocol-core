use crate::mqtt::packet::GenericStorePacket;
use crate::mqtt::packet::ResponsePacket;
use crate::mqtt::result_code::MqttError;
use crate::mqtt::packet::IsPacketId;
use indexmap::IndexMap;
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

/// A store that holds packets in insertion order and allows O(1) insert/remove by id.
pub struct GenericStore<PacketIdType: IsPacketId> {
    map: IndexMap<PacketIdType, GenericStorePacket<PacketIdType>>,
}

pub type Store = GenericStore<u16>;

impl<PacketIdType: IsPacketId> GenericStore<PacketIdType> {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
        }
    }

    /// Add a packet to the store.
    /// Returns true if inserted, false if a packet with same id already exists.
    pub fn add(&mut self, packet: GenericStorePacket<PacketIdType>) -> Result<(), MqttError> {
        let id = packet.packet_id();
        if self.map.contains_key(&id) {
            return Err(MqttError::PacketIdentifierConflict);
        }
        self.map.insert(id, packet);
        Ok(())
    }

    /// Erase a packet by its response type and packet id.
    /// Returns true if removed, false otherwise.
    pub fn erase(&mut self, response: ResponsePacket, packet_id: PacketIdType) -> bool {
        if let Some((index, _, pkt)) = self.map.get_full(&packet_id) {
            if pkt.response_packet() == response {
                self.map.shift_remove_index(index);
                return true;
            }
        }
        false
    }
    /// Erase a publish packet by packet id only.
    /// Returns true if removed, false otherwise.
    pub fn erase_publish(&mut self, packet_id: PacketIdType) -> bool {
        if let Some((index, _, pkt)) = self.map.get_full(&packet_id) {
            if matches!(
                pkt.response_packet(),
                ResponsePacket::V3_1_1Puback
                    | ResponsePacket::V3_1_1Pubrec
                    | ResponsePacket::V5_0Puback
                    | ResponsePacket::V5_0Pubrec
            ) {
                self.map.shift_remove_index(index);
                return true;
            }
        }
        false
    }

    /// Clear all stored packets.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Iterate over packets in insertion order.
    /// The provided function returns true to keep the packet, or false to remove it.
    pub fn for_each<F>(&mut self, mut func: F)
    where
        F: FnMut(&GenericStorePacket<PacketIdType>) -> bool,
    {
        let mut to_remove = Vec::new();
        for (id, pkt) in &self.map {
            if !func(pkt) {
                to_remove.push(*id);
            }
        }
        for id in to_remove {
            self.map.shift_remove(&id);
            println!("[store] removed pid: {:?}", id);
        }
    }

    /// Return a vector of all stored packets in insertion order.
    pub fn get_stored(&self) -> Vec<GenericStorePacket<PacketIdType>> {
        self.map.values().cloned().collect()
    }
}
