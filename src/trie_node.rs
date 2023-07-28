use std::cmp::Ordering;
use std::collections::HashMap;

/// Helper struct for returning multiple values for deleting data.
/// It is needed because the 'must_keep' value will at some point change
/// from false to true, but the data stays the same from the beginning of
/// unwinding.
pub(crate) struct RemoveData<D> {
    must_keep: bool,
    pub(crate) data: Option<Vec<D>>
}

/// Singular trie node that represents one character,
/// it's children and data associated with the character
/// if it's a word.
#[derive(Debug, Default)]
pub(crate) struct TrieNode<'a, D> {
    pub(crate) children: HashMap<&'a str, TrieNode<'a, D>>,
    pub(crate) associated_data: Option<Vec<D>>,
}

impl<'a, D> TrieNode<'a, D> {
    /// Returns a new instance of a TrieNode with the given character.
    pub(crate) fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            associated_data: None,
        }
    }

    /// Recursive function for getting the number of words from a given node.
    pub(crate) fn number_of_words(&self) -> usize {
        self.children
            .values()
            .map(|x| x.number_of_words())
            .sum::<usize>()
            + (self.associated_data.is_some()) as usize
    }

    /// Recursive function for inserting found words from the given node and
    /// given starting substring.
    pub(crate) fn find_words(&self, substring: &str, found_words: &mut Vec<String>) {
        if self.associated_data.is_some() {
            found_words.push(substring.to_string());
        }

        self.children.iter().for_each(|(&character, node)| {
            node.find_words(&(substring.to_owned() + character), found_words)
        });
    }

    /// The recursive function for finding a vector of shortest and longest words in the TrieNode consists of:
    /// - the DFS tree traversal part for getting to every child node;
    /// - matching lengths of found words in combination with the passed ordering.
    pub(crate) fn words_min_max(&self, substring: &str, found_words: &mut Vec<String>, ord: Ordering) {
        'word: {
            if self.associated_data.is_some() {
                if let Some(found) = found_words.first() {
                    match substring.len().cmp(&found.len()) {
                        Ordering::Less if ord == Ordering::Less => {
                            found_words.clear();
                        }
                        Ordering::Greater if ord == Ordering::Greater => {
                            found_words.clear();
                        }
                        Ordering::Equal => (),
                        _ => break 'word,
                    }
                }
                found_words.push(substring.to_string());
            }
        }

        self.children.iter().for_each(|(&character, node)| {
            node.words_min_max(&(substring.to_owned() + character), found_words, ord)
        });
    }

    /// Recursive function for removing and freeing memory of a word that is not needed anymore.
    /// The algorithm first finds the last node of a word given in the form of a character iterator,
    /// then it frees the maps and unwinds to the first node that should not be deleted.
    /// The first node that should not be deleted is either:
    /// - the root node
    /// - the node that has multiple words branching from it
    /// - the node that represents an end to some word with the same prefix
    /// The last node's data is propagated all the way to the final return with the help
    /// of auxiliary 'RemoveData<D>' struct.
    pub(crate) fn remove_one_word<'b, I>(&mut self, mut characters: I) -> RemoveData<D>
        where
            I: Iterator<Item = &'b str>,
    {
        let next_character = match characters.next() {
            None => return RemoveData {
                must_keep: false,
                data: self.clear_data(false)
            },
            Some(char) => char
        };

        let next_node = self.children.get_mut(next_character).unwrap();
        let must_keep = next_node.remove_one_word(characters);

        if self.children.len() > 1 || must_keep.must_keep {
            return RemoveData {
                must_keep: true,
                data: must_keep.data
            }
        }
        self.clear_children();

        RemoveData {
            must_keep: self.associated_data.is_some(),
            data: must_keep.data
        }
    }

    /// Recursive function that drops all children maps and collects data
    /// regardless of having multiple words branching from them or not.
    pub(crate) fn remove_all_words_collect(&mut self, data_vec: &mut Vec<D>) {
        self.children.values_mut().for_each(|x| {
            x.remove_all_words_collect(data_vec);
        });

        if self.associated_data.is_some() {
            data_vec.extend(self.clear_data(false).unwrap());
        }

        self.clear_children();
    }

    /// Recursive function that drops all children maps
    /// regardless of having multiple words branching from them or not..
    pub(crate) fn remove_all_words(&mut self) {
        self.children.values_mut().for_each(|x| {
            x.remove_all_words();
        });

        self.clear_children();
    }

    /// Recursive function finds every node that is an end of a word and appends
    /// it's data to the passed vector.
    pub(crate) fn generate_all_data<'b>(&'b self, found_data: &mut Vec<&'b D>) {
        if self.associated_data.is_some() {
            found_data.extend(self.associated_data.as_ref().unwrap().iter());
        }

        self.children
            .values()
            .for_each(|x| x.generate_all_data(found_data));
    }

    /// Function resets the data of a word.
    pub(crate) fn clear_data(&mut self, keep_word: bool) -> Option<Vec<D>> {
        let return_data = std::mem::take(&mut self.associated_data);
        if keep_word {
            self.associated_data = Some(Vec::new());
        } else {
            self.associated_data = None;
        }

        return_data
    }

    /// Function adds data to a node.
    pub(crate) fn add_data(&mut self, data: D) {
        if self.associated_data.is_none() {
            self.associated_data = Some(Vec::new());
        }
        self.associated_data.as_mut().unwrap().push(data);
    }

    /// Function removes all children of node.
    pub(crate) fn clear_children(&mut self) {
        self.children = HashMap::new();
    }
}