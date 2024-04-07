use std::cmp::Ordering;
use std::ops;

use arrayvec::ArrayString;
#[cfg(feature = "serde")]
use serde_crate::{Deserialize, Serialize};

use crate::trie::get_characters;
use crate::trie_node::TrieDatalessNode;

#[derive(Debug, Default, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct Trie {
    root: TrieDatalessNode,
    len: usize,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieDatalessNode::new(),
            len: 0,
        }
    }

    /// Insert a word into the trie, with no corresponding data.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1");
    /// assert_eq!(vec![String::from("word1")], trie.get_all());
    /// ```
    pub fn insert(&mut self, word: &str) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current
                .children
                .entry(ArrayString::from(character).unwrap())
                .or_default();
        }

        if !current.is_associated() {
            self.len += 1;
        }

        current.associate();
    }

    /// Removes a word from the trie.
    /// If the word is a prefix to some word, some word
    /// isn't removed from the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word");
    /// trie.insert("wording");
    ///
    /// trie.remove("word");
    /// assert_eq!(vec![String::from("wording")], trie.get("word").unwrap());
    ///
    /// trie.remove("wording");
    /// assert_eq!(Vec::<String>::new(), trie.get_all());
    /// ```
    pub fn remove(&mut self, word: &str) {
        let Some(current) = self.get_final_node_mut(word) else {
            return;
        };

        let characters = get_characters(word);

        if !current.children.is_empty() {
            return if current.is_associated() {
                current.disassociate();
                self.len -= 1;
            };
        }

        self.root.remove_one_word(characters.into_iter());
        self.len -= 1;
    }

    /// Removes every word that begins with 'prefix'.
    /// Not including the word 'prefix' if it's present.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("eat");
    /// trie.insert("eats");
    /// trie.insert("eating");
    /// trie.insert("eatings");
    /// trie.insert("ea");
    ///
    /// trie.remove_prefix("ea");
    ///
    /// assert_eq!(vec![String::from("ea")], trie.get_all());
    /// ```
    pub fn remove_prefix(&mut self, prefix: &str) {
        let Some(current) = self.get_final_node_mut(prefix) else {
            return;
        };

        // (current.is_associated() as usize) is added (subtracted twice) to
        // not remove the current word from the count. Literal '1' is not used
        // because of calling this function on the root node where 1 should
        // not be added.
        self.len -= current.remove_all_words() - (current.is_associated() as usize);
    }

    /// Returns an option enum with a vector of owned strings
    /// representing all found words that begin with 'query'.
    /// If the word 'query' doesn't exist, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1");
    /// trie.insert("word2");
    ///
    /// let all_correct_words = vec![String::from("word1"), String::from("word2")];
    /// let mut found_words = trie.get("word").unwrap();
    /// found_words.sort();
    /// assert_eq!(all_correct_words, found_words);
    /// ```
    pub fn get(&self, query: &str) -> Option<Vec<String>> {
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

        Some(words_vec)
    }

    /// Returns the vector of longest words found in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("shortwrd");
    /// trie.insert("verylongword");
    /// trie.insert("somelongword");
    ///
    /// let longest_words = vec![String::from("somelongword"), String::from("verylongword")];
    /// let mut found_words = trie.get_longest();
    /// found_words.sort();
    /// assert_eq!(longest_words, found_words);
    /// ```
    pub fn get_longest(&self) -> Vec<String> {
        let mut words = Vec::new();
        self.root.words_min_max("", &mut words, Ordering::Greater);
        words
    }

    /// Returns the vector of shortest words found in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("shortwrd");
    /// trie.insert("rlyshort");
    /// trie.insert("verylongword");
    ///
    /// let shortest_word = vec![String::from("rlyshort"), String::from("shortwrd")];
    /// let mut found_words = trie.get_shortest();
    /// found_words.sort();
    /// assert_eq!(shortest_word, found_words);
    /// ```
    pub fn get_shortest(&self) -> Vec<String> {
        let mut words = Vec::new();
        self.root.words_min_max("", &mut words, Ordering::Less);
        words
    }

    /// Returns the number of words in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1");
    /// trie.insert("word2");
    /// trie.insert("word3");
    /// trie.insert("word4");
    /// assert_eq!(4, trie.len());
    ///
    /// trie.remove("word1");
    /// assert_eq!(3, trie.len());
    ///
    /// trie.remove_prefix("w");
    /// assert_eq!(0, trie.len());
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns an option enum with a vector of owned strings
    /// representing all words in the trie.
    /// Order is not guaranteed.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
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
    /// let mut found_words = trie.get_all();
    /// found_words.sort();
    ///
    /// assert_eq!(all_words, found_words);
    /// ```
    pub fn get_all(&self) -> Vec<String> {
        self.get("").unwrap()
    }

    /// Returns true if the trie contains 'query' as a word.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word");
    /// assert!(trie.contains("word"));
    /// assert!(!trie.contains("notfound"));
    /// ```
    pub fn contains(&self, query: &str) -> bool {
        self.get_final_node(query)
            .map_or(false, |node| node.is_associated())
    }

    /// Returns true if no words are in the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word");
    /// trie.remove("word");
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
    /// use basic_trie::Trie;
    /// let mut trie = Trie::new();
    ///
    /// trie.insert("word1");
    /// trie.insert("word2");
    /// trie.insert("word3");
    /// trie.insert("word4");
    ///
    /// trie.clear();
    /// assert!(trie.is_empty());
    /// assert_eq!(0, trie.len());
    /// ```
    pub fn clear(&mut self) {
        self.root.clear_children();
        self.len = 0;
    }

    /// Function for getting the last node in a character sequence.
    fn get_final_node(&self, query: &str) -> Option<&TrieDatalessNode> {
        let mut current = &self.root;

        for character in get_characters(query) {
            current = match current.children.get(character) {
                None => return None,
                Some(next_node) => next_node,
            }
        }

        Some(current)
    }

    /// Function for getting the last node in a character sequence (mutable).
    fn get_final_node_mut(&mut self, query: &str) -> Option<&mut TrieDatalessNode> {
        let mut current = &mut self.root;

        for character in get_characters(query) {
            current = match current.children.get_mut(character) {
                None => return None,
                Some(next_node) => next_node,
            }
        }

        Some(current)
    }
}

