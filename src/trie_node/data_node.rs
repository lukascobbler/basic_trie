use std::ops;
use crate::data::YesData;
use crate::trie_node::{TrieNode, NodeAssociation};

pub type TrieDataNode<D> = TrieNode<D, YesData>;

/// Methods only on nodes that have data.
impl<D> TrieDataNode<D> {
    /// Recursive function that drops all children maps and collects data
    /// regardless of having multiple words branching from them or not.
    pub(crate) fn remove_all_words_collect(&mut self, found_data: &mut Vec<D>) -> usize {
        let num_removed = self.children.values_mut().map(
            |child| child.remove_all_words_collect(found_data)
        ).sum::<usize>() + self.is_associated() as usize;

        if let NodeAssociation::Data(data_vec) = std::mem::take(&mut self.word_end_association) {
            found_data.extend(data_vec);
        }

        self.clear_children();

        num_removed
    }

    /// Recursive function finds every node that is an end of a word and appends
    /// it's data to the passed vector.
    pub(crate) fn generate_all_data<'a>(&'a self, found_data: &mut Vec<&'a D>) {
        if let NodeAssociation::Data(data_vec) = &self.word_end_association {
            found_data.extend(data_vec.iter());
        }

        self.children
            .values()
            .for_each(|x| x.generate_all_data(found_data));
    }

    /// Function pushes data to the association vector.
    pub(crate) fn push_data(&mut self, data: D) {
        if let NodeAssociation::Data(vec) = &mut self.word_end_association {
            vec.push(data);
        }
    }
}

impl<D> ops::AddAssign for TrieDataNode<D> {
    fn add_assign(&mut self, rhs: Self) {
        for (char, mut rhs_next_node) in rhs.children {
            match self.children.remove(&*char) {
                Some(mut self_next_node) => {
                    // Edge case: associate self node if the other node is also associated
                    // Example: when adding 'word' to 'word1', 'd' on 'word' needs to be associated
                    if let NodeAssociation::Data(data_vec_rhs) = std::mem::take(&mut rhs_next_node.word_end_association) {
                        if let NodeAssociation::Data(data_vec_self) = &mut self_next_node.word_end_association {
                            data_vec_self.extend(data_vec_rhs);
                        } else {
                            self_next_node.word_end_association = NodeAssociation::Data(data_vec_rhs);
                        }
                    }
                    self_next_node += rhs_next_node;
                    self.children.insert(char, self_next_node);
                },
                None => {
                    self.children.insert(char, rhs_next_node);
                }
            }
        }
    }
}

impl<D: PartialEq> PartialEq for TrieDataNode<D> {
    fn eq(&self, other: &Self) -> bool {
        if self.children.keys().ne(other.children.keys()) {
            return false;
        }

        if self.word_end_association != other.word_end_association {
            return false;
        }

        self.children.iter()
            .map(|(char, self_child)|
                (self_child, other.children.get(char).unwrap())
            )
            .all(|(self_child, other_child)|
                other_child == self_child
            )
    }
}