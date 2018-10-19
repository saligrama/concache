//! A concurrent hash map implementation with [crossbeam memory
//! reclamation](https://docs.rs/crossbeam-epoch/).
//!
//! This implementation provides a lock-free hash map using buckets that hold [lock-free linked
//! lists](https://www.microsoft.com/en-us/research/wp-content/uploads/2001/10/2001-disc.pdf).
//! Memory is safely destructed and reclaimed using
//! [`crossbeam::epoch`](https://docs.rs/crossbeam-epoch/). Table resizing is not yet supported,
//! but the map will also never fill due to the linked implementation; instead, performance will
//! decrease as the map is filled with more keys.
//!
//! The interface to this map is somewhat different from `HashMap` to support concurrent operation.
//! When you create a new [`Map`],you are given a [`MapHandle`], which allows access to the map's
//! data. To read or mutate the map for elsewhere, you call [`MapHandle::clone`], which gives you
//! a new `MapHandle` that provides concurrent access to the same map.
//!
//! Similarly to [`crossbeam::epoch`](https://docs.rs/crossbeam-epoch/), this `Map` does not
//! guarantee that destructors are called. In practice though, as long as threads do not leak
//! `MapHandle`s, destructors will all eventually be called.
//!
//! Note that unlike `HashMap`, this `Map` requires its values to be `Copy`. This greatly
//! simplifies the map's interface; accesses to the map's data have to be carefully guarded, and
//! there is no simple way to expose references into the map through a method call. Later, we may
//! provide temporary access through closures, similar to `evmap`'s
//! [`ReadHandle::get_and`](https://docs.rs/evmap/4/evmap/struct.ReadHandle.html#method.get_and),
//! but for the time being, values have to be `Copy`.

mod linked_list;

use self::linked_list::LinkedList;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

/// A handle to a shared [`Map`].
///
/// Any operation performed on this handle affects the map seen by all other related `MapHandle`
/// instances. To get another handle to the `Map`, simply clone any of its handles.
#[derive(Clone)]
pub struct MapHandle<K, V> {
    bsize: usize,
    size: Arc<AtomicUsize>,
    mp: Arc<Vec<LinkedList<K, V>>>,
}

/// A shared, concurrent hash map.
///
/// See [`MapHandle`] for how to interact with this map.
pub type Map<K, V> = MapHandle<K, V>;

impl<K, V> MapHandle<K, V> {
    /// Create a new, shared map and return a handle to it.
    ///
    /// The map will have `nbuckets` buckets to distribute stored keys among. If there are many
    /// more keys than buckets, performance will suffer, as all the keys in a key's bucket must be
    /// searched to read or update that key.
    pub fn with_capacity(nbuckets: usize) -> Self {
        let mut v = Vec::with_capacity(nbuckets);

        for _i in 0..nbuckets {
            v.push(LinkedList::default());
        }

        Map {
            bsize: nbuckets,
            size: Arc::new(AtomicUsize::new(0)),
            mp: Arc::new(v),
        }
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::crossbeam::Map;
    ///
    /// let mut a = Map::with_capacity(16);
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.size.load(Ordering::SeqCst)
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::crossbeam::Map;
    ///
    /// let mut a = Map::with_capacity(16);
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.size.load(Ordering::SeqCst) == 0
    }
}

impl<K, V> Map<K, V>
where
    K: Eq + Hash,
    V: Copy,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `true` is returned.
    ///
    /// If the map did have this key present, the value is updated, and `false` is returned.
    /// The key is not updated, though; this matters for types that can be `==` without being
    /// identical.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::crossbeam::Map;
    ///
    /// let mut map = Map::with_capacity(16);
    /// assert_eq!(map.insert(37, "a"), true);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), false);
    /// assert_eq!(map.get(&37), Some("c"));
    /// ```
    pub fn insert(&self, key: K, value: V) -> Option<*mut V> {
        let mut hsh = DefaultHasher::new();
        key.hash(&mut hsh);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;
        let ret = self.mp[ndx].insert((key, value));
        if ret.is_none() {
            self.size.fetch_add(1, Ordering::SeqCst);
        }
        ret
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::crossbeam::Map;
    ///
    /// let mut map = Map::with_capacity(16);
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some("a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get(&self, key: &K) -> Option<V> {
        let mut hsh = DefaultHasher::new();
        key.hash(&mut hsh);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        self.mp[ndx].get(key)
    }

    /// Removes a key from the map, returning `true` if the key was previously in the map.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use concache::crossbeam::Map;
    ///
    /// let mut map = Map::with_capacity(16);
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), true);
    /// assert_eq!(map.remove(&1), false);
    /// ```
    pub fn remove(&self, key: &K) -> bool {
        let mut hsh = DefaultHasher::new();
        key.hash(&mut hsh);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        if self.mp[ndx].remove(key) {
            self.size.fetch_sub(1, Ordering::SeqCst);
            return true;
        }
        false
    }
}

