use std::hash::Hasher;

use fasthash::{CityHasher, FastHasher, MurmurHasher};

const DEFAULT_FALSE_POSITIVE_RATE: f32 = 0.4f32;

struct BloomFilter {
    false_positive_rate: f32,
    number_of_bits: u32,
    number_of_elements: u32,
    number_of_hash_functions: u8,
    buffer: Vec<bool>,
    items_added: u32,
}

impl BloomFilter {
    pub fn new(
        false_positive_rate_opt: Option<f32>,
        number_of_elements: u32,
        number_of_bits: u32,
    ) -> Result<Self, String> {
        if number_of_elements == 0 {
            return Err("The number of elements could not be 0.".to_owned());
        }

        if number_of_bits == 0 {
            return Err("The number of bits could not be 0.".to_owned());
        }

        if false_positive_rate_opt <= Some(0f32) {
            return Err("The false positive rate could not be 0 or less.".to_owned());
        }

        let number_of_elements_per_bit: f32 = number_of_bits as f32 / number_of_elements as f32;
        let false_positive_rate: f32 =
            false_positive_rate_opt.unwrap_or(DEFAULT_FALSE_POSITIVE_RATE);
        let number_of_hash_functions: u8 =
            (-false_positive_rate.ln() * number_of_elements_per_bit) as u8;

        Ok(Self {
            false_positive_rate: false_positive_rate,
            number_of_elements: number_of_elements,
            number_of_bits: number_of_bits,
            number_of_hash_functions: number_of_hash_functions,
            buffer: vec![false; number_of_bits as usize],
            items_added: 0,
        })
    }

    pub fn insert(&mut self, item: &str) -> bool {
        if self.items_added < self.number_of_elements {
            for i in 0..self.number_of_hash_functions {
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
        for i in 0..self.number_of_hash_functions {
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
        let mut bloom_filter = match BloomFilter::new(Some(0.35f32), 100, 200) {
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
        let mut bloom_filter = match BloomFilter::new(Some(0.35f32), 100, 200) {
            Ok(bloom_filter) => bloom_filter,
            Err(msg) => panic!("{}", msg),
        };

        bloom_filter.insert(item);

        let probably_present: bool = bloom_filter.is_probably_present(item);

        assert_eq!(probably_present, true);
    }

    #[test]
    #[should_panic(expected = "The false positive rate could not be 0 or less.")]
    fn test_init_with_wrong_false_positive_rate() {
        match BloomFilter::new(Some(0f32), 100, 200) {
            Ok(_) => (),
            Err(msg) => panic!("{}", msg),
        };
    }

    #[test]
    #[should_panic(expected = "The number of elements could not be 0.")]
    fn test_init_with_wrong_number_of_elements() {
        match BloomFilter::new(Some(0f32), 0, 200) {
            Ok(_) => (),
            Err(msg) => panic!("{}", msg),
        };
    }

    #[test]
    #[should_panic(expected = "The number of bits could not be 0.")]
    fn test_init_with_wrong_number_of_bits() {
        match BloomFilter::new(Some(0f32), 100, 0) {
            Ok(_) => (),
            Err(msg) => panic!("{}", msg),
        };
    }

    #[test]
    fn test_insert_over_capacity() {
        let items: [&str; 3] = ["John Green", "Steve Red", "Mark Adams"];
        let last_item: &str = "John Doe";

        let mut bloom_filter = match BloomFilter::new(Some(0.35f32), 3, 10) {
            Ok(bloom_filter) => bloom_filter,
            Err(msg) => panic!("{}", msg),
        };

        for item in items {
            assert_eq!(bloom_filter.insert(item), true);
        }

        assert_eq!(bloom_filter.insert(last_item), false);
    }
}
