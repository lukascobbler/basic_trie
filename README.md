# Basic Trie

[![Test CI](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml/badge.svg)](https://github.com/lukascobbler/basic_trie/actions/workflows/rust.yml)

The trie data structure is used for quick access to words and
data that should be associated with them.

**Basic Trie** is implemented as a tree where each node holds a single character
that could point at any other character thus allowing insertion of arbitrary words.
Each node also holds a vector of the data that is associated with it.

For example, when inserting a whole book in the trie, you could insert every word with
the corresponding page number it's on. Later when searching for the word, you could get all
the pages the word is on with no added performance cost.

## Features
- insertion / removal of words
- finding words based on prefix
- finding data of words based on exact match or prefix
- longest / shortest words in the trie
- number of complete words in the trie

## Optional features
- unicode support via the 'unicode' feature with the 'unicode-segmentation' crate (enabled by default)

## Dependencies
- unicode-segmentation (enabled by default)

## License

The software is licensed under the MIT license.