impl<K, V> fmt::Debug for Map<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: use https://doc.rust-lang.org/std/fmt/struct.DebugMap.html
        let mut all = String::new();
        for i in 0..self.bsize {
            // TODO: I'm _sure_ there's a better way to do this
            all.push_str(&format!("{:?}", &self.mp[i]));
        }
        let ret: String = all.chars().skip(0).take(all.len() - 2).collect();
        write!(f, "[{}]", ret)
    }
}

#[cfg(all(test, feature = "bench"))]
mod benchmarks {
    use super::*;
    use rand::{thread_rng, Rng};
    use test::Bencher;

    //BENCHMARKS
    #[inline]
    fn getn(b: &mut Bencher, n: usize) {
        let handle = Map::with_capacity(1024);
        for key in 0..n {
            handle.insert(key, 0);
        }
        let mut rng = thread_rng();

        b.iter(|| {
            let key = rng.gen_range(0, n);
            handle.get(key);
        });
    }

    //get
    #[bench]
    fn get0128(b: &mut Bencher) {
        getn(b, 128);
    }

    #[bench]
    fn get0256(b: &mut Bencher) {
        getn(b, 256);
    }

    #[bench]
    fn get0512(b: &mut Bencher) {
        getn(b, 512);
    }

    #[bench]
    fn get1024(b: &mut Bencher) {
        getn(b, 1024);
    }

    #[bench]
    fn get2048(b: &mut Bencher) {
        getn(b, 2048);
    }

    #[bench]
    fn get4096(b: &mut Bencher) {
        getn(b, 4096);
    }

    #[bench]
    fn get8192(b: &mut Bencher) {
        getn(b, 8192);
    }

    #[inline]
    fn updaten(b: &mut Bencher, n: usize) {
        let handle = Map::with_capacity(1024);
        for key in 0..n {
            handle.insert(key, 0);
        }
        let mut rng = thread_rng();

        b.iter(|| {
            let key = rng.gen_range(0, n);
            handle.insert(key, 1);
        });
    }

    //update
    #[bench]
    fn update0128(b: &mut Bencher) {
        updaten(b, 128);
    }

    #[bench]
    fn update0256(b: &mut Bencher) {
        updaten(b, 256);
    }

    #[bench]
    fn update0512(b: &mut Bencher) {
        updaten(b, 512);
    }

    #[bench]
    fn update1024(b: &mut Bencher) {
        updaten(b, 1024);
    }

    #[bench]
    fn update2048(b: &mut Bencher) {
        updaten(b, 2048);
    }

    #[bench]
    fn update4096(b: &mut Bencher) {
        updaten(b, 4096);
    }

    #[bench]
    fn update8192(b: &mut Bencher) {
        updaten(b, 8192);
    }

    fn removen(b: &mut Bencher, n: usize) {
        let handle = Map::with_capacity(1024);
        for key in 0..n {
            handle.insert(key, 0);
        }
        let mut rng = thread_rng();

        b.iter(|| {
            let key = rng.gen_range(0, n);
            handle.remove(key);
            handle.insert(key, 0);
        });
    }

    //remove
    #[bench]
    fn remove0128(b: &mut Bencher) {
        removen(b, 128);
    }

    #[bench]
    fn remove0256(b: &mut Bencher) {
        removen(b, 256);
    }

    #[bench]
    fn remove0512(b: &mut Bencher) {
        removen(b, 512);
    }

