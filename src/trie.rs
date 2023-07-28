//! Module containing the implementation details of the Trie and the TrieNode.
//! Users of this library do not need to have interactions with singular TrieNodes.

use std::cmp::Ordering;
use std::fmt::Debug;

#[cfg(feature = "unicode")]
use unicode_segmentation::UnicodeSegmentation;
use crate::trie_node::TrieNode;

/// Trie data structure. Each word has a list of data associated to it. The associated data
/// can be of any type.
#[derive(Debug, Default)]
pub struct Trie<'a, D> {
    root: TrieNode<'a, D>,
}

impl<'a, D> Trie<'a, D> {
    /// Returns a new instance of the trie data structure.
    pub fn new() -> Self {
        Trie {
            root: TrieNode::<D>::new(),
        }
    }

    /// Insert a word into the trie, with the corresponding data.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1", "somedata");
    /// assert_eq!(vec![String::from("word1")], trie.all_words().unwrap());
    /// ```
    pub fn insert(&mut self, word: &'a str, associated_data: D) {
        let mut current = &mut self.root;

        let characters = get_characters(word);

        for character in characters {
            current = current.children.entry(character).or_insert_with(TrieNode::new);
        }

        current.add_data(associated_data);
    }

    /// Removes a word from the trie and returns data associated with that word.
    /// If the word is a prefix to some word, some word
    /// isn't removed from the trie.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word", "somedata");
    /// trie.insert("wording", "somedata2");
    ///
    /// let removed_data1 = trie.remove_word("word");
    /// assert_eq!(vec![String::from("wording")], trie.find_words("word").unwrap());
    /// assert_eq!(vec![&"somedata2"], trie.find_data_of_word("word", true).unwrap());
    /// assert_eq!(vec!["somedata"], removed_data1.unwrap());
    ///
    /// let removed_data2 = trie.remove_word("wording");
    /// assert_eq!(None, trie.all_words());
    /// assert_eq!(vec!["somedata2"], removed_data2.unwrap());
    /// ```
    pub fn remove_word(&mut self, word: &str) -> Option<Vec<D>> {
        let characters = get_characters(word);

        let mut current = &mut self.root;

        for character in characters.iter() {
            current = match current.children.get_mut(*character) {
                None => return None,
                Some(next_node) => next_node
            };
        }

        if !current.children.is_empty() {
            return current.clear_data(false);
        }

        self.root.remove_one_word(characters.into_iter()).data
    }

    /// Removes every word that begins with 'prefix'.
    /// Not including the word 'prefix' if it's present.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("eat", "somedata");
    /// trie.insert("eats", "somedata2");
    /// trie.insert("eating", "somedata3");
    /// trie.insert("eatings", "somedata4");
    /// trie.insert("ea", "somedata5");
    ///
    /// let mut removed_data = trie.remove_words_from_prefix("ea").unwrap();
    /// removed_data.sort();
    ///
    /// assert_eq!(vec![String::from("ea")], trie.all_words().unwrap());
    /// assert_eq!(vec!["somedata", "somedata2", "somedata3", "somedata4"], removed_data);
    /// ```
    pub fn remove_words_from_prefix(&mut self, prefix: &str) -> Option<Vec<D>> {
        let characters = get_characters(prefix);

        let mut current = &mut self.root;

        for character in characters {
            current = match current.children.get_mut(character) {
                None => return None,
                Some(next_node) => next_node,
            };
        }

        let mut data_vec = Vec::new();
        current.children.values_mut().for_each(|child|
            child.remove_all_words_collect(&mut data_vec)
        );
        if !data_vec.is_empty() {
            Some(data_vec)
        } else {
            None
        }
    }

