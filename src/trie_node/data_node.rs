use std::collections::HashMap;
use std::marker::PhantomData;
use crate::data::YesData;
use crate::trie_node::{TrieNode, NodeAssociation};

/// Helper struct for returning multiple values for deleting data.
/// It is needed because the 'must_keep' value will at some point change
/// from false to true, but the data stays the same from the beginning of
/// unwinding.
pub(crate) struct RemoveData<D> {
    must_keep: bool,
    pub(crate) data: NodeAssociation<D>
}

pub type TrieDataNode<'a, D> = TrieNode<'a, D, YesData>;

impl<'a, D> TrieDataNode<'a, D> {
    /// Returns a new instance of a TrieNode with the given character.
    pub(crate) fn new() -> Self {
        TrieDataNode {
            children: HashMap::new(),
            word_end_association: NodeAssociation::NoAssociation,
            pd: PhantomData::<YesData>,
        }
    }

    /// Recursive function for removing and freeing memory of a word that is not needed anymore.
    /// The algorithm first finds the last node of a word given in the form of a character iterator,
    /// then it frees the maps and unwinds to the first node that should not be deleted.
    /// The first node that should not be deleted is either:
    /// - the root node
    /// - the node that has multiple words branching from it
    /// - the node that represents an end to some word with the same prefix
    /// The last node's data (or lack thereof in case of dataless trie)
    /// is propagated all the way to the final return with the help of auxiliary 'RemoveData<D>' struct.
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
            must_keep: self.word_end_association.is_associated(),
            data: must_keep.data
        }
    }

    /// Recursive function that drops all children maps and collects data
    /// regardless of having multiple words branching from them or not.
    pub(crate) fn remove_all_words_collect(&mut self, found_data: &mut Vec<D>) {
        self.children.values_mut().for_each(|child| {
            child.remove_all_words_collect(found_data);
        });

        if let NodeAssociation::Data(data_vec) = self.clear_word_end_association(false) {
            found_data.extend(data_vec);
        }

        self.clear_children();
    }

    /// Recursive function finds every node that is an end of a word and appends
    /// it's data to the passed vector.
    pub(crate) fn generate_all_data<'b>(&'b self, found_data: &mut Vec<&'b D>) {
        if let NodeAssociation::Data(data_vec) = &self.word_end_association {
            found_data.extend(data_vec.iter());
        }

        self.children
            .values()
            .for_each(|x| x.generate_all_data(found_data));
    }

    /// Function resets the association of a word.
    pub(crate) fn clear_word_end_association(&mut self, keep_word: bool) -> NodeAssociation<D> {
        let return_data = std::mem::take(&mut self.word_end_association);
        if keep_word {
            if let NodeAssociation::Data(_) = return_data {
                self.word_end_association = NodeAssociation::Data(Vec::new())
            }
        } else {
            self.word_end_association = NodeAssociation::NoAssociation;
        }

        return_data
    }
}