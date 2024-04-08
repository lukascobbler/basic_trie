use fxhash::FxHashMap;
use std::cmp::Ordering;
use std::ops;
use thin_vec::ThinVec;

#[cfg(feature = "serde")]
use serde_crate::{Deserialize, Serialize};

type WordEnd<D> = Option<ThinVec<D>>;

/// Helper struct for returning multiple values for deleting data.
/// It is needed because the 'must_keep' value will at some point change
/// from false to true, but the data stays the same from the beginning of
/// unwinding.
pub(crate) struct RemoveData<D> {
    must_keep: bool,
    pub(crate) data: WordEnd<D>,
}

/// Singular trie node that represents its children and a marker for word ending.
#[derive(Debug, Default, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct TrieDataNode<D> {
    #[cfg_attr(feature = "serde", serde(rename = "c"))]
    pub(crate) children: Box<FxHashMap<arrayvec::ArrayString<4>, TrieDataNode<D>>>,
    #[cfg_attr(feature = "serde", serde(rename = "wed"))]
    word_end_data: WordEnd<D>,
}

/// Methods only on nodes that have data.
impl<D> TrieDataNode<D> {
    /// Returns a new instance of a TrieNode.
    pub(crate) fn new() -> Self {
        TrieDataNode {
            children: Default::default(),
            word_end_data: None,
        }
    }

    /// Recursive function that drops all children maps and collects data
    /// regardless of having multiple words branching from them or not.
    pub(crate) fn remove_all_words_collect(&mut self, found_data: &mut Vec<D>) -> usize {
        let num_removed = self
            .children
            .values_mut()
            .map(|child| child.remove_all_words_collect(found_data))
            .sum::<usize>()
            + self.is_associated() as usize;

        if let Some(data_vec) = self.disassociate() {
            found_data.extend(data_vec);
        }

        self.clear_children();

        num_removed
    }

    /// Recursive function finds every node that is an end of a word and appends
    /// its data as references to the passed vector.
    pub(crate) fn generate_all_data<'a>(&'a self, found_data: &mut Vec<&'a D>) {
        if let Some(data_vec) = &self.word_end_data {
            found_data.extend(data_vec.iter());
        }

        self.children
            .values()
            .for_each(|x| x.generate_all_data(found_data));
    }

    /// Recursive function finds every node that is an end of a word and appends
    /// its data as mutable references to the passed vector.
    pub(crate) fn generate_all_data_mut<'a>(&'a mut self, found_data: &mut Vec<&'a mut D>) {
        if let Some(data_vec) = &mut self.word_end_data {
            found_data.extend(data_vec.iter_mut());
        }

        self.children
            .values_mut()
            .for_each(|x| x.generate_all_data_mut(found_data));
    }

    /// Function pushes data to the association vector.
    pub(crate) fn push_data(&mut self, data: D) {
        self.get_association_mut().as_mut().unwrap().push(data);
    }

    /// Recursive function for inserting found words from the given node and
    /// given starting substring.
    pub(crate) fn find_words(&self, substring: &str, found_words: &mut Vec<String>) {
        if self.is_associated() {
            found_words.push(substring.to_string());
        }

        self.children.iter().for_each(|(character, node)| {
            node.find_words(&(substring.to_owned() + character), found_words)
        });
    }

    /// The recursive function for finding a vector of shortest and longest words in the TrieNode consists of:
    /// - the DFS tree traversal part for getting to every child node;
    /// - matching lengths of found words in combination with the passed ordering.
    pub(crate) fn words_min_max(
        &self,
        substring: &str,
        found_words: &mut Vec<String>,
        ord: Ordering,
    ) {
        'word: {
            if self.is_associated() {
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

        self.children.iter().for_each(|(character, node)| {
            node.words_min_max(&(substring.to_owned() + character), found_words, ord)
        });
    }

    /// Function resets the association of a word and returns the
    /// previous association. If 'keep_word' is true, the association is only
    /// reset.
    pub(crate) fn clear_word_end_association(&mut self, keep_word: bool) -> WordEnd<D> {
        let return_data = self.disassociate();

        if keep_word && return_data.is_some() {
            self.associate();
        }

        return_data
    }

    /// Recursive function for removing and freeing memory of a word that is not needed anymore.
    /// The algorithm first finds the last node of a word given in the form of a character iterator,
    /// then it frees the maps and unwinds to the first node that should not be deleted.
    /// The first node that should not be deleted is either:
    /// - the root node
    /// - the node that has multiple words branching from it
    /// - the node that represents an end to some word with the same prefix
    /// The last node's data is propagated all the way to the final return
    /// with the help of auxiliary 'RemoveData<D>' struct.
    pub(crate) fn remove_one_word<'b>(
        &mut self,
        mut characters: impl Iterator<Item = &'b str>,
    ) -> RemoveData<D> {
        let next_character = match characters.next() {
            None => {
                return RemoveData {
                    must_keep: false,
                    data: self.disassociate()
                }
            }
            Some(char) => char,
        };

        let next_node = self.children.get_mut(next_character).unwrap();
        let must_keep = next_node.remove_one_word(characters);

        if self.children.len() > 1 || must_keep.must_keep {
            return RemoveData {
                must_keep: true,
                data: must_keep.data,
            };
        }
        self.clear_children();

        RemoveData {
            must_keep: self.is_associated(),
            data: must_keep.data,
        }
    }

    /// Function marks the node as an end of a word.
    pub(crate) fn associate(&mut self) {
        self.word_end_data = Some(ThinVec::new());
    }

    /// Function unmarks the node as an end of a word and returns the data.
    pub(crate) fn disassociate(&mut self) -> WordEnd<D> {
        self.word_end_data.take()
    }

    /// Function returns true if an association is found for the word.
    pub(crate) fn is_associated(&self) -> bool {
        self.word_end_data.is_some()
    }

    /// Function returns the node association.
    pub(crate) fn get_association(&self) -> &WordEnd<D> {
        &self.word_end_data
    }

    /// Function returns the mutable node association.
    pub(crate) fn get_association_mut(&mut self) -> &mut WordEnd<D> {
        &mut self.word_end_data
    }

    /// Function removes all children of a node.
    pub(crate) fn clear_children(&mut self) {
        self.children = Default::default();
    }
}

