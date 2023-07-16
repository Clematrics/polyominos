use std::{
    collections::{BTreeMap, HashSet},
    iter::repeat,
};

use crate::{board::Board, polyomino::Polyomino};

#[repr(transparent)]
pub struct Database(pub Vec<BTreeMap<(u8, u8), HashSet<Board>>>);

fn get_mut_or<K, V, F>(map: &mut BTreeMap<K, V>, key: K, f: F) -> &mut V
where
    K: Ord + Copy,
    F: Fn() -> V,
{
    if !map.contains_key(&key) {
        map.insert(key, f());
    }

    map.get_mut(&key).unwrap()
}

impl Database {
    /// Returns true if the Polyomino was not seen before
    /// Returns false otherwise
    pub fn add_or_reject(&mut self, p: &Polyomino) -> bool {
        if self.0.len() < p.square_count as usize {
            self.0
                .extend(repeat(BTreeMap::new()).take(p.square_count as usize - self.0.len()))
        }

        let set = get_mut_or(
            &mut self.0[p.square_count as usize - 1],
            p.dimension,
            || HashSet::new(),
        );

        if set.contains(&p.repr) {
            false
        } else {
            set.insert(p.repr);
            true
        }
    }
}
