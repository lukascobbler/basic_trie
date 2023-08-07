use std::collections::HashMap;
use std::marker::PhantomData;
use crate::data::{NoData};
use crate::trie_node::{NodeAssociation, TrieNode};

pub type TrieDatalessNode<'a> = TrieNode<'a, (), NoData>;

impl<'a> TrieDatalessNode<'a> {
    /// Returns a new instance of a TrieNode with the given character.
    pub(crate) fn new() -> Self {
        TrieDatalessNode {
            children: HashMap::new(),
            word_end_association: NodeAssociation::NoAssociation,
            pd: PhantomData::<NoData>,
        }
    }

    /// Recursive function for removing and freeing memory of a word that is not needed anymore.
    /// The algorithm first finds the last node of a word given in the form of a character iterator,
    /// then it frees the maps and unwinds to the first node that should not be deleted.
    /// The first node that should not be deleted is either:
    /// - the root node
    /// - the node that has multiple words branching from it
    /// - the node that represents an end to some word with the same prefix
    pub(crate) fn remove_one_word<'b, I>(&mut self, mut characters: I) -> bool
        where
            I: Iterator<Item = &'b str>,
    {
        let next_character = match characters.next() {
            None => {
                self.word_end_association = NodeAssociation::NoAssociation;
                return false;
            },
            Some(char) => char
        };

        let next_node = self.children.get_mut(next_character).unwrap();
        let must_keep = next_node.remove_one_word(characters);

        if self.children.len() > 1 || must_keep {
            return true;
        }

        self.clear_children();

        self.word_end_association.is_associated()
    }
}