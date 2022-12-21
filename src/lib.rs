#![allow(dead_code, unused_variables)]

use std::fs::File;
use std::hash::Hasher;
use std::io::{Read, Result as IoResult, Write};
use std::path::Path;

use fasthash::city::Hasher64 as CityHasher64;
use fasthash::murmur::Hasher32 as MurmurHasher32;
use fasthash::{CityHasher, FastHasher, MurmurHasher};

use serde::{Deserialize, Serialize};

pub const DEFAULT_FALSE_POSITIVE_PROBABILITY: f32 = 0.4f32;

/// A structure representing a bloom filter.
/// The structure should be created \w ::new syntax.
/// Consider the fact that constructor returns Result<BloomFilter, String>
///
/// ```rust
/// use bfilters::BloomFilter;
/// let expected_items_count: u32 = 233_092;
/// let expected_false_positive_probability: f32 = 0.01;
///
/// let mut bloom_filter = match BloomFilter::new(Some(expected_false_positive_probability), expected_items_count) {
///     Ok(bloom_filter) => bloom_filter,
///     Err(msg) => panic!("{}", msg),
/// };
/// ```
///
/// The bloom filter is often used to test false positive assumptions.
/// For example it can be used to check whether the element is not in the filter.
/// This feature can be used for a quick time search.
///
/// In the example below we check whether the Coke was memorized with a bloom filter or not.
///
/// ```rust
/// use bfilters::BloomFilter;
///
/// let expected_items_count: u32 = 233_999;
/// let expected_false_positive_probability: f32 = 0.01;
///
/// let mut bloom_filter = match BloomFilter::new(Some(expected_false_positive_probability), expected_items_count) {
///     Ok(bloom_filter) => bloom_filter,
///     Err(msg) => panic!("{}", msg),
/// };
///
/// let item_present: &str = "Vinegar";
/// let item_absent: &str = "Coke";
///
/// bloom_filter.insert(item_present);
///
/// assert!(!bloom_filter.is_probably_present(item_absent));
/// ```
///
/// Also (if needed) all the parameters could be initialized.
///
/// ```rust
/// use bfilters::BloomFilter;
/// let test_item: &str = "Vinegar";
/// let test_absent_item: &str = "Coke";
/// let test_false_positive_probability: f32 = 0.01;
/// let test_items_count: u32 = 923578;
/// let test_capacity: u32 = 923578 * 10;
/// let test_number_of_hashes: u32 = 4;
///
/// let mut bloom_filter: BloomFilter = match BloomFilter::custom(
///     test_items_count,
///     Some(test_false_positive_probability),
///     Some(test_capacity),
///     Some(test_number_of_hashes),
/// ) {
///     Ok(bloom_filter) => bloom_filter,
///     Err(msg) => panic!("{}", msg),
/// };
///
/// bloom_filter.insert(test_item);
///
/// let probably_present: bool = bloom_filter.is_probably_present(test_absent_item);
///
/// assert_eq!(probably_present, false);
#[derive(Serialize, Deserialize)]
pub struct BloomFilter {
    false_positive_probability: f32,
    number_of_bits: u32,
    items_count: u32,
    number_of_hashes: u32,
    buffer: Vec<bool>,
    items_added: u32,
}

impl BloomFilter {
    /// Creates a new instance of the Bloom Filter.
    pub fn new(
        false_positive_probability_opt: Option<f32>,
        items_count: u32,
    ) -> Result<Self, String> {
        if items_count == 0 {
            return Err("The bloom filter's items count could not be 0.".to_owned());
        }

        let false_positive_probability: f32 =
            false_positive_probability_opt.unwrap_or(DEFAULT_FALSE_POSITIVE_PROBABILITY);

        if false_positive_probability <= 0.0 || false_positive_probability >= 1.0 {
            return Err(
                "The bloom filter's false positive probability should be in range from 0 to 1."
                    .to_owned(),
            );
        }

        let number_of_bits: u32 =
            Self::calc_best_number_of_bits(items_count, false_positive_probability);
        let number_of_hashes: u32 =
            Self::calc_best_number_of_hashes(false_positive_probability) as u32;

        Ok(Self {
            false_positive_probability,
            number_of_bits,
            items_count,
            number_of_hashes,
            buffer: vec![false; number_of_bits as usize],
            items_added: 0,
        })
    }

