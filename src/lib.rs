//! # Basic Trie
//!
//! [![Test CI](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml/badge.svg)](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml)
//!
//! The trie data structure is used for quick access to words and
//! data that should (could) be associated with them.
//!
//! **Basic Trie** is implemented as a tree where each node holds a single character
//! that could point at any other character thus allowing insertion of arbitrary words.
//!
//! #### There are two major implementations:
//! - Dataless Trie where words are inserted with nothing attached to them
//! - Data Trie where each word has a corresponding vector of data attached to it
//!
//! Dataless tries are often used for word lookups and prefix matching, and data tries are
//! often used for finding all data that is connected to some prefix.
//!
//! For example, when inserting a whole book in the trie, you could insert every word with
//! the corresponding page number it's on. Later when searching for the word, you could get all
//! the pages the word is on with no added performance cost.
//!
//! ## Global features
//! - insertion / removal of words
//! - finding words based on prefix
//! - longest / shortest words in the trie
//! - number of complete words in the trie
//! - generic methods: `is_empty`, `contains`, `clear`
//! - Trie equality with `==`
//! - Trie merging with `+` or `+=`
//!
//! ## Data Trie features
//! - generic type implementation for associating a word to any type, with zero trait constraints
//! - finding data of words based on exact match or prefix
//!
//! ## Optional features
//! - unicode support via the 'unicode' feature with the `unicode-segmentation` crate (enabled by default)
//! - data trie support via the 'data' feature (enabled by default)
//! - serialization and deserialization via the 'serde' feature with the `serde` crate
//!
//! ## Dependencies
//! - `unicode-segmentation` (enabled by default)
//! - `serde` (only with 'serde' feature flag)
//! - `fxhash`
//!
//! ## License
//! The software is licensed under the MIT license.
//!
//! ## Examples
//!
//!  ```rust
//!  use basic_trie::DatalessTrie;
//!
//!  let mut dataless_trie = DatalessTrie::new();
//!  dataless_trie.insert("eat");
//!  dataless_trie.insert("eating");
//!  dataless_trie.insert("wizard");
//!
//!  let mut found_longest_words = dataless_trie.longest_words().unwrap();
//!  found_longest_words.sort();
//!
//!  assert_eq!(vec![String::from("eating"), String::from("wizard")], found_longest_words);
//!  assert_eq!(vec![String::from("eat")], dataless_trie.shortest_words().unwrap());
//!  assert_eq!(3, dataless_trie.number_of_words());
//!  ```
//!
//!  ```rust
//!  use basic_trie::DataTrie;
//!
//!  let mut data_trie = DataTrie::<u32>::new();
//!  data_trie.insert("apple", 1);
//!  data_trie.insert("apple", 2);
//!  data_trie.insert_no_data("banana");
//!  data_trie.insert("avocado", 15);
//!
//! let mut found_data = data_trie.find_data_of_word("apple", false).unwrap();
//! found_data.sort();
//! assert_eq!(vec![&1, &2], found_data);
//!
//! let mut found_data = data_trie.find_data_of_word("a", true).unwrap();
//! found_data.sort();
//! assert_eq!(vec![&1, &2, &15], found_data);
//!
//! assert_eq!(vec![15], data_trie.remove_word("avocado").unwrap());
//!  ```
//!
//! ## Changelog
//! - **1.2.0** – Equality and addition operators support between
//! same Trie types via `==`, `+` and `+=`.
//! - **1.1.1** – Adding `FxHashMap` dependency for boosted performance.
//! - **1.1.0** – Serialization with the `serde` crate and the 'serde' feature.
//! - **1.0.3** – Optimization of `number_of_words()`. Removing lifetime requirements
//! for word insertion for much better flexibility at the same logical memory cost.
//! - **1.0.2** – Bug fixes.
//! - **1.0.1** – `insert_no_data()` for `DataTrie`. Bugfixes.
//! - **1.0.0** – Separation of `DataTrie` and `DatalessTrie`. Optimizing
//! performance for `DatalessTrie`. Incompatible with older versions.
//! - **<1.0.0** – Simple `Trie` with data and base features.

