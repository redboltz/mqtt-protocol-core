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
mod arc_payload;
pub use arc_payload::{ArcPayload, IntoPayload};

mod value_allocator;
pub use value_allocator::ValueAllocator;

mod cursor;
pub use cursor::Cursor;
pub use cursor::CursorError;

pub(crate) mod tracing;

/// Type alias for HashSet to provide a stable API abstraction over the underlying hash set implementation.
///
/// This alias allows the library to use a high-performance hash set implementation
/// (currently hashbrown::HashSet) while providing API stability to users. It also
/// ensures consistent usage across the entire codebase.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::common::HashSet;
///
/// let mut set = HashSet::new();
/// set.insert("key");
/// ```
pub type HashSet<T> = hashbrown::HashSet<T>;

/// Type alias for HashMap to provide a stable API abstraction over the underlying hash map implementation.
///
/// This alias allows the library to use a high-performance hash map implementation
/// (currently hashbrown::HashMap) while providing API stability to users. It also
/// ensures consistent usage across the entire codebase.
///
/// # Examples
///
/// ```ignore
/// use mqtt_protocol_core::mqtt::common::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("key", "value");
/// ```
pub type HashMap<K, V> = hashbrown::HashMap<K, V>;
