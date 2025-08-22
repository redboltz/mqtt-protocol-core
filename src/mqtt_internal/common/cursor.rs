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

/// Errors that can occur when reading from a cursor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorError {
    /// Attempted to read beyond the end of the data
    UnexpectedEof,
}

/// A cursor which wraps an in-memory buffer and provides positioned reading
///
/// `Cursor` is a simple wrapper that allows reading from various in-memory
/// data types (like `&[u8]`) with a tracked position. This is useful for
/// parsing protocols where you need to read sequentially through a buffer.
///
/// Unlike `std::io::Cursor`, this implementation is designed for `no_std`
/// environments and focuses on reading operations only.
///
/// # Examples
///
/// ```ignore
/// mqtt_protocol_core::make_default_aliases!();
/// use mqtt::common::Cursor;
///
/// let data = &b"hello world"[..];
/// let mut cursor = Cursor::new(data);
///
/// // Read individual bytes
/// assert_eq!(cursor.read_u8(), Some(b'h'));
/// assert_eq!(cursor.position(), 1);
///
/// // Read multiple bytes at once
/// let chunk = cursor.read_bytes(5).unwrap();
/// assert_eq!(chunk, b"ello ");
/// assert_eq!(cursor.position(), 6);
/// ```ignore
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}

impl<T> Cursor<T> {
    /// Creates a new cursor with the provided data
    ///
    /// The cursor starts at position 0.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let cursor = Cursor::new(&b"hello"[..]);
    /// assert_eq!(cursor.position(), 0);
    /// ```
    #[inline]
    pub fn new(inner: T) -> Self {
        Cursor { inner, pos: 0 }
    }

    /// Returns the current position of the cursor
    ///
    /// # Returns
    ///
    /// The current position as a `u64` value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let mut cursor = Cursor::new(&b"hello"[..]);
    /// assert_eq!(cursor.position(), 0);
    ///
    /// cursor.set_position(3);
    /// assert_eq!(cursor.position(), 3);
    /// ```ignore
    #[inline]
    pub fn position(&self) -> u64 {
        self.pos
    }

    /// Sets the position of the cursor
    ///
    /// # Parameters
    ///
    /// * `pos` - New position for the cursor
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let mut cursor = Cursor::new(&b"hello"[..]);
    /// cursor.set_position(3);
    /// assert_eq!(cursor.position(), 3);
    /// ```ignore
    #[inline]
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }

    /// Gets a reference to the underlying value
    ///
    /// # Returns
    ///
    /// A reference to the underlying data of type `&T`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let data = &b"hello"[..];
    /// let cursor = Cursor::new(data);
    /// assert_eq!(cursor.get_ref(), &data);
    /// ```ignore
    #[inline]
    pub fn get_ref(&self) -> &T {
        &self.inner
    }
}

impl Cursor<&[u8]> {
    /// Returns a slice of the remaining unread data
    ///
    /// This returns all data from the current position to the end of the buffer.
    /// If the position is beyond the buffer length, returns an empty slice.
    ///
    /// # Returns
    ///
    /// * `&[u8]` - Slice containing all unread data from current position to end
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let mut cursor = Cursor::new(&b"hello"[..]);
    /// cursor.set_position(2);
    /// assert_eq!(cursor.remaining_slice(), b"llo");
    /// ```ignore
    #[inline]
    pub fn remaining_slice(&self) -> &[u8] {
        let pos = self.pos as usize;
        if pos <= self.inner.len() {
            &self.inner[pos..]
        } else {
            &[]
        }
    }

    /// Reads exactly `count` bytes from the cursor
    ///
    /// Advances the cursor position by `count` bytes and returns a slice
    /// to the read data. Returns `None` if there are not enough bytes
    /// remaining to satisfy the request.
    ///
    /// # Parameters
    ///
    /// * `count` - Number of bytes to read
    ///
    /// # Returns
    ///
    /// * `Some(&[u8])` - Slice containing exactly `count` bytes
    /// * `None` - Not enough data available to read `count` bytes
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let mut cursor = Cursor::new(&b"hello world"[..]);
    /// assert_eq!(cursor.read_bytes(5), Some(&b"hello"[..]));
    /// assert_eq!(cursor.position(), 5);
    /// assert_eq!(cursor.read_bytes(20), None); // Not enough data
    /// ```ignore
    #[inline]
    pub fn read_bytes(&mut self, count: usize) -> Option<&[u8]> {
        let pos = self.pos as usize;
        if pos.saturating_add(count) <= self.inner.len() {
            let data = &self.inner[pos..pos + count];
            self.pos += count as u64;
            Some(data)
        } else {
            None
        }
    }

