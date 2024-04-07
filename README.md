# Basic Trie

[![Test CI](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml/badge.svg)](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml)

The trie data structure is used for quick access to words and
data that should (could) be associated with them.

**Basic Trie** is implemented as a tree where each node holds a single character
that could point at any other character thus allowing insertion of arbitrary words.

##### There are two major implementations:
- Trie where words are inserted with nothing attached to them
- Data Trie where each word has a corresponding vector of data attached to it

Regular tries are often used for word lookups and prefix matching, and data tries are
often used for finding all data that is connected to some prefix.

For example, when inserting a whole book in the trie, you could insert every word with
the corresponding page number it's on. Later when searching for the word, you could get all
the pages the word is on with no added performance cost.

### Global features
- insertion / removal of words
- fast contains check
- finding words based on a prefix
- longest / shortest words in the trie
- generic methods: `is_empty`, `len`, `clear`
- Trie equality with `==`
- Trie merging with `+` or `+=`

### Data Trie features
- generic type implementation for associating a word to any type, with zero trait constraints
- finding data of words based on exact match or prefix

### Optional features
- unicode support via the 'unicode' feature with the `unicode-segmentation` crate (enabled by default)
- data trie support via the 'data' feature (enabled by default)
- serialization and deserialization via the 'serde' feature with the `serde` crate

### Dependencies
- `unicode-segmentation` (enabled by default)
- `serde` (only with 'serde' feature flag)
- `fxhash`
- `thin-vec`
- `arrayvec`

### License
The software is licensed under the MIT license.

### Examples

```rust
 use basic_trie::Trie;

 let mut trie = Trie::new();
 trie.insert("eat");
 trie.insert("eating");
 trie.insert("wizard");

 let mut found_longest_words = trie.get_longest();
 found_longest_words.sort();

 assert!(trie.contains("wizard"));
 assert_eq!(vec![String::from("eating"), String::from("wizard")], found_longest_words);
 assert_eq!(vec![String::from("eat")], trie.get_shortest());
 assert_eq!(3, trie.len());
 ```

 ```rust
 use basic_trie::DataTrie;

 let mut data_trie = DataTrie::<u32>::new();
 data_trie.insert("apple", 1);
 data_trie.insert("apple", 2);
 data_trie.insert_no_data("banana");
 data_trie.insert("avocado", 15);

let mut found_data = data_trie.get_data("apple", false).unwrap();
found_data.sort();
assert_eq!(vec![&1, &2], found_data);

let mut found_data = data_trie.get_data("a", true).unwrap();
found_data.sort();
assert_eq!(vec![&1, &2, &15], found_data);

assert_eq!(vec![15], data_trie.remove("avocado").unwrap());
 ```

## Changelog
- **2.0.0** - Major redesign: increased memory efficiency for the regular Trie (used to be Dataless Trie);
Changed API names to better match the standard library; splitting the two implementations code-wise thus
fixing the documentation not rendering bug.
- **1.2.3** – Adding dependencies for even more memory layout optimisations.
- **1.2.2** – More memory optimisations with Box.
- **1.2.1** – Memory performance upgrade with Box. Mutable data retrieval.
- **1.2.0** – Equality and addition operators support between
same Trie types via `==`, `+` and `+=`.
- **1.1.1** – Adding `FxHashMap` dependency for boosted performance.
- **1.1.0** – Serialization with the `serde` crate and the 'serde' feature.
- **1.0.3** – Optimisation of `number_of_words()`. Removing lifetime requirements
for word insertion for much better flexibility at the same logical memory cost.
- **1.0.2** – Bug fixes.
- **1.0.1** – `insert_no_data()` for `DataTrie`. Bugfixes.
- **1.0.0** – Separation of `DataTrie` and `DatalessTrie`. Optimizing
performance for `DatalessTrie`. Incompatible with older versions.
- **<1.0.0** – Simple `Trie` with data and base features.

