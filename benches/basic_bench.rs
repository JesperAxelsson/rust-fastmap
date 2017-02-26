#![feature(test)]

extern crate fastmap;
extern crate rand;
extern crate test;

extern crate ordermap;

use fastmap::FastMap;
use std::collections::HashMap;
use ordermap::OrderMap;


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;


    const VEC_COUNT: usize = 1000;

    // ********** Built in **********

    #[bench]
    fn u64_insert_built_in(b: &mut Bencher) {
        let data = get_random_range(VEC_COUNT);
        let mut map = HashMap::with_capacity(data.len());

        b.iter(|| {
            map.clear();

            for s in data.iter() {
               test::black_box(map.insert(s, s));
            }

        });
    }

    #[bench]
    fn u64_get_built_in(b: &mut Bencher) {
        let data = get_random_range(VEC_COUNT);
        let mut map: HashMap<&u64, &u64>  = HashMap::with_capacity(data.len());

        for s in data.iter() {
            test::black_box(map.insert(s, s)
            );
        }

        b.iter(|| {
            for s in data.iter() {
                test::black_box({
                    map.contains_key(s);

                });
            }
        });
    }

    // ********** Ordermap **********

    #[bench]
    fn u64_insert_ordermap(b: &mut Bencher) {
        let data = get_random_range(VEC_COUNT);
        let mut map = OrderMap::with_capacity(data.len());

        b.iter(|| {
            map.clear();

            for s in data.iter() {
               test::black_box(map.insert(s, s));
            }

        });
    }

    #[bench]
    fn u64_get_ordermap(b: &mut Bencher) {
        let data = get_random_range(VEC_COUNT);
        let mut map: OrderMap<&u64, &u64>  = OrderMap::with_capacity(data.len());

        for s in data.iter() {
            test::black_box(map.insert(s, s)
            );
        }

        b.iter(|| {
            for s in data.iter() {
                test::black_box({
                    map.contains_key(s);
                });
            }
        });
    }


    // ********** Intmap **********

    #[bench]
    fn u64_insert_fastmap(b: &mut Bencher) {
        let data = get_random_range(VEC_COUNT);

        let mut map = FastMap::with_capacity(data.len());

        b.iter(|| {
            map.clear();
            for s in data.iter() {
                test::black_box({map.insert(*s, s)});
            }
        });
    }


    #[bench]
    fn u64_get_fastmap(b: &mut Bencher) {
        let data = get_random_range(VEC_COUNT);

        let mut map = FastMap::with_capacity(data.len());
        for s in data.iter() {
            map.insert(*s, s);
        }

        b.iter(|| {
            for s in data.iter() {
                test::black_box(map.contains_key(*s));
            }
        });
    }

    // ********** Misc **********


    fn get_random_range(count: usize) -> Vec<u64> {
        use rand::{Rng, SeedableRng, StdRng};

        let mut vec = Vec::new();

        let seed: &[_] = &[4, 2, 4, 2];
        let mut rng: StdRng = SeedableRng::from_seed(seed);

        for _ in 0..count {
            vec.push(rng.gen::<u64>());
        }

        vec.sort();
        vec.dedup();

        vec
    }

}
