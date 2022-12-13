#![allow(dead_code, unused_variables)]

use std::hash::Hasher;

use fasthash::{CityHasher, FastHasher, MurmurHasher};

const DEFAULT_FALSE_POSITIVE_PROBABILITY: f32 = 0.4f32;

struct BloomFilter {
    false_positive_probability: f32,
    number_of_bits: usize,
    items_count: u32,
    number_of_hashes: u8,
    buffer: Vec<bool>,
    items_added: u32,
}

impl BloomFilter {
    pub fn new(
        false_positive_probability_opt: Option<f32>,
        items_count: u32,
    ) -> Result<Self, String> {
        if items_count == 0 {
            return Err("The bloom filter's items count could not be 0.".to_owned());
        }

        if false_positive_probability_opt <= Some(0.0)
            || false_positive_probability_opt >= Some(1.0)
        {
            return Err(
                "The bloom filter's false positive probability should be in range from 0 to 1."
                    .to_owned(),
            );
        }

        let false_positive_probability: f32 =
            false_positive_probability_opt.unwrap_or(DEFAULT_FALSE_POSITIVE_PROBABILITY);
        let number_of_bits: usize =
            Self::calc_best_number_of_bits(items_count, false_positive_probability);
        let number_of_hashes: u8 =
            Self::calc_best_number_of_hashes(false_positive_probability) as u8;

        Ok(Self {
            false_positive_probability,
            number_of_bits,
            items_count,
            number_of_hashes,
            buffer: vec![false; number_of_bits],
            items_added: 0,
        })
    }

    pub fn calc_best_number_of_bits(items_count: u32, false_positive_probability: f32) -> usize {
        -(items_count as f32 * false_positive_probability.ln() / f32::powf(f32::ln(2.0), 2.0))
            as usize
    }

    pub fn calc_best_number_of_hashes(false_positive_probability: f32) -> i8 {
        -f32::log2(false_positive_probability) as i8
    }

    pub fn insert(&mut self, item: &str) -> bool {
        if self.items_added < self.items_count {
            for i in 0..self.number_of_hashes {
                // TODO: Refactor initialization should happen only onces
                let mut murmur_hasher = MurmurHasher::new();
                let mut city_hasher = CityHasher::new();

                murmur_hasher.write(item.as_bytes());
                city_hasher.write(item.as_bytes());

                // Solution is based on answer: https://stackoverflow.com/questions/24676237/generating-random-hash-functions-for-lsh-minhash-algorithm#answer-24685697
                let aka_random_hash: u128 =
                    murmur_hasher.finish() as u128 + (i as u128) * city_hasher.finish() as u128;
                let item_hash_index = (aka_random_hash % self.number_of_bits as u128) as usize;

                self.buffer[item_hash_index] = true;
            }

            self.items_added += 1;

            true
        } else {
            false
        }
    }

    pub fn is_probably_present(&self, item: &str) -> bool {
        for i in 0..self.number_of_hashes {
            // TODO: Refactor initialization should happen only onces
            let mut murmur_hasher = MurmurHasher::new();
            let mut city_hasher = CityHasher::new();

            murmur_hasher.write(item.as_bytes());
            city_hasher.write(item.as_bytes());

            // Solution is based on answer: https://stackoverflow.com/questions/24676237/generating-random-hash-functions-for-lsh-minhash-algorithm#answer-24685697
            let aka_random_hash: u128 =
                murmur_hasher.finish() as u128 + (i as u128) * city_hasher.finish() as u128;
            let item_hash_index = (aka_random_hash % self.number_of_bits as u128) as usize;

            if !self.buffer[item_hash_index] {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::BloomFilter;

    #[test]
    fn test_item_not_present() {
        let item: &str = "John Green";
        let wrong_item: &str = "John White";
        let mut bloom_filter = match BloomFilter::new(Some(0.35), 100) {
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

        let calculated_best_number_of_bits: usize = BloomFilter::calc_best_number_of_bits(
            expected_items_count,
            expected_false_positive_probability,
        );

        assert!(calculated_best_number_of_bits > 0);
        assert!(calculated_best_number_of_bits / expected_items_count as usize > 8);
    }

    #[test]
    fn test_calc_best_number_of_hashes() {
        let expected_false_positive_probability: f32 = 0.01;

        let calculated_best_number_of_hashes: i8 =
            BloomFilter::calc_best_number_of_hashes(expected_false_positive_probability);

        assert!(calculated_best_number_of_hashes > 0);
    }
}
