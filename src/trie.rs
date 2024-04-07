#[cfg(feature = "unicode")]
use unicode_segmentation::UnicodeSegmentation;

#[cfg(feature = "data")]
mod data_trie;

#[cfg(feature = "data")]
pub use data_trie::DataTrie;

mod regular_trie;

pub use regular_trie::Trie;

/// Function returns true characters if the 'unicode' feature is enabled,
/// else it splits on "" and removes the first and last element, which may
/// result in wrong data if used with unicode text.
fn get_characters(word: &str) -> Vec<&str> {
    #[cfg(feature = "unicode")]
    return UnicodeSegmentation::graphemes(word, true).collect();

    #[cfg(not(feature = "unicode"))]
    {
        word.split("")
            .collect::<Vec<&str>>()
            .iter()
            .skip(1)
            .rev()
            .skip(1)
            .rev()
            .cloned()
            .collect()
    }
}
