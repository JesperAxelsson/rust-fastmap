# rust-fastmap
Fast hashmap implementation for Rust.

Might be missing some functionality but you can remove, add, get and clear for now.

Be aware that no effort is made against DoS attacks.

Performace compared to the standard hashmap:

````
test tests::u64_get_built_in    ... bench:      23,047 ns/iter (+/- 1,862)
test tests::u64_get_fastmap     ... bench:       9,599 ns/iter (+/- 379)
test tests::u64_get_ordermap    ... bench:      22,449 ns/iter (+/- 2,228)
test tests::u64_insert_built_in ... bench:      28,119 ns/iter (+/- 1,818)
test tests::u64_insert_fastmap  ... bench:      20,824 ns/iter (+/- 1,016)
test tests::u64_insert_ordermap ... bench:      26,269 ns/iter (+/- 3,025)
````

# How to use
Simple example.

````rust
extern crate fastmap;

use fastmap::FastMap;

let mut map = FastMap::new();

for i in 0..20_000 {
    map.insert(i, format!("item: {:?}", i));
}
````

# How can it be so much faster?
I use mumurhash which is faster then the built in hash, but is more sensitive to DoS attacks. The internal cache is also kept a power of 2 which inrease memory usage but can increase lookup speed. Worst case performance is worse then the built in hashmap (O(n)), but should be closer to O(1) in practice.

