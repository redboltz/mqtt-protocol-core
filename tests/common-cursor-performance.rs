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

// Include test prelude and setup unique mqtt module
include!("prelude.rs");
setup_mqtt_for_test!(mqtt_cursor_performance);

use mqtt::common::Cursor as MqttCursor;
use std::io::{Cursor as StdCursor, Read};
use std::time::Instant;
mod common;

#[test]
fn compare_cursor_performance() {
    common::init_tracing();
    const DATA_SIZE: usize = 16384;
    const ITERATIONS: usize = 100000;

    let data = vec![0xAAu8; DATA_SIZE];

    // Test std::io::Cursor
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = StdCursor::new(&data);
        let mut buf = vec![0u8; DATA_SIZE];
        cursor.read_exact(&mut buf).unwrap();
    }
    let std_duration = start.elapsed();

    // Test mqtt_protocol_core::Cursor
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = MqttCursor::new(data.as_slice());
        let mut buf = vec![0u8; DATA_SIZE];
        cursor.read_exact(&mut buf).unwrap();
    }
    let mqtt_duration = start.elapsed();

    println!(
        "Performance comparison for {} iterations of {}KB reads:",
        ITERATIONS,
        DATA_SIZE / 1024
    );
    println!(
        "std::io::Cursor:     {:?} ({:.1}ns per read)",
        std_duration,
        std_duration.as_nanos() as f64 / ITERATIONS as f64
    );
    println!(
        "mqtt Cursor:         {:?} ({:.1}ns per read)",
        mqtt_duration,
        mqtt_duration.as_nanos() as f64 / ITERATIONS as f64
    );
    let speedup = std_duration.as_nanos() as f64 / mqtt_duration.as_nanos() as f64;
    println!(
        "Performance ratio:   {:.2}x {}",
        speedup,
        if speedup > 1.0 {
            "(mqtt faster)"
        } else {
            "(std faster)"
        }
    );
}

#[test]
fn compare_single_byte_reads() {
    common::init_tracing();
    const DATA_SIZE: usize = 4096;
    const ITERATIONS: usize = 10000;

    let data = vec![0xAAu8; DATA_SIZE];

    // Test std::io::Cursor single byte reads
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = StdCursor::new(&data);
        for _ in 0..100 {
            let mut buf = [0u8; 1];
            cursor.read_exact(&mut buf).unwrap();
        }
    }
    let std_duration = start.elapsed();

    // Test mqtt_protocol_core::Cursor read_u8
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = MqttCursor::new(data.as_slice());
        for _ in 0..100 {
            cursor.read_u8().unwrap();
        }
    }
    let mqtt_duration = start.elapsed();

    println!(
        "Single byte read comparison ({} iterations of 100 reads):",
        ITERATIONS
    );
    println!(
        "std::io::Cursor:     {:?} ({:.1}ns per read)",
        std_duration,
        std_duration.as_nanos() as f64 / (ITERATIONS * 100) as f64
    );
    println!(
        "mqtt Cursor:         {:?} ({:.1}ns per read)",
        mqtt_duration,
        mqtt_duration.as_nanos() as f64 / (ITERATIONS * 100) as f64
    );
    let speedup = std_duration.as_nanos() as f64 / mqtt_duration.as_nanos() as f64;
    println!(
        "Performance ratio:   {:.2}x {}",
        speedup,
        if speedup > 1.0 {
            "(mqtt faster)"
        } else {
            "(std faster)"
        }
    );
}

#[test]
fn compare_position_operations() {
    common::init_tracing();
    const ITERATIONS: usize = 100000;
    let data = vec![0xAAu8; 4096];

    // Test std::io::Cursor position operations
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = StdCursor::new(&data);
        for i in 0..10 {
            cursor.set_position(i * 100);
            let _ = cursor.position();
        }
    }
    let std_duration = start.elapsed();

    // Test mqtt_protocol_core::Cursor position operations
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = MqttCursor::new(data.as_slice());
        for i in 0..10 {
            cursor.set_position(i * 100);
            let _ = cursor.position();
        }
    }
    let mqtt_duration = start.elapsed();

    println!(
        "Position operations comparison ({} iterations of 10 ops):",
        ITERATIONS
    );
    println!(
        "std::io::Cursor:     {:?} ({:.1}ns per op)",
        std_duration,
        std_duration.as_nanos() as f64 / (ITERATIONS * 10) as f64
    );
    println!(
        "mqtt Cursor:         {:?} ({:.1}ns per op)",
        mqtt_duration,
        mqtt_duration.as_nanos() as f64 / (ITERATIONS * 10) as f64
    );
    let speedup = std_duration.as_nanos() as f64 / mqtt_duration.as_nanos() as f64;
    println!(
        "Performance ratio:   {:.2}x {}",
        speedup,
        if speedup > 1.0 {
            "(mqtt faster)"
        } else {
            "(std faster)"
        }
    );
}
