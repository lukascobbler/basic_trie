use crate::trie::{get_characters, Trie};
use crate::trie_node::{NodeAssociation, TrieDataNode};
use crate::data::YesData;

pub type DataTrie<D> = Trie<D, YesData>;

impl <D> DataTrie<D> {
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
    pub fn insert(&mut self, word: &str, associated_data: D) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current.children.entry(Box::from(character)).or_insert_with(TrieDataNode::new);
        }

        current.associate(true);
        current.push_data(associated_data);
        self.len += 1;
    }

    /// Insert a word into the trie, with no corresponding data.
    /// This function is very different from inserting a word into
    /// a dataless trie, since it enables later attachment of data
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
    /// assert_eq!(vec![String::from("word1")], trie.all_words().unwrap());
    ///
    /// trie.insert("word1", "somedata");
    /// assert_eq!(vec![&"somedata"], trie.find_data_of_word("word1", false).unwrap());
    /// ```
    pub fn insert_no_data(&mut self, word: &str) {
        let characters = get_characters(word);
        let mut current = &mut self.root;

        for character in characters {
            current = current.children.entry(Box::from(character)).or_insert_with(TrieDataNode::new);
        }

        current.associate(true);
        self.len += 1;
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
        let Some(current) = self.get_final_node_mut(word) else { return None };

        if !current.children.is_empty() {
            return match current.clear_word_end_association(false) {
                NodeAssociation::Data(data_vec) if !data_vec.is_empty() => {
                    self.len -= 1;
                    Some(*data_vec)
                },
                _ => None
            }
        }

        let characters = get_characters(word);

        match self.root.remove_one_word(characters.into_iter()).data {
            NodeAssociation::Data(data_vec) if !data_vec.is_empty() => {
                self.len -= 1;
                Some(*data_vec)
            },
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
        let Some(current) = self.get_final_node_mut(prefix) else { return None };

        let mut data_vec = Vec::new();

        // Sum must be applied to the node's children and not to the node
        // itself because the recursive function must disassociate a node
        // to put it's data in the vector. The optimization of adding one
        // to the count when the node in question isn't root can't be used
        // since the original node would've been already disassociated therefore
        // not accounted for in self.len.
        let word_count = current.children.values_mut().map(
            |child| child.remove_all_words_collect(&mut data_vec)
        ).sum::<usize>();
        current.clear_children();

        self.len -= word_count;

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
        let Some(current) = self.get_final_node(query) else { return None };

        return if soft_match {
            let mut soft_match_data = Vec::new();
            current.generate_all_data(&mut soft_match_data);

            if soft_match_data.is_empty() {
                return None;
            }

            Some(soft_match_data)
        } else {
            match current.get_association() {
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
    pub fn clear_word_data(&mut self, word: &str) -> Option<Vec<D>> {
        let Some(current) = self.get_final_node_mut(word) else { return None };

        match current.clear_word_end_association(true) {
            NodeAssociation::Data(data_vec) if !data_vec.is_empty() =>
                Some(*data_vec),
            _ => None
        }
    }
}