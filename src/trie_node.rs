#[cfg(feature = "data")]
mod data_node;

mod regular_node;

#[cfg(feature = "data")]
pub(crate) use data_node::TrieDataNode;

pub(crate) use regular_node::TrieDatalessNode;
