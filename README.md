# smolmap

A hashmap on the stack in Rust, for fun, not really a serious project for now.

```rust
let mut map: SmolMap<u32, u32, 4> = SmolMap::new(RandomState::new());
map.insert(1, 3);
map.insert(2, 2);
map.insert(3, 1);
assert_eq!(map.get(&1), Some(&3));
assert_eq!(map.get(&2), Some(&2));
assert_eq!(map.get(&3), Some(&1));
assert_eq!(map.get(&6), None);
 ```