    /// Reads a single byte from the cursor
    ///
    /// Advances the cursor position by 1 byte and returns the byte value.
    /// Returns `None` if there are no more bytes to read.
    ///
    /// # Returns
    ///
    /// * `Some(u8)` - The next byte from the cursor
    /// * `None` - No more bytes available to read
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let mut cursor = Cursor::new(&b"hi"[..]);
    /// assert_eq!(cursor.read_u8(), Some(b'h'));
    /// assert_eq!(cursor.read_u8(), Some(b'i'));
    /// assert_eq!(cursor.read_u8(), None); // End of data
    /// ```ignore
    #[inline]
    pub fn read_u8(&mut self) -> Option<u8> {
        let pos = self.pos as usize;
        if pos < self.inner.len() {
            let val = self.inner[pos];
            self.pos += 1;
            Some(val)
        } else {
            None
        }
    }
}

impl<T: AsRef<[u8]>> Cursor<T> {
    /// Pulls some bytes from this cursor into the specified buffer
    ///
    /// This method is compatible with `std::io::Read::read()`. It reads at most
    /// `buf.len()` bytes from the cursor into the provided buffer, advancing
    /// the cursor position accordingly.
    ///
    /// # Parameters
    ///
    /// * `buf` - Buffer to read data into
    ///
    /// # Returns
    ///
    /// * `Ok(n)` - Number of bytes read (0 to `buf.len()`)
    ///
    /// This method currently never returns an error, but returns a `Result`
    /// for compatibility with `std::io::Read::read()`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::Cursor;
    ///
    /// let mut cursor = Cursor::new(&b"hello world"[..]);
    /// let mut buf = [0u8; 5];
    /// let n = cursor.read(&mut buf).unwrap();
    /// assert_eq!(n, 5);
    /// assert_eq!(&buf, b"hello");
    /// assert_eq!(cursor.position(), 5);
    /// ```ignore
    #[inline]
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, CursorError> {
        let pos = self.pos as usize;
        let available = self.inner.as_ref().len().saturating_sub(pos);
        let to_read = core::cmp::min(buf.len(), available);

        if to_read > 0 {
            buf[..to_read].copy_from_slice(&self.inner.as_ref()[pos..pos + to_read]);
            self.pos += to_read as u64;
        }

        Ok(to_read)
    }

    /// Reads the exact number of bytes required to fill `buf`
    ///
    /// This method is compatible with `std::io::Read::read_exact()`. It reads
    /// exactly `buf.len()` bytes from the cursor into the buffer, or returns
    /// an error if not enough data is available.
    ///
    /// # Parameters
    ///
    /// * `buf` - Buffer to read data into (must be completely filled)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully read exactly `buf.len()` bytes
    /// * `Err(CursorError::UnexpectedEof)` - Not enough data available
    ///
    /// # Examples
    ///
    /// ```ignore
    /// mqtt_protocol_core::make_default_aliases!();
    /// use mqtt::common::{Cursor, CursorError};
    ///
    /// let mut cursor = Cursor::new(&b"hello"[..]);
    /// let mut buf = [0u8; 3];
    /// cursor.read_exact(&mut buf).unwrap();
    /// assert_eq!(&buf, b"hel");
    ///
    /// // Trying to read more than available
    /// let mut buf2 = [0u8; 10];
    /// assert_eq!(cursor.read_exact(&mut buf2), Err(CursorError::UnexpectedEof));
    /// ```ignore
    #[inline]
    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), CursorError> {
        let pos = self.pos as usize;
        let available = self.inner.as_ref().len().saturating_sub(pos);

        if available < buf.len() {
            return Err(CursorError::UnexpectedEof);
        }

        buf.copy_from_slice(&self.inner.as_ref()[pos..pos + buf.len()]);
        self.pos += buf.len() as u64;
        Ok(())
    }
}
