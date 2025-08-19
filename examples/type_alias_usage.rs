// Example demonstrating the usage of type alias macros

fn main() {
    println!("Testing type alias macros...");

    // Demonstrate make_size_aliases! (uses u16 packet IDs)
    {
        // Generate type aliases with custom buffer sizes
        mqtt_protocol_core::make_size_aliases!(128, 64, 256);

        // Test v5.0 packets
        let auth = v5_0::Auth::builder().build().unwrap();
        println!("Created Auth packet: size = {}", auth.size());

        let publish = match v5_0::Publish::builder().topic_name("test/topic") {
            Ok(builder) => match builder.payload(b"Hello, MQTT!").build() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Failed to build publish: {:?}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("Failed to set topic_name: {:?}", e);
                return;
            }
        };
        println!("Created Publish packet: size = {}", publish.size());

        let connect = v5_0::Connect::builder()
            .client_id("test_client")
            .unwrap()
            .build()
            .unwrap();
        println!("Created Connect packet: size = {}", connect.size());

        // Test v3.1.1 packets
        let connect_v3 = v3_1_1::Connect::builder()
            .client_id("test_client_v3")
            .unwrap()
            .build()
            .unwrap();
        println!(
            "Created v3.1.1 Connect packet: size = {}",
            connect_v3.size()
        );

        match v3_1_1::Publish::builder().topic_name("v3/topic") {
            Ok(builder) => match builder.payload(b"Hello, MQTT v3.1.1!").build() {
                Ok(publish_v3) => println!(
                    "Created v3.1.1 Publish packet: size = {}",
                    publish_v3.size()
                ),
                Err(e) => eprintln!("Failed to build v3.1.1 publish: {:?}", e),
            },
            Err(e) => eprintln!("Failed to set topic for v3.1.1 publish: {:?}", e),
        }
    }

    // Demonstrate make_type_size_aliases! with u32 packet IDs in a different scope
    {
        mqtt_protocol_core::make_type_size_aliases!(u32, 64, 64, 128);

        match v5_0::Puback::builder().packet_id(42u32).build() {
            Ok(puback) => println!(
                "Created Puback with u32 packet ID: size = {}",
                puback.size()
            ),
            Err(e) => eprintln!("Failed to create Puback: {:?}", e),
        }

        match v5_0::Publish::builder().topic_name("u32/topic") {
            Ok(builder) => match builder.build() {
                Ok(publish_u32) => println!(
                    "Created Publish with u32 packet ID: size = {}",
                    publish_u32.size()
                ),
                Err(e) => eprintln!("Failed to build publish with u32: {:?}", e),
            },
            Err(e) => eprintln!("Failed to set topic for u32 publish: {:?}", e),
        }
    }

    println!("All type aliases work correctly!");
}
