// Test prelude - provides a macro to create unique mqtt modules per test file

macro_rules! setup_mqtt_for_test {
    ($module_name:ident) => {
        mqtt_protocol_core::make_size_aliases!($module_name, 32, 32, 128);
        use $module_name as mqtt;
    };
}
