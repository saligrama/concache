use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;

mod linked_list;
use self::linked_list::{LinkedList, Node};

const OSC: Ordering = Ordering::SeqCst;
const REFRESH_RATE: usize = 100;


struct Table {
    nbuckets: usize,
    map: Vec<LinkedList>,
    nitems: AtomicUsize,
}

impl Table {
    fn new(num_of_buckets: usize) -> Self {
        let mut t = Table {
            nbuckets: num_of_buckets,
            map: Vec::with_capacity(num_of_buckets),
            nitems: AtomicUsize::new(0),
        };

        for _ in 0..num_of_buckets {
            t.map.push(LinkedList::new());
        }

        t
    }

    fn insert(
        &self,
        key: usize,
        value: usize,
        remove_nodes: &mut Vec<*mut Node>,
    ) -> Option<*mut usize> {
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

    fn get(&self, key: usize, remove_nodes: &mut Vec<*mut Node>) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        self.map[index].get(key, remove_nodes)
    }

    fn delete(&self, key: usize, remove_nodes: &mut Vec<*mut Node>) -> Option<usize> {
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

pub struct MapHandle {
    map: Arc<Map>,
    epoch_counter: Arc<AtomicUsize>,
    remove_nodes: Vec<*mut Node>,
    remove_val: Vec<*mut usize>,
    refresh: usize, 
}

unsafe impl Send for MapHandle {}


impl MapHandle {
    pub fn cleanup(&mut self) {
        // println!("Cleaning");
        //epoch set up, load all of the values
        let mut started = Vec::new();
        let handles_map = self.map.handles.read().unwrap();
        for h in handles_map.iter() {
            started.push(h.load(OSC));
        }
        for (i, h) in handles_map.iter().enumerate() {
            if started[i] % 2 == 0 {
                continue;
            }
            let mut check = h.load(OSC);
            let mut iter = 0;
            while (check <= started[i]) && (check % 2 == 1) {
                if iter % 4 == 0 {
                    // we may be waiting for a thread that isn't currently running
                    thread::yield_now();
                }
                check = h.load(OSC);
                iter += 1;
                //do nothing, epoch spinning
            }
        }

        //physical deletion, epoch has rolled over so we are safe to proceed with physical deletion
        //epoch rolled over, so we know we have exclusive access to the node

        // println!("{:?}", &self.remove_nodes.len());
        for to_drop in &self.remove_nodes {
            let n = unsafe { (&**to_drop).val.load(OSC) };
            self.remove_val.push(n);
            //[drop the value inside of the node, or add to remove_val]
            drop(unsafe { Box::from_raw(*to_drop) });
        }

        // println!("{:?}", &self.remove_val.len());
        for to_drop in &self.remove_val {
            drop(unsafe { Box::from_raw(*to_drop) });
        }

        //reset
        self.remove_nodes = Vec::new();
        self.remove_val = Vec::new();
    }

    pub fn insert(&mut self, key: usize, value: usize) -> Option<usize> {
        self.refresh += 1;

        self.epoch_counter.fetch_add(1, OSC);
        let val = self.map.insert(key, value, &mut self.remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);

        let mut ret = None;

        if let Some(v) = val {
            ret = Some(unsafe { *v });
            self.remove_val.push(v);
        }

        if self.refresh == REFRESH_RATE {
            self.refresh = 0;
            self.cleanup();
        }

        ret
    }

    pub fn get(&mut self, key: usize) -> Option<usize> {
        self.refresh = (self.refresh + 1) % REFRESH_RATE;

        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.get(key, &mut self.remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);

        if self.refresh == REFRESH_RATE {
            self.refresh = 0;
            self.cleanup();
        }

        ret
    }

    pub fn delete(&mut self, key: usize) -> Option<usize> {
        self.refresh = (self.refresh + 1) % REFRESH_RATE;

        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.delete(key, &mut self.remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);

        if self.refresh == REFRESH_RATE {
            self.refresh = 0;
            self.cleanup();
        }

        ret
    }
}

impl Clone for MapHandle {
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

pub struct Map {
    table: Table,
    handles: RwLock<Vec<Arc<AtomicUsize>>>, //(started, finished)
}

impl Map {
    pub fn with_capacity(num_items: usize) -> MapHandle {
        let new_hashmap = Map {
            table: Table::new(num_items),
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

    fn insert(
        &self,
        key: usize,
        value: usize,
        remove_nodes: &mut Vec<*mut Node>,
    ) -> Option<*mut usize> {
        self.table.insert(key, value, remove_nodes)
    }

    fn get(&self, key: usize, remove_nodes: &mut Vec<*mut Node>) -> Option<usize> {
        self.table.get(key, remove_nodes)
    }

    fn delete(&self, key: usize, remove_nodes: &mut Vec<*mut Node>) -> Option<usize> {
        self.table.delete(key, remove_nodes)
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
                        let v = new_handle.get(val);
                        if v.is_some() {
                            assert_eq!(v.unwrap(), val);
                        }
                    } else {
                        new_handle.delete(val);
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
        assert_eq!(handle.get(1).unwrap(), 3);
        assert_eq!(handle.delete(1).unwrap(), 3);
        assert_eq!(handle.get(1), None);
        assert_eq!(handle.delete(2).unwrap(), 5);
        assert_eq!(handle.delete(16).unwrap(), 3);
        assert_eq!(handle.get(16), None);
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

        assert_eq!(new_hashmap.get(20).unwrap(), 5);
        assert_eq!(new_hashmap.get(12).unwrap(), 5);
        assert_eq!(new_hashmap.get(1).unwrap(), 1);
        assert_eq!(new_hashmap.get(0).unwrap(), 0);
        assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed

        // try the same assert_eqs
        assert_eq!(new_hashmap.get(20).unwrap(), 5);
        assert_eq!(new_hashmap.get(12).unwrap(), 5);
        assert_eq!(new_hashmap.get(1).unwrap(), 1);
        assert_eq!(new_hashmap.get(0).unwrap(), 0);
        assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed
    }
}
