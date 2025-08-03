# mqtt-protocol-core

[![Crates.io](https://img.shields.io/crates/v/mqtt-protocol-core.svg)](https://crates.io/crates/mqtt-protocol-core)
[![Documentation](https://docs.rs/mqtt-protocol-core/badge.svg)](https://docs.rs/mqtt-protocol-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A **Sans-I/O** style MQTT protocol library for Rust that supports both **MQTT v5.0** and **v3.1.1**.

## Features

- **Sans-I/O Design**: Pure protocol implementation independent of I/O operations
- **MQTT v5.0 & v3.1.1 Support**: Full compatibility with both protocol versions
- **Client & Server (Broker) Support**: Can be used to build both MQTT clients and brokers
- **Synchronous API**: Event-driven architecture where users handle returned events to integrate with I/O
- **Automatic Configuration**: Automatically configures packets and properties according to MQTT specifications
- **Generic Packet ID Support**: Supports both u16 (standard) and u32 (extended for broker clusters) packet IDs

### Optional Features

- **Automatic TopicAlias Application**: Automatically applies topic aliases for efficiency
- **Automatic TopicAlias Numbering**: Manages topic alias assignments automatically
- **Automatic Publish Responses**: Handles Puback, Pubrec, Pubrel, and Pubcomp responses automatically
- **Automatic Pingreq Responses**: Automatically responds to ping requests
- **Ping Timeout Management**: Configurable timeout settings for Pingreq to Pingresp cycles

### I/O Integration

This library can be combined with various I/O implementations:
- **std::net**: For synchronous TCP networking
- **tokio**: For asynchronous networking
- **Any custom I/O**: The Sans-I/O design allows integration with any transport layer

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
mqtt-protocol-core = "0.1.3"
```

## Quick Start

### Basic Client Example

```rust
use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::prelude::*;
use std::io::{Cursor, Read, Write};
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create connection
    let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

    // Connect to broker
    let mut stream = TcpStream::connect("localhost:1883")?;

    // Build CONNECT packet
    let connect_packet = mqtt::packet::v5_0::Connect::builder()
        .client_id("my_client")
        .unwrap()
        .build()?;

    // Send through connection (returns events to handle)
    let events = connection.checked_send(connect_packet);
    handle_events(&mut stream, &mut connection, events)?;

    // Publish a message
    let publish_packet = mqtt::packet::v5_0::Publish::builder()
        .topic_name("test/topic")
        .unwrap()
        .qos(mqtt::packet::Qos::AtLeastOnce)
        .payload(b"Hello, MQTT!")
        .packet_id(connection.acquire_packet_id()?)
        .build()?;

    let events = connection.checked_send(publish_packet);
    handle_events(&mut stream, &mut connection, events)?;

    Ok(())
}

fn handle_events(
    stream: &mut TcpStream,
    connection: &mut mqtt::Connection<mqtt::role::Client>,
    events: Vec<mqtt::connection::Event>,
) -> Result<(), Box<dyn std::error::Error>> {
    for event in events {
        match event {
            mqtt::connection::Event::RequestSendPacket { packet, .. } => {
                // Send packet over network
                let buffers = packet.to_buffers();
                stream.write_vectored(&buffers)?;
            }
            mqtt::connection::Event::NotifyPacketReceived(packet) => {
                // Handle received packet
                println!("Received: {}", packet.packet_type());
            }
            mqtt::connection::Event::NotifyError(error) => {
                eprintln!("MQTT Error: {:?}", error);
            }
            // Handle other events...
            _ => {}
        }
    }
    Ok(())
}
```

### Subscription Example

```rust
use mqtt_protocol_core::mqtt;
use mqtt_protocol_core::mqtt::prelude::*;

// Create connection and connect (same as above)
let mut connection = mqtt::Connection::<mqtt::role::Client>::new(mqtt::Version::V5_0);

// Subscribe to topic
let packet_id = connection.acquire_packet_id()?;
let sub_opts = mqtt::packet::SubOpts::new().set_qos(mqtt::packet::Qos::AtLeastOnce);
let sub_entry = mqtt::packet::SubEntry::new("sensor/+", sub_opts)?;

let subscribe_packet = mqtt::packet::v5_0::Subscribe::builder()
    .entries(vec![sub_entry])
    .packet_id(packet_id)
    .build()?;

let events = connection.checked_send(subscribe_packet);
// Handle events to send SUBSCRIBE and receive SUBACK...
```

## Architecture

This library follows the **Sans-I/O** pattern, which means:

1. **Pure Protocol Logic**: The library handles MQTT protocol state and packet processing
2. **Event-Driven**: All I/O operations are communicated through events
3. **Transport Agnostic**: Works with any underlying transport (TCP, WebSocket, etc.)
4. **User Controls I/O**: Your application handles actual network operations

### Event Flow

```rust
// 1. Create and send packets through connection
let events = connection.checked_send(packet);

// 2. Handle events (your code decides how to do I/O)
for event in events {
    match event {
        RequestSendPacket { packet, .. } => {
            // Your code: send packet over network
        }
        NotifyPacketReceived(packet) => {
            // Your code: process received packet
        }
        RequestTimerReset { kind, duration_ms } => {
            // Your code: set up timer
        }
        // ... handle other events
    }
}

// 3. When data arrives, feed it to connection
let events = connection.recv(&mut cursor);
// Handle resulting events...
```

## Examples

Complete examples can be found in the `examples/` directory:

- **publish.rs**: Connects and publishes a message
- **subscribe.rs**: Connects and subscribes to receive messages

Run examples with:
```bash
cargo run --example publish localhost 1883 test/topic 1 "Hello World"
cargo run --example subscribe localhost 1883 test/topic 1
```

## MQTT Protocol Support

### MQTT v5.0 Features
- Properties support
- Reason codes
- Topic aliases
- User properties
- Session expiry
- Message expiry
- And more...

### MQTT v3.1.1 Features
- Full protocol compliance
- QoS levels 0, 1, 2
- Retained messages
- Clean/persistent sessions
- Last Will and Testament (LWT)

## Generic Packet ID Support

The library supports generic packet ID types for advanced use cases:

```rust
// Standard u16 packet IDs (default)
type Connection = mqtt::Connection<mqtt::role::Client>;

// Extended u32 packet IDs (for broker clusters)
type ExtendedConnection = mqtt::GenericConnection<mqtt::role::Client, u32>;
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for details about changes in each version.
