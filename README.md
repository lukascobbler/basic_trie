# Basic Trie

[![Test CI](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml/badge.svg)](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml)

The trie data structure is used for quick access to words and
data that should (could) be associated with them.

**Basic Trie** is implemented as a tree where each node holds a single character
that could point at any other character thus allowing insertion of arbitrary words.

#### There are two major implementations:
- Dataless Trie where words are inserted with nothing attached to them
- Data Trie where each word has a corresponding vector of data attached to it

Dataless tries are often used for word lookups and prefix matching, and data tries are
often used for finding all data that is connected to some prefix.

For example, when inserting a whole book in the trie, you could insert every word with
the corresponding page number it's on. Later when searching for the word, you could get all
the pages the word is on with no added performance cost.

## Global features
- insertion / removal of words
- finding words based on prefix
- longest / shortest words in the trie
- number of complete words in the trie
- generic methods: `is_empty`, `contains`, `clear`

## Data Trie features
- generic type implementation for associating a word to any type, with zero trait constraints
- finding data of words based on exact match or prefix 

## Optional features
- unicode support via the 'unicode' feature with the `unicode-segmentation` crate (enabled by default)
- data trie support via the 'data' feature (enabled by default)
- serialization and deserialization via the 'serde' feature with the `serde` crate 

## Dependencies
- `unicode-segmentation` (enabled by default)
- `serde` (only with 'serde' feature flag)

## License
The software is licensed under the MIT license.

## Examples

 ```rust
 use basic_trie::DatalessTrie;

 let mut dataless_trie = DatalessTrie::new();
 dataless_trie.insert("eat");
 dataless_trie.insert("eating");
 dataless_trie.insert("wizard");

 let mut found_longest_words = dataless_trie.longest_words().unwrap();
 found_longest_words.sort();

 assert_eq!(vec![String::from("eating"), String::from("wizard")], found_longest_words);
 assert_eq!(vec![String::from("eat")], dataless_trie.shortest_words().unwrap());
 assert_eq!(3, dataless_trie.number_of_words());
 ```

 ```rust
 use basic_trie::DataTrie;

 let mut data_trie = DataTrie::<u32>::new();
 data_trie.insert("apple", 1);
 data_trie.insert("apple", 2);
 data_trie.insert_no_data("banana");
 data_trie.insert("avocado", 15);

let mut found_data = data_trie.find_data_of_word("apple", false).unwrap();
found_data.sort();
assert_eq!(vec![&1, &2], found_data);

let mut found_data = data_trie.find_data_of_word("a", true).unwrap();
found_data.sort();
assert_eq!(vec![&1, &2, &15], found_data);

assert_eq!(vec![15], data_trie.remove_word("avocado").unwrap());
 ```

## Changelog
- **1.1.0** – Serialization with the `serde` crate and the 'serde' feature.
- **1.0.3** – Optimization of `number_of_words()`. Removing lifetime requirements
for word insertion for much better flexibility at the same logical memory cost.
- **1.0.2** – Bug fixes.
- **1.0.1** – `insert_no_data()` for `DataTrie`. Bugfixes.
- **1.0.0** – Separation of `DataTrie` and `DatalessTrie`. Optimizing
performance for `DatalessTrie`. Incompatible with older versions.
- **<1.0.0** – Simple `Trie` with data and base features.