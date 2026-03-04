use fxhash::FxHashMap;
use std::fmt;
use std::fmt::Debug;
use thin_vec::ThinVec;

/// A multi-typed container for storing child nodes.
/// - An empty discriminant (no allocations) is used when there are no children
/// - A vector (ThinVec) variant is used when there are less than 32 children
/// - A fast hash map (FxHashMap) is used when there are more than 32 children
///
/// This preserves much more space than always keeping the allocated hash map.
#[derive(Default)]
pub enum ChildStorage<NodeType> {
    #[default]
    Empty,
    Small(ThinVec<(char, NodeType)>),
    Large(Box<FxHashMap<char, NodeType>>),
}

impl<NodeType> ChildStorage<NodeType> {
    /// Inserts a new defaulted node into the children container.
    pub fn insert_new(&mut self, key: char)
    where
        NodeType: Default,
    {
        self.insert_direct(key, NodeType::default());
    }

    /// Returns the number of children
    pub fn len(&self) -> usize {
        match self {
            ChildStorage::Empty => 0,
            ChildStorage::Small(v) => v.len(),
            ChildStorage::Large(m) => m.len(),
        }
    }

    /// Returns a reference to a child by its character key
    pub fn get(&self, key: char) -> Option<&NodeType> {
        match self {
            ChildStorage::Empty => None,
            ChildStorage::Small(v) => v.iter().find(|(c, _)| *c == key).map(|(_, node)| node),
            ChildStorage::Large(m) => m.get(&key),
        }
    }

    /// Returns a mutable reference to a child by its character key
    pub fn get_mut(&mut self, key: char) -> Option<&mut NodeType> {
        match self {
            ChildStorage::Empty => None,
            ChildStorage::Small(v) => v.iter_mut().find(|(c, _)| *c == key).map(|(_, node)| node),
            ChildStorage::Large(m) => m.get_mut(&key),
        }
    }

    /// Unified iterator over (&char, &TrieDatalessNode)
    pub fn iter(&self) -> Box<dyn Iterator<Item = (&char, &NodeType)> + '_> {
        match self {
            ChildStorage::Empty => Box::new(std::iter::empty()),
            ChildStorage::Small(v) => Box::new(v.iter().map(|(c, n)| (c, n))),
            ChildStorage::Large(m) => Box::new(m.iter()),
        }
    }

    /// Unified iterator over &TrieDatalessNode
    pub fn values(&self) -> Box<dyn Iterator<Item = &NodeType> + '_> {
        match self {
            ChildStorage::Empty => Box::new(std::iter::empty()),
            ChildStorage::Small(v) => Box::new(v.iter().map(|(_, n)| n)),
            ChildStorage::Large(m) => Box::new(m.values()),
        }
    }

    /// Unified iterator over &mut TrieDatalessNode
    pub fn values_mut(&mut self) -> Box<dyn Iterator<Item = &mut NodeType> + '_> {
        match self {
            ChildStorage::Empty => Box::new(std::iter::empty()),
            ChildStorage::Small(v) => Box::new(v.iter_mut().map(|(_, n)| n)),
            ChildStorage::Large(m) => Box::new(m.values_mut()),
        }
    }

    /// Whether the node has children.
    pub fn is_empty(&self) -> bool {
        match self {
            ChildStorage::Empty => true,
            ChildStorage::Small(_) | ChildStorage::Large(_) => false,
        }
    }

    /// Removes a child based on the passed character.
    pub fn remove(&mut self, key: char) -> Option<NodeType> {
        match self {
            ChildStorage::Empty => None,
            ChildStorage::Small(vec) => {
                let pos = vec.iter().position(|(k, _)| *k == key)?;
                let (_, node) = vec.remove(pos);

                if vec.is_empty() {
                    *self = ChildStorage::Empty;
                }
                Some(node)
            }
            ChildStorage::Large(map) => map.remove(&key),
        }
    }

    /// Whether two nodes have the same keys.
    pub fn has_same_keys(&self, other: &ChildStorage<NodeType>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        match (self, other) {
            (ChildStorage::Empty, ChildStorage::Empty) => true,
            (ChildStorage::Small(v1), ChildStorage::Small(v2)) => {
                v1.iter().all(|(k1, _)| v2.iter().any(|(k2, _)| k1 == k2))
            }
            (ChildStorage::Large(m1), ChildStorage::Large(m2)) => {
                m1.keys().all(|k| m2.contains_key(k))
            }
            (ChildStorage::Small(v), ChildStorage::Large(m))
            | (ChildStorage::Large(m), ChildStorage::Small(v)) => {
                v.iter().all(|(k, _)| m.contains_key(k))
            }
            _ => false,
        }
    }

    /// Inserts the passed node into the collection. Switches up the type of
    /// collection in case the limit gets passed.
    pub fn insert_direct(&mut self, key: char, node: NodeType) {
        match self {
            ChildStorage::Empty => {
                let mut v = ThinVec::with_capacity(1);
                v.push((key, node));
                *self = ChildStorage::Small(v);
            }
            ChildStorage::Small(vec) => {
                if vec.len() < 32 {
                    vec.push((key, node));
                } else {
                    let mut map = FxHashMap::default();
                    for (k, v) in vec.drain(..) {
                        map.insert(k, v);
                    }
                    map.insert(key, node);
                    *self = ChildStorage::Large(Box::new(map));
                }
            }
            ChildStorage::Large(map) => {
                map.insert(key, node);
            }
        }
    }
}

impl<NodeType: PartialEq> PartialEq for ChildStorage<NodeType> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Small(v1), Self::Small(v2)) => v1.iter().all(|(k1, n1)| {
                v2.iter()
                    .find(|(k2, _)| k1 == k2)
                    .map(|(_, n2)| n1 == n2)
                    .unwrap_or(false)
            }),
            (Self::Large(m1), Self::Large(m2)) => m1 == m2,
            _ => self
                .iter()
                .all(|(char_key, node)| other.get(*char_key) == Some(node)),
        }
    }
}

impl<NodeType: Debug> Debug for ChildStorage<NodeType> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "{{}}"),
            Self::Small(vec) => {
                let mut map = f.debug_map();
                for (c, node) in vec {
                    map.entry(c, node);
                }
                map.finish()
            }
            Self::Large(map) => f.debug_map().entries(map.iter()).finish(),
        }
    }
}

/// Unified iterator over all the variants.
pub enum ChildIntoIter<NodeType> {
    Empty,
    Small(thin_vec::IntoIter<(char, NodeType)>),
    Large(std::collections::hash_map::IntoIter<char, NodeType>),
}

impl<NodeType> Iterator for ChildIntoIter<NodeType> {
    type Item = (char, NodeType);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ChildIntoIter::Empty => None,
            ChildIntoIter::Small(iter) => iter.next(),
            ChildIntoIter::Large(iter) => iter.next(),
        }
    }
}

impl<NodeType> IntoIterator for ChildStorage<NodeType> {
    type Item = (char, NodeType);
    type IntoIter = ChildIntoIter<NodeType>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            ChildStorage::Empty => ChildIntoIter::Empty,
            ChildStorage::Small(vec) => ChildIntoIter::Small(vec.into_iter()),
            ChildStorage::Large(map) => ChildIntoIter::Large(map.into_iter()),
        }
    }
}
