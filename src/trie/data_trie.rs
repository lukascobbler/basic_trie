use std::marker::PhantomData;
use crate::trie::{get_characters, Trie};
use crate::trie_node::{NodeAssociation, TrieDataNode};
use crate::data::YesData;

pub type DataTrie<'a, D> = Trie<'a, D, YesData>;

impl <'a, D> DataTrie<'a, D> {
    /// Returns a new instance of the data trie.
    pub fn new() -> Self {
        Trie {
            root: TrieDataNode::<D>::new(),
            pd: PhantomData::<YesData>
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
    /// assert_eq!(vec![String::from("word1")], trie.all_words().unwrap());
    /// ```
    pub fn insert(&mut self, word: &'a str, associated_data: D) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current.children.entry(character).or_insert_with(TrieDataNode::new);
        }

        if !current.word_end_association.is_associated() {
            current.word_end_association = NodeAssociation::Data(Vec::new());
        }

        current.word_end_association.push_data(associated_data);
    }

    /// Removes a word from the trie and returns data associated with that word.
    /// If the word is a prefix to some word, some word
    /// isn't removed from the trie.
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
            return match current.clear_word_end_association(false) {
                NodeAssociation::Data(data_vec) => Some(data_vec),
                _ => None
            }
        }

        match self.root.remove_one_word(characters.into_iter()).data {
            NodeAssociation::Data(data_vec) => Some(data_vec),
            _ => None
        }
    }

    /// Removes every word that begins with 'prefix' and collects all removed data.
    /// Not including the word 'prefix' if it's present.
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

    /// Returns all data associated to "query" wrapped in the Option enum, in case there is no data
    /// for the specified arguments.
    /// Parameter soft_match dictates if the returned data should be associated only with "query"
    /// or with all words that begin with "query".
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
    /// assert_eq!(hard_data, trie.find_data_of_word("word1", false).unwrap());
    ///
    /// let soft_data = vec![&"somedata", &"somemoredata"];
    /// let mut found_data = trie.find_data_of_word("word", true).unwrap();
    /// found_data.sort();
    /// assert_eq!(soft_data, found_data);
    /// ```
    pub fn find_data_of_word(&self, query: &str, soft_match: bool) -> Option<Vec<&D>> {
        let current = match self.get_final_node(query) {
            None => return None,
            Some(node) => node
        };

        return if soft_match {
            let mut soft_match_data = Vec::new();
            current.generate_all_data(&mut soft_match_data);

            if soft_match_data.is_empty() {
                return None;
            }

            Some(soft_match_data)
        } else {
            match &current.word_end_association {
                NodeAssociation::Data(data_vec) if !data_vec.is_empty() =>
                    Some(data_vec.iter().collect()),
                _ => None
            }
        };
    }

    /// Clears and returns data of some word. If the word is not found,
    /// or there is no data associated with the word, nothing happens.
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
    /// let found_data = trie.clear_word_data("word");
    ///
    /// assert_eq!(None, trie.find_data_of_word("word", false));
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

        match current.clear_word_end_association(true) {
            NodeAssociation::Data(data_vec) if !data_vec.is_empty() =>
                Some(data_vec),
            _ => None
        }
    }
}
