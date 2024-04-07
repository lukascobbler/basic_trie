use crate::trie::get_characters;
use crate::trie_node::TrieDataNode;
use arrayvec::ArrayString;
use std::cmp::Ordering;
use std::ops;

#[cfg(feature = "serde")]
use serde_crate::{Deserialize, Serialize};

#[derive(Debug, Default, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct DataTrie<D> {
    root: TrieDataNode<D>,
    len: usize,
}

impl<D> DataTrie<D> {
    /// Returns a new instance of the trie.
    pub fn new() -> Self {
        DataTrie {
            root: TrieDataNode::new(),
            len: 0,
        }
    }

    /// Insert a word into the trie, with the corresponding data.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut trie = DataTrie::new();
    ///
    /// trie.insert("word1", "somedata");
    /// assert_eq!(vec![String::from("word1")], trie.get_all());
    /// ```
    pub fn insert(&mut self, word: &str, associated_data: D) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current
                .children
                .entry(ArrayString::from(character).unwrap())
                .or_insert_with(TrieDataNode::new);
        }

        if !current.is_associated() {
            self.len += 1;
            current.associate();
        }

        current.push_data(associated_data);
    }

    /// Insert a word into the trie, with no corresponding data.
    /// This function is very different from inserting a word into
    /// a regular trie, since it enables later attachment of data
    /// onto the inserted word. Type of trie must be annotated if
    /// this is the first function call.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut trie = DataTrie::<&str>::new();
    ///
    /// trie.insert_no_data("word1");
    /// assert_eq!(vec![String::from("word1")], trie.get_all());
    ///
    /// trie.insert("word1", "somedata");
    /// assert_eq!(vec![&"somedata"], trie.get_data("word1", false).unwrap());
    /// ```
    pub fn insert_no_data(&mut self, word: &str) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current
                .children
                .entry(ArrayString::from(character).unwrap())
                .or_insert_with(TrieDataNode::new);
        }

        if !current.is_associated() {
            self.len += 1;
            current.associate();
        }
    }

    /// Removes a word from the trie and returns data associated with that word.
    /// If the word is a prefix to some word, some word isn't removed from the trie.
    /// If the word is not found, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut trie = DataTrie::new();
    ///
    /// trie.insert("word", "somedata");
    /// trie.insert("wording", "somedata2");
    ///
    /// let removed_data1 = trie.remove("word");
    /// assert_eq!(vec![String::from("wording")], trie.get("word").unwrap());
    /// assert_eq!(vec![&"somedata2"], trie.get_data("word", true).unwrap());
    /// assert_eq!(vec!["somedata"], removed_data1.unwrap());
    ///
    /// let removed_data2 = trie.remove("wording");
    /// assert_eq!(Vec::<String>::new(), trie.get_all());
    /// assert_eq!(vec!["somedata2"], removed_data2.unwrap());
    /// ```
    pub fn remove(&mut self, word: &str) -> Option<Vec<D>> {
        let current = self.get_final_node_mut(word)?;

        if !current.children.is_empty() {
            return current.clear_word_end_association(false).map(|data_vec| {
                self.len -= 1;
                data_vec.into_iter().collect()
            });
        }

        let characters = get_characters(word);

        self.root
            .remove_one_word(characters.into_iter())
            .data
            .map_or(Some(Vec::new()), |data_vec| {
                self.len -= 1;
                Some(data_vec.into_iter().collect())
            })
    }

    /// Removes every word that begins with 'prefix' and collects all removed data.
    /// Not including the word 'prefix' if it's present.
    /// If the word 'prefix' is not found, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut trie = DataTrie::new();
    ///
    /// trie.insert("eat", "somedata");
    /// trie.insert("eats", "somedata2");
    /// trie.insert("eating", "somedata3");
    /// trie.insert("eatings", "somedata4");
    /// trie.insert("ea", "somedata5");
    ///
    /// let mut removed_data = trie.remove_prefix("ea").unwrap();
    /// removed_data.sort();
    ///
    /// assert_eq!(vec![String::from("ea")], trie.get_all());
    /// assert_eq!(vec!["somedata", "somedata2", "somedata3", "somedata4"], removed_data);
    /// ```
    pub fn remove_prefix(&mut self, prefix: &str) -> Option<Vec<D>> {
        let current = self.get_final_node_mut(prefix)?;

        let mut data_vec = Vec::new();

        // Sum must be applied to the node's children and not to the node
        // itself because the recursive function must disassociate a node
        // to put its data in the vector. The optimization of adding one
        // to the count when the node in question isn't root can't be used
        // since the original node would've been already disassociated therefore
        // not accounted for in self.len.
        let word_count = current
            .children
            .values_mut()
            .map(|child| child.remove_all_words_collect(&mut data_vec))
            .sum::<usize>();
        current.clear_children();

        self.len -= word_count;

        Some(data_vec)
    }

    /// Returns a vector of references to data of some word or references
    /// to all found data of some word prefix when 'soft_match' is set to true.
    /// If the word is not found and 'soft_match' is set to false, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut trie = DataTrie::new();
    ///
    /// trie.insert("word1", "somedata");
    /// trie.insert("word2", "somemoredata");
    ///
    /// let hard_data = vec![&"somedata"];
    /// assert_eq!(hard_data, trie.get_data("word1", false).unwrap());
    ///
    /// let soft_data = vec![&"somedata", &"somemoredata"];
    /// let mut found_data = trie.get_data("word", true).unwrap();
    /// found_data.sort();
    /// assert_eq!(soft_data, found_data);
    /// ```
    pub fn get_data(&self, query: &str, soft_match: bool) -> Option<Vec<&D>> {
        let current = self.get_final_node(query)?;

        return if soft_match {
            let mut soft_match_data = Vec::new();
            current.generate_all_data(&mut soft_match_data);

            Some(soft_match_data)
        } else {
            current
                .get_association()
                .as_ref()
                .map(|data_vec| data_vec.iter().collect())
        };
    }

    /// Returns a vector of mutable references to data of some word that equals 'query'
    /// or mutable references to all found data of words that begin with 'query'
    /// when 'soft_match' is set to true.
    /// If the word is not found and 'soft_match' is set to false, None is returned.
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut trie = DataTrie::new();
    ///
    /// trie.insert("word1", "somedata");
    /// trie.insert("word2", "somemoredata");
    /// trie.insert("word1", "evenmoredata");
    ///
    /// let mut found_data = trie.get_data_mut("word1", false).unwrap();
    ///
    /// *found_data[0] = "changeddata";
    /// *found_data[1] = "bigchanges";
    ///
    /// let hard_data = vec![&"changeddata", &"bigchanges"];
    /// assert_eq!(hard_data, trie.get_data("word1", false).unwrap());
    ///
    /// let soft_data = vec![&"0", &"1", &"2"];
    /// let mut found_data_mut = trie.get_data_mut("word", true).unwrap();
    /// found_data_mut.sort();
    /// *found_data_mut[0] = "0";
    /// *found_data_mut[1] = "1";
    /// *found_data_mut[2] = "2";
    /// assert_eq!(soft_data, found_data_mut);
    /// ```
    pub fn get_data_mut(&mut self, query: &str, soft_match: bool) -> Option<Vec<&mut D>> {
        let current = self.get_final_node_mut(query)?;

        return if soft_match {
            let mut soft_match_data = Vec::new();
            current.generate_all_data_mut(&mut soft_match_data);

            Some(soft_match_data)
        } else {
            current
                .get_association_mut()
                .as_mut()
                .map(|data_vec| data_vec.iter_mut().collect())
        };
    }

    /// Clears and returns data of some word. If the word is not found returns None.
    /// If there is no data associated to the word, an empty vector is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut trie = DataTrie::new();
    ///
    /// trie.insert("word", "data1");
    /// trie.insert("word", "data2");
    /// trie.insert("word", "data3");
    /// let found_data = trie.clear_data("word");
    ///
    /// assert_eq!(Vec::<&&str>::new(), trie.get_data("word", false).unwrap());
    /// assert_eq!(vec!["data1", "data2", "data3"], found_data.unwrap());
    /// ```
    pub fn clear_data(&mut self, word: &str) -> Option<Vec<D>> {
        let current = self.get_final_node_mut(word)?;

        current
            .clear_word_end_association(true)
            .map(|data_vec| {
                data_vec.into_iter().collect()
            })
    }

    /// Returns an option enum with a vector of owned strings
    /// representing all found words that begin with 'query'.
    /// If the word 'query' doesn't exist, None is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut data_trie = DataTrie::new();
    ///
    /// data_trie.insert("word1", 1);
    /// data_trie.insert("word2", 2);
    ///
    /// let all_correct_words = vec![String::from("word1"), String::from("word2")];
    /// let mut found_words = data_trie.get("word").unwrap();
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
    /// use basic_trie::DataTrie;
    /// let mut data_trie = DataTrie::new();
    ///
    /// data_trie.insert("shortwrd", 1);
    /// data_trie.insert("verylongword", 2);
    /// data_trie.insert("somelongword", 2);
    ///
    /// let longest_words = vec![String::from("somelongword"), String::from("verylongword")];
    /// let mut found_words = data_trie.get_longest();
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
    /// use basic_trie::DataTrie;
    /// let mut data_trie = DataTrie::new();
    ///
    /// data_trie.insert("shortwrd", 1);
    /// data_trie.insert("rlyshort", 2);
    /// data_trie.insert("verylongword", 3);
    ///
    /// let shortest_word = vec![String::from("rlyshort"), String::from("shortwrd")];
    /// let mut found_words = data_trie.get_shortest();
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
    /// use basic_trie::DataTrie;
    /// let mut data_trie = DataTrie::new();
    ///
    /// data_trie.insert("word1", 1);
    /// data_trie.insert("word2", 2);
    /// data_trie.insert("word3", 3);
    /// data_trie.insert("word4", 4);
    /// assert_eq!(4, data_trie.len());
    ///
    /// data_trie.remove("word1");
    /// assert_eq!(3, data_trie.len());
    ///
    /// data_trie.remove_prefix("w");
    /// assert_eq!(0, data_trie.len());
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
    /// use basic_trie::DataTrie;
    /// let mut data_trie = DataTrie::new();
    ///
    /// data_trie.insert("word1", 1);
    /// data_trie.insert("word2", 2);
    /// data_trie.insert("word3", 3);
    /// data_trie.insert("word4", 4);
    /// data_trie.insert("word5", 5);
    ///
    /// let all_words = vec![
    ///     String::from("word1"), String::from("word2"), String::from("word3"),
    ///     String::from("word4"), String::from("word5")
    /// ];
    ///
    /// let mut found_words = data_trie.get_all();
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
    /// use basic_trie::DataTrie;
    /// let mut data_trie = DataTrie::new();
    ///
    /// data_trie.insert("word", 0);
    /// assert!(data_trie.contains("word"));
    /// assert!(!data_trie.contains("notfound"));
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
    /// let mut data_trie = Trie::new();
    ///
    /// data_trie.insert("word");
    /// data_trie.remove("word");
    ///
    /// assert!(data_trie.is_empty());
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
    /// let mut data_trie = Trie::new();
    ///
    /// data_trie.insert("word1");
    /// data_trie.insert("word2");
    /// data_trie.insert("word3");
    /// data_trie.insert("word4");
    ///
    /// data_trie.clear();
    /// assert!(data_trie.is_empty());
    /// assert_eq!(0, data_trie.len());
    /// ```
    pub fn clear(&mut self) {
        self.root.clear_children();
        self.len = 0;
    }

    /// Function for getting the last node in a character sequence.
    fn get_final_node(&self, query: &str) -> Option<&TrieDataNode<D>> {
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
    fn get_final_node_mut(&mut self, query: &str) -> Option<&mut TrieDataNode<D>> {
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

impl<D> ops::Add for DataTrie<D> {
    type Output = DataTrie<D>;

    /// Operation + merges two tries, leaving out duplicate words.
    /// The smaller trie is always added to the larger one for efficiency.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut data_trie_1 = DataTrie::new();
    /// data_trie_1.insert("word1", 1);
    /// data_trie_1.insert("word2", 2);
    /// data_trie_1.insert("word", 0);
    ///
    /// let mut data_trie_2 = DataTrie::new();
    /// data_trie_2.insert("word3", 3);
    /// data_trie_2.insert_no_data("word");
    ///
    /// let mut correct = DataTrie::new();
    /// correct.insert("word", 0);
    /// correct.insert("word1", 1);
    /// correct.insert("word2", 2);
    /// correct.insert("word3", 3);
    ///
    /// let data_trie_3 = data_trie_1 + data_trie_2;
    ///
    /// assert_eq!(data_trie_3, correct);
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

impl<D> ops::AddAssign for DataTrie<D> {
    /// Operation += merges two tries, leaving out duplicate words.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut data_trie_1 = DataTrie::new();
    /// data_trie_1.insert("word1", 1);
    /// data_trie_1.insert("word2", 2);
    /// data_trie_1.insert("word", 0);
    ///
    /// let mut data_trie_2 = DataTrie::new();
    /// data_trie_2.insert("word3", 3);
    /// data_trie_2.insert_no_data("word");
    ///
    /// let mut correct = DataTrie::new();
    /// correct.insert("word", 0);
    /// correct.insert("word1", 1);
    /// correct.insert("word2", 2);
    /// correct.insert("word3", 3);
    ///
    /// data_trie_1 += data_trie_2;
    ///
    /// assert_eq!(data_trie_1, correct);
    /// ```
    fn add_assign(&mut self, rhs: Self) {
        self.root += rhs.root;
    }
}

impl<D: PartialEq> PartialEq for DataTrie<D> {
    /// Operation '==' can be applied only to tries whose data implements PartialEq.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DataTrie;
    /// let mut data_trie_1 = DataTrie::new();
    /// data_trie_1.insert("test", 1);
    ///
    /// let mut data_trie_2 = DataTrie::new();
    /// data_trie_2.insert("test", 1);
    ///
    /// assert_eq!(data_trie_1, data_trie_2);
    ///
    /// data_trie_2.insert("test2", 2);
    ///
    /// assert_ne!(data_trie_1, data_trie_2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
    }
}
