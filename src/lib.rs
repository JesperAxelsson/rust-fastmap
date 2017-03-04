use std::hash::Hash;
use std::hash::BuildHasher;
use std::hash::Hasher;

// struct Kv<V> {
//     key: u64,
//     value: V
// }

pub struct FastMap<K: Eq + Hash, V, S = Murmur2_64a> {
    cache:  Vec<Bucket<K, V>>,
    // indices:  Vec<(K, V)>,
    size: u32,
    mod_mask: u64,
    count: usize,
    hasher: S,
}

enum Bucket<K: Eq + Hash, V> {
    Value(u64, K, V),
    Deleted,
    Empty,
}

impl<K, V> FastMap<K, V>
    where K: Eq + Hash {
    /// Creates a new FastMap.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::new();
    /// ```
    pub fn new() -> Self {
        FastMap::with_capacity(4)
    }


    /// Creates a new FastMap with a at least capacity, all sizes is a power of 2.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::with_capacity(20);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        let m = Murmur2_64a::new();

        let mut map = FastMap {
            cache: Vec::new(),
            size: 0,
            count: 0,
            mod_mask: 0,
            hasher: m,
        };

        map.increase_cache();

        while map.lim() < capacity {
            map.increase_cache();
        }

        // flame::clear();
        map
    }


    /// Insert key/value into the FastMap.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map = FastMap::new();
    /// map.insert(21, "Eat my shorts");
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> bool {

        self.insert_internal(key, value);

        if (self.count & 4) == 4 {
            self.ensure_load_rate();
        }

        true
    }

    fn insert_internal(&mut self, key: K, value: V) -> bool {
        // let _guard = flame::start_guard("insert");
        let (hash, mut ix) = self.calc_index(&key);
        // let ix_start = ix;

        {
            loop {
                match self.cache[ix] {
                    Bucket::Value(h, ref k, _) => {
                        if h == hash && *k == key {
                            return false;
                        } else {
                            ix += 1;
                        }
                    }
                    Bucket::Deleted => ix += 1,
                    Bucket::Empty => break, // Got free spot!
                }
            }

            self.count += 1;
            self.cache[ix] = Bucket::Value(hash, key, value);
        }

        true
    }

    /// Get value from the FastMap.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::new();
    /// map.insert(21, 42);
    /// let val = map.get(21);
    /// assert!(val.is_some());
    /// assert_eq!(*val.unwrap(), 42);
    /// assert!(map.contains_key(21));
    /// ```
    pub fn get(&self, key: K) -> Option<&V> {
        let (hash, mut ix) = self.calc_index(&key);

        loop {
            match self.cache[ix] {
                Bucket::Value(h, ref k, ref v) => {
                    if h == hash && *k == key {
                        return Some(&v);
                    } else {
                        ix += 1;
                    }
                }
                Bucket::Deleted => ix += 1,
                Bucket::Empty => return None,
            }
        }

    }

    /// Get mutable value from the FastMap.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::new();
    /// map.insert(21, 42);
    ///
    /// assert_eq!(*map.get(21).unwrap(), 42);
    /// assert!(map.contains_key(21));
    ///
    /// {
    ///     let mut val = map.get_mut(21).unwrap();
    ///     *val+=1;
    /// }
    ///     assert_eq!(*map.get(21).unwrap(), 43);
    /// ```
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let (hash, mut ix) = self.calc_index(&key);

        loop {
            match self.cache[ix] {
                Bucket::Value(h, ref k, _) => {
                    if h == hash && *k == key {
                        break;
                    } else {
                        ix += 1;
                    }
                }
                Bucket::Deleted => ix += 1,
                Bucket::Empty => return None,
            }
        }

        if let Bucket::Value(_, _, ref mut v) = self.cache[ix] {
            return Some(&mut *v);
        } else {
            panic!("get_mut item we want to give away were not there anymore!");
        }
    }

    /// Remove value from the FastMap.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::new();
    /// map.insert(21, 42);
    /// let val = map.remove(21);
    /// assert!(val.is_some());
    /// assert_eq!(val.unwrap(), 42);
    /// assert!(!map.contains_key(21));
    /// ```
    pub fn remove(&mut self, key: K) -> Option<V> {
        let (hash, mut ix) = self.calc_index(&key);


        loop {
            match self.cache[ix] {
                Bucket::Value(h, ref k, _) => {
                    if h == hash && *k == key {
                        break;
                    } else {
                        ix += 1;
                    }
                }
                Bucket::Deleted => ix += 1,
                Bucket::Empty => return None,
            }
        }

        self.count -= 1;

        // Push elemtent to switch with
        self.cache.push(Bucket::Deleted);

        let val = self.cache.swap_remove(ix);
        self.cache[ix] = Bucket::Deleted;

        match val {
            Bucket::Value(_, _, v) => return Some(v),
            _ => panic!("Item that we wanted to remove is gone!"),
        }
    }

    /// Returns true if key is in map.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::new();
    /// map.insert(21, 42);
    /// assert!(map.contains_key(21));
    /// ```
    pub fn contains_key(&self, key: K) -> bool {
        match self.get(key) {
            Some(_) => true,
            None    => false
        }
    }


    /// Removes all elements from map.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::new();
    /// map.insert(21, 42);
    /// map.clear();
    /// assert_eq!(map.len(), 0);
    /// ```
    pub fn clear(&mut self) {
        for i in 0..self.cache.len() {
            self.cache[i] = Bucket::Empty;
        }

        self.count = 0;
    }

    /// Returns true if map is empty
    ///
    /// # Examples
    ///
    /// ```
    /// use fastmap::FastMap;
    ///
    /// let mut map: FastMap<u64, u64> = FastMap::new();
    /// map.insert(21, 42);
    /// assert!(!map.is_empty());
    /// map.remove(21);
    /// assert!(map.is_empty());
    /// ```
    pub fn is_empty(&mut self) -> bool {
        self.count == 0
    }


    //**** Iterators *****

    pub fn iter<'a>(&self) -> Iter<K, V> {
        Iter {
            outer: self.cache.iter()
        }
    }

    pub fn keys(&mut self) -> Keys<K, V> {
        Keys { inner: self.iter() }
    }

    pub fn values(&mut self) -> Values<K, V> {
        Values { inner: self.iter() }
    }

    pub fn iter_mut<'a>(&mut self) -> IterMut<K, V> {
        // IterMut::new(&mut self.cache)
        IterMut {
            outer: self.cache.iter_mut()
        }
    }


    //**** Internal hash stuff *****

    // #[inline]
    // fn hash_u64(seed: u64) -> u64 {
    //     let a = 11400714819323198549u64;
    //     let val = a.wrapping_mul(seed);
    //     val
    // }

    #[inline]
    fn calc_index(&self, key: &K) -> (u64, usize) {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        // Faster modulus
        let ix = self.ix(hash);
        (hash, ix)
    }

    #[inline]
    fn ix(&self, hash: u64) -> usize {
        (hash & self.mod_mask) as usize
    }


    #[inline]
    fn lim(&self) -> usize {
        2u64.pow(self.size) as usize
    }


    fn increase_cache(&mut self) {
        // let _guard = flame::start_guard("increase_cache");
        self.size += 1;
        let new_lim = self.lim();
        self.mod_mask = (new_lim as u64) - 1;

        self.rebuild_cache();
    }


    fn rebuild_cache(&mut self) {
        let old_count = self.count;
        self.count = 0;

        let mut vec: Vec<Bucket<K, V>> = Vec::new();

        vec.append(&mut self.cache);

        for _ in 0..self.lim() + 10 {
            self.cache.push(Bucket::Empty);
        }

        while let Some(item) = vec.pop() {

            match item {
                Bucket::Value(_, k, v) => {
                    self.insert_internal(k, v);
                    // let ix = self.ix(h);
                    // self.cache[ix] = Bucket::Value(h, k, v);
                }
                _ => (),
            }
        }

         // debug_assert!(self.cache.len() == self.lim(), "cache vector the wrong length, lim: {:?} cache: {:?}", self.lim(), self.cache.len());
        debug_assert_eq!(old_count, self.count, "Different count after increase cache! Old: {}, New: {}", old_count, self.count);
    }

    fn ensure_load_rate(&mut self) {
        // let _guard2 = flame::start_guard("ensure_load_rate");
        while ((self.count*100) / self.cache.len()) > 70 {
            self.increase_cache();
        }
    }


    /// Number of elements in map.
    ///
    pub fn len(&self) -> usize {
        self.count as usize
    }


    /// Force count number of slots filled.
    ///
    pub fn load(&self) -> u64 {
        let mut count = 0;

        for item in self.cache.iter() {
            if let Bucket::Value(_, _, _) = *item {
                count += 1;
            }
        }

        count
    }



    pub fn load_rate(&self) -> f64 {
        (self.count as f64) / (self.cache.len() as f64) * 100f64
    }


    /// Total number of slots available.
    ///
    pub fn capacity(&self) -> usize {
        self.cache.len()
    }


    pub fn assert_count(&self) -> bool {
        let mut count = 0;

        for item in self.cache.iter() {
            match *item {
                Bucket::Value(_, _, _) => count += 1,
                _ => ()
            }
        }

        self.count == count
    }


    pub fn collisions(&self) -> FastMap<u64, u64> {
        let map = FastMap::new();

        // for s in self.cache.iter() {
        //     let key = s.len() as u64;
        //     if key > 0 {
        //         if !map.contains_key(key) {
        //             map.insert(key, 1);
        //         } else {
        //             let counter = map.get_mut(key).unwrap();
        //             *counter += 1;
        //         }
        //     }
        // }

        // map.sort();

        map
    }
}





