use std::{
    collections::{BTreeMap, HashMap, VecDeque},
    hash::Hash,
    mem::replace,
};

use crate::{board::Board, polyomino::Polyomino};

/// The database holds three things:
/// - the number of polyominoes with some square amount, if all have been processed
/// - the queue of unprocessed polyominoes of the last square amount
/// - the cache of polyominoes for the next amount
/// - stats by square amount
pub struct Database {
    counts: Vec<u128>,
    queue: VecDeque<Polyomino>,
    cache: BTreeMap<(u8, u8), HashMap<Board, Board>>,
    stats: Vec<u128>,
}

fn treemap_get_mut_or<K, V, F>(map: &mut BTreeMap<K, V>, key: K, f: F) -> &mut V
where
    K: Ord + Copy,
    F: Fn() -> V,
{
    if !map.contains_key(&key) {
        map.insert(key, f());
    }

    map.get_mut(&key).unwrap()
}

/// Returns None if the value is added, returns the value already stored otherwise
fn hashmap_get_mut_or<K, V>(map: &mut HashMap<K, V>, key: K, v: V) -> Option<&mut V>
where
    K: Ord + Copy + Hash,
{
    if !map.contains_key(&key) {
        map.insert(key, v);
        None
    } else {
        map.get_mut(&key)
    }
}

impl Database {
    pub fn new() -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(Polyomino::trivial());

        Self {
            counts: vec![1],
            queue,
            cache: BTreeMap::new(),
            stats: vec![1, 0],
        }
    }

    pub fn pop(&mut self) -> Option<Polyomino> {
        self.queue.pop_front()
    }

    /// Register the polyomino in the cache
    pub fn register(&mut self, p: Polyomino) {
        let mut map = treemap_get_mut_or(&mut self.cache, p.dimension, || HashMap::new());

        *self.stats.last_mut().unwrap() += 1;

        match hashmap_get_mut_or(&mut map, p.repr, p.mask) {
            Some(mask) => *mask |= p.mask,
            None => (),
        }
    }

    /// Flush the cache into the queue, ready to start processing the new polyominoes with a new square,
    /// also adding the number of polyominoes to the database.
    /// Panics if the queue is not empty.
    /// WARNING: if another process picks the last element of the queue and flush is called before
    /// this last element was processed and registered to the database, the count could be wrong
    pub fn flush(&mut self) {
        if !self.queue.is_empty() {
            panic!("The queue database is not empty!")
        }

        let cache = replace(&mut self.cache, BTreeMap::new());
        for (dim, hashmap) in cache.into_iter() {
            for (repr, mask) in hashmap.into_iter() {
                let p = Polyomino::from(dim, repr, mask);

                // println!("Flushing:");
                // println!("{p:?}");

                self.queue.push_back(p);
            }
        }

        self.counts.push(self.queue.len() as u128);
        self.stats.push(0);
    }

    /// Returns Some number of polyominoes with [n] squares,
    /// or None if the count is unkown
    pub fn count(&self, n: usize) -> Option<&u128> {
        if n == 0 {
            panic!("There are no polyominoes with zero square");
        }

        self.counts.get(n - 1)
    }

    // Return an iterator on all counts
    pub fn counts(&self) -> std::slice::Iter<'_, u128> {
        self.counts.iter()
    }

    pub fn stats(&self) -> std::slice::Iter<'_, u128> {
        self.stats.iter()
    }
}
