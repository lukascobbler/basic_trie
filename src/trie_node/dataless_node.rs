use crate::data::{NoData};
use crate::trie_node::{TrieNode};

pub type TrieDatalessNode<'a> = TrieNode<'a, (), NoData>;

/// Methods only on nodes that don't have data.
impl<'a> TrieDatalessNode<'a> {

}