impl<D> ops::AddAssign for TrieDataNode<D> {
    /// Overriding the += operator on nodes.
    /// Function adds two nodes based on the principle:
    /// for every child node and character in the 'rhs' node:
    /// - if the self node doesn't have that character in its children map,
    /// simply move the pointer to the self's children map without any extra cost;
    /// - if the self node has that character, the node of that character (self's child)
    /// is added with the 'rhc's' node.
    /// An edge case exists when the 'rhc's' node has an association but self's node doesn't.
    /// That association is handled based on the result of 'rhc_next_node.word_end_data'.
    /// On Some(data), the self node vector is initialized with the 'rhc' node vector.
    fn add_assign(&mut self, rhs: Self) {
        for (char, mut rhs_next_node) in rhs.children.into_iter() {
            // Does self contain the character?
            match self.children.remove(&*char) {
                // The whole node is removed, as owned, operated on and returned in self's children.
                Some(mut self_next_node) => {
                    // Edge case: associate self node if the other node is also associated
                    // Example: when adding 'word' to 'word1', 'd' on 'word' needs to be associated
                    if let Some(data_vec_rhs) = rhs_next_node.word_end_data.take() {
                        if let Some(data_vec_self) = &mut self_next_node.word_end_data {
                            data_vec_self.extend(data_vec_rhs);
                        } else {
                            self_next_node.word_end_data = Some(data_vec_rhs);
                        }
                    }

                    self_next_node += rhs_next_node;
                    self.children.insert(char, self_next_node);
                }
                // Self doesn't contain the character, no conflict arises.
                // The whole 'rhs' node is just moved from 'rhs' into self.
                None => {
                    self.children.insert(char, rhs_next_node);
                }
            }
        }
    }
}

impl<D: PartialEq> PartialEq for TrieDataNode<D> {
    /// Operation == can be applied only to TrieNodes whose data implements PartialEq.
    fn eq(&self, other: &Self) -> bool {
        // If keys aren't equal, nodes aren't equal.
        if !(self.children.len() == other.children.len() && self.children.keys().all(|k| other.children.contains_key(k))) {
            return false;
        }

        // If associations aren't equal, two nodes aren't equal.
        if self.word_end_data != other.word_end_data {
            return false;
        }

        // Every child node that has the same key (character) must be equal.
        self.children
            .iter()
            .map(|(char, self_child)| (self_child, other.children.get(char).unwrap()))
            .all(|(self_child, other_child)| other_child == self_child)
    }
}
