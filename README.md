# rust-fastmap
Fast hashmap implementation for Rust.

Might be missing some functionality but you can remove, add, get and clear for now.

Be aware that no effort is made against DoS attacks.

Performace compared to the standard hashmap:

````
test tests::string_get_built_in    ... bench:  11,616,188 ns/iter (+/- 1,819,299)
test tests::string_get_fastmap     ... bench:  12,451,457 ns/iter (+/- 1,569,140)
test tests::string_get_ordermap    ... bench:   8,925,038 ns/iter (+/- 542,242)
test tests::string_insert_built_in ... bench:  12,997,246 ns/iter (+/- 795,416)
test tests::string_insert_fastmap  ... bench:  14,976,130 ns/iter (+/- 920,800)
test tests::string_insert_ordermap ... bench:   9,371,283 ns/iter (+/- 724,981)
test tests::u64_get_built_in       ... bench:      21,706 ns/iter (+/- 1,210)
test tests::u64_get_fastmap        ... bench:       9,905 ns/iter (+/- 496)
test tests::u64_get_ordermap       ... bench:      23,074 ns/iter (+/- 1,199)
test tests::u64_insert_built_in    ... bench:      30,466 ns/iter (+/- 2,532)
test tests::u64_insert_fastmap     ... bench:      20,977 ns/iter (+/- 991)
test tests::u64_insert_ordermap    ... bench:      25,572 ns/iter (+/- 1,607)
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