    /// Returns an option enum with a vector of owned strings
    /// representing all found words that begin with "query".
    /// If no words are found, None is returned.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1", "somedata");
    /// trie.insert("word2", "somemoredata");
    ///
    /// let all_correct_words = vec![String::from("word1"), String::from("word2")];
    /// let mut found_words = trie.find_words("word").unwrap();
    /// found_words.sort();
    /// assert_eq!(all_correct_words, found_words);
    /// ```
    pub fn find_words(&self, query: &str) -> Option<Vec<String>> {
        let mut current_node = &self.root;
        let mut substring = String::new();

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
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("shortwrd", "somedata");
    /// trie.insert("verylongword", "somemoredata");
    /// trie.insert("somelongword", "somemoredata");
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
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("shortwrd", "somedata");
    /// trie.insert("rlyshort", "somedata");
    /// trie.insert("verylongword", "somemoredata");
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
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1", "");
    /// trie.insert("word2", "");
    /// trie.insert("word3", "");
    /// trie.insert("word4", "");
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
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1", "1");
    /// trie.insert("word2", "2");
    /// trie.insert("word3", "3");
    /// trie.insert("word4", "4");
    /// trie.insert("word5", "5");
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

    /// Returns all data associated to "query" wrapped in the Option enum, in case there is no data
    /// for the specified arguments.
    /// Parameter soft_match dictates if the returned data should be associated only with "query"
    /// or with all words that begin with "query".
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1", "somedata");
    /// trie.insert("word2", "somemoredata");
    ///
    /// let hard_data = vec![&"somedata"];
    /// assert_eq!(hard_data, trie.find_data_of_word("word1", false).unwrap());
    ///
    /// let soft_data = vec![&"somedata", &"somemoredata"];
    /// let mut found_data = trie.find_data_of_word("word", true).unwrap();
    /// found_data.sort();
    /// assert_eq!(soft_data, found_data);
    /// ```
    pub fn find_data_of_word(&self, query: &str, soft_match: bool) -> Option<Vec<&D>> {
        let characters = get_characters(query);
        let mut current_node = &self.root;

        for character in characters {
            current_node = match current_node.children.get(character) {
                None => return None,
                Some(trie_node) => trie_node,
            };
        }

        return if soft_match {
            let mut soft_match_data = Vec::new();
            current_node.generate_all_data(&mut soft_match_data);

            if soft_match_data.is_empty() {
                return None;
            }

            Some(soft_match_data)
        } else {
            current_node
                .associated_data
                .as_ref()
                .map(|data_vec| data_vec.iter().collect())
        };
    }

    /// Clears and returns data of some word. If the word is not found,
    /// or there is no data associated with the word, nothing happens.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word", "data1");
    /// trie.insert("word", "data2");
    /// trie.insert("word", "data3");
    /// let found_data = trie.clear_word_data("word");
    ///
    /// assert_eq!(Some(vec![]), trie.find_data_of_word("word", false));
    /// assert_eq!(vec!["data1", "data2", "data3"], found_data.unwrap());
    /// ```
    pub fn clear_word_data(&mut self, word: &'a str) -> Option<Vec<D>> {
        let mut current = &mut self.root;
        let characters = get_characters(word);

        for character in characters {
            current = match current.children.get_mut(character) {
                None => return None,
                Some(trie_node) => trie_node,
            }
        }

        current.clear_data(true)
    }

    /// Returns true if the trie contains 'query' as a word.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word", "");
    /// assert!(trie.contains("word"));
    /// assert!(!trie.contains("notfound"));
    /// ```
    pub fn contains(&self, query: &str) -> bool {
        let characters = get_characters(query);
        let mut current_node = &self.root;

        for character in characters {
            current_node = match current_node.children.get(character) {
                None => return false,
                Some(next_node) => next_node
            }
        }

        current_node.associated_data.is_some()
    }

    /// Returns true if no words are in the trie.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word", "");
    /// trie.remove_word("word");
    ///
    /// assert!(trie.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.root.children.is_empty()
    }

    /// Removes all words from the trie
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1", "");
    /// trie.insert("word2", "");
    /// trie.insert("word3", "");
    /// trie.insert("word4", "");
    ///
    /// trie.clear();
    /// assert!(trie.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.root.remove_all_words();
    }
}

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