use std::ops;
use crate::data::{NoData};
use crate::trie_node::{TrieNode};

pub type TrieDatalessNode = TrieNode<(), NoData>;

/// Methods only on nodes that don't have data.
impl TrieDatalessNode {

}

impl ops::AddAssign for TrieDatalessNode {
    fn add_assign(&mut self, rhs: Self) {
        for (char, rhs_next_node) in rhs.children {
            match self.children.remove(&*char) {
                Some(mut self_next_node) => {
                    // Edge case: associate self node if the other node is also associated
                    // Example: when adding 'word' to 'word1', 'd' on 'word' needs to be associated
                    if rhs_next_node.is_associated() {
                        self_next_node.associate(false);
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

impl PartialEq for TrieDatalessNode {
    fn eq(&self, other: &Self) -> bool {
        if self.children.keys().ne(other.children.keys()) {
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