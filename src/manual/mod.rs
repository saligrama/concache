//! A concurrent hash map implementation with a hand-written epoch-based memory management scheme.
//!
//! This implementation provides a lock-free hash map using buckets that hold [lock-free linked
//! lists](https://www.microsoft.com/en-us/research/wp-content/uploads/2001/10/2001-disc.pdf).
//! Memory is safely destructed and reclaimed using a simplified variant of _Quiescent-State-Based
//! Reclamation_. Table resizing is not yet supported, but the map will also never fill due to the
//! linked implementation; instead, performance will decrease as the map is filled with more keys.
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

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

mod linked_list;
use self::linked_list::{LinkedList, Node};

const OSC: Ordering = Ordering::SeqCst;
const REFRESH_RATE: usize = 100;

struct Table<K, V> {
    nbuckets: usize,
    map: Vec<LinkedList<K, V>>,
    nitems: AtomicUsize,
}

impl<K, V> Table<K, V> {
    fn new(num_of_buckets: usize) -> Self {
        let mut t = Table {
            nbuckets: num_of_buckets,
            map: Vec::with_capacity(num_of_buckets),
            nitems: AtomicUsize::new(0),
        };

        for _ in 0..num_of_buckets {
            t.map.push(LinkedList::default());
        }

        t
    }
}

impl<K, V> Table<K, V>
where
    K: Hash + Ord,
    V: Copy,
{
    fn insert(&self, key: K, value: V, remove_nodes: &mut Vec<*mut Node<K, V>>) -> Option<*mut V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let ret = self.map[index].insert(key, value, remove_nodes);

        if ret.is_none() {
            self.nitems.fetch_add(1, OSC);
        }

        ret
    }

    fn get(&self, key: &K, remove_nodes: &mut Vec<*mut Node<K, V>>) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        self.map[index].get(key, remove_nodes)
    }

    fn delete(&self, key: &K, remove_nodes: &mut Vec<*mut Node<K, V>>) -> Option<V> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let ret = self.map[index].delete(key, remove_nodes);

        if ret.is_some() {
            self.nitems.fetch_sub(1, OSC);
        }

        ret
    }
}

/// A handle to a shared [`Map`].
///
/// Any operation performed on this handle affects the map seen by all other related `MapHandle`
/// instances. To get another handle to the `Map`, simply clone any of its handles.
pub struct MapHandle<K, V> {
    map: Arc<Map<K, V>>,
    epoch_counter: Arc<AtomicUsize>,
    remove_nodes: Vec<*mut Node<K, V>>,
    remove_val: Vec<*mut V>,
    refresh: usize,
}

unsafe impl<K, V> Send for MapHandle<K, V>
where
    K: Send + Sync,
    V: Send,
{
}

// impl<K, V> MapHandle<K, V> {
//     fn cleanup(&mut self) {
//         //epoch set up, load all of the values
//         let mut started = Vec::new();
//         let handles_map = self.map.handles.read().unwrap();
//         for h in handles_map.iter() {
//             started.push(h.load(OSC));
//         }
//         for (i, h) in handles_map.iter().enumerate() {
//             if started[i] % 2 == 0 {
//                 continue;
//             }
//             let mut check = h.load(OSC);
//             let mut iter = 0;
//             while (check <= started[i]) && (check % 2 == 1) {
//                 if iter % 4 == 0 {
//                     // we may be waiting for a thread that isn't currently running
//                     thread::yield_now();
//                 }
//                 check = h.load(OSC);
//                 iter += 1;
//                 //do nothing, epoch spinning
//             }
//         }

//         //physical deletion, epoch has rolled over so we are safe to proceed with physical deletion
//         //epoch rolled over, so we know we have exclusive access to the node

//         // println!("{:?}", &self.remove_nodes.len());
//         for to_drop in &self.remove_nodes {
//             let n = unsafe { (&**to_drop).val.load(OSC) };
//             self.remove_val.push(n);
//             //[drop the value inside of the node, or add to remove_val]
//             drop(unsafe { Box::from_raw(*to_drop) });
//         }

//         // println!("{:?}", &self.remove_val.len());
//         for to_drop in &self.remove_val {
//             drop(unsafe { Box::from_raw(*to_drop) });
//         }

//         //reset
//         self.remove_nodes = Vec::new();
//         self.remove_val = Vec::new();
//     }
// }

