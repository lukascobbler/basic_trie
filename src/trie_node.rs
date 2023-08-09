use std::cmp::Ordering;
use std::collections::HashMap;
use std::marker::PhantomData;

mod data_node;
mod dataless_node;

pub(crate) use data_node::TrieDataNode;
pub(crate) use dataless_node::TrieDatalessNode;
use crate::data::CData;

/// Helper enum for deciding if a singular TrieNode is an end of a word, and what type
/// of word end is it. It is used as a generic implementation for both the Dataless and Data
/// Tries.
#[derive(Debug)]
pub(crate) enum NodeAssociation<D> {
    Data(Vec<D>),
    NoData,
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

/// Singular trie node that represents one character,
/// it's children and a marker for word ending.
#[derive(Debug, Default)]
pub struct TrieNode<'a, D, HasData: CData> {
    pub(crate) children: HashMap<&'a str, TrieNode<'a, D, HasData>>,
    word_end_association: NodeAssociation<D>,
    pd: PhantomData<HasData>
}

impl<'a, D, HasData: CData> TrieNode<'a, D, HasData> {
    /// Returns a new instance of a TrieNode with the given character.
    pub(crate) fn new() -> Self {
        TrieNode {
            children: HashMap::new(),
            word_end_association: NodeAssociation::NoAssociation,
            pd: PhantomData::<HasData>,
        }
    }

    /// Recursive function for getting the number of words from a given node.
    pub(crate) fn number_of_words(&self) -> usize {
        self.children
            .values()
            .map(|x| x.number_of_words())
            .sum::<usize>()
            + (self.is_associated()) as usize
    }

    /// Recursive function for inserting found words from the given node and
    /// given starting substring.
    pub(crate) fn find_words(&self, substring: &str, found_words: &mut Vec<String>) {
        if self.is_associated() {
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

        self.children.iter().for_each(|(&character, node)| {
            node.words_min_max(&(substring.to_owned() + character), found_words, ord)
        });
    }

    /// Recursive function that drops all children maps
    /// regardless of having multiple words branching from them or not.
    pub(crate) fn remove_all_words(&mut self) {
        self.children.values_mut().for_each(|child| {
            child.remove_all_words();
        });

        self.clear_children();
    }

    /// Function resets the association of a word.
    pub(crate) fn clear_word_end_association(&mut self, keep_word: bool) -> NodeAssociation<D> {
        let return_data = std::mem::take(&mut self.word_end_association);
        if keep_word {
            // If is present to ensure not calling this function
            // on a node that has no association.
            // In case of calling this function on a dataless node, there
            // is nothing to be done if the word should be kept.
            if let NodeAssociation::Data(_) = return_data {
                self.word_end_association = NodeAssociation::Data(Vec::new())
            }
        } else {
            self.word_end_association = NodeAssociation::NoAssociation;
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
    pub(crate) fn remove_one_word<'b, I>(&mut self, mut characters: I) -> RemoveData<D>
        where
            I: Iterator<Item = &'b str>,
    {
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
                true => self.word_end_association = NodeAssociation::Data(Vec::new()),
                false => self.word_end_association = NodeAssociation::NoData
            }
        }
    }

    /// Function unmarks the node as an end of a word.
    pub(crate) fn disassociate(&mut self) {
        self.word_end_association = NodeAssociation::NoAssociation;
    }

    /// Function returns true if any type of association is found for the word.
    pub(crate) fn is_associated(&self) -> bool {
        !matches!(self.word_end_association, NodeAssociation::NoAssociation)
    }

    /// Function returns the node association.
    pub(crate) fn get_association(&self) -> &NodeAssociation<D> {
        &self.word_end_association
    }

    /// Function removes all children of a node.
    fn clear_children(&mut self) {
        self.children = HashMap::new();
    }
}