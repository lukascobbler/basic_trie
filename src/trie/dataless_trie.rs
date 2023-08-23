use crate::trie::{get_characters, Trie};
use crate::trie_node::{TrieDatalessNode};
use crate::data::NoData;

pub type DatalessTrie = Trie<(), NoData>;

impl DatalessTrie {
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
    pub fn insert(&mut self, word: &str) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current.children.entry(Box::from(character)).or_insert_with(TrieDatalessNode::new);
        }

        current.associate(false);
        self.len += 1;
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
        let Some(current) = self.get_final_node_mut(word) else { return };

        let characters = get_characters(word);

        if !current.children.is_empty() {
            return if current.disassociate() {
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
        let Some(current) = self.get_final_node_mut(prefix) else { return };

        // (current.is_associated() as usize) is added (subtracted twice) to
        // not remove the current word from the count. Literal '1' is not used
        // because of calling this function on the root node where 1 should
        // not be added.
        self.len -= current.remove_all_words() - (current.is_associated() as usize);
    }
}