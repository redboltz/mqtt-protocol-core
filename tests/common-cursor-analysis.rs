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
use mqtt_protocol_core::mqtt::common::Cursor as MqttCursor;
use std::io::{Cursor as StdCursor, Read};
use std::time::Instant;
mod common;

#[test]
fn analyze_memory_access_patterns() {
    common::init_tracing();
    const ITERATIONS: usize = 1000000;
    let data = vec![0xAAu8; 1024];

    // Test sequential small reads with std::io::Cursor
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = StdCursor::new(&data);
        let mut buf = [0u8; 8];
        for _ in 0..10 {
            cursor.read_exact(&mut buf).unwrap();
        }
    }
    let std_sequential = start.elapsed();

    // Test sequential small reads with mqtt Cursor
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = MqttCursor::new(data.as_slice());
        let mut buf = [0u8; 8];
        for _ in 0..10 {
            cursor.read_exact(&mut buf).unwrap();
        }
    }
    let mqtt_sequential = start.elapsed();

    // Test using read_bytes for mqtt cursor (more optimal)
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = MqttCursor::new(data.as_slice());
        for _ in 0..10 {
            cursor.read_bytes(8).unwrap();
        }
    }
    let mqtt_read_bytes = start.elapsed();

    println!(
        "Sequential 8-byte reads ({} iterations × 10 reads):",
        ITERATIONS
    );
    println!(
        "std::io::Cursor:     {:?} ({:.1}ns per read)",
        std_sequential,
        std_sequential.as_nanos() as f64 / (ITERATIONS * 10) as f64
    );
    println!(
        "mqtt read_exact:     {:?} ({:.1}ns per read)",
        mqtt_sequential,
        mqtt_sequential.as_nanos() as f64 / (ITERATIONS * 10) as f64
    );
    println!(
        "mqtt read_bytes:     {:?} ({:.1}ns per read)",
        mqtt_read_bytes,
        mqtt_read_bytes.as_nanos() as f64 / (ITERATIONS * 10) as f64
    );

    let ratio1 = std_sequential.as_nanos() as f64 / mqtt_sequential.as_nanos() as f64;
    let ratio2 = std_sequential.as_nanos() as f64 / mqtt_read_bytes.as_nanos() as f64;
    println!(
        "Performance ratio (read_exact): {:.2}x {}",
        ratio1,
        if ratio1 > 1.0 {
            "(mqtt faster)"
        } else {
            "(std faster)"
        }
    );
    println!(
        "Performance ratio (read_bytes): {:.2}x {}",
        ratio2,
        if ratio2 > 1.0 {
            "(mqtt faster)"
        } else {
            "(std faster)"
        }
    );

    // Performance regression detection
    // Allow some variance due to system load, but ensure mqtt cursor is competitive
    assert!(
        ratio1 > 0.9,
        "mqtt cursor read_exact performance regression: {:.2}x ratio (expected > 0.9)",
        ratio1
    );
    assert!(
        ratio2 > 0.9,
        "mqtt cursor read_bytes performance regression: {:.2}x ratio (expected > 0.9)",
        ratio2
    );
}

#[test]
fn analyze_copy_overhead() {
    common::init_tracing();
    const ITERATIONS: usize = 100000;
    const SIZE: usize = 4096;
    let data = vec![0xAAu8; SIZE];

    // Measure just the copy operation overhead
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut buf = vec![0u8; SIZE];
        buf.copy_from_slice(&data);
        std::hint::black_box(buf);
    }
    let copy_baseline = start.elapsed();

    // std::io::Cursor with read_exact
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = StdCursor::new(&data);
        let mut buf = vec![0u8; SIZE];
        cursor.read_exact(&mut buf).unwrap();
        std::hint::black_box(buf);
    }
    let std_duration = start.elapsed();

    // mqtt Cursor with read_exact
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = MqttCursor::new(data.as_slice());
        let mut buf = vec![0u8; SIZE];
        cursor.read_exact(&mut buf).unwrap();
        std::hint::black_box(buf);
    }
    let mqtt_duration = start.elapsed();

    println!(
        "Copy overhead analysis ({}KB × {} iterations):",
        SIZE / 1024,
        ITERATIONS
    );
    println!(
        "Raw copy_from_slice: {:?} ({:.1}ns per copy)",
        copy_baseline,
        copy_baseline.as_nanos() as f64 / ITERATIONS as f64
    );
    println!(
        "std::io::Cursor:     {:?} ({:.1}ns per copy)",
        std_duration,
        std_duration.as_nanos() as f64 / ITERATIONS as f64
    );
    println!(
        "mqtt Cursor:         {:?} ({:.1}ns per copy)",
        mqtt_duration,
        mqtt_duration.as_nanos() as f64 / ITERATIONS as f64
    );

    let std_overhead = std_duration.as_nanos() as f64 / copy_baseline.as_nanos() as f64;
    let mqtt_overhead = mqtt_duration.as_nanos() as f64 / copy_baseline.as_nanos() as f64;
    println!("std overhead:        {:.2}x baseline", std_overhead);
    println!("mqtt overhead:       {:.2}x baseline", mqtt_overhead);

    let ratio = std_duration.as_nanos() as f64 / mqtt_duration.as_nanos() as f64;
    println!(
        "Performance ratio:   {:.2}x {}",
        ratio,
        if ratio > 1.0 {
            "(mqtt faster)"
        } else {
            "(std faster)"
        }
    );
}

#[test]
fn analyze_inlining_effectiveness() {
    common::init_tracing();
    const ITERATIONS: usize = 10000000;
    let data = vec![0xAAu8; 64];

    // Test position operations (should be highly optimizable)
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = StdCursor::new(&data);
        cursor.set_position(10);
        std::hint::black_box(cursor.position());
        cursor.set_position(20);
        std::hint::black_box(cursor.position());
    }
    let std_position = start.elapsed();

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let mut cursor = MqttCursor::new(data.as_slice());
        cursor.set_position(10);
        std::hint::black_box(cursor.position());
        cursor.set_position(20);
        std::hint::black_box(cursor.position());
    }
    let mqtt_position = start.elapsed();

    println!("Position operation inlining ({} iterations):", ITERATIONS);
    println!(
        "std::io::Cursor:     {:?} ({:.1}ns per op)",
        std_position,
        std_position.as_nanos() as f64 / (ITERATIONS * 4) as f64
    );
    println!(
        "mqtt Cursor:         {:?} ({:.1}ns per op)",
        mqtt_position,
        mqtt_position.as_nanos() as f64 / (ITERATIONS * 4) as f64
    );

    let ratio = std_position.as_nanos() as f64 / mqtt_position.as_nanos() as f64;
    println!(
        "Performance ratio:   {:.2}x {}",
        ratio,
        if ratio > 1.0 {
            "(mqtt faster)"
        } else {
            "(std faster)"
        }
    );
}
