use std::cmp::Ordering;
use fxhash::FxHashMap;
use std::marker::PhantomData;
use std::ops;

#[cfg(feature = "serde")]
use serde_crate::{Deserialize, Serialize};

#[cfg(feature = "data")]
mod data_node;

mod dataless_node;

#[cfg(feature = "data")]
pub(crate) use data_node::TrieDataNode;

pub(crate) use dataless_node::TrieDatalessNode;
use crate::data::CData;

/// Helper enum for deciding if a singular TrieNode is an end of a word, and what type
/// of word end is it. It is used as a generic implementation for both the Dataless and Data
/// Tries.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub(crate) enum NodeAssociation<D> {
    #[cfg_attr(feature = "serde", serde(rename = "D"))]
    Data(Box<Vec<D>>),
    #[cfg_attr(feature = "serde", serde(rename = "NoD"))]
    NoData,
    #[cfg_attr(feature = "serde", serde(rename = "NoA"))]
    NoAssociation
}

/// Default is having no association on a TrieNode.
impl<D> Default for NodeAssociation<D> {
    fn default() -> Self {
        NodeAssociation::NoAssociation
    }
}

/// Helper struct for returning multiple values for deleting data.
/// It is needed because the 'must_keep' value will at some point change
/// from false to true, but the data stays the same from the beginning of
/// unwinding.
pub(crate) struct RemoveData<D> {
    must_keep: bool,
    pub(crate) data: NodeAssociation<D>
}

/// Singular trie node that represents it's children and a marker for word ending.
#[derive(Debug, Default, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct TrieNode<D, HasData: CData> {
    pub(crate) children: FxHashMap<Box<str>, TrieNode<D, HasData>>,
    #[cfg_attr(feature = "serde", serde(rename = "wea"))]
    word_end_association: NodeAssociation<D>,
    #[cfg_attr(feature = "serde", serde(skip))]
    pd: PhantomData<HasData>
}