mod trie;
mod trie_node;

mod data {
    #[cfg(feature = "serde")]
    use serde_crate::{Deserialize, Serialize};

    pub trait CData {}

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(
        feature = "serde",
        derive(Serialize, Deserialize),
        serde(crate = "serde_crate")
    )]
    pub struct YesData;

    #[derive(Debug, Clone, PartialEq, Eq)]
    #[cfg_attr(
        feature = "serde",
        derive(Serialize, Deserialize),
        serde(crate = "serde_crate")
    )]
    pub struct NoData;

    impl CData for YesData {}
    impl CData for NoData {}
}

#[cfg(feature = "data")]
pub use trie::DataTrie;

pub use trie::DatalessTrie;

// Tests which are the same for both implementations,
// Dataless is used for less verbose code.
#[cfg(test)]
mod general_trie_tests {
    use super::DatalessTrie;
    
    #[test]
    fn find_words() {
        let found_words_correct = vec![
            String::from("word1"),
            String::from("word2"),
            String::from("word3"),
        ];

        let mut trie = DatalessTrie::new();

        trie.insert("word1");
        trie.insert("word2");
        trie.insert("word3");

        let mut found_words = trie.find_words("word").unwrap();
        found_words.sort();
        assert_eq!(found_words, found_words_correct);
    }

    #[test]
    fn longest_word() {
        let mut trie = DatalessTrie::new();

        trie.insert("a");
        assert_eq!(trie.longest_words().unwrap(), vec![String::from("a")]);

        trie.insert("aa");
        assert_eq!(trie.longest_words().unwrap(), vec![String::from("aa")]);

        trie.insert("aaa");
        assert_eq!(trie.longest_words().unwrap(), vec![String::from("aaa")]);

        trie.insert("aaaa");
        assert_eq!(trie.longest_words().unwrap(), vec![String::from("aaaa")]);

        trie.insert("a");
        assert_eq!(trie.longest_words().unwrap(), vec![String::from("aaaa")]);
    }

    #[test]
    fn multiple_longest_words() {
        let mut trie = DatalessTrie::new();

        trie.insert("abba");
        trie.insert("cddc");

        let mut found_words = trie.longest_words().unwrap();
        found_words.sort();

        assert_eq!(
            vec![String::from("abba"), String::from("cddc")],
            found_words
        );
    }

    #[test]
    fn shortest_word() {
        let mut trie = DatalessTrie::new();

        trie.insert("a");
        assert_eq!(trie.shortest_words().unwrap(), vec![String::from("a")]);

        trie.insert("aa");
        assert_eq!(trie.shortest_words().unwrap(), vec![String::from("a")]);

        trie.insert("aaa");
        assert_eq!(trie.shortest_words().unwrap(), vec![String::from("a")]);

        trie.insert("aaaa");
        assert_eq!(trie.shortest_words().unwrap(), vec![String::from("a")]);

        trie.insert("a");
        assert_eq!(trie.shortest_words().unwrap(), vec![String::from("a")]);
    }

    #[test]
    fn multiple_shortest_words() {
        let mut trie = DatalessTrie::new();

        trie.insert("aaa");
        trie.insert("aaaa");
        trie.insert("aa");
        trie.insert("bb");

        let mut found_words = trie.shortest_words().unwrap();
        found_words.sort();

        assert_eq!(vec![String::from("aa"), String::from("bb")], found_words);
    }

    #[test]
    fn number_of_words() {
        let mut trie = DatalessTrie::new();

        trie.insert("a");
        trie.insert("b");
        trie.insert("c");
        trie.insert("d");

        assert_eq!(4, trie.number_of_words());
    }

    #[test]
    fn same_word_twice() {
        let mut trie = DatalessTrie::new();

        trie.insert("twice");
        trie.insert("twice");

        assert_eq!(vec!["twice"], trie.find_words("twice").unwrap());
    }

