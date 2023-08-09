use crate::trie::{get_characters, Trie};
use crate::trie_node::{TrieDatalessNode};
use crate::data::NoData;

pub type DatalessTrie<'a> = Trie<'a, (), NoData>;

impl<'a> DatalessTrie<'a> {
    /// Insert a word into the trie, with no corresponding data.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word1");
    /// assert_eq!(vec![String::from("word1")], trie.all_words().unwrap());
    /// ```
    pub fn insert(&mut self, word: &'a str) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current.children.entry(character).or_insert_with(TrieDatalessNode::new);
        }

        current.associate(false);
    }

    /// Removes a word from the dataless trie.
    /// If the word is a prefix to some word, some word
    /// isn't removed from the trie.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("word");
    /// trie.insert("wording");
    ///
    /// trie.remove_word("word");
    /// assert_eq!(vec![String::from("wording")], trie.find_words("word").unwrap());
    ///
    /// trie.remove_word("wording");
    /// assert_eq!(None, trie.all_words());
    /// ```
    pub fn remove_word(&mut self, word: &str) {
        let characters = get_characters(word);

        let mut current = &mut self.root;

        for character in characters.iter() {
            current = match current.children.get_mut(*character) {
                None => return,
                Some(next_node) => next_node
            };
        }

        if !current.children.is_empty() {
            current.disassociate();
            return;
        }

        self.root.remove_one_word(characters.into_iter());
    }

    /// Removes every word that begins with 'prefix'.
    /// Not including the word 'prefix' if it's present.
    ///
    /// # Examples
    ///
    /// ```
    /// use basic_trie::DatalessTrie;
    /// let mut trie = DatalessTrie::new();
    ///
    /// trie.insert("eat");
    /// trie.insert("eats");
    /// trie.insert("eating");
    /// trie.insert("eatings");
    /// trie.insert("ea");
    ///
    /// trie.remove_words_from_prefix("ea");
    ///
    /// assert_eq!(vec![String::from("ea")], trie.all_words().unwrap());
    /// ```
    pub fn remove_words_from_prefix(&mut self, prefix: &str) {
        let characters = get_characters(prefix);
        let mut current = &mut self.root;

        for character in characters {
            current = match current.children.get_mut(character) {
                None => return,
                Some(next_node) => next_node,
            };
        }

        current.remove_all_words();
    }
}