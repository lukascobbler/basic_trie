use crate::data::{NoData};
use crate::trie_node::{TrieNode};

pub type TrieDatalessNode = TrieNode<(), NoData>;

/// Methods only on nodes that don't have data.
impl TrieDatalessNode {

}