    /// Constructor that allowed to set all the parameters manually. The false_positive_probability,
    /// number_of_bits_opt, number_of_hashes_opt will be computed only if None will be passed.
    pub fn custom(
        items_count: u32,
        false_positive_probability_opt: Option<f32>,
        number_of_bits_opt: Option<u32>,
        number_of_hashes_opt: Option<u32>,
    ) -> Result<Self, String> {
        if items_count == 0 {
            return Err("The bloom filter's items count could not be 0.".to_owned());
        }

        let false_positive_probability: f32 =
            false_positive_probability_opt.unwrap_or(DEFAULT_FALSE_POSITIVE_PROBABILITY);

        if false_positive_probability <= 0.0 || false_positive_probability >= 1.0 {
            return Err(
                "The bloom filter's false positive probability should be in range from 0 to 1."
                    .to_owned(),
            );
        }

        let number_of_bits: u32 = number_of_bits_opt.unwrap_or(Self::calc_best_number_of_bits(
            items_count,
            false_positive_probability,
        ));
        let number_of_hashes: u32 = number_of_hashes_opt
            .unwrap_or(Self::calc_best_number_of_hashes(false_positive_probability) as u32);

        Ok(Self {
            false_positive_probability,
            number_of_bits,
            items_count,
            number_of_hashes,
            buffer: vec![false; number_of_bits as usize],
            items_added: 0,
        })
    }

    /// Tries to instantiate a new instance of the bloom filter from the given file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut _file = File::open(path).unwrap();
        let mut _buffer: String = String::new();

        _file.read_to_string(&mut _buffer).unwrap();