impl<D, HasData: CData> TrieNode<D, HasData> {
    /// Returns a new instance of a TrieNode.
    pub(crate) fn new() -> Self {
        TrieNode {
            children: FxHashMap::default(),
            word_end_association: NodeAssociation::NoAssociation,
            pd: PhantomData::<HasData>,
        }
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
    pub(crate) fn words_min_max(&self, substring: &str, found_words: &mut Vec<String>, ord: Ordering) {
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

    /// Recursive function that drops all children maps
    /// regardless of having multiple words branching from them or not.
    /// Counts the number of words removed.
    pub(crate) fn remove_all_words(&mut self) -> usize {
        let num_removed = self.children.values_mut().map(
            |child| child.remove_all_words()
        ).sum::<usize>() + self.is_associated() as usize;

        self.clear_children();

        num_removed
    }

    /// Function resets the association of a word and returns the
    /// previous association. If 'keep_word' is true, the association is only
    /// reset.
    pub(crate) fn clear_word_end_association(&mut self, keep_word: bool) -> NodeAssociation<D> {
        let return_data = std::mem::take(&mut self.word_end_association);
        if keep_word {
            // If is present to ensure not calling this function
            // on a node that has no association.
            if let NodeAssociation::Data(_) = return_data {
                self.associate(true);
            } else if let NodeAssociation::NoData = return_data {
                self.associate(false);
            }
        } else {
            self.disassociate();
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
    pub(crate) fn remove_one_word<'b>(&mut self, mut characters: impl Iterator<Item = &'b str>) -> RemoveData<D> {
        let next_character = match characters.next() {
            None => return RemoveData {
                must_keep: false,
                data: self.clear_word_end_association(false)
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
            must_keep: self.is_associated(),
            data: must_keep.data
        }
    }

    /// Function marks the node as an end of a word.
    pub(crate) fn associate(&mut self, data: bool) {
        if !self.is_associated() {
            match data {
                true => self.word_end_association = NodeAssociation::Data(Box::new(Vec::new())),
                false => self.word_end_association = NodeAssociation::NoData
            }
        }
    }

    /// Function unmarks the node as an end of a word. If a word was
    /// previously associated, returns true else false.
    pub(crate) fn disassociate(&mut self) -> bool {
        if self.is_associated() {
            self.word_end_association = NodeAssociation::NoAssociation;
            true
        } else {
            false
        }
    }

    /// Function returns true if any type of association is found for the word.
    pub(crate) fn is_associated(&self) -> bool {
        !matches!(self.word_end_association, NodeAssociation::NoAssociation)
    }

    /// Function returns the node association.
    pub(crate) fn get_association(&self) -> &NodeAssociation<D> {
        &self.word_end_association
    }

    /// Function returns the mutable node association.
    pub(crate) fn get_association_mut(&mut self) -> &mut NodeAssociation<D> {
        &mut self.word_end_association
    }

    /// Function removes all children of a node.
    pub(crate) fn clear_children(&mut self) {
        self.children = FxHashMap::default();
    }
}

impl<D, HasData: CData> ops::AddAssign for TrieNode<D, HasData> {
    /// Overriding the += operator on nodes.
    /// Function adds two nodes based on the principle:
    /// for every child node and character in the 'rhs' node:
    /// - if the self node doesn't have that character in it's children map,
    /// simply move the pointer to the self's children map without any extra cost;
    /// - if the self node has that character, the node of that character (self's child)
    /// is added with the 'rhc's' node.
    /// An edge case exists when the 'rhc's' node has an association but self's node doesn't.
    /// That association is handled based on the 'NodeAssociation' struct result of
    /// 'rhc_next_node.word_end_association'. On 'NodeAssociation::Data', the self node vector
    /// is either extended by the 'rhc' node vector or initialized with it.
    /// On 'NodeAssociation::NoData', the self node association is only initialized as
    /// 'NodeAssociation::NoData'.
    fn add_assign(&mut self, rhs: Self) {
        for (char, mut rhs_next_node) in rhs.children {
            // Does self contain the character?
            match self.children.remove(&*char) {
                // The whole node is removed, as owned, operated on and returned in self's children.
                Some(mut self_next_node) => {

                    // Edge case: associate self node if the other node is also associated
                    // Example: when adding 'word' to 'word1', 'd' on 'word' needs to be associated
                    match std::mem::take(&mut rhs_next_node.word_end_association) {
                        NodeAssociation::Data(data_vec_rhs) => {
                            if let NodeAssociation::Data(data_vec_self) = &mut self_next_node.word_end_association {
                                data_vec_self.extend(*data_vec_rhs);
                            } else {
                                self_next_node.word_end_association = NodeAssociation::Data(data_vec_rhs);
                            }
                        },
                        NodeAssociation::NoData => {
                            self_next_node.associate(false);
                        },
                        NodeAssociation::NoAssociation => {}
                    }

                    self_next_node += rhs_next_node;
                    self.children.insert(char, self_next_node);
                },
                // Self doesn't contain the character, no conflict arises.
                // The whole 'rhs' node is just moved from 'rhs' into self.
                None => {
                    self.children.insert(char, rhs_next_node);
                }
            }
        }
    }
}

impl<D: PartialEq, HasData: CData> PartialEq for TrieNode<D, HasData> {
    /// Operation == can be applied only to TrieNodes whose data implements PartialEq.
    fn eq(&self, other: &Self) -> bool {
        // If keys aren't equal, nodes aren't equal.
        if self.children.keys().ne(other.children.keys()) {
            return false;
        }

        // If associations aren't equal, two nodes aren't equal.
        if self.word_end_association != other.word_end_association {
            return false;
        }

        // Every child node that has the same key (character) must be equal.
        self.children.iter()
            .map(|(char, self_child)|
                (self_child, other.children.get(char).unwrap())
            )
            .all(|(self_child, other_child)|
                other_child == self_child
            )
    }
}