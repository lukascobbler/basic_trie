//! Module containing the implementation details of the Trie and the TrieNode.
//! Users of this library do not need to have interactions with singular TrieNodes.

use unicode_segmentation::UnicodeSegmentation;
use std::cmp::Ordering;
use std::collections::{HashMap};
use std::fmt::Debug;

/// Singular trie node that represents one character,
/// it's children and data associated with the character
/// if it's a word.
#[derive(Debug)]
struct TrieNode<'a, D> {
    children: HashMap<&'a str, TrieNode<'a, D>>,
    associated_data: Option<Vec<D>>
}

impl<'a, D> TrieNode<'a, D> {
    /// Returns a new instance of a TrieNode with the given character.
    fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            associated_data: None
        }
    }

    /// Recursive function for getting the number of words from a given node.
    fn number_of_words(&self) -> usize {
        self.children.values()
            .map(
                |x| x.number_of_words()
            )
            .sum::<usize>() +
            (self.associated_data.is_some()) as usize
    }

    /// Recursive function for inserting found words from the given node and
    /// given starting substring.
    fn find_words(&self, substring: &str, found_words: &mut Vec<String>) {
        if self.associated_data.is_some() {
            found_words.push(substring.to_string());
        }

        self.children.iter().for_each(|(&character, node)|
            node.find_words(&(substring.to_owned() + character), found_words)
        );
    }

    /// The recursive function for finding a vector of shortest and longest words in the TrieNode consists of:
    /// - the DFS tree traversal part for getting to every child node;
    /// - matching lengths of found words in combination with the passed ordering.
    fn words_min_max(&self, substring: &str, found_words: &mut Vec<String>, ord: Ordering) {
        'word: {
            if self.associated_data.is_some() {
                if let Some(found) = found_words.first() {
                    match substring.len().cmp(&found.len()) {
                        Ordering::Less if ord == Ordering::Less => {
                            found_words.clear();
                        },
                        Ordering::Greater if ord == Ordering::Greater => {
                            found_words.clear();
                        },
                        Ordering::Equal => (),
                        _ => break 'word
                    }
                }
                found_words.push(substring.to_string());
            }
        }

        self.children.iter().for_each(|(&character, node)|
            node.words_min_max(&(substring.to_owned() + character), found_words, ord)
        );
    }

    /// Recursive function for removing and freeing memory of a word that is not needed anymore.
    /// The algorithm first finds the last node of a word given in the form of a character iterator,
    /// then it frees the maps and unwinds to the first node that should not be deleted.
    /// The first node that should not be deleted is either:
    /// - the root node
    /// - the node that has multiple words branching from it
    fn remove_one_word<'b, I>(&mut self, mut characters: I) -> bool
        where
            I: Iterator<Item = &'b str>,
    {
        let next_character = match characters.next() {
            Some(char) => char,
            None => return false
        };

        let next_node = self.children.get_mut(next_character).unwrap();
        let must_keep = next_node.remove_one_word(characters);

        if self.children.len() > 1 || must_keep {
            return true;
        }
        self.children = HashMap::new();

        self.associated_data.is_some()
    }

    /// Recursive function that drops all children maps
    /// regardless of having multiple words branching from them or not.
    fn remove_all_words(&mut self) {
        self.children.values_mut().for_each(|x| {
            x.remove_all_words();
        });

        self.children.clear();
    }

    /// Recursive function finds every node that is an end of a word and appends
    /// it's data to the passed vector.
    fn generate_all_data<'b>(&'b self, found_data: &mut Vec<&'b D>) {
        if self.associated_data.is_some() {
            found_data.extend(self.associated_data.as_ref().unwrap().iter());
        }

        self.children.values().for_each(|x|
            x.generate_all_data(found_data)
        );
    }

    /// Function resets the data of a word.
    fn clear_data(&mut self) {
        self.associated_data = None;
    }

    /// Function adds data to a node
    fn add_data(&mut self, data: D) {
        if self.associated_data.is_none() {
            self.associated_data = Some(Vec::new());
        }
        self.associated_data.as_mut().unwrap().push(data);
    }
}

/// Trie data structure. Each word has a list of data associated to it. The associated data
/// can be of any type.
///
#[derive(Debug)]
pub struct Trie<'a, D> {
    root: TrieNode<'a, D>
}

impl<'a, D> Trie<'a, D> {
    /// Returns a new instance of the trie data structure.
    pub fn new() -> Self {
        Trie {
            root: TrieNode::<D>::new()
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
        let characters = UnicodeSegmentation::graphemes(word, true).collect::<Vec<&str>>();

        for character in characters {
            current = current.children.entry(character).or_insert(TrieNode::new());
        }

        current.add_data(associated_data);
    }

    /// Removes a word from the trie. If the word is a prefix to some word, some word
    /// isn't removed from the trie. The associated data is also removed.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word", "somedata");
    /// trie.insert("wording", "somedata2");
    ///
    /// trie.remove_word("word");
    /// assert_eq!(vec![String::from("wording")], trie.find_words("word").unwrap());
    /// assert_eq!(vec![&"somedata2"], trie.find_data_of_word("word", true).unwrap());
    ///
    /// trie.remove_word("wording");
    /// assert_eq!(None, trie.all_words());
    /// ```
    pub fn remove_word(&mut self, word: &str) {
        let characters = UnicodeSegmentation::graphemes(word, true).collect::<Vec<&str>>();

        let mut current = &mut self.root;

        for character in characters.iter() {
            current = match current.children.get_mut(*character) {
                Some(next_node) => next_node,
                None => return
            };
        }

        if !current.children.is_empty() {
            current.clear_data();
            return;
        }

        self.root.remove_one_word(characters.into_iter());
    }

    /// Removes every word that begins with 'prefix'.
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
    /// trie.remove_words_from_prefix("ea");
    /// assert_eq!(vec![String::from("ea")], trie.all_words().unwrap());
    /// ```
    pub fn remove_words_from_prefix(&mut self, prefix: &str) {
        let characters = UnicodeSegmentation::graphemes(prefix, true).collect::<Vec<&str>>();

        let mut current = &mut self.root;

        for character in characters {
            current = match current.children.get_mut(character) {
                None => return,
                Some(next_node) => next_node
            };
        }

        current.remove_all_words();
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

        let characters = UnicodeSegmentation::graphemes(query, true).collect::<Vec<&str>>();

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
        current_node.find_words(&mut substring, &mut words_vec);

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
        self.root.words_min_max("", &mut empty_vec, Ordering::Greater);
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
    pub fn find_data_of_word(&self, word: &str, soft_match: bool) -> Option<Vec<&D>> {
        let characters = UnicodeSegmentation::graphemes(word, true).collect::<Vec<&str>>();
        let mut current_node = &self.root;

        for character in characters {
            current_node = match current_node.children.get(character) {
                None => return None,
                Some(trie_node) => trie_node
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
            match &current_node.associated_data {
                Some(data_vec) => Some(data_vec.iter().collect()),
                None => None
            }
        }
    }

    /// Clears data of some word. If the word is not found,
    /// or there is no data associated with the word, nothing happens.
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word", "data1");
    /// trie.insert("word", "data2");
    /// trie.insert("word", "data3");
    /// trie.clear_word_data("word");
    ///
    /// assert_eq!(None, trie.find_data_of_word("word", false));
    /// ```
    pub fn clear_word_data(&mut self, word: &'a str) {
        let mut current = &mut self.root;
        let characters = UnicodeSegmentation::graphemes(word, true).collect::<Vec<&str>>();

        for character in characters {
            current = match current.children.get_mut(character) {
                None => return,
                Some(trie_node) => trie_node
            }
        }

        current.clear_data();
    }
}