    #[test]
    fn all_words() {
        let mut trie = DatalessTrie::new();

        trie.insert("a");
        trie.insert("ab");
        trie.insert("abc");
        trie.insert("abcd");

        let all_words = vec![
            String::from("a"),
            String::from("ab"),
            String::from("abc"),
            String::from("abcd"),
        ];

        assert_eq!(all_words, trie.all_words().unwrap())
    }

    #[cfg(feature = "unicode")]
    #[test]
    fn unicode() {
        let mut trie = DatalessTrie::new();

        trie.insert("а");
        trie.insert("аб");
        trie.insert("абц");
        trie.insert("абцд");

        let all_words = vec![
            String::from("а"),
            String::from("аб"),
            String::from("абц"),
            String::from("абцд"),
        ];

        assert_eq!(all_words, trie.all_words().unwrap())
    }

    #[test]
    fn clear() {
        let mut trie = DatalessTrie::new();
        trie.insert("word1");
        trie.insert("word2");
        trie.insert("word3");
        trie.insert("word4");
        trie.insert("word5");

        trie.clear();
    }
}

#[cfg(feature = "data")]
#[cfg(test)]
mod data_trie_tests {
    use super::DataTrie;

    #[test]
    fn find_data_soft_match() {
        let found_data_correct = vec![&1, &2, &3];

        let mut trie = DataTrie::new();

        trie.insert("word1", 1);
        trie.insert("word2", 2);
        trie.insert("word3", 3);

        let mut found_data = trie.find_data_of_word("word", true).unwrap();
        found_data.sort();
        assert_eq!(found_data, found_data_correct);
    }

    #[test]
    fn find_str_data_soft_match() {
        let found_data_correct = vec![&"data1", &"data2", &"data3"];

        let mut trie = DataTrie::new();

        trie.insert("word1", "data1");
        trie.insert("word2", "data2");
        trie.insert("word3", "data3");

        let mut found_data = trie.find_data_of_word("word", true).unwrap();
        found_data.sort();
        assert_eq!(found_data, found_data_correct);
    }

    #[test]
    fn find_data_hard_match() {
        let found_data_correct = vec![&1];

        let mut trie = DataTrie::new();

        trie.insert("word1", 1);
        trie.insert("word2", 2);
        trie.insert("word3", 3);

        let mut found_data = trie.find_data_of_word("word1", false).unwrap();
        found_data.sort();
        assert_eq!(found_data, found_data_correct);
    }

    #[test]
    fn find_data_hard_match_not_found() {
        let found_data_correct = None;

        let mut trie = DataTrie::new();

        trie.insert("word1", 1);
        trie.insert("word2", 2);
        trie.insert("word3", 3);

        let found_data = trie.find_data_of_word("word", false);

        assert_eq!(found_data, found_data_correct);
    }
    
    #[test]
    fn same_word_twice_different_data() {
        let mut trie = DataTrie::new();

        trie.insert("twice", 5);
        trie.insert("twice", 3);

        assert_eq!(vec![&5, &3], trie.find_data_of_word("twice", true).unwrap());
    }

    #[test]
    fn clear_word_data() {
        let mut trie = DataTrie::new();

        trie.insert("twice", 5);
        let data = trie.clear_word_data("twice");
        trie.insert("twice", 3);

        assert_eq!(vec![&3], trie.find_data_of_word("twice", true).unwrap());
        assert_eq!(vec![5], data.unwrap());
    }

    #[test]
    fn clear_word_no_data() {
        let mut trie = DataTrie::new();

        trie.insert("word1", 5);
        let data = trie.clear_word_data("word2");

        assert_eq!(None, data);
    }

    #[test]
    fn remove_word1() {
        let mut trie = DataTrie::new();

        trie.insert("a", 5);
        trie.insert("ab", 5);
        trie.insert("abc", 5);
        trie.insert("abcd", 5);

        trie.remove_word("a");

        let all_words = vec![
            String::from("ab"),
            String::from("abc"),
            String::from("abcd"),
        ];

        assert_eq!(all_words, trie.all_words().unwrap())
    }

