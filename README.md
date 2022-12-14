# Bloom Filter
Simple bloom filter implementation in Rust.

To use this crate in any Rust project just one of the following dependencies.

## Install

```yaml
[dependencies]
...
bfilters = { git = "https://github.com/alexanderbakhmach/bloom-filter", branch = "<desired-branch>", version = "<desired-version>"}
```

For example for dev branch with version 0.1.1 the dependecy will look the following.

```yaml
[dependencies]
...
bfilters = { git = "https://github.com/alexanderbakhmach/bloom-filter", branch = "dev", version = "0.1.3"}
```

Or as a registered create
```yaml
[dependencies]
...
bfilters = "0.1.3"
```

## Use
The example below illustrates the bloom filter usage.

```rust
use bfilters::BloomFilter;

...

let items_capacity: u32 = 933_333;
let false_positive_probability: f32 = 0.04;

let mut bloom_filter: BloomFilter =
    match BloomFilter::new(Some(false_positive_probability), items_capacity) {
        Ok(bloom_filter) => bloom_filter,
        Err(msg) => panic!("Can not create bloom filter due to error: {}", msg),
    };

let item_to_save: &str = "Erc20Token";
let item_absent: &str = "Erc721Token";

bloom_filter.insert(item_to_save);

assert!(!bloom_filter.is_probably_present(item_absent));
```

Also false_positive_probability could be ```None``` then it will be computed with a formula.

```rust
use bfilters::BloomFilter;

...

let items_capacity: u32 = 933_333;

let mut bloom_filter: BloomFilter =
    match BloomFilter::new(None, items_capacity) {
        Ok(bloom_filter) => bloom_filter,
        Err(msg) => panic!("Can not create bloom filter due to error: {}", msg),
    };

let item_to_save: &str = "Erc20Token";
let item_absent: &str = "Erc721Token";

bloom_filter.insert(item_to_save);

assert!(!bloom_filter.is_probably_present(item_absent));
```

## Docs
Rust provides you with a beautiful documentation autogeneration tool. To generate documentation in your browser simply run the following command from the root of this project.

```bash
cargo doc --no-deps --open
```