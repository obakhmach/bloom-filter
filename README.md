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

The bloom filter could be serialized and deserialized in the JSON format. 

```rust
use std::{fs, path::Path};
use bfilters::BloomFilter;

// Define the bloom filter state
let test_false_positive_probability: f32 = 0.01;
let test_items_count: u32 = 923578;
let test_capacity: u32 = 923578 * 10;
let test_number_of_hashes: u32 = 4;

// Define the bloom filter test items
let test_item: &str = "Vinegar";
let test_absent_item: &str = "Coke";

// Instantiate a bloom filter
let mut bloom_filter: BloomFilter = match BloomFilter::custom(
    test_items_count,
    Some(test_false_positive_probability),
    Some(test_capacity),
    Some(test_number_of_hashes),
) {
    Ok(bloom_filter) => bloom_filter,
    Err(msg) => panic!("{}", msg),
};

// Validate that the bloom filter is working
bloom_filter.insert(test_item);

let probably_present: bool = bloom_filter.is_probably_present(test_absent_item);

assert_eq!(probably_present, false);

// Serializing bloom filter into test tmp file
let tmp_save_path: &Path = std::path::Path::new("./bfilter_tmp.json");

bloom_filter.save(tmp_save_path).unwrap();

// Initialize a new bloom filter from the file
let mut deserialized_bloom_filter: BloomFilter = BloomFilter::from_file(tmp_save_path).unwrap();

// Validating that the deserialized bloom filter is working as before
let probably_present: bool = deserialized_bloom_filter.is_probably_present(test_absent_item);
```

## Docs
Rust provides you with a beautiful documentation autogeneration tool. To generate documentation in your browser simply run the following command from the root of this project.

```bash
cargo doc --no-deps --open
```