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

use crate::mqtt_internal::packet::IsPacketId;
use crate::mqtt_internal::result_code::MqttError;
use crate::mqtt_internal::ValueAllocator;

pub struct PacketIdManager<T>
where
    T: IsPacketId,
{
    allocator: ValueAllocator<T>,
}

impl<T> PacketIdManager<T>
where
    T: IsPacketId,
{
    /// Create a new packet ID manager with valid IDs in range [1, T::max_value()]
    pub fn new() -> Self {
        Self {
            allocator: ValueAllocator::new(T::one(), T::max_value()),
        }
    }

    /// Acquire a new unique packet ID.
    /// Returns `Ok(T)` if successful, `Err(MqttError)` if no IDs are available.
    pub fn acquire_unique_id(&mut self) -> Result<T, MqttError> {
        self.allocator
            .allocate()
            .ok_or(MqttError::PacketIdentifierFullyUsed)
    }

    /// Register a packet ID externally acquired or reused.
    /// Returns `Ok(())` if successful, `Err(MqttError)` if the ID is already in use.
    pub fn register_id(&mut self, packet_id: T) -> Result<(), MqttError> {
        self.allocator
            .use_value(packet_id)
            .then_some(())
            .ok_or(MqttError::PacketIdentifierConflict)
    }

    /// Check whether a packet ID is in use.
    pub fn is_used_id(&self, packet_id: T) -> bool {
        self.allocator.is_used(packet_id)
    }

    /// Release a previously acquired or registered packet ID.
    pub fn release_id(&mut self, packet_id: T) {
        self.allocator.deallocate(packet_id);
    }

    /// Clear all state: all packet IDs become available again.
    pub fn clear(&mut self) {
        self.allocator.clear();
    }
}
