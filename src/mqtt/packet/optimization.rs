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

/// Stack buffer optimization configuration for MQTT data types
///
/// This trait provides a centralized way to configure stack buffer sizes for
/// MqttString, MqttBinary, and ArcPayload types. It allows compile-time
/// customization of Small Buffer Optimization (SBO) parameters across
/// the entire MQTT protocol implementation.
///
/// # Type Parameters
///
/// * `STRING_BUFFER_SIZE` - Stack buffer size for MqttString (default: 32 bytes)
/// * `BINARY_BUFFER_SIZE` - Stack buffer size for MqttBinary (default: 32 bytes)
/// * `PAYLOAD_BUFFER_SIZE` - Stack buffer size for ArcPayload (default: 32 bytes)
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::packet::StackOptimization;
///
/// // Use default optimization (32 bytes for all types)
/// type DefaultOpt = mqtt::packet::DefaultStackOptimization;
///
/// // Custom optimization with different buffer sizes
/// struct CustomOpt;
/// impl StackOptimization for CustomOpt {
///     const STRING_BUFFER_SIZE: usize = 64;
///     const BINARY_BUFFER_SIZE: usize = 128;
///     const PAYLOAD_BUFFER_SIZE: usize = 64;
/// }
/// ```
pub trait StackOptimization {
    /// Stack buffer size for MqttString in bytes
    const STRING_BUFFER_SIZE: usize = 32;

    /// Stack buffer size for MqttBinary in bytes
    const BINARY_BUFFER_SIZE: usize = 32;

    /// Stack buffer size for ArcPayload in bytes
    const PAYLOAD_BUFFER_SIZE: usize = 32;
}

/// Default stack optimization configuration
///
/// Provides 32-byte stack buffers for all three types (MqttString, MqttBinary, ArcPayload).
/// This is the recommended configuration for most MQTT applications, providing a good
/// balance between stack usage and heap allocation avoidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultStackOptimization;

impl StackOptimization for DefaultStackOptimization {
    const STRING_BUFFER_SIZE: usize = 32;
    const BINARY_BUFFER_SIZE: usize = 32;
    const PAYLOAD_BUFFER_SIZE: usize = 32;
}

/// High-performance stack optimization configuration
///
/// Provides larger stack buffers (64 bytes) for all types to minimize heap allocations
/// in applications with larger typical message sizes. Use this when you have more
/// stack space available and want to avoid heap allocations for larger messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighPerformanceStackOptimization;

impl StackOptimization for HighPerformanceStackOptimization {
    const STRING_BUFFER_SIZE: usize = 64;
    const BINARY_BUFFER_SIZE: usize = 64;
    const PAYLOAD_BUFFER_SIZE: usize = 64;
}

/// Low-memory stack optimization configuration
///
/// Provides smaller stack buffers (16 bytes) for all types to minimize stack usage
/// in memory-constrained environments. Use this in embedded or resource-limited
/// applications where stack space is precious.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LowMemoryStackOptimization;

impl StackOptimization for LowMemoryStackOptimization {
    const STRING_BUFFER_SIZE: usize = 16;
    const BINARY_BUFFER_SIZE: usize = 16;
    const PAYLOAD_BUFFER_SIZE: usize = 16;
}

/// Minimal stack optimization configuration
///
/// Provides very small stack buffers (8 bytes) for all types. This is primarily
/// intended for testing or extremely memory-constrained environments where even
/// small stack allocations matter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinimalStackOptimization;

impl StackOptimization for MinimalStackOptimization {
    const STRING_BUFFER_SIZE: usize = 8;
    const BINARY_BUFFER_SIZE: usize = 8;
    const PAYLOAD_BUFFER_SIZE: usize = 8;
}

/// Large buffer stack optimization configuration
///
/// Provides very large stack buffers (128 bytes) for all types to handle large
/// messages efficiently. Use this when you have abundant stack space and frequently
/// work with large MQTT messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LargeBufferStackOptimization;

impl StackOptimization for LargeBufferStackOptimization {
    const STRING_BUFFER_SIZE: usize = 128;
    const BINARY_BUFFER_SIZE: usize = 128;
    const PAYLOAD_BUFFER_SIZE: usize = 128;
}
