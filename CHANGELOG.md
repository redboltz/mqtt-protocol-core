# 0.3.1

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
