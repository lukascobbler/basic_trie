//! Module containing the implementation details of the Trie.
//! Users of this library do not need to have interactions with singular TrieNodes.

mod data_trie;
mod dataless_trie;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::marker::PhantomData;

#[cfg(feature = "unicode")]
use unicode_segmentation::UnicodeSegmentation;
use crate::trie_node::TrieNode;

pub use data_trie::DataTrie;
pub use dataless_trie::DatalessTrie;

/// Trie data structure. Generic implementation for common methods
/// between Dataless and Data tries.
#[derive(Debug, Default)]
pub struct Trie<'a, D, HasData> {
    root: TrieNode<'a, D, HasData>,
    pd: PhantomData<HasData>
}

impl<'a, D, HasData> Trie<'a, D, HasData> {
    /// Returns an option enum with a vector of owned strings
    /// representing all found words that begin with "query".
    /// If no words are found, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word1");
    /// trie.insert("word2");
    ///
    /// let all_correct_words = vec![String::from("word1"), String::from("word2")];
    /// let mut found_words = trie.find_words("word").unwrap();
    /// found_words.sort();
    /// assert_eq!(all_correct_words, found_words);
    /// ```
    pub fn find_words(&self, query: &str) -> Option<Vec<String>> {
        let mut substring = String::new();
        let mut current_node = &self.root;
        let characters = get_characters(query);

        for character in characters {
            current_node = match current_node.children.get(character) {
                None => return None,
                Some(trie_node) => {
                    substring.push_str(character);
                    trie_node
                }
            }
        }

        let mut words_vec = Vec::new();
        current_node.find_words(&substring, &mut words_vec);

        if words_vec.is_empty() {
            return None;
        }

        Some(words_vec)
    }

    /// Returns the vector of longest words found in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("shortwrd");
    /// trie.insert("verylongword");
    /// trie.insert("somelongword");
    ///
    /// let longest_words = vec![String::from("somelongword"), String::from("verylongword")];
    /// let mut found_words = trie.longest_words();
    /// found_words.sort();
    /// assert_eq!(longest_words, found_words);
    /// ```
    pub fn longest_words(&self) -> Vec<String> {
        let mut empty_vec = Vec::new();
        self.root
            .words_min_max("", &mut empty_vec, Ordering::Greater);
        empty_vec
    }

    /// Returns the vector of shortest words found in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("shortwrd");
    /// trie.insert("rlyshort");
    /// trie.insert("verylongword");
    ///
    /// let shortest_word = vec![String::from("rlyshort"), String::from("shortwrd")];
    /// let mut found_words = trie.shortest_words();
    /// found_words.sort();
    /// assert_eq!(shortest_word, found_words);
    /// ```
    pub fn shortest_words(&self) -> Vec<String> {
        let mut empty_vec = Vec::new();
        self.root.words_min_max("", &mut empty_vec, Ordering::Less);
        empty_vec
    }

    /// Returns the number of words in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word1");
    /// trie.insert("word2");
    /// trie.insert("word3");
    /// trie.insert("word4");
    ///
    /// let number_of_words = 4;
    /// assert_eq!(number_of_words, trie.number_of_words());
    /// ```
    pub fn number_of_words(&self) -> usize {
        self.root.number_of_words()
    }

    /// Returns an option enum with a vector of owned strings
    /// representing all words in the trie.
    /// Order is not guaranteed.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word1");
    /// trie.insert("word2");
    /// trie.insert("word3");
    /// trie.insert("word4");
    /// trie.insert("word5");
    ///
    /// let all_words = vec![
    ///     String::from("word1"), String::from("word2"), String::from("word3"),
    ///     String::from("word4"), String::from("word5")
    /// ];
    ///
    /// let mut found_words = trie.all_words().unwrap();
    /// found_words.sort();
    ///
    /// assert_eq!(all_words, found_words);
    /// ```
    pub fn all_words(&self) -> Option<Vec<String>> {
        self.find_words("")
    }

    /// Returns true if the trie contains 'query' as a word.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word");
    /// assert!(trie.contains("word"));
    /// assert!(!trie.contains("notfound"));
    /// ```
    pub fn contains(&self, query: &str) -> bool {
        match self.get_final_node(query) {
            None => false,
            Some(node) => node.word_end_association.is_associated()
        }
    }

    /// Returns true if no words are in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word");
    /// trie.remove_word("word");
    ///
    /// assert!(trie.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.root.children.is_empty()
    }

    /// Removes all words from the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// // DataTrie and DatalessTrie common method.
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word1");
    /// trie.insert("word2");
    /// trie.insert("word3");
    /// trie.insert("word4");
    ///
    /// trie.clear();
    /// assert!(trie.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.root.remove_all_words();
    }

    #[doc(hidden)]
    /// Function for getting the last node in a character sequence.
    /// Cannot make a mutable version of this function because the lifetimes
    /// can't be calibrated to fit the rules.
    fn get_final_node(&self, query: &str) -> Option<&TrieNode<D, HasData>> {
        let mut current = &self.root;

        for character in get_characters(query) {
            current = match current.children.get(character) {
                None => return None,
                Some(next_node) => next_node
            }
        }

        Some(current)
    }
}

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
            .cloned().collect()
    }
}