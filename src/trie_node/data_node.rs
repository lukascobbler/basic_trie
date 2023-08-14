use crate::data::YesData;
use crate::trie_node::{TrieNode, NodeAssociation};

pub type TrieDataNode<D> = TrieNode<D, YesData>;

/// Methods only on nodes that have data.
impl<D> TrieDataNode<D> {
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

    /// Function pushes data to the association vector.
    pub(crate) fn push_data(&mut self, data: D) {
        if let NodeAssociation::Data(vec) = &mut self.word_end_association {
            vec.push(data);
        }
    }
}