use std::slice::Iter as SliceIter;
use std::slice::IterMut as SliceIterMut;

// // ***************** Iter *********************

pub struct Iter<'a, K: 'a, V: 'a>
    where K: Eq + Hash {
    outer: SliceIter<'a, Bucket<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
    where K: Eq + Hash{
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            match self.outer.next() {
                Some(r) => {
                    match *r {
                        Bucket::Value(_, ref k, ref v) => return Some((&k, &v)),
                        _ => (),
                    }
                },
                None => return None,
            }
        }
    }
}


// ***************** Iter Mut *********************

pub struct IterMut<'a, K: 'a, V: 'a>
    where K: Eq + Hash {
    outer: SliceIterMut<'a, Bucket<K, V>>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V>
    where K: Eq + Hash{
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
         loop {
            match self.outer.next() {
                Some(r) => {
                    match *r {
                        Bucket::Value(_, ref k, ref mut v) => return Some((&k, &mut *v)),
                        _ => (),
                    }
                },
                None => return None,
            }
        }
    }
}


// ***************** Values Iter *********************

pub struct Values<'a, K:'a, V: 'a>
    where K: Eq + Hash{
    inner: Iter<'a, K, V>
}


impl<'a, K, V> Iterator for Values<'a, K, V>
     where K: Eq + Hash{
    type Item = &'a V;

    #[inline] fn next(&mut self) -> Option<(&'a V)> { self.inner.next().map(|kv| kv.1) }
    #[inline] fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

// ***************** Keys Iter *********************

pub struct Keys<'a, K: 'a, V: 'a>
    where K: Eq + Hash {
    inner: Iter<'a, K, V>
}

impl<'a, K, V> Iterator for Keys<'a, K, V>
    where K: Eq + Hash{
    type Item = &'a K;

    #[inline] fn next(&mut self) -> Option<&'a K> { self.inner.next().map(|kv| kv.0) }
    #[inline] fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

// // ***************** Values Mut *********************

// pub struct ValuesMut<'a, V: 'a> {
//     inner: Iter<'a, V>
// }


// impl<'a, V> Iterator for ValuesMut<'a, V> {
//     type Item = &'a V;

//     #[inline] fn next(&mut self) -> Option<(&'a V)> { self.inner.next().map(|kv| kv.1) }
//     #[inline] fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
// }


// use std::hash::{Hasher, BuildHasher};

// ***** Murmur2_64a *****

#[allow(non_camel_case_types)]
pub struct Murmur2_64a {
    seed: u64
}

impl Murmur2_64a {
    pub fn new() -> Murmur2_64a {
        Murmur2_64a{ seed: 0 }
    }
}


impl Hasher for Murmur2_64a {
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.seed = murmur_hash64a(msg, self.seed);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.seed as u64
    }
}

impl BuildHasher for Murmur2_64a {
    type Hasher = Murmur2_64a;
    fn build_hasher(&self) -> Self::Hasher {
        let mut murm = Murmur2_64a::new();
        murm.seed = self.seed;
        murm
    }
}

pub fn murmur_hash64a(key: &[u8], seed: u64) -> u64 {
    let m : u64 = 0xc6a4a7935bd1e995;
    let r : u8 = 47;

    let len = key.len();
    let mut hash : u64 = seed ^ ((len as u64).wrapping_mul(m));

    // let end = len >> 3;
    let nblocks = (len >> 3) as isize;

    let mut k: u64;
    let blocks = key.as_ptr() as *mut u64;

    unsafe {
        for i in 0..nblocks {
            k = *blocks.offset(i);

            k = k.wrapping_mul(m);
            k ^= k >> r;
            k = k.wrapping_mul(m);

            hash ^= k;
            hash = hash.wrapping_mul(m);
        }

        let tail = blocks.offset(nblocks) as *mut u8;

        match len & 7 {
            7 => {
                hash ^= (*tail.offset(6) as u64) << 48;
                hash ^= (*tail.offset(5) as u64) << 40;
                hash ^= (*tail.offset(4) as u64) << 32;
                hash ^= (*tail.offset(3) as u64) << 24;
                hash ^= (*tail.offset(2) as u64) << 16;
                hash ^= (*tail.offset(1) as u64) << 8;
                hash ^= *tail as u64;
                hash = hash.wrapping_mul(m);
            },
            6 => {
                hash ^= (*tail.offset(5) as u64) << 40;
                hash ^= (*tail.offset(4) as u64) << 32;
                hash ^= (*tail.offset(3) as u64) << 24;
                hash ^= (*tail.offset(2) as u64) << 16;
                hash ^= (*tail.offset(1) as u64) << 8;
                hash ^= *tail as u64;
                hash = hash.wrapping_mul(m);
            },
            5 => {
                hash ^= (*tail.offset(4) as u64) << 32;
                hash ^= (*tail.offset(3) as u64) << 24;
                hash ^= (*tail.offset(2) as u64) << 16;
                hash ^= (*tail.offset(1) as u64) << 8;
                hash ^= *tail as u64;
                hash = hash.wrapping_mul(m);
            },
            4 => {
                hash ^= (*tail.offset(3) as u64) << 24;
                hash ^= (*tail.offset(2) as u64) << 16;
                hash ^= (*tail.offset(1) as u64) << 8;
                hash ^= *tail as u64;
                hash = hash.wrapping_mul(m);
            },
            3 => {
                hash ^= (*tail.offset(2) as u64) << 16;
                hash ^= (*tail.offset(1) as u64) << 8;
                hash ^= *tail as u64;
                hash = hash.wrapping_mul(m);
            },
            2 => {
                hash ^= (*tail.offset(1) as u64) << 8;
                hash ^= *tail as u64;
                hash = hash.wrapping_mul(m);
            },
            1 => {
                hash ^= *tail as u64;
                hash = hash.wrapping_mul(m);
            },
            _ => {},
        }
    }

    hash ^= hash >> r;
    hash = hash.wrapping_mul(m);
    hash ^= hash >> r;
    hash
}
