use fxhash::FxHashMap;
use std::cmp::Ordering;
use std::ops;

#[cfg(feature = "serde")]
use serde_crate::{Deserialize, Serialize};

/// Singular trie node that represents its children and a marker for word ending.
#[derive(Debug, Default, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
pub struct TrieDatalessNode {
    #[cfg_attr(feature = "serde", serde(rename = "c"))]
    pub(crate) children: Box<FxHashMap<arrayvec::ArrayString<4>, TrieDatalessNode>>,
    #[cfg_attr(feature = "serde", serde(rename = "we"))]
    word_end: bool,
}

impl TrieDatalessNode {
    /// Returns a new instance of a TrieNode.
    pub(crate) fn new() -> Self {
        TrieDatalessNode {
            children: Default::default(),
            word_end: false,
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

    /// Recursive function that drops all children maps
    /// regardless of having multiple words branching from them or not.
    /// Counts the number of words removed.
    pub(crate) fn remove_all_words(&mut self) -> usize {
        let num_removed = self
            .children
            .values_mut()
            .map(|child| child.remove_all_words())
            .sum::<usize>()
            + self.is_associated() as usize;

        self.clear_children();

        num_removed
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
    ) -> bool {
        let next_character = match characters.next() {
            None => {
                self.disassociate();
                return false;
            }
            Some(char) => char,
        };

        let next_node = self.children.get_mut(next_character).unwrap();
        let must_keep = next_node.remove_one_word(characters);

        if self.children.len() > 1 || must_keep {
            return true;
        }
        self.clear_children();

        self.is_associated()
    }

    /// Function marks the node as an end of a word.
    pub(crate) fn associate(&mut self) {
        self.word_end = true;
    }

    /// Function unmarks the node as an end of a word.
    pub(crate) fn disassociate(&mut self) {
        self.word_end = false;
    }

    pub(crate) fn is_associated(&self) -> bool {
        self.word_end
    }

    /// Function removes all children of a node.
    pub(crate) fn clear_children(&mut self) {
        self.children = Default::default();
    }
}

impl ops::AddAssign for TrieDatalessNode {
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
        for (char, rhs_next_node) in rhs.children.into_iter() {
            // Does self contain the character?
            match self.children.remove(&*char) {
                // The whole node is removed, as owned, operated on and returned in self's children.
                Some(mut self_next_node) => {
                    // Edge case: associate self node if the other node is also associated
                    // Example: when adding 'word' to 'word1', 'd' on 'word' needs to be associated
                    if rhs_next_node.word_end {
                        self_next_node.word_end = true;
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

impl PartialEq for TrieDatalessNode {
    fn eq(&self, other: &Self) -> bool {
        // If keys aren't equal, nodes aren't equal.
        if self.children.keys().ne(other.children.keys()) {
            return false;
        }

        // If the node on one trie is a word end, and on the other it isn't, two nodes aren't equal.
        if self.word_end != other.word_end {
            return false;
        }

        // Every child node that has the same key (character) must be equal.
        self.children
            .iter()
            .map(|(char, self_child)| (self_child, other.children.get(char).unwrap()))
            .all(|(self_child, other_child)| other_child == self_child)
    }
}
