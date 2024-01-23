use serde::{de::DeserializeOwned, Serialize};
use sled::{self, IVec, Tree};
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

pub trait OrderedStore<O, V>: Send + Sync {
    fn len(&self) -> usize;
    fn first_key_value(&self) -> Option<(O, V)>;
    fn last_key_value(&self) -> Option<(O, V)>;
    fn get(&self, key: &O) -> Option<V>;
    fn insert(&mut self, key: O, value: V) -> Option<V>;
    fn remove(&mut self, key: &O) -> Option<V>;
}
impl<V: Serialize + DeserializeOwned> OrderedStore<IVec, V> for Tree {
    fn len(&self) -> usize {
        Tree::len(self)
    }
    fn first_key_value(&self) -> Option<(IVec, V)> {
        match self.first() {
            Ok(Some((k, v))) => serde_json::from_slice(v.as_ref()).ok().map(|v| (k, v)),
            _ => None,
        }
    }
    fn last_key_value(&self) -> Option<(IVec, V)> {
        match self.last() {
            Ok(Some((k, v))) => serde_json::from_slice(v.as_ref()).ok().map(|v| (k, v)),
            _ => None,
        }
    }
    fn get(&self, key: &IVec) -> Option<V> {
        match self.get(key) {
            Ok(Some(v)) => serde_json::from_slice(v.as_ref()).ok(),
            _ => None,
        }
    }
    fn insert(&mut self, key: IVec, value: V) -> Option<V> {
        match Tree::insert(self, key, serde_json::to_vec(&value).unwrap()) {
            Ok(Some(v)) => serde_json::from_slice(v.as_ref()).ok(),
            _ => None,
        }
    }
    fn remove(&mut self, key: &IVec) -> Option<V> {
        match Tree::remove(self, key).map(|v| v) {
            Ok(Some(v)) => serde_json::from_slice(&v).ok(),
            _ => None,
        }
    }
}
impl<O: Ord + Copy + Send + Sync, V: Clone + Send + Sync> OrderedStore<O, V> for BTreeMap<O, V> {
    fn len(&self) -> usize {
        BTreeMap::len(self)
    }
    fn first_key_value(&self) -> Option<(O, V)> {
        BTreeMap::first_key_value(self).map(|(o, v)| (*o, v.clone()))
    }
    fn last_key_value(&self) -> Option<(O, V)> {
        BTreeMap::last_key_value(self).map(|(o, v)| (*o, v.clone()))
    }
    fn get(&self, key: &O) -> Option<V> {
        BTreeMap::get(self, key).map(|v| v.clone())
    }
    fn insert(&mut self, key: O, value: V) -> Option<V> {
        BTreeMap::insert(self, key, value)
    }
    fn remove(&mut self, key: &O) -> Option<V> {
        BTreeMap::remove(self, key)
    }
}
/// A hashmap that also maintains a BTreeMap of keys ordered by a given value
/// This is useful for structures that need fast O(1) lookups, but also need to evict the oldest or least recently used entries
pub struct OrderedHashMap<K, O, V>(
    (
        HashMap<K, (O, V)>,
        Box<dyn OrderedStore<O, Vec<K>> + Send + Sync>,
    ),
);

impl<K: Clone + Send + Sync, O: Ord + Copy + Send + Sync, V> OrderedHashMap<K, O, V> {
    pub fn new(order: impl OrderedStore<O, Vec<K>> + 'static) -> Self {
        Self((HashMap::new(), Box::new(order)))
    }
}

impl<K: Hash + Eq + Clone, O: Ord + Copy, V: Clone> OrderedHashMap<K, O, V> {
    pub fn len(&self) -> usize {
        let (lookup, _) = &self.0;
        lookup.len()
    }
    pub fn get(&self, key: &K) -> Option<&(O, V)> {
        let (lookup, _) = &self.0;
        lookup.get(key)
    }
    fn get_key_value(
        &self,
        selector: Box<
            dyn Fn(&Box<dyn OrderedStore<O, Vec<K>> + Send + Sync>) -> Option<(O, Vec<K>)>,
        >,
    ) -> Option<(K, O, V)> {
        let (lookup, ordered_lookup) = &self.0;
        selector(ordered_lookup).and_then(|(_, keys)| {
            keys.first().and_then(|key| {
                lookup
                    .get(key)
                    .and_then(|(o, v)| Some((key.clone(), *o, v.clone())))
            })
        })
    }
    /// gets the entry with the lowest order value
    pub fn get_first_key_value(&self) -> Option<(K, O, V)> {
        self.get_key_value(Box::new(|ordered_lookup| ordered_lookup.first_key_value()))
    }
    /// gets the entry with the highest order value
    pub fn get_last_key_value(&self) -> Option<(K, O, V)> {
        self.get_key_value(Box::new(|ordered_lookup| ordered_lookup.last_key_value()))
    }
    /// re-orders the entry with the given new order
    pub fn re_order(&mut self, key: &K, new_order: O) {
        if let Some((_, value)) = self.remove(key) {
            self.insert(key.clone(), value, new_order);
        }
    }
    /// inserts a new entry with the given key and value and order
    pub fn insert(&mut self, key: K, value: V, order: O) -> Option<V> {
        let (lookup, order_lookup) = &mut self.0;

        if let Some((old_order, _)) = lookup.get(&key) {
            // if entry already exists, remove it from the btree
            if let Some(mut keys) = order_lookup.remove(old_order) {
                keys.retain(|k| *k != key);
                // insert modified keys back into btree
                if !keys.is_empty() {
                    order_lookup.insert(*old_order, keys);
                }
            }
        }
        let keys = match order_lookup.remove(&order) {
            Some(mut ks) => {
                ks.push(key.clone());
                ks
            }
            None => vec![key.clone()],
        };
        order_lookup.insert(order, keys);
        lookup
            .insert(key, (order, value))
            .and_then(|(_, v)| Some(v))
    }
    /// removes the entry with the given key
    pub fn remove(&mut self, key: &K) -> Option<(O, V)> {
        let (lookup, order_lookup) = &mut self.0;
        lookup.remove(key).and_then(|(order, v)| {
            match order_lookup.remove(&order) {
                Some(mut keys) => {
                    keys.retain(|k| k != key);
                    // insert remaining keys back in
                    if !keys.is_empty() {
                        order_lookup.insert(order, keys);
                    }
                }
                None => {}
            }
            Some((order, v))
        })
    }
    /// removes the entry with the lowest order value
    pub fn remove_first(&mut self) -> Option<(K, O, V)> {
        let first_key = self.get_first_key_value().map(|(k, _, _)| k.clone());
        if let Some(first_key) = first_key {
            self.remove(&first_key)
                .map(|(order, v)| (first_key, order, v))
        } else {
            None
        }
    }
}
