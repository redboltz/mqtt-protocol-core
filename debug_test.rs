use mqtt_protocol_core as mqtt_core;

fn main() {
    mqtt_core::make_default_aliases!();
    
    // Create a 33-byte payload
    let boundary_data = "a".repeat(33);
    let payload = mqtt::common::ArcPayload::from(boundary_data.as_bytes());
    
    println!("Payload length: {}", payload.len());
    println!("Uses Arc: {:?}", payload.arc_data().is_some());
    println!("Expected: true (should use Arc for >32 bytes)");
}
