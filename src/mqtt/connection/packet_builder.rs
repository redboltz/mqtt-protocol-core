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
use crate::mqtt::common::Cursor;
use crate::mqtt::result_code::MqttError;
use alloc::{sync::Arc, vec::Vec};

#[derive(Debug, Clone)]
pub enum PacketData {
    Normal(Vec<u8>),
    Publish(Arc<[u8]>),
}

impl PacketData {
    pub fn as_slice(&self) -> &[u8] {
        match self {
            PacketData::Normal(vec) => vec.as_slice(),
            PacketData::Publish(arc) => arc.as_ref(),
        }
    }

    pub fn len(&self) -> u32 {
        match self {
            PacketData::Normal(vec) => vec.len().try_into().unwrap(),
            PacketData::Publish(arc) => arc.len().try_into().unwrap(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone)]
pub struct RawPacket {
    fixed_header: u8,
    pub data: PacketData,
}

impl RawPacket {
    pub fn data_as_slice(&self) -> &[u8] {
        self.data.as_slice()
    }

    pub fn packet_type(&self) -> u8 {
        self.fixed_header >> 4
    }

    pub fn flags(&self) -> u8 {
        self.fixed_header & 0x0F
    }

    pub fn is_publish(&self) -> bool {
        self.packet_type() == 3
    }

    pub fn remaining_length(&self) -> u32 {
        self.data.len()
    }
}

/// Enum representing packet construction results
#[derive(Debug)]
pub enum PacketBuildResult {
    /// Packet construction completed
    Complete(RawPacket),
    /// Packet building in progress (more data needed)
    Incomplete,
    /// Error occurred
    Error(MqttError),
}

/// Builder for constructing MQTT packet byte sequences
pub struct PacketBuilder {
    /// Current read state
    state: ReadState,
    /// Buffer for header and remaining length
    header_buf: Vec<u8>,
    /// Remaining length
    remaining_length: usize,
    /// Multiplier for variable-length integer decoding
    multiplier: u32,
    /// Buffer for entire packet
    raw_buf: Option<Vec<u8>>,
    /// Current position in buffer
    raw_buf_offset: usize,
}

/// Packet reading state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReadState {
    /// Reading fixed header
    FixedHeader,
    /// Reading remaining length
    RemainingLength,
    /// Reading payload
    Payload,
}

impl PacketBuilder {
    /// Create new packet builder
    pub fn new() -> Self {
        Self {
            state: ReadState::FixedHeader,
            header_buf: Vec::with_capacity(5),
            remaining_length: 0,
            multiplier: 1,
            raw_buf: None,
            raw_buf_offset: 0,
        }
    }

    /// Reset builder for reuse
    pub fn reset(&mut self) {
        self.state = ReadState::FixedHeader;
        self.header_buf.clear();
        self.remaining_length = 0;
        self.multiplier = 1;
        self.raw_buf = None;
        self.raw_buf_offset = 0;
    }

    /// Get packet type (first byte of fixed header)
    fn get_packet_type(&self) -> u8 {
        if !self.header_buf.is_empty() {
            self.header_buf[0]
        } else {
            0
        }
    }

    /// Determine if packet is PUBLISH
    fn is_publish_packet(&self) -> bool {
        (self.get_packet_type() & 0xF0) == 0x30
    }

    /// Build packet from data stream
    pub fn feed(&mut self, data: &mut Cursor<&[u8]>) -> PacketBuildResult {
        let available = data.get_ref().len() as u64 - data.position();
        if available == 0 {
            return PacketBuildResult::Incomplete;
        }

        let mut byte = [0u8; 1];

        loop {
            match self.state {
                ReadState::FixedHeader => {
                    if data.read_exact(&mut byte).is_err() {
                        return PacketBuildResult::Incomplete;
                    }

                    self.header_buf.push(byte[0]);
                    self.state = ReadState::RemainingLength;
                }

                ReadState::RemainingLength => {
                    if data.read_exact(&mut byte).is_err() {
                        return PacketBuildResult::Incomplete;
                    }

                    self.header_buf.push(byte[0]);
                    let encoded_byte = byte[0];

                    self.remaining_length +=
                        ((encoded_byte & 0x7F) as usize) * (self.multiplier as usize);
                    self.multiplier *= 128;

                    // Variable-length integer limit check
                    if self.multiplier > 128 * 128 * 128 {
                        self.reset();
                        return PacketBuildResult::Error(MqttError::MalformedPacket);
                    }

                    if (encoded_byte & 0x80) == 0 {
                        if self.remaining_length == 0 {
                            let fixed_header = self.header_buf[0];
                            let packet_data = if self.is_publish_packet() {
                                // Use Arc for PUBLISH packets
                                PacketData::Publish(Arc::from([]))
                            } else {
                                // Use Vec for other packets
                                PacketData::Normal(Vec::new())
                            };

                            let packet = RawPacket {
                                fixed_header,
                                data: packet_data,
                            };
                            self.reset();
                            return PacketBuildResult::Complete(packet);
                        } else {
                            self.raw_buf = Some(Vec::with_capacity(self.remaining_length));
                            self.raw_buf_offset = 0;
                            self.state = ReadState::Payload;
                        }
                    }
                }

                ReadState::Payload => {
                    let raw_buf = self.raw_buf.as_mut().unwrap();
                    let bytes_remaining = self.remaining_length;

                    let position = data.position();
                    let available = data.get_ref().len() as u64 - position;
                    let bytes_to_read = bytes_remaining.min(available as usize);

                    if bytes_to_read == 0 {
                        return PacketBuildResult::Incomplete;
                    }

                    raw_buf.resize(self.raw_buf_offset + bytes_to_read, 0);

                    let read_slice =
                        &mut raw_buf[self.raw_buf_offset..self.raw_buf_offset + bytes_to_read];
                    let bytes_read = data.read(read_slice).unwrap();

                    self.raw_buf_offset += bytes_read;
                    self.remaining_length -= bytes_read;

                    if self.remaining_length == 0 {
                        let raw_buf = self.raw_buf.take().unwrap();
                        let fixed_header = self.header_buf[0];

                        let packet_data = if self.is_publish_packet() {
                            // Use Arc for PUBLISH packets
                            PacketData::Publish(Arc::from(raw_buf.into_boxed_slice()))
                        } else {
                            // Use Vec for other packets
                            PacketData::Normal(raw_buf)
                        };

                        let packet = RawPacket {
                            fixed_header,
                            data: packet_data,
                        };
                        self.reset();
                        return PacketBuildResult::Complete(packet);
                    }
                    return PacketBuildResult::Incomplete;
                }
            }
        }
    }
}
