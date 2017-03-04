use std::hash::Hash;
use std::hash::BuildHasher;
use std::hash::Hasher;

// struct Kv<V> {
//     key: u64,
//     value: V
// }

pub struct FastMap<K, V, S = Murmur2_64a>
    where K: Eq + Hash{
    cache:  Vec<Vec<Bucket<K, V>>>,
    // indices:  Vec<(K, V)>,
    size: u32,
    mod_mask: u64,
    count: usize,
    hasher: S,
}

enum Bucket<K, V>
    where K: Eq + Hash {
    Value(u64, K, V),
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
        // let _guard = flame::start_guard("insert");
        let (hash, ix) = self.calc_index(&key);

        {
            let ref mut vals = self.cache[ix];
            for kv in vals.iter() {
                match *kv {
                    Bucket::Value(h, ref k, _) => {
                        if h == hash && *k == key {
                            return false;
                        }
                    }
                    Bucket::Empty => {}
                }
            }

            self.count += 1;
            vals.push(Bucket::Value(hash, key, value));
        }
        if (self.count & 4) == 4 {
            self.ensure_load_rate();
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
        // let _guard = flame::start_guard("get");
        let (hash, ix) = self.calc_index(&key);

        let ref vals = self.cache[ix];

        if vals.len() > 0 {
            // let _guard2 = flame::start_guard("get_search");

            for kv in vals.iter() {
                match *kv {
                    Bucket::Value(h, ref k, ref v) => {
                        if h == hash && *k == key {
                            return Some(&v);
                        }
                    }
                    Bucket::Empty => {}
                }
            }

            return None;

        } else {
            return None;
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
        let (hash, ix) = self.calc_index(&key);

        let ref mut vals = self.cache[ix];

        if vals.len() > 0 {
            for kv in vals {
                match *kv {
                    Bucket::Value(h, ref k, ref mut v) => {
                        if h == hash && *k == key {
                            return Some(&mut *v);
                        }
                    }
                    Bucket::Empty => {}
                }
            }

            return None;

        } else {
            return None;
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
        let (hash, ix) = self.calc_index(&key);

        let ref mut vals = self.cache[ix];

        if vals.len() > 0 {

            for i in 0..vals.len() {
                let key_match;
                {
                    let ref peek = vals[i];

                    match *peek {
                        Bucket::Value(h, ref k, _) => {
                            if h == hash && *k == key {
                                key_match = true;
                            } else {
                                key_match = false;
                            }
                        }
                        Bucket::Empty => {key_match = false;}
                    }
                }

                if key_match {
                    self.count -= 1;
                    let kv = vals.swap_remove(i);
                    match kv {
                        Bucket::Value(_, _, v) => return Some(v),
                        Bucket::Empty => panic!("Bucket empty when it should not be!"),
                    }
                }
            }

            return None;

        } else {
            return None;
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
            self.cache[i].clear();
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
        Iter::new(&self.cache)
    }

    pub fn keys(&mut self) -> Keys<K, V> {
        Keys { inner: self.iter() }
    }

    pub fn values(&mut self) -> Values<K, V> {
        Values { inner: self.iter() }
    }

    pub fn iter_mut<'a>(&mut self) -> IterMut<K, V> {
        IterMut::new(&mut self.cache)
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
        // let _guard =flame::start_guard("calc_index");
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        // Faster modulus
        let ix = (hash & self.mod_mask) as usize;
        (hash, ix)
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

        let mut vec: Vec<Vec<Bucket<K, V>>> = Vec::new();

        vec.append(&mut self.cache);

        for _ in 0..new_lim {
            self.cache.push(Vec::with_capacity(0));
        }

        while vec.len() > 0 {
            let mut values = vec.pop().unwrap();
            while values.len() > 0 {
                if let Some(kv) = values.pop() {
                    if let Bucket::Value(h, k, v) = kv {
                        let (hash, ix) = self.calc_index(&k);

                        let ref mut vals = self.cache[ix];
                        vals.push(Bucket::Value(hash, k, v));
                    }
                }
            }
        }

        debug_assert!(self.cache.len() == self.lim(), "cache vector the wrong length, lim: {:?} cache: {:?}", self.lim(), self.cache.len());
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

        for i in 0..self.cache.len() {
            if self.cache[i].len() > 0 {
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

        for i in 0..self.cache.len() {
            for _ in self.cache[i].iter() {
                count += 1;
            }
        }

        self.count == count
    }


    pub fn collisions(&self) -> FastMap<u64, u64> {
        let mut map = FastMap::new();

        for s in self.cache.iter() {
            let key = s.len() as u64;
            if key > 0 {
                if !map.contains_key(key) {
                    map.insert(key, 1);
                } else {
                    let counter = map.get_mut(key).unwrap();
                    *counter += 1;
                }
            }
        }

        // map.sort();

        map
    }

    pub fn write_flame(&self) {
        use std::fs::File;
        // flame::dump_html(&mut File::create("flame-graph.html").unwrap()).unwrap();
    }
}



    // #[derive(Debug)]
    // pub struct IterMut2<'a, V: 'a> {
    //     iter: std::iter::Map<'a, Vec<(u64, V)>>,
    //     // inner: SliceIterMut<'a, (u64, V)>,
    // }


use std::slice::Iter as SliceIter;
use std::slice::IterMut as SliceIterMut;

// ***************** Iter *********************

pub struct Iter<'a, K: 'a, V: 'a>
    where K: Eq + Hash {
    outer: SliceIter<'a, Vec<Bucket<K, V>>>,
    inner: SliceIter<'a, Bucket<K, V>>,
}

impl<'a, K, V> Iter<'a, K, V>
    where K: Eq + Hash{
    fn new(vec: &'a Vec<Vec<Bucket<K, V>>>) -> Self {
        let mut outer = vec.iter();
        let inner = outer.next()
                         .map(|v| v.iter())
                         .unwrap_or_else(|| (&[]).iter());

        Iter {
            outer: outer,
            inner: inner,
         }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V>
    where K: Eq + Hash{
    type Item = (&'a K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        loop {
            match self.inner.next() {
                Some(r) => {
                    match *r {
                        Bucket::Value(_, ref k, ref v) => return Some((&k, &v)),
                        Bucket::Empty => (),
                    }

                },
                None => (),
            }

            match self.outer.next() {
                Some(v) => self.inner = v.iter(),
                None => return None,
            }
        }
    }
}


// ***************** Iter Mut *********************

pub struct IterMut<'a, K: 'a, V: 'a>
    where K: Eq + Hash {
    outer: SliceIterMut<'a, Vec<Bucket<K, V>>>,
    inner: SliceIterMut<'a, Bucket<K, V>>,
}

impl<'a, K, V> IterMut<'a, K, V>
    where K: Eq + Hash{
    fn new(vec: &'a mut Vec<Vec<Bucket<K, V>>>) -> IterMut<'a, K, V> {
        let mut outer = vec.iter_mut();
        let inner = outer.next()
                         .map(|v| v.iter_mut())
                         .unwrap_or_else(|| (&mut []).iter_mut() );

        IterMut {
            outer: outer,
            inner: inner,
        }
    }
}


impl<'a, K, V> Iterator for IterMut<'a, K, V>
    where K: Eq + Hash{
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<(&'a K, &'a mut V)> {
        loop {
            match self.inner.next() {
                Some(r) => {
                    match *r {
                        Bucket::Value(_, ref k, ref mut v) => return Some((&k, &mut *v)),
                        Bucket::Empty => (),
                    }
                }
                None => (),
            }

            match self.outer.next() {
                Some(v) => self.inner = v.iter_mut(),
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
