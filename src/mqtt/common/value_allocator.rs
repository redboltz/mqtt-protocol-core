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
use alloc::collections::BTreeSet;
use core::fmt::Debug;

use num_traits::{One, PrimInt};

#[derive(Debug, Clone, Eq, PartialEq)]
struct ValueInterval<T> {
    low: T,
    high: T,
}

impl<T: PrimInt> ValueInterval<T> {
    fn new_single(value: T) -> Self {
        Self {
            low: value,
            high: value,
        }
    }

    fn new_range(low: T, high: T) -> Self {
        Self { low, high }
    }

    fn contains(&self, value: T) -> bool {
        self.low <= value && value <= self.high
    }

    fn low(&self) -> T {
        self.low
    }

    fn high(&self) -> T {
        self.high
    }
}

impl<T: PrimInt> PartialOrd for ValueInterval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: PrimInt> Ord for ValueInterval<T> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.high < other.low {
            core::cmp::Ordering::Less
        } else if other.high < self.low {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}

#[derive(Clone)]
pub struct ValueAllocator<T>
where
    T: PrimInt + One + Debug,
{
    pool: BTreeSet<ValueInterval<T>>,
    lowest: T,
    highest: T,
}

impl<T> ValueAllocator<T>
where
    T: PrimInt + One + Debug,
{
    pub fn new(lowest: T, highest: T) -> Self {
        assert!(lowest <= highest);
        let mut pool = BTreeSet::new();
        pool.insert(ValueInterval::new_range(lowest, highest));
        Self {
            pool,
            lowest,
            highest,
        }
    }

    pub fn allocate(&mut self) -> Option<T> {
        let iv = self.pool.iter().next()?.clone();
        let value = iv.low();

        self.pool.remove(&iv);
        if value < iv.high() {
            self.pool
                .insert(ValueInterval::new_range(value + T::one(), iv.high()));
        }

        Some(value)
    }

    pub fn first_vacant(&self) -> Option<T> {
        self.pool.iter().next().map(|iv| iv.low())
    }

    pub fn deallocate(&mut self, value: T) {
        assert!(self.lowest <= value && value <= self.highest);

        let right = self
            .pool
            .range(ValueInterval::new_single(value)..)
            .next()
            .cloned();
        let left = self
            .pool
            .range(..ValueInterval::new_single(value))
            .next_back()
            .cloned();

        match (left, right) {
            (Some(l), Some(r)) if l.high + T::one() == value && value + T::one() == r.low => {
                self.pool.remove(&l);
                self.pool.remove(&r);
                self.pool.insert(ValueInterval::new_range(l.low, r.high));
            }
            (Some(l), _) if l.high + T::one() == value => {
                self.pool.remove(&l);
                self.pool.insert(ValueInterval::new_range(l.low, value));
            }
            (_, Some(r)) if value + T::one() == r.low => {
                self.pool.remove(&r);
                self.pool.insert(ValueInterval::new_range(value, r.high));
            }
            _ => {
                self.pool.insert(ValueInterval::new_single(value));
            }
        }
    }

    pub fn use_value(&mut self, value: T) -> bool {
        if let Some(iv) = self.pool.iter().find(|iv| iv.contains(value)).cloned() {
            self.pool.remove(&iv);
            if iv.low < value {
                self.pool
                    .insert(ValueInterval::new_range(iv.low, value - T::one()));
            }
            if value < iv.high {
                self.pool
                    .insert(ValueInterval::new_range(value + T::one(), iv.high()));
            }
            true
        } else {
            false
        }
    }

    pub fn is_used(&self, value: T) -> bool {
        !self.pool.iter().any(|iv| iv.contains(value))
    }

    pub fn clear(&mut self) {
        self.pool.clear();
        self.pool
            .insert(ValueInterval::new_range(self.lowest, self.highest));
    }

    pub fn interval_count(&self) -> usize {
        self.pool.len()
    }

    pub fn dump(&self) {
        for iv in &self.pool {
            println!("{iv:?}");
        }
    }
}