    #[bench]
    fn remove1024(b: &mut Bencher) {
        removen(b, 1024);
    }

    #[bench]
    fn remove2048(b: &mut Bencher) {
        removen(b, 2048);
    }

    #[bench]
    fn remove4096(b: &mut Bencher) {
        removen(b, 4096);
    }

    #[bench]
    fn remove8192(b: &mut Bencher) {
        removen(b, 8192);
    }

    #[bench]
    fn insert(b: &mut Bencher) {
        let handle = Map::with_capacity(1024);

        b.iter(|| {
            handle.insert(1, 0);
            handle.remove(1);
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::thread;

    /*
    the data produced is a bit strange because of the way I take mod to test only even values
    are inserted so the end number of values should be n/2 (computer style) and the capacity
    of the map should be equal to the greatest power of 2 less than n/2.
    */
    #[test]
    fn hashmap_concurr() {
        let handle = Map::with_capacity(8); //changed this,
        let mut threads = vec![];
        let nthreads = 5;
        // let handle = MapHandle::new(Arc::clone(&new_hashmap).table.read().unwrap());
        for _ in 0..nthreads {
            let mut new_handle = handle.clone();

            threads.push(thread::spawn(move || {
                let num_iterations = 1000000;
                for _ in 0..num_iterations {
                    let mut rng = thread_rng();
                    let val = rng.gen_range(0, 128);
                    let two = rng.gen_range(0, 3);

                    if two % 3 == 0 {
                        new_handle.insert(val, val);
                    } else if two % 3 == 1 {
                        let v = new_handle.get(&val);
                        if v.is_some() {
                            assert_eq!(v.unwrap(), val);
                        }
                    } else {
                        new_handle.remove(&val);
                    }
                }
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
    }

    #[test]
    fn hashmap_delete() {
        let handle = Map::with_capacity(8);
        handle.insert(1, 3);
        handle.insert(2, 5);
        handle.insert(3, 8);
        handle.insert(4, 3);
        handle.insert(5, 4);
        handle.insert(6, 5);
        handle.insert(7, 3);
        handle.insert(8, 3);
        handle.insert(9, 3);
        handle.insert(10, 3);
        handle.insert(11, 3);
        handle.insert(12, 3);
        handle.insert(13, 3);
        handle.insert(14, 3);
        handle.insert(15, 3);
        handle.insert(16, 3);
        assert_eq!(handle.get(&1).unwrap(), 3);
        assert_eq!(handle.remove(&1), true);
        assert_eq!(handle.get(&1), None);
        assert_eq!(handle.remove(&2), true);
        assert_eq!(handle.remove(&16), true);
        assert_eq!(handle.get(&16), None);
    }

    #[test]
    fn hashmap_basics() {
        let new_hashmap = Map::with_capacity(8); //init with 2 buckets
                                                     //input values
        new_hashmap.insert(1, 1);
        new_hashmap.insert(2, 5);
        new_hashmap.insert(12, 5);
        new_hashmap.insert(13, 7);
        new_hashmap.insert(0, 0);

        new_hashmap.insert(20, 3);
        new_hashmap.insert(3, 2);
        new_hashmap.insert(4, 1);

        assert_eq!(new_hashmap.insert(20, 5), false); //repeated new
        assert_eq!(new_hashmap.insert(3, 8), false); //repeated new

        new_hashmap.insert(3, 8); //repeated

        assert_eq!(new_hashmap.get(&20).unwrap(), 5);
        assert_eq!(new_hashmap.get(&12).unwrap(), 5);
        assert_eq!(new_hashmap.get(&1).unwrap(), 1);
        assert_eq!(new_hashmap.get(&0).unwrap(), 0);
        assert!(new_hashmap.get(&3).unwrap() != 2); // test that it changed

        // try the same assert_eqs
        assert_eq!(new_hashmap.get(&20).unwrap(), 5);
        assert_eq!(new_hashmap.get(&12).unwrap(), 5);
        assert_eq!(new_hashmap.get(&1).unwrap(), 1);
        assert_eq!(new_hashmap.get(&0).unwrap(), 0);
        assert!(new_hashmap.get(&3).unwrap() != 2); // test that it changed
    }
}
