//! Module containing the implementation details of the Trie.
//! Users of this library do not need to have interactions with singular TrieNodes.

#[cfg(feature = "data")]
mod data_trie;

mod dataless_trie;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops;

#[cfg(feature = "unicode")]
use unicode_segmentation::UnicodeSegmentation;

#[cfg(feature = "serde")]
use serde_crate::{Serialize, Deserialize};

use crate::trie_node::TrieNode;
use crate::data::CData;

#[cfg(feature = "data")]
pub use data_trie::DataTrie;

pub use dataless_trie::DatalessTrie;

/// Trie data structure. Generic implementation for common methods
/// between Dataless and Data tries. Phantom data as a state machine
/// is used together with zero sized structs
/// 'YesData' and 'NoData' to differentiate between two types.
#[derive(Debug, Default, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Trie<D, HasData: CData> {
    root: TrieNode<D, HasData>,
    pd: PhantomData<HasData>,
    len: usize
}

impl<D, HasData: CData> Trie<D, HasData> {
    /// Returns a new instance of the trie.
    pub fn new() -> Self {
        Trie {
            root: TrieNode::new(),
            pd: PhantomData::<HasData>,
            len: 0
        }
    }

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
            None
        } else {
            Some(words_vec)
        }
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
    /// let mut found_words = trie.longest_words().unwrap();
    /// found_words.sort();
    /// assert_eq!(longest_words, found_words);
    /// ```
    pub fn longest_words(&self) -> Option<Vec<String>> {
        let mut words = Vec::new();
        self.root
            .words_min_max("", &mut words, Ordering::Greater);
        if words.is_empty() {
            None
        } else {
            Some(words)
        }
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
    /// let mut found_words = trie.shortest_words().unwrap();
    /// found_words.sort();
    /// assert_eq!(shortest_word, found_words);
    /// ```
    pub fn shortest_words(&self) -> Option<Vec<String>> {
        let mut words = Vec::new();
        self.root.words_min_max("", &mut words, Ordering::Less);
        if words.is_empty() {
            None
        } else {
            Some(words)
        }
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
    /// assert_eq!(4, trie.number_of_words());
    ///
    /// trie.remove_word("word1");
    /// assert_eq!(3, trie.number_of_words());
    ///
    /// trie.remove_words_from_prefix("w");
    /// assert_eq!(0, trie.number_of_words());
    /// ```
    pub fn number_of_words(&self) -> usize {
        self.len
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
            Some(node) => node.is_associated()
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
        self.len == 0
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
    /// assert_eq!(0, trie.number_of_words());
    /// ```
    pub fn clear(&mut self) {
        self.root.clear_children();
        self.len = 0;
    }

    /// Function for getting the last node in a character sequence.
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

    /// Function for getting the last node in a character sequence (mutable).
    fn get_final_node_mut(&mut self, query: &str) -> Option<&mut TrieNode<D, HasData>> {
        let mut current = &mut self.root;

        for character in get_characters(query) {
            current = match current.children.get_mut(character) {
                None => return None,
                Some(next_node) => next_node
            }
        }

        Some(current)
    }
}

impl<D, HasData: CData> ops::Add for Trie<D, HasData> {
    type Output = Trie<D, HasData>;

    /// Operation + merges two tries, leaving out duplicate words.
    /// The smaller trie is always added to the larger one for efficiency.
    /// In a DataTrie context, all data of the same word is moved
    /// to the destination.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DatalessTrie;
    /// let mut dataless_trie_1 = DatalessTrie::new();
    /// dataless_trie_1.insert("word1");
    /// dataless_trie_1.insert("word2");
    /// dataless_trie_1.insert("word");
    ///
    /// let mut dataless_trie_2 = DatalessTrie::new();
    /// dataless_trie_2.insert("word3");
    /// dataless_trie_2.insert("word");
    ///
    /// let mut correct = DatalessTrie::new();
    /// correct.insert("word");
    /// correct.insert("word1");
    /// correct.insert("word2");
    /// correct.insert("word3");
    ///
    /// let dataless_trie_3 = dataless_trie_1 + dataless_trie_2;
    ///
    /// assert_eq!(dataless_trie_3, correct);
    /// ```
    fn add(self, rhs: Self) -> Self::Output {
        let (smaller, mut bigger) = if self.len < rhs.len {
            (self, rhs)
        } else {
            (rhs, self)
        };

        bigger.root += smaller.root;

        bigger
    }
}

impl<D, HasData: CData> ops::AddAssign for Trie<D, HasData> {
    /// Operation += merges two tries, leaving out duplicate words.
    /// In a DataTrie context, all data of the same word is moved
    /// to the destination.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DatalessTrie;
    /// let mut dataless_trie_1 = DatalessTrie::new();
    /// dataless_trie_1.insert("word1");
    /// dataless_trie_1.insert("word2");
    /// dataless_trie_1.insert("word");
    ///
    /// let mut dataless_trie_2 = DatalessTrie::new();
    /// dataless_trie_2.insert("word3");
    /// dataless_trie_2.insert("word");
    ///
    /// let mut correct = DatalessTrie::new();
    /// correct.insert("word");
    /// correct.insert("word1");
    /// correct.insert("word2");
    /// correct.insert("word3");
    ///
    /// dataless_trie_1 += dataless_trie_2;
    ///
    /// assert_eq!(dataless_trie_1, correct);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        self.root += rhs.root;
    }
}

impl<D: PartialEq, HasData: CData> PartialEq for Trie<D, HasData> {
    /// Operation == can be applied only to tries whose data implements PartialEq.
    /// This includes the DatalessTrie by default.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DatalessTrie;
    /// let mut dataless_trie_1 = DatalessTrie::new();
    /// dataless_trie_1.insert("test");
    ///
    /// let mut dataless_trie_2 = DatalessTrie::new();
    /// dataless_trie_2.insert("test");
    ///
    /// assert_eq!(dataless_trie_1, dataless_trie_2);
    ///
    /// dataless_trie_2.insert("test2");
    ///
    /// assert_ne!(dataless_trie_1, dataless_trie_2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
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