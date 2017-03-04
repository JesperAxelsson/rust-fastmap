# rust-fastmap
Fast hashmap implementation for Rust.

Might be missing some functionality but you can remove, add, get and clear for now.

Be aware that no effort is made against DoS attacks.

Performace compared to the standard hashmap:

````
test tests::string_get_built_in    ... bench:  10,146,486 ns/iter (+/- 350,242)
test tests::string_get_fastmap     ... bench:   7,005,050 ns/iter (+/- 126,438)
test tests::string_get_ordermap    ... bench:   7,568,866 ns/iter (+/- 220,864)
test tests::string_insert_built_in ... bench:  11,378,783 ns/iter (+/- 322,641)
test tests::string_insert_fastmap  ... bench:   9,659,826 ns/iter (+/- 183,317)
test tests::string_insert_ordermap ... bench:   7,926,658 ns/iter (+/- 327,210)
test tests::u64_get_built_in       ... bench:      21,275 ns/iter (+/- 379)
test tests::u64_get_fastmap        ... bench:      11,477 ns/iter (+/- 240)
test tests::u64_get_ordermap       ... bench:      22,823 ns/iter (+/- 270)
test tests::u64_insert_built_in    ... bench:      30,093 ns/iter (+/- 381)
test tests::u64_insert_fastmap     ... bench:      18,079 ns/iter (+/- 444)
test tests::u64_insert_ordermap    ... bench:      24,937 ns/iter (+/- 625)
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

# Pros and Cons
Pros:
* Faster then the built in hashmap
Cons:
* Vunerable to DoS attacks
* Probably use more memory
* Worse worst case performace
