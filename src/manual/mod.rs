use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};

mod linked_list;
use self::linked_list::{LinkedList, Node};

const OSC: Ordering = Ordering::SeqCst;

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
}

impl MapHandle {
    pub fn insert(&self, key: usize, value: usize) -> Option<usize> {
        let mut remove_nodes: Vec<*mut Node> = Vec::new();

        self.epoch_counter.fetch_add(1, OSC);
        let val = self.map.insert(key, value, &mut remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);

        let mut ret = None;

        if let Some(v) = val {
            ret = Some(unsafe { *v });
            self.free_val(v);
        }

        if !remove_nodes.is_empty() {
            self.free_nodes(&remove_nodes);
        }

        ret
    }

    pub fn get(&self, key: usize) -> Option<usize> {
        let mut remove_nodes: Vec<*mut Node> = Vec::new();

        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.get(key, &mut remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);
        if !remove_nodes.is_empty() {
            self.free_nodes(&remove_nodes);
        }

        ret
    }

    pub fn delete(&self, key: usize) -> Option<usize> {
        let mut remove_nodes: Vec<*mut Node> = Vec::new();

        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.delete(key, &mut remove_nodes);
        self.epoch_counter.fetch_add(1, OSC);
        if !remove_nodes.is_empty() {
            self.free_nodes(&remove_nodes);
        }

        ret
    }

    fn free_val(&self, remove_val: *mut usize) {
        //epoch set up, load all of the values
        let mut started = Vec::new();
        let handles_map = self.map.handles.read().unwrap();
        for h in handles_map.iter() {
            started.push(h.load(OSC));
        }
        for (i, h) in handles_map.iter().enumerate() {
            let mut check = h.load(OSC);
            while (check <= started[i]) && (check % 2 == 1) {
                check = h.load(OSC);
                //do nothing, epoch spinning
            }
            //now finished is greater than or equal to started
        }

        //physical deletion, epoch has rolled over so we are safe to proceed with physical deletion
        // epoch rolled over, so we know we have exclusive access to the node

        drop(unsafe { Box::from_raw(remove_val) });
    }

    fn free_nodes(&self, remove_nodes: &[*mut Node]) {
        //epoch set up, load all of the values
        let mut started = Vec::new();
        let handles_map = self.map.handles.read().unwrap();
        for h in handles_map.iter() {
            started.push(h.load(OSC));
        }
        for (i, h) in handles_map.iter().enumerate() {
            let mut check = h.load(OSC);
            while (check <= started[i]) && (check % 2 == 1) {
                check = h.load(OSC);
                //do nothing, epoch spinning
            }
            //now finished is greater than or equal to started
        }

        //physical deletion, epoch has rolled over so we are safe to proceed with physical deletion
        // epoch rolled over, so we know we have exclusive access to the node

        for to_drop in remove_nodes {
            drop(unsafe { Box::from_raw(*to_drop) });
        }
    }
}

impl Clone for MapHandle {
    fn clone(&self) -> Self {
        let ret = Self {
            map: Arc::clone(&self.map),
            epoch_counter: Arc::new(AtomicUsize::new(0)),
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
            let new_handle = handle.clone();

            threads.push(thread::spawn(move || {
                let num_iterations = 100000;
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
    fn hashmap_handle_cloning() {
        let handle = Arc::new(Map::with_capacity(8)); //init with 16 bucket
        println!("{:?}", handle.epoch_counter);
        handle.insert(1, 3);
        assert_eq!(handle.get(1).unwrap(), 3);

        //create a new handle
        let new_handle = Arc::clone(&handle);
        assert_eq!(new_handle.get(1).unwrap(), 3);
        new_handle.insert(2, 5);

        assert_eq!(handle.get(2).unwrap(), 5);
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
        assert_eq!(handle.get(1).unwrap(), 3);
        assert_eq!(handle.delete(1).unwrap(), 3);
        assert_eq!(handle.get(1), None);
        assert_eq!(handle.delete(2).unwrap(), 5);
        assert_eq!(handle.delete(16).unwrap(), 3);
        assert_eq!(handle.get(16), None);
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

    fn deleten(b: &mut Bencher, n: usize) {
        let handle = Map::with_capacity(1024);
        for key in 0..n {
            handle.insert(key, 0);
        }
        let mut rng = thread_rng();

        b.iter(|| {
            let key = rng.gen_range(0, n);
            handle.delete(key);
            handle.insert(key, 0);
        });
    }

    //delete
    #[bench]
    fn delete0128(b: &mut Bencher) {
        deleten(b, 128);
    }

    #[bench]
    fn delete0256(b: &mut Bencher) {
        deleten(b, 256);
    }

    #[bench]
    fn delete0512(b: &mut Bencher) {
        deleten(b, 512);
    }

    #[bench]
    fn delete1024(b: &mut Bencher) {
        deleten(b, 1024);
    }

    #[bench]
    fn delete2048(b: &mut Bencher) {
        deleten(b, 2048);
    }

    #[bench]
    fn delete4096(b: &mut Bencher) {
        deleten(b, 4096);
    }

    #[bench]
    fn delete8192(b: &mut Bencher) {
        deleten(b, 8192);
    }

    #[bench]
    fn insert(b: &mut Bencher) {
        let handle = Map::with_capacity(1024);

        b.iter(|| {
            handle.insert(1, 0);
            handle.delete(1);
        })
    }
}
