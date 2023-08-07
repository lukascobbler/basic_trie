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
- unicode support via the 'unicode' feature with the 'unicode-segmentation' crate (enabled by default)

## Dependencies
- unicode-segmentation (enabled by default)

## License

The software is licensed under the MIT license.