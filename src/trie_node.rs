use std::cmp::Ordering;
use std::collections::HashMap;

mod data_node;
mod dataless_node;

pub(crate) use data_node::TrieDataNode;
pub(crate) use dataless_node::TrieDatalessNode;

/// Helper enum for deciding if a singular TrieNode is an end of a word, and what type
/// of word end is it. It is used as a generic implementation for both the Dataless and Data
/// Tries.
#[derive(Debug)]
pub(crate) enum NodeAssociation<D> {
    Data(Vec<D>),
    NoData,
    NoAssociation
}

impl<D> NodeAssociation<D> {
    /// Function pushes data to the 'Data' variant's inner vector.
    pub(crate) fn push_data(&mut self, data: D) {
        if let NodeAssociation::Data(vec) = self {
            vec.push(data);
        }
    }

    /// Returns true if any type of association is matched.
    pub(crate) fn is_associated(&self) -> bool {
        !matches!(self, NodeAssociation::NoAssociation)
    }
}

/// Default is having no association on a TrieNode.
impl<D> Default for NodeAssociation<D> {
    fn default() -> Self {
        NodeAssociation::NoAssociation
    }
}

/// Singular trie node that represents one character,
/// it's children and data associated with the character
/// if it's a word.
#[derive(Debug, Default)]
pub struct TrieNode<'a, D, HasData> {
    pub(crate) children: HashMap<&'a str, TrieNode<'a, D, HasData>>,
    pub(crate) word_end_association: NodeAssociation<D>,
    pd: std::marker::PhantomData<HasData>
}

impl<'a, D, HasData> TrieNode<'a, D, HasData> {
    /// Recursive function for getting the number of words from a given node.
    pub(crate) fn number_of_words(&self) -> usize {
        self.children
            .values()
            .map(|x| x.number_of_words())
            .sum::<usize>()
            + (self.word_end_association.is_associated()) as usize
    }

    /// Recursive function for inserting found words from the given node and
    /// given starting substring.
    pub(crate) fn find_words(&self, substring: &str, found_words: &mut Vec<String>) {
        if self.word_end_association.is_associated() {
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
            if self.word_end_association.is_associated() {
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

    /// Function removes all children of a node.
    fn clear_children(&mut self) {
        self.children = HashMap::new();
    }

    /// Recursive function that drops all children maps
    /// regardless of having multiple words branching from them or not.
    pub(crate) fn remove_all_words(&mut self) {
        self.children.values_mut().for_each(|child| {
            child.remove_all_words();
        });

        self.clear_children();
    }
}