    #[test]
    fn remove_word_final() {
        let mut trie = DataTrie::new();

        trie.insert("a", 5);
        trie.insert("ab", 5);
        trie.insert("abc", 5);
        trie.insert("abcd", 5);

        trie.remove_word("abcd");

        let all_correct_words = vec![String::from("a"), String::from("ab"), String::from("abc")];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_2() {
        let mut trie = DataTrie::new();

        trie.insert("a", 5);
        trie.insert("ab", 5);
        trie.insert("abc", 5);
        trie.insert("abcd", 5);

        trie.remove_word("abc");

        let all_correct_words = vec![String::from("a"), String::from("ab"), String::from("abcd")];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
        assert_eq!(vec![&5, &5, &5], trie.find_data_of_word("a", true).unwrap());
    }

    #[test]
    fn remove_word_3() {
        let mut trie = DataTrie::new();

        trie.insert("eat", 5);
        trie.insert("eating", 5);
        trie.insert("eats", 5);
        trie.insert("eatings", 5);

        trie.remove_word("eating");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eatings"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_4() {
        let mut trie = DataTrie::new();

        trie.insert("eat", 5);
        trie.insert("eating", 5);
        trie.insert("eats", 5);
        trie.insert("eatings", 5);

        trie.remove_word("eatings");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eating"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_5() {
        let mut trie = DataTrie::new();

        trie.insert("eat", 5);
        trie.insert("eating", 5);
        trie.insert("eats", 5);
        trie.insert("eatings", 5);

        let data = trie.remove_word("eatin");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eating"),
            String::from("eatings"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
        assert_eq!(None, data);
    }

    #[test]
    fn remove_word_6() {
        let mut trie = DataTrie::new();

        trie.insert("eat", 5);
        trie.insert("eatings", 5);

        trie.remove_word("eatings");

        let all_correct_words = vec![
            String::from("eat"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_7() {
        let mut trie = DataTrie::new();

        trie.insert("eat", 3);
        trie.insert("eatings", 5);

        let data1 = trie.remove_word("eatings");

        let all_correct_words = vec![
            String::from("eat"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);

        assert_eq!(vec![5], data1.unwrap());

        let data2 = trie.remove_word("eat");

        assert_eq!(vec![3], data2.unwrap());
    }

    #[test]
    fn remove_word_8() {
        let mut trie = DataTrie::new();

        trie.insert("eat", 3);
        trie.insert("eats", 4);
        trie.insert("eatings", 5);

        let data = trie.remove_word("eats");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eatings")
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
        assert_eq!(vec![4], data.unwrap());

        let mut remaining_data = trie.find_data_of_word("eat", true).unwrap();
        remaining_data.sort();

        assert_eq!(vec![&3, &5], remaining_data);
    }

    #[test]
    fn remove_prefix_1() {
        let mut trie = DataTrie::new();

        trie.insert("eat", 3);
        trie.insert("eating", 4);
        trie.insert("eats", 5);
        trie.insert("eatings", 6);
        trie.insert("ea", 7);

        let mut removed_data = trie.remove_words_from_prefix("ea").unwrap();
        removed_data.sort();

        assert_eq!(vec![String::from("ea")], trie.all_words().unwrap());
        assert_eq!(vec![3, 4, 5, 6], removed_data);
        assert_eq!(1, trie.number_of_words());
    }

    #[test]
    fn remove_prefix_2() {
        let mut trie = DataTrie::new();

        trie.insert("a1", 3);
        trie.insert("b2", 4);
        trie.insert("c3", 5);

        let mut removed_data = trie.remove_words_from_prefix("").unwrap();
        removed_data.sort();

        assert_eq!(None, trie.all_words());
        assert!(trie.is_empty());
        assert_eq!(0, trie.number_of_words());
        assert_eq!(vec![3, 4, 5], removed_data);
    }

    #[cfg(feature = "unicode")]
    #[test]
    fn unicode_data() {
        let mut trie = DataTrie::new();

        trie.insert("а", 5);
        trie.insert("аб", 5);
        trie.insert("абц", 5);
        trie.insert("абцд", 5);

        let all_data = vec![&5, &5, &5, &5];

        assert_eq!(all_data, trie.find_data_of_word("а", true).unwrap())
    }

    #[test]
    fn insert_no_data() {
        let mut trie = DataTrie::<&str>::new();

        trie.insert_no_data("word1");
        assert_eq!(vec![String::from("word1")], trie.all_words().unwrap());

        trie.insert("word1", "somedata");
        assert_eq!(vec![&"somedata"], trie.find_data_of_word("word1", false).unwrap());
    }

    #[test]
    fn equals() {
        let mut data_trie_1 = DataTrie::new();
        data_trie_1.insert("test", 1);

        let mut data_trie_2 = DataTrie::new();
        data_trie_2.insert("test", 1);

        assert_eq!(data_trie_1, data_trie_2);
    }

    #[test]
    fn add_two_tries_1() {
        let mut t1 = DataTrie::<i32>::new();
        t1.insert("word1", 1000);
        t1.insert("word2", 1000);
        t1.insert("apple", 1000);
        t1.insert("banana", 1000);

        let mut t2 = DataTrie::<i32>::new();
        t2.insert("word3", 1000);
        t2.insert("word4", 1000);
        t2.insert("potato", 1000);
        t2.insert("watermelon", 1000);

        let t3 = t1 + t2;

        let mut correct = DataTrie::<i32>::new();
        correct.insert("word1", 1000);
        correct.insert("word2", 1000);
        correct.insert("apple", 1000);
        correct.insert("banana", 1000);
        correct.insert("word3", 1000);
        correct.insert("word4", 1000);
        correct.insert("potato", 1000);
        correct.insert("watermelon", 1000);

        let mut t3_words = t3.all_words().unwrap();
        let mut correct_words = correct.all_words().unwrap();

        t3_words.sort();
        correct_words.sort();
        assert_eq!(t3_words, correct_words);

        let t3_data = t3.find_data_of_word("", true).unwrap();
        assert_eq!(t3_data, Vec::from([&1000; 8]));
    }

    #[test]
    fn add_two_tries_2() {
        let mut t1 = DataTrie::<i32>::new();
        t1.insert("word1", 1000);
        t1.insert("word2", 1000);
        t1.insert("apple", 1000);
        t1.insert("banana", 1000);

        let mut t2 = DataTrie::<i32>::new();
        t2.insert("word3", 1000);
        t2.insert("word4", 1000);
        t2.insert("potato", 1000);
        t2.insert("watermelon", 1000);

        t1 += t2;

        let mut correct = DataTrie::<i32>::new();
        correct.insert("word1", 1000);
        correct.insert("word2", 1000);
        correct.insert("apple", 1000);
        correct.insert("banana", 1000);
        correct.insert("word3", 1000);
        correct.insert("word4", 1000);
        correct.insert("potato", 1000);
        correct.insert("watermelon", 1000);

        let mut t1_words = t1.all_words().unwrap();
        let mut correct_words = correct.all_words().unwrap();

        t1_words.sort();
        correct_words.sort();
        assert_eq!(t1_words, correct_words);

        let t1_data = t1.find_data_of_word("", true).unwrap();
        assert_eq!(t1_data, Vec::from([&1000; 8]));
    }

    #[test]
    fn add_two_tries_3() {
        let mut t1 = DataTrie::<i32>::new();
        t1.insert("word1", 500);

        let mut t2 = DataTrie::<i32>::new();
        t2.insert("word2", 500);
        t2.insert("word", 500);

        t1 += t2;

        let mut correct = DataTrie::<i32>::new();
        correct.insert("word", 500);
        correct.insert("word1", 500);
        correct.insert("word2", 500);

        let mut t1_words = t1.all_words().unwrap();
        let mut correct_words = correct.all_words().unwrap();

        t1_words.sort();
        correct_words.sort();
        assert_eq!(t1_words, correct_words);

        let t1_data = t1.find_data_of_word("", true).unwrap();
        assert_eq!(t1_data, Vec::from([&500; 3]));
    }

    #[test]
    fn add_two_tries_4() {
        let mut t1 = DataTrie::<i32>::new();
        t1.insert("word1", 500);
        t1.insert("word1", 500);
        t1.insert("word1", 500);

        let mut t2 = DataTrie::<i32>::new();
        t2.insert("word1", 500);
        t2.insert("word1", 500);
        t2.insert("word1", 500);

        t1 += t2;

        let mut correct = DataTrie::<i32>::new();
        correct.insert("word1", 500);

        let mut t1_words = t1.all_words().unwrap();
        let mut correct_words = correct.all_words().unwrap();

        t1_words.sort();
        correct_words.sort();
        assert_eq!(t1_words, correct_words);

        let t1_data = t1.find_data_of_word("", true).unwrap();
        assert_eq!(t1_data, Vec::from([&500; 6]));
    }
}

#[cfg(test)]
mod dataless_trie_tests {
    use super::DatalessTrie;

    #[test]
    fn insert_no_data() {
        let mut trie = DatalessTrie::new();

        let found_words_correct = vec![
            String::from("word1"),
            String::from("word2"),
            String::from("word3"),
        ];

        trie.insert("word1");
        trie.insert("word2");
        trie.insert("word3");

        let mut found_words = trie.find_words("word").unwrap();
        found_words.sort();

        assert_eq!(found_words, found_words_correct);
    }

    #[test]
    fn remove_word1() {
        let mut trie = DatalessTrie::new();

        trie.insert("a");
        trie.insert("ab");
        trie.insert("abc");
        trie.insert("abcd");

        trie.remove_word("a");

        let all_words = vec![
            String::from("ab"),
            String::from("abc"),
            String::from("abcd"),
        ];

        assert_eq!(all_words, trie.all_words().unwrap())
    }

    #[test]
    fn remove_word_final() {
        let mut trie = DatalessTrie::new();

        trie.insert("a");
        trie.insert("ab");
        trie.insert("abc");
        trie.insert("abcd");

        trie.remove_word("abcd");

        let all_correct_words = vec![String::from("a"), String::from("ab"), String::from("abc")];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_2() {
        let mut trie = DatalessTrie::new();

        trie.insert("a");
        trie.insert("ab");
        trie.insert("abc");
        trie.insert("abcd");

        trie.remove_word("abc");

        let all_correct_words = vec![String::from("a"), String::from("ab"), String::from("abcd")];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_3() {
        let mut trie = DatalessTrie::new();

        trie.insert("eat");
        trie.insert("eating");
        trie.insert("eats");
        trie.insert("eatings");

        trie.remove_word("eating");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eatings"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_4() {
        let mut trie = DatalessTrie::new();

        trie.insert("eat");
        trie.insert("eating");
        trie.insert("eats");
        trie.insert("eatings");

        trie.remove_word("eatings");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eating"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_5() {
        let mut trie = DatalessTrie::new();

        trie.insert("eat");
        trie.insert("eating");
        trie.insert("eats");
        trie.insert("eatings");

        trie.remove_word("eatin");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eating"),
            String::from("eatings"),
            String::from("eats"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_6() {
        let mut trie = DatalessTrie::new();

        trie.insert("eat");
        trie.insert("eatings");

        trie.remove_word("eatings");

        let all_correct_words = vec![
            String::from("eat"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_7() {
        let mut trie = DatalessTrie::new();

        trie.insert("eat");
        trie.insert("eatings");

        trie.remove_word("eatings");

        let all_correct_words = vec![
            String::from("eat"),
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_8() {
        let mut trie = DatalessTrie::new();

        trie.insert("eat");
        trie.insert("eats");
        trie.insert("eating");

        trie.remove_word("eats");

        let all_correct_words = vec![
            String::from("eat"),
            String::from("eating")
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_word_9() {
        let mut trie = DatalessTrie::new();

        trie.insert("123");
        trie.insert("1234");
        trie.insert("12345");

        trie.remove_word("1234");

        let all_correct_words = vec![
            String::from("123"),
            String::from("12345")
        ];

        let mut all_words = trie.all_words().unwrap();
        all_words.sort();

        assert_eq!(all_correct_words, all_words);
    }

    #[test]
    fn remove_prefix_1() {
        let mut trie = DatalessTrie::new();

        trie.insert("eat");
        trie.insert("eating");
        trie.insert("eats");
        trie.insert("eatings");
        trie.insert("ea");

        trie.remove_words_from_prefix("ea");

        assert_eq!(vec![String::from("ea")], trie.all_words().unwrap());
        assert_eq!(1, trie.number_of_words());
    }

    #[test]
    fn remove_prefix_2() {
        let mut trie = DatalessTrie::new();

        trie.insert("a1");
        trie.insert("b2");
        trie.insert("c3");

        trie.remove_words_from_prefix("");

        assert_eq!(None, trie.all_words());
        assert!(trie.is_empty());
        assert_eq!(0, trie.number_of_words());
    }

    #[test]
    fn equals() {
        let mut dataless_trie_1 = DatalessTrie::new();
        dataless_trie_1.insert("test");

        let mut dataless_trie_2 = DatalessTrie::new();
        dataless_trie_2.insert("test");

        assert_eq!(dataless_trie_1, dataless_trie_2);
    }

    #[test]
    fn add_two_tries_1() {
        let mut t1 = DatalessTrie::new();
        t1.insert("word1");
        t1.insert("word2");
        t1.insert("apple");
        t1.insert("banana");

        let mut t2 = DatalessTrie::new();
        t2.insert("word3");
        t2.insert("word4");
        t2.insert("potato");
        t2.insert("pineapple");

        let t3 = t1 + t2;

        let mut correct = DatalessTrie::new();
        correct.insert("word1");
        correct.insert("word2");
        correct.insert("apple");
        correct.insert("banana");
        correct.insert("word3");
        correct.insert("word4");
        correct.insert("potato");
        correct.insert("pineapple");

        let mut t3_words = t3.all_words().unwrap();
        let mut correct_words = correct.all_words().unwrap();

        t3_words.sort();
        correct_words.sort();
        assert_eq!(t3_words, correct_words);
    }

    #[test]
    fn add_two_tries_2() {
        let mut t1 = DatalessTrie::new();
        t1.insert("word1");
        t1.insert("word2");
        t1.insert("apple");
        t1.insert("banana");

        let mut t2 = DatalessTrie::new();
        t2.insert("word3");
        t2.insert("word4");
        t2.insert("potato");
        t2.insert("watermelon");

        t1 += t2;

        let mut correct = DatalessTrie::new();
        correct.insert("word1");
        correct.insert("word2");
        correct.insert("apple");
        correct.insert("banana");
        correct.insert("word3");
        correct.insert("word4");
        correct.insert("potato");
        correct.insert("watermelon");

        let mut t1_words = t1.all_words().unwrap();
        let mut correct_words = correct.all_words().unwrap();

        t1_words.sort();
        correct_words.sort();
        assert_eq!(t1_words, correct_words);
    }

    #[test]
    fn add_two_tries_3() {
        let mut t1 = DatalessTrie::new();
        t1.insert("word1");

        let mut t2 = DatalessTrie::new();
        t2.insert("word2");
        t2.insert("word");

        t1 += t2;

        let mut correct = DatalessTrie::new();
        correct.insert("word");
        correct.insert("word1");
        correct.insert("word2");

        let mut t1_words = t1.all_words().unwrap();
        let mut correct_words = correct.all_words().unwrap();

        t1_words.sort();
        correct_words.sort();
        assert_eq!(t1_words, correct_words);
    }
}