impl ops::Add for Trie {
    type Output = Trie;

    /// Operation + merges two tries, leaving out duplicate words.
    /// The smaller trie is always added to the larger one for efficiency.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie_1 = Trie::new();
    /// trie_1.insert("word1");
    /// trie_1.insert("word2");
    /// trie_1.insert("word");
    ///
    /// let mut trie_2 = Trie::new();
    /// trie_2.insert("word3");
    /// trie_2.insert("word");
    ///
    /// let mut correct = Trie::new();
    /// correct.insert("word");
    /// correct.insert("word1");
    /// correct.insert("word2");
    /// correct.insert("word3");
    ///
    /// let trie_3 = trie_1 + trie_2;
    ///
    /// assert_eq!(trie_3, correct);
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

impl ops::AddAssign for Trie {
    /// Operation += merges two tries, leaving out duplicate words.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie_1 = Trie::new();
    /// trie_1.insert("word1");
    /// trie_1.insert("word2");
    /// trie_1.insert("word");
    ///
    /// let mut trie_2 = Trie::new();
    /// trie_2.insert("word3");
    /// trie_2.insert("word");
    ///
    /// let mut correct = Trie::new();
    /// correct.insert("word");
    /// correct.insert("word1");
    /// correct.insert("word2");
    /// correct.insert("word3");
    ///
    /// trie_1 += trie_2;
    ///
    /// assert_eq!(trie_1, correct);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        self.root += rhs.root;
    }
}

impl PartialEq for Trie {
    /// # Examples
    ///
    /// ```
    /// use basic_trie::Trie;
    /// let mut trie_1 = Trie::new();
    /// trie_1.insert("test");
    ///
    /// let mut trie_2 = Trie::new();
    /// trie_2.insert("test");
    ///
    /// assert_eq!(trie_1, trie_2);
    ///
    /// trie_2.insert("test2");
    ///
    /// assert_ne!(trie_1, trie_2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}
