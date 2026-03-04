#[cfg(feature = "data")]
mod data_trie;

#[cfg(feature = "data")]
pub use data_trie::DataTrie;

mod regular_trie;

pub use regular_trie::Trie;

#[cfg(feature = "unicode")]
use unicode_normalization::UnicodeNormalization;

#[cfg(not(feature = "unicode"))]
use std::str::Chars;

#[cfg(feature = "unicode")]
pub fn get_characters(word: &str) -> impl Iterator<Item = char> + '_ {
    word.nfc()
}

#[cfg(not(feature = "unicode"))]
pub fn get_characters(word: &str) -> Chars<'_> {
    word.chars()
}
