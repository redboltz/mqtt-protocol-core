# 0.7.0

## Breaking changes

* Remove is_publish_processing() method from connection. #40

## Other updates

* Fix send_post_process() on offline publish. #38
* Fix MqttError code. #37

# 0.6.0

## Breaking changes

* Replace set_pingresp_recv_timeout() timeout_ms type. #36
  * u64 is used instead of Option<u64>.
* Re-designed PINGREQ sending interval management. #35
  * GenericConnection::set_pingreq_send_interval() duration parameter becomes `Option<u64>`.
  * Prioritize PINGREQ sending interval settings.
     1. User setting by GenericConnection::set_pingreq_send_interval().
     2. ServerKeepAlive Property value in received CONNACK packet.
     3. KeepAlive(default 0) value in sent CONNECT packet.

## Other updates

* Fix PINGRESP receiving timeout timer management. #34
* Refine error handling. #32
* Remove unused trait `RecvBehavior`. #31
* Fix v5_0::GenericPublish property_length from Option<VariableByteInteger> to VariableByteInteger. #30
* Add ServerKeepAlive timer handling on sending CONNACK. #29
* Refine common methods. #28
* Add RequestClose on CONNACK with error sending.  #27
* Fix invalid ConnectReasonCode. #27
* Refine tests. #26, #27, #28, #29, #32
* Fix panic on connack generation. #25
  * When errornous CONNECT packet is received, a CONNACK packet with rc is automatically generated but it caused panic due to lack of session_present field.

# 0.5.0
## Breaking changes

* Add TopicAlias extract mechanism on PUBLISH packet receiving. #24
  * When an endpoint receives a MQTTv5.0 PUBLISH packet that has a validly registerd TopicAlias propery
    and has empty TopicName, the endpoint automatically extract and add the registered TopicName.
    The TopicAlias is not changed. RemainingLength is re-caluclated.
    In addition, you can check the TopicAlias field is extracted one (originally empty) using `topic_name_extracted()` method.
    NOTE: Eq comparison fails if topic_name_extracted field is different.

* Add SSO(Small Size Optimization) for MqttString, MqttBinary, and ArcPayload. #23
  * The following feature flags are supported:
     * sso-min-32bit = []  # MqttString/MqttBinary: 12, ArcPayload: 15 - 32bit enum size optimized
     * sso-min-64bit = []  # MqttString/MqttBinary: 24, ArcPayload: 31 - 64bit enum size optimized
     * sso-lv10 = []       # MqttString/MqttBinary: 24, ArcPayload: 127 - Level 10 optimization
     * sso-lv20 = []       # MqttString/MqttBinary: 48, ArcPayload: 255 - Level 20 optimization
  * If multiple sso-* features are set, then the highest level is used.

## Other updates

* Refine tests. #23

# 0.4.0

## Breaking changes

* mqtt::common::HashSet::default() should be called instead of mqtt::common::HashSet::new(). #21
* Fix v5_0::GenericPublish::props() no Option. #20
  * According to the MQTT v5.0 spec, Property Length is always required even if it is 0 and payload is also empty.

## Other updates

* Fix no-std support. #21
* Refine documents. #19
* Separate tracing feature. #18

# 0.3.0

## Breaking changes

* Support no-std (required core and alloc). #17
  * HashSet, HashMap, and Cursor are in `mqtt::common` instead of `std::*`.

## Other updates

* Add to_continuous_buffer() method for packets. #17
* Refine CI. #9
* Refine TopicAlias for sending. # 15, #16
* Add tests. #8, #10, #11, #12, #13

# 0.2.0

## Breaking changes

* Re-organize tree. #7

# 0.1.3

* Add CI. #5

# 0.1.2

* Add documentation for crates.io

# 0.1.1

* Fix Cargo.toml edition.
* Remove .vscode

# 0.1.0

* Initial import.
