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

mod common;
use common::mqtt;
use mqtt::ValueAllocator;

#[test]
fn one() {
    common::init_tracing();
    let mut a = ValueAllocator::new(0usize, 0);
    assert_eq!(a.interval_count(), 1);

    if let Some(value) = a.allocate() {
        assert_eq!(value, 0);
        assert_eq!(a.interval_count(), 0);
    } else {
        panic!("expected Some(0)");
    }

    assert!(a.allocate().is_none());

    a.deallocate(0);
    assert_eq!(a.interval_count(), 1);

    if let Some(value) = a.allocate() {
        assert_eq!(value, 0);
        assert_eq!(a.interval_count(), 0);
    } else {
        panic!("expected Some(0)");
    }

    assert!(a.allocate().is_none());
    assert_eq!(a.use_value(0), false);
    assert_eq!(a.use_value(1), false);
    a.deallocate(0);
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.use_value(0), true);
    assert_eq!(a.interval_count(), 0);
    assert_eq!(a.use_value(1), false);
    assert!(a.allocate().is_none());
    a.deallocate(0);
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(0));
    assert_eq!(a.interval_count(), 0);
}

#[test]
fn offset() {
    common::init_tracing();
    let mut a = ValueAllocator::new(5usize, 5);
    assert_eq!(a.allocate(), Some(5));
    assert!(a.allocate().is_none());
    a.deallocate(5);
    assert_eq!(a.allocate(), Some(5));
    assert!(a.allocate().is_none());
    assert_eq!(a.use_value(5), false);
    assert_eq!(a.use_value(1), false);
    a.deallocate(5);
    assert_eq!(a.use_value(5), true);
    assert_eq!(a.use_value(1), false);
    assert!(a.allocate().is_none());
    a.deallocate(5);
    assert_eq!(a.allocate(), Some(5));
}

#[test]
fn allocate() {
    common::init_tracing();
    let mut a = ValueAllocator::new(0usize, 4);
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(0));
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(1));
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(2));
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(3));
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(4));
    assert_eq!(a.interval_count(), 0);
    assert!(a.allocate().is_none());

    a.deallocate(2);
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(2));
    assert_eq!(a.interval_count(), 0);
}

#[test]
fn use_value() {
    common::init_tracing();
    let mut a = ValueAllocator::new(0usize, 4);
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(1));
    assert_eq!(a.interval_count(), 2);
    assert!(a.use_value(3));
    assert_eq!(a.interval_count(), 3);
    assert!(a.use_value(2));
    assert_eq!(a.interval_count(), 2);
    assert!(a.use_value(0));
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(4));
    assert_eq!(a.interval_count(), 0);
    assert!(!a.use_value(0));
    assert!(!a.use_value(1));
    assert!(!a.use_value(2));
    assert!(!a.use_value(3));
    assert!(!a.use_value(4));
    a.deallocate(2);
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(2));
    assert_eq!(a.interval_count(), 0);
}

#[test]
fn clear() {
    common::init_tracing();
    let mut a = ValueAllocator::new(0usize, 4);
    assert_eq!(a.allocate(), Some(0));
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(1));
    assert_eq!(a.allocate(), Some(2));
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(3));
    assert_eq!(a.allocate(), Some(4));
    assert_eq!(a.interval_count(), 0);

    a.clear();
    assert_eq!(a.interval_count(), 1);
    assert_eq!(a.allocate(), Some(0));
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(1));
    assert_eq!(a.allocate(), Some(2));
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(3));
    assert_eq!(a.allocate(), Some(4));
    assert_eq!(a.interval_count(), 0);
}

#[test]
fn interval_management() {
    common::init_tracing();
    let mut a = ValueAllocator::new(0usize, 4);
    assert!(a.use_value(0));
    assert!(a.use_value(1));
    assert!(a.use_value(2));
    assert!(a.use_value(3));
    assert!(a.use_value(4));

    {
        let mut ca = a.clone();
        ca.deallocate(0);
        assert_eq!(ca.interval_count(), 1);
        ca.deallocate(4);
        assert_eq!(ca.interval_count(), 2);
        ca.deallocate(2);
        assert_eq!(ca.interval_count(), 3);
        ca.deallocate(1);
        assert_eq!(ca.interval_count(), 2);
        ca.deallocate(3);
        assert_eq!(ca.interval_count(), 1);
    }
    {
        let mut ca = a.clone();
        ca.deallocate(3);
        assert_eq!(ca.interval_count(), 1);
        ca.deallocate(0);
        assert_eq!(ca.interval_count(), 2);
        ca.deallocate(4);
        assert_eq!(ca.interval_count(), 2);
    }
    {
        let mut ca = a.clone();
        ca.deallocate(3);
        assert_eq!(ca.interval_count(), 1);
        ca.deallocate(2);
        assert_eq!(ca.interval_count(), 1);
    }
    {
        let mut ca = a.clone();
        ca.deallocate(0);
        assert_eq!(ca.interval_count(), 1);
        ca.deallocate(4);
        assert_eq!(ca.interval_count(), 2);
        ca.deallocate(3);
        assert_eq!(ca.interval_count(), 2);
        ca.deallocate(1);
        assert_eq!(ca.interval_count(), 2);
    }
    {
        let mut ca = a.clone();
        ca.deallocate(2);
        assert_eq!(ca.interval_count(), 1);
        ca.deallocate(1);
        assert_eq!(ca.interval_count(), 1);
    }
}

#[test]
fn signed_value() {
    common::init_tracing();
    let mut a = ValueAllocator::new(-2, 3);
    assert_eq!(a.interval_count(), 1);
    assert!(a.use_value(2));
    assert_eq!(a.interval_count(), 2);
    assert_eq!(a.allocate(), Some(-2));
    assert_eq!(a.interval_count(), 2);
    assert!(a.use_value(0));
    assert_eq!(a.interval_count(), 3);
}