        Ok(serde_json::from_str::<Self>(&_buffer).unwrap())
    }

    /// Calculates the best number of bits for the bloom filter's bit array.
    /// The formula uses the "expected items" count we want our filter to save (also known as capacity)
    /// and a "false positive probability" (also known as an error rate)
    ///
    /// The formula is:
    ///
    /// number_of_bits = - items_count * ln(false_positive_probability) / ln(2) ^ 2
    ///
    /// For more information please use <https://hur.st/bloomfilter> and <https://www.youtube.com/watch?v=-jiOPKt7avE>
    pub fn calc_best_number_of_bits(items_count: u32, false_positive_probability: f32) -> u32 {
        -(items_count as f32 * false_positive_probability.ln() / f32::powf(f32::ln(2.0), 2.0))
            as u32
    }

    /// Calculates the best number of hash functions to be used to store the single string item.
    /// The formula uses the "false positive probability" (also known as an error rate)
    ///
    /// The formula is:
    ///
    /// best_number_of_hashes = - log2(false_positive_probability)
    ///
    /// For more information please use <https://hur.st/bloomfilter> and <https://www.youtube.com/watch?v=-jiOPKt7avE>
    pub fn calc_best_number_of_hashes(false_positive_probability: f32) -> i8 {
        -f32::log2(false_positive_probability) as i8
    }

    /// Calculates the index for the given single string item in the bit array.
    /// Uses a simplified formula to replace a necessity to pick a random function.
    /// The simplified formula to simulate picking of random hash function is:
    ///
    /// hash_function_1_return_value + integer_seed * hash_function_2_return_value
    ///
    /// For more information please use <https://stackoverflow.com/questions/24676237/generating-random-hash-functions-for-lsh-minhash-algorithm#answer-24685697>
    /// Or the original paper: <https://www.eecs.harvard.edu/~michaelm/postscripts/rsa2008.pdf>
    pub fn _calc_random_bit_array_index(&mut self, item: &str, seed: u32) -> usize {
        let mut murmur_hasher: MurmurHasher32 = MurmurHasher::new();
        let mut city_hasher: CityHasher64 = CityHasher::new();

        murmur_hasher.write(item.as_bytes());
        city_hasher.write(item.as_bytes());

        // Solution is based on answer:
        // https://stackoverflow.com/questions/24676237/generating-random-hash-functions-for-lsh-minhash-algorithm#answer-24685697
        let aka_random_hash: u128 =
            murmur_hasher.finish() as u128 + (seed as u128) * city_hasher.finish() as u128;

        (aka_random_hash % self.number_of_bits as u128) as usize
    }

    /// Saving a given item to the bloom filter.
    /// Returning false if the bloom filter is full.
    /// Returning true if the insertion was successful.
    pub fn insert(&mut self, item: &str) -> bool {
        if self.items_added < self.items_count {
            for i in 0..self.number_of_hashes {
                let item_hash_index: usize = self._calc_random_bit_array_index(item, i);

                self.buffer[item_hash_index] = true;
            }

            self.items_added += 1;

            true
        } else {
            false
        }
    }

    /// Given the negative or false positive answer about the item presence in the bloom filter.
    pub fn is_probably_present(&mut self, item: &str) -> bool {
        for i in 0..self.number_of_hashes {
            let item_hash_index: usize = self._calc_random_bit_array_index(item, i);

            if !self.buffer[item_hash_index] {
                return false;
            }
        }

        true
    }

    /// With given path to a file saves a state of the current bloom filter in order
    /// to be able to deserialize it later.
    /// Returns an empty std::io::Result as IoResult
    pub fn save<P: AsRef<Path>>(&self, path: P) -> IoResult<()> {
        let mut _file = File::create(path)?;

        let _serialized_bfilter: String = serde_json::to_string(self)?;
        _file.write_all(_serialized_bfilter.as_str().as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BloomFilter;

    #[test]
    fn test_item_not_present() {
        let item: &str = "John Green";
        let wrong_item: &str = "John White";
        let items_capacity = 250_000_000; // 500 millions because the number of smart contracts in ethereum is 2,5 million
                                          // we aim to test with 100 bigger number
        let mut bloom_filter = match BloomFilter::new(Some(0.35), 2_000_0000) {
            Ok(bloom_filter) => bloom_filter,
            Err(msg) => panic!("{}", msg),
        };

        bloom_filter.insert(item);

        let probably_present: bool = bloom_filter.is_probably_present(wrong_item);

        assert_eq!(probably_present, false);
    }

    #[test]
    fn test_item_not_present_empty() {
        let item: &str = "John Green";
        let wrong_item: &str = "John White";
        let items_capacity = 250_000_000; // 500 millions because the number of smart contracts in ethereum is 2,5 million
                                          // we aim to test with 100 bigger number
        let mut bloom_filter = match BloomFilter::new(None, 2_000_0000) {
            Ok(bloom_filter) => bloom_filter,
            Err(msg) => panic!("{}", msg),
        };

        bloom_filter.insert(item);

        let probably_present: bool = bloom_filter.is_probably_present(wrong_item);

        assert_eq!(probably_present, false);
    }

    #[test]
    fn test_item_probably_present() {
        let item: &str = "John Green";
        let mut bloom_filter = match BloomFilter::new(Some(0.35), 100) {
            Ok(bloom_filter) => bloom_filter,
            Err(msg) => panic!("{}", msg),
        };

        bloom_filter.insert(item);

        let probably_present: bool = bloom_filter.is_probably_present(item);

        assert_eq!(probably_present, true);
    }

    #[test]
    #[should_panic(
        expected = "The bloom filter's false positive probability should be in range from 0 to 1."
    )]
    fn test_init_with_wrong_false_positive_rate_smaller_zero() {
        match BloomFilter::new(Some(0.0), 100) {
            Ok(_) => (),
            Err(msg) => panic!("{}", msg),
        };
    }

    #[test]
    #[should_panic(
        expected = "The bloom filter's false positive probability should be in range from 0 to 1."
    )]
    fn test_init_with_wrong_false_positive_rate_bigger_one() {
        match BloomFilter::new(Some(1.2), 100) {
            Ok(_) => (),
            Err(msg) => panic!("{}", msg),
        };
    }

    #[test]
    #[should_panic(expected = "The bloom filter's items count could not be 0.")]
    fn test_init_with_wrong_number_of_elements() {
        match BloomFilter::new(Some(0.32), 0) {
            Ok(_) => (),
            Err(msg) => panic!("{}", msg),
        };
    }

    #[test]
    fn test_insert_over_capacity() {
        let items: [&str; 3] = ["John Green", "Steve Red", "Mark Adams"];
        let last_item: &str = "John Doe";

        let mut bloom_filter = match BloomFilter::new(Some(0.35), 3) {
            Ok(bloom_filter) => bloom_filter,
            Err(msg) => panic!("{}", msg),
        };

        for item in items {
            assert_eq!(bloom_filter.insert(item), true);
        }

        assert_eq!(bloom_filter.insert(last_item), false);
    }

    #[test]
    fn test_calc_best_number_of_bits_valid() {
        // This test is made considering article here https://freecontent.manning.com/all-about-bloom-filters/
        // The article says that the number of bits divided by items count should be bigger 8 to support the false
        // positive probability lowe 3%.

        let expected_items_count: u32 = 233_092;
        let expected_false_positive_probability: f32 = 0.01;

        let calculated_best_number_of_bits: u32 = BloomFilter::calc_best_number_of_bits(
            expected_items_count,
            expected_false_positive_probability,
        );

        assert!(calculated_best_number_of_bits > 0);
        assert!(calculated_best_number_of_bits / expected_items_count as u32 > 8);
    }

    #[test]
    fn test_calc_best_number_of_hashes() {
        let expected_false_positive_probability: f32 = 0.01;

        let calculated_best_number_of_hashes: i8 =
            BloomFilter::calc_best_number_of_hashes(expected_false_positive_probability);

        assert!(calculated_best_number_of_hashes > 0);
    }

    #[test]
    fn test_calc_random_bit_array_index() {
        let test_item: &str = "Hello test world!";
        let test_seed: u32 = 2;
        let test_false_positive_probability: f32 = 0.01;
        let test_items_count: u32 = 923578;

        let mut bloom_filter: BloomFilter =
            match BloomFilter::new(Some(test_false_positive_probability), test_items_count) {
                Ok(bloom_filter) => bloom_filter,
                Err(msg) => panic!("{}", msg),
            };

        for i in 0..9999 {
            bloom_filter._calc_random_bit_array_index(test_item, test_seed);
        }
    }

    #[test]
    fn test_with_custom_parameters() {
        let test_item: &str = "Hello test world!";
        let test_absent_item: &str = "Absent";
        let test_false_positive_probability: f32 = 0.01;
        let test_items_count: u32 = 923578;
        let test_capacity: u32 = 923578 * 10;
        let test_number_of_hashes: u32 = 4;

        let mut bloom_filter: BloomFilter = match BloomFilter::custom(
            test_items_count,
            Some(test_false_positive_probability),
            Some(test_capacity),
            Some(test_number_of_hashes),
        ) {
            Ok(bloom_filter) => bloom_filter,
            Err(msg) => panic!("{}", msg),
        };

        bloom_filter.insert(test_item);

        let probably_present: bool = bloom_filter.is_probably_present(test_absent_item);

        assert_eq!(probably_present, false);
    }

    #[test]
    fn test_with_custom_parameters_optional_empty() {
        let test_item: &str = "Hello test world!";
        let test_absent_item: &str = "Absent";
        let test_items_count: u32 = 923578;

        let mut bloom_filter: BloomFilter =
            match BloomFilter::custom(test_items_count, None, None, None) {
                Ok(bloom_filter) => bloom_filter,
                Err(msg) => panic!("{}", msg),
            };

        bloom_filter.insert(test_item);

        let probably_present: bool = bloom_filter.is_probably_present(test_absent_item);

        assert_eq!(probably_present, false);
    }
}
