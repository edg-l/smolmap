#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![allow(clippy::cast_possible_truncation)]

use std::{
    collections::hash_map::RandomState,
    hash::{BuildHasher, Hash, Hasher},
    mem::MaybeUninit,
};

#[derive(Debug)]
pub struct SmolMap<K, V, const N: usize, S = RandomState> {
    storage: [MaybeUninit<(K, V)>; N],
    tags: [bool; N],
    len: usize,
    state: S,
}

impl<K, V, const N: usize, S> SmolMap<K, V, N, S> {
    pub const fn new(hasher: S) -> Self {
        Self {
            storage: unsafe { MaybeUninit::uninit().assume_init() },
            tags: [false; N],
            len: 0,
            state: hasher,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<K, V, const N: usize, S> SmolMap<K, V, N, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    /// # Panics
    /// If length == N and the key doesn't exist.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let mut hasher = self.state.build_hasher();
        key.hash(&mut hasher);
        let start_idx = hasher.finish() as usize;
        let mut iter_idx = 0;

        while iter_idx < N {
            let idx_mod: usize = (start_idx + iter_idx) % self.storage.len();

            if self.tags[idx_mod] {
                if unsafe { self.storage[idx_mod].assume_init_ref() }
                    .0
                    .eq(&key)
                {
                    let new_val = MaybeUninit::new((key, value));
                    let old_val = std::mem::replace(&mut self.storage[idx_mod], new_val);
                    return Some(unsafe { old_val.assume_init().1 });
                }
            } else {
                let new_val = MaybeUninit::new((key, value));
                self.tags[idx_mod] = true;
                self.storage[idx_mod] = new_val;
                self.len += 1;
                return None;
            }

            iter_idx += 1;
        }

        panic!("no slot could be found")
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = self.state.build_hasher();
        key.hash(&mut hasher);

        let start_idx = hasher.finish() as usize;
        let mut iter_idx = 0;

        while iter_idx < N {
            let idx_mod: usize = (start_idx + iter_idx) % self.storage.len();

            if self.tags[idx_mod] {
                let value = unsafe { self.storage[idx_mod].assume_init_ref() };
                if value.0.eq(key) {
                    return Some(&value.1);
                }
            } else {
                return None;
            }

            iter_idx += 1;
        }

        None
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let mut hasher = self.state.build_hasher();
        key.hash(&mut hasher);

        let start_idx = hasher.finish() as usize;
        let mut iter_idx = 0;
        let len = self.storage.len();

        // to work around rust mutability in loops.
        let mut found_idx = None;

        while iter_idx < N {
            let idx_mod: usize = (start_idx + iter_idx) % len;

            if self.tags[idx_mod] {
                let value = unsafe { self.storage[idx_mod].assume_init_ref() };
                if value.0.eq(key) {
                    found_idx = Some(idx_mod);
                    break;
                }
            } else {
                return None;
            }

            iter_idx += 1;
        }

        found_idx.map(|x| {
            let value = unsafe { self.storage[x].assume_init_mut() };
            &mut value.1
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let map: SmolMap<u32, u32, 8> = SmolMap::new(RandomState::new());
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn insert() {
        let mut map: SmolMap<u32, u32, 4> = SmolMap::new(RandomState::new());
        map.insert(1, 2);
        map.insert(2, 2);
        map.insert(3, 2);
        map.insert(4, 2);
        map.insert(1, 3);
        map.insert(2, 4);
        map.insert(3, 5);
        map.insert(4, 6);
    }

    #[test]
    #[should_panic]
    fn insert_panic() {
        let mut map: SmolMap<u32, u32, 4> = SmolMap::new(RandomState::new());
        map.insert(1, 2);
        map.insert(2, 2);
        map.insert(3, 2);
        map.insert(4, 2);
        map.insert(5, 2);
    }

    #[test]
    fn get() {
        let mut map: SmolMap<u32, u32, 4> = SmolMap::new(RandomState::new());
        map.insert(1, 3);
        map.insert(2, 2);
        map.insert(3, 1);
        assert_eq!(map.get(&1), Some(&3));
        assert_eq!(map.get(&2), Some(&2));
        assert_eq!(map.get(&3), Some(&1));
        assert_eq!(map.get(&6), None);
    }

    #[test]
    fn get_mut() {
        let mut map: SmolMap<u32, u32, 4> = SmolMap::new(RandomState::new());
        map.insert(1, 3);
        map.insert(2, 2);
        map.insert(3, 1);
        assert_eq!(map.get(&1), Some(&3));
        let val = map.get_mut(&1);

        if let Some(val) = val {
            *val = 4;
        }
        assert_eq!(map.get(&1), Some(&4));
    }
}
