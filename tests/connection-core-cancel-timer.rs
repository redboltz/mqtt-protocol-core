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
mod common;
use common::*;

#[test]
fn v5_0_client_send_connect_keep_alive() {
    common::init_tracing();
    let mut con = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);
    con.set_pingreq_send_interval(Some(3000));
    con.set_pingresp_recv_timeout(1000);
    v5_0_client_establish_connection(&mut con);

    let packet = mqtt::packet::v5_0::Pingreq::new();
    let _events = con.checked_send(packet);

    let events = con.notify_closed();
    assert_eq!(events.len(), 2);

    if let mqtt::connection::GenericEvent::RequestTimerCancel(kind) = events[0] {
        assert_eq!(kind, mqtt::connection::TimerKind::PingreqSend);
    } else {
        assert!(
            false,
            "Expected RequestTimerCancel event with PingreqSend, but got: {:?}",
            events[1]
        );
    }

    if let mqtt::connection::GenericEvent::RequestTimerCancel(kind) = events[1] {
        assert_eq!(kind, mqtt::connection::TimerKind::PingrespRecv);
    } else {
        assert!(
            false,
            "Expected RequestTimerCancel event with PingrespRecv, but got: {:?}",
            events[1]
        );
    }
}
