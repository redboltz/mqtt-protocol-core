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
use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::prelude::*;
use std::sync::Arc;
mod common;

#[test]
fn test_arc_payload_serialize() {
    common::init_tracing();
    let data = vec![1, 2, 3, 4, 5];
    let payload: mqtt::common::ArcPayload = data.into_payload();

    // Test Serialize trait (line 104)
    let serialized = serde_json::to_string(&payload).unwrap();
    assert_eq!(serialized, "[1,2,3,4,5]");
}

#[test]
fn test_arc_payload_debug() {
    common::init_tracing();
    let data = vec![1, 2, 3];
    let payload: mqtt::common::ArcPayload = data.into_payload();

    // Test Debug trait (line 259)
    let debug_output = format!("{payload:?}");
    assert!(debug_output.contains("ArcPayload"));
    assert!(debug_output.contains("[1, 2, 3]"));
}

#[test]
fn test_arc_payload_small_from_slice() {
    common::init_tracing();
    // Test creating Small variant from &[u8] (line 340, 346)
    let small_data = &[1u8, 2, 3]; // Small enough for SSO
    let payload: mqtt::common::ArcPayload = small_data.into_payload();

    assert_eq!(payload.len(), 3);
    assert_eq!(payload.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_arc_payload_large_from_slice() {
    common::init_tracing();
    // Test creating Large variant from &[u8] (line 346)
    let large_data = &[0u8; 1000]; // Large enough to force Large variant
    let payload: mqtt::common::ArcPayload = large_data.into_payload();

    assert_eq!(payload.len(), 1000);
    assert_eq!(payload.as_slice().len(), 1000);
}

#[test]
fn test_arc_payload_small_from_vec() {
    common::init_tracing();
    // Test creating Small variant from Vec<u8> (line 389)
    let small_vec = vec![10u8, 20, 30]; // Small enough for SSO
    let payload: mqtt::common::ArcPayload = small_vec.into_payload();

    assert_eq!(payload.len(), 3);
    assert_eq!(payload.as_slice(), &[10, 20, 30]);
}

#[test]
fn test_arc_payload_large_from_vec() {
    common::init_tracing();
    // Test creating Large variant from Vec<u8>
    let large_vec = vec![42u8; 1000]; // Large enough to force Large variant
    let payload: mqtt::common::ArcPayload = large_vec.into_payload();

    assert_eq!(payload.len(), 1000);
    assert!(payload.as_slice().iter().all(|&x| x == 42));
}

#[test]
fn test_arc_payload_small_from_arc() {
    common::init_tracing();
    // Test creating Small variant from Arc<[u8]> (line 439)
    let small_arc: Arc<[u8]> = Arc::from(&[7u8, 8, 9] as &[u8]);
    let payload: mqtt::common::ArcPayload = small_arc.into_payload();

    assert_eq!(payload.len(), 3);
    assert_eq!(payload.as_slice(), &[7, 8, 9]);
}

#[test]
fn test_arc_payload_large_from_arc() {
    common::init_tracing();
    // Test creating Large variant from Arc<[u8]>
    let large_arc: Arc<[u8]> = Arc::from(&[99u8; 1000] as &[u8]);
    let payload: mqtt::common::ArcPayload = large_arc.into_payload();

    assert_eq!(payload.len(), 1000);
    assert!(payload.as_slice().iter().all(|&x| x == 99));
}

#[test]
fn test_arc_payload_identity_conversion() {
    common::init_tracing();
    // Test ArcPayload identity conversion (line 461)
    let original_payload: mqtt::common::ArcPayload = vec![1, 2, 3, 4].into_payload();
    let converted_payload: mqtt::common::ArcPayload = original_payload.clone().into_payload();

    assert_eq!(original_payload.as_slice(), converted_payload.as_slice());
    assert_eq!(original_payload.len(), converted_payload.len());
}

#[test]
fn test_arc_payload_unit_conversion() {
    common::init_tracing();
    // Test unit type conversion
    let payload: mqtt::common::ArcPayload = ().into_payload();

    assert_eq!(payload.len(), 0);
    assert!(payload.is_empty());
    assert_eq!(payload.as_slice(), &[] as &[u8]);
}

#[test]
fn test_arc_payload_various_sizes_from_slice() {
    common::init_tracing();
    // Test different sizes to ensure Small variant creation is covered (line 340)

    // Very small data (1 byte) - should be Small if SSO enabled
    let tiny_data = &[42u8];
    let payload: mqtt::common::ArcPayload = tiny_data.into_payload();
    assert_eq!(payload.len(), 1);
    assert_eq!(payload.as_slice(), &[42]);

    // Edge case: 10 bytes - within smallest SSO buffer (15 bytes)
    let medium_data = &[1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let payload: mqtt::common::ArcPayload = medium_data.into_payload();
    assert_eq!(payload.len(), 10);
    assert_eq!(payload.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
}

#[test]
fn test_arc_payload_various_sizes_from_vec() {
    common::init_tracing();
    // Test different sizes to ensure Small variant creation from Vec is covered (line 389)

    // Very small Vec (1 byte) - should be Small if SSO enabled
    let tiny_vec = vec![99u8];
    let payload: mqtt::common::ArcPayload = tiny_vec.into_payload();
    assert_eq!(payload.len(), 1);
    assert_eq!(payload.as_slice(), &[99]);

    // Edge case Vec: 10 bytes - within smallest SSO buffer (15 bytes)
    let medium_vec = vec![10u8, 20, 30, 40, 50, 60, 70, 80, 90, 100];
    let payload: mqtt::common::ArcPayload = medium_vec.into_payload();
    assert_eq!(payload.len(), 10);
    assert_eq!(
        payload.as_slice(),
        &[10, 20, 30, 40, 50, 60, 70, 80, 90, 100]
    );
}