impl<K, V> MapHandle<K, V>
where
    K: Hash + Ord,
    V: Copy,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    /// The key is not updated, though; this matters for types that can be `==` without being
    /// identical.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::manual::Map;
    ///
    /// let mut map = Map::with_capacity(16);
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map.get(&37), Some("c"));
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // self.refresh = (self.refresh + 1) % REFRESH_RATE;

        self.epoch_counter.fetch_add(1, OSC);
        let val = self.map.table.insert(key, value, &mut self.remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);

        let mut ret = None;

        if let Some(v) = val {
            ret = Some(unsafe { *v });
            self.remove_val.push(v);
        }

        // if self.refresh == REFRESH_RATE {
        //     self.refresh = 0;
        //     self.cleanup();
        // }

        ret
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::manual::Map;
    ///
    /// let mut map = Map::with_capacity(16);
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some("a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get(&mut self, key: &K) -> Option<V> {
        // self.refresh = (self.refresh + 1) % REFRESH_RATE;

        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.table.get(key, &mut self.remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);

        // if self.refresh == REFRESH_RATE {
        //     self.refresh = 0;
        //     self.cleanup();
        // }

        ret
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::manual::Map;
    ///
    /// let mut map = Map::with_capacity(16);
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    pub fn remove(&mut self, key: &K) -> Option<V> {
        // self.refresh = (self.refresh + 1) % REFRESH_RATE;

        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.table.delete(key, &mut self.remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);

        // if self.refresh == REFRESH_RATE {
        //     self.refresh = 0;
        //     self.cleanup();
        // }

        ret
    }

    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::manual::Map;
    ///
    /// let mut a = Map::with_capacity(16);
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.map.table.nitems.load(OSC)
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use concache::manual::Map;
    ///
    /// let mut a = Map::with_capacity(16);
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.map.table.nitems.load(OSC) == 0
    }
}

impl<K, V> Clone for MapHandle<K, V> {
    fn clone(&self) -> Self {
        let ret = Self {
            map: Arc::clone(&self.map),
            epoch_counter: Arc::new(AtomicUsize::new(0)),
            remove_nodes: Vec::new(),
            remove_val: Vec::new(),
            refresh: 0,
        };

        let mut handles_vec = self.map.handles.write().unwrap(); //handles vector
        handles_vec.push(Arc::clone(&ret.epoch_counter));

        ret
    }
}

/// A shared, concurrent hash map.
///
/// See [`MapHandle`] for how to interact with this map.
pub struct Map<K, V> {
    table: Table<K, V>,
    handles: RwLock<Vec<Arc<AtomicUsize>>>, //(started, finished)
}

impl<K, V> Map<K, V> {
    /// Create a new, shared map and return a handle to it.
    ///
    /// The map will have `nbuckets` buckets to distribute stored keys among. If there are many
    /// more keys than buckets, performance will suffer, as all the keys in a key's bucket must be
    /// searched to read or update that key.
    pub fn with_capacity(nbuckets: usize) -> MapHandle<K, V> {
        let new_hashmap = Map {
            table: Table::new(nbuckets),
            handles: RwLock::new(Vec::new()),
        };
        let ret = MapHandle {
            map: Arc::new(new_hashmap),
            epoch_counter: Arc::new(AtomicUsize::new(0)),
            remove_nodes: Vec::new(),
            remove_val: Vec::new(),
            refresh: 0,
        };

        //push the first maphandle into the epoch system
        let hashmap = Arc::clone(&ret.map);
        let mut handles_vec = hashmap.handles.write().unwrap();
        handles_vec.push(Arc::clone(&ret.epoch_counter));
        ret
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
                assert_eq!(new_handle.epoch_counter.load(OSC), num_iterations * 2);
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
    }

    #[test]
    fn hashmap_delete() {
        let mut handle = Map::with_capacity(8);
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
        assert_eq!(handle.remove(&1).unwrap(), 3);
        assert_eq!(handle.get(&1), None);
        assert_eq!(handle.remove(&2).unwrap(), 5);
        assert_eq!(handle.remove(&16).unwrap(), 3);
        assert_eq!(handle.get(&16), None);
    }

    #[test]
    fn hashmap_basics() {
        let mut new_hashmap = Map::with_capacity(8); //init with 2 buckets
                                                     //input values
        new_hashmap.insert(1, 1);
        new_hashmap.insert(2, 5);
        new_hashmap.insert(12, 5);
        new_hashmap.insert(13, 7);
        new_hashmap.insert(0, 0);

        new_hashmap.insert(20, 3);
        new_hashmap.insert(3, 2);
        new_hashmap.insert(4, 1);

        assert_eq!(new_hashmap.insert(20, 5).unwrap(), 3); //repeated
        assert_eq!(new_hashmap.insert(3, 8).unwrap(), 2); //repeated
        assert_eq!(new_hashmap.insert(5, 5), None); //repeated

        let cln = Arc::clone(&new_hashmap.map);
        assert_eq!(cln.table.nitems.load(OSC), 9);

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
