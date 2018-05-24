#![feature(integer_atomics)]

extern crate rand;

use std::sync::{Mutex, RwLock, atomic::*, Arc};
use std::ptr;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::thread;
use rand::{thread_rng, Rng};

const AVG_PER_BIN_THRESH : usize = 4;

struct Node {
    kv: (usize, Mutex<usize>),
    next: AtomicPtr<Node>
}

struct LinkedList {
    size: AtomicUsize,
    first: AtomicPtr<Node>,
}

struct Table {
    bsize: AtomicUsize,
    size: AtomicUsize,
    mp: Vec<LinkedList>
}

struct HashMap {
    table: RwLock<Table>
}

impl Node {
    fn new (k : usize, v : usize) -> Self {
        Node {
            kv: (k, Mutex::new(v)),
            next: AtomicPtr::new(ptr::null_mut())
        }
    }
}

impl LinkedList {
    fn new () -> Self {
        LinkedList {
            size: AtomicUsize::new(0),
            first: AtomicPtr::new(ptr::null_mut()),
        }
    }

    fn insert (&self, kv : (usize, usize)) {
        let ins = Box::new(Node::new(kv.0, kv.1));

        if self.first.load(Ordering::SeqCst).is_null() {
            // nonexistent first node
            self.first.compare_and_swap(ptr::null_mut(), Box::into_raw(ins), Ordering::SeqCst);
        } else {
            let mut not_mutated = false;
            let p = Box::into_raw(ins);

            while !not_mutated {
                let mut node_cur : &Node;
                let mut ptr_cur = &self.first;
                let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);
                let mut swap = true;

                while !ptr_raw.is_null() {
                    node_cur = unsafe { &*ptr_raw };
                    ptr_cur = &node_cur.next;
                    ptr_raw = ptr_cur.load(Ordering::SeqCst);

                    // found same key
                    if node_cur.kv.0 == kv.0 {
                        let mut change = node_cur.kv.1.lock().unwrap();
                        *change = kv.1;
                        swap = false;
                        not_mutated = false;
                        break;
                    }
                }

                if swap {
                    let ret = ptr_cur.compare_and_swap(ptr::null_mut(), p, Ordering::SeqCst);
                    if ret == ptr::null_mut() {
                        not_mutated = false;
                    }
                }
            }
        }
    }

    fn get (&self, key : usize) -> Option<usize> {
        if !self.first.load(Ordering::SeqCst).is_null() {

            let mut node_cur : &Node;
            let mut ptr_cur = &self.first;
            let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);

            while !ptr_raw.is_null() {
                node_cur = unsafe { &*ptr_raw };
                if node_cur.kv.0 == key {
                    let value = node_cur.kv.1.lock().unwrap();
                    return Some(*value);
                }

                ptr_cur = &node_cur.next;
                ptr_raw = ptr_cur.load(Ordering::SeqCst);
            }
        }
        None
    }
}

impl Table {
    fn new (nbuckets : usize) -> Self {
        let mut v = Vec::with_capacity(nbuckets);

        for _i in 0..nbuckets {
            v.push(LinkedList::new());
        }

        let ret = Table {
            bsize: AtomicUsize::new(nbuckets),
            size: AtomicUsize::new(0),
            mp: v
        };

        ret
    }

    fn resize (&mut self, nbuckets : usize) {
        let new = Table::new(nbuckets);
        let bsize = self.bsize.load(Ordering::SeqCst);
        for i in 0..bsize {
            let ll = &self.mp[i];

            let mut ptr_cur = &ll.first;
            let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);

            while !ptr_raw.is_null() {
                let mut node_cur = unsafe { &*ptr_raw };

                new.insert(node_cur.kv.0, *node_cur.kv.1.lock().unwrap());

                ptr_cur = &node_cur.next;
                ptr_raw = ptr_cur.load(Ordering::SeqCst);
            }
        }

        self.mp = new.mp;
        self.bsize.compare_and_swap(bsize, nbuckets, Ordering::SeqCst);
    }

    fn insert (&self, key : usize, value : usize) {
        let bsize = self.bsize.load(Ordering::SeqCst);

        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % bsize;

        &self.mp[ndx].insert((key, value));
        self.size.fetch_add(1, Ordering::SeqCst);
    }

    fn get (&self, key : usize) -> Option<usize> {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize.load(Ordering::SeqCst);

        (&self).mp[ndx].get(key)
    }
}

impl HashMap {
    fn new () -> Self {
        HashMap {
            table: RwLock::new(Table::new(4))
        }
    }

    fn insert (&self, key : usize, val : usize) {
        let t = (&self).table.read().unwrap();
        let bsize = t.bsize.load(Ordering::SeqCst);
        let size = t.size.load(Ordering::SeqCst);
        if size / bsize > AVG_PER_BIN_THRESH {
            self.resize(bsize * 2);
        }

        t.insert(key, val);
    }

    fn get (&self, key : usize) -> Option<usize> {
        let t = (&self).table.read().unwrap();
        t.get(key)
    }

    fn resize (&self, nbuckets : usize) {
        let mut t = (&self).table.write().unwrap();
        t.resize(nbuckets);
    }
}

fn main() {
	// let mut new_linked_list = LinkedList::new();
	// println!("{:?}", new_linked_list);
	// new_linked_list.insert((3, 2));
	// new_linked_list.insert((3, 4));
	// new_linked_list.insert((5, 8));
	// new_linked_list.insert((4, 6));
	// new_linked_list.insert((1, 8));
	// new_linked_list.insert((6, 6));
	// new_linked_list.print();

	// assert_eq!(new_linked_list.get(3).unwrap(), 4);
	// assert_eq!(new_linked_list.get(5).unwrap(), 8);
	// assert_eq!(new_linked_list.get(2), None);

    println!("Started");
    let mut new_HashMap = HashMap::new(); //init with 16 buckets
	// new_HashMap.mp[0].push((1,2)); //manually push
    //input values
    new_HashMap.insert(1, 1);
    new_HashMap.insert(2, 5);
    new_HashMap.insert(12, 5);
    new_HashMap.insert(13, 7);
    new_HashMap.insert(0, 0);

    println!("testing for 4");
    assert_eq!(new_HashMap.table.read().unwrap().mp.capacity(), 4); //should be 4 after you attempt the 5th insert

    new_HashMap.insert(20, 3);
    new_HashMap.insert(3, 2);
    new_HashMap.insert(4, 1);
    new_HashMap.insert(5, 5);

    new_HashMap.insert(20, 5); //repeated
    new_HashMap.insert(3, 8); //repeated
    println!("testing for 8");
    assert_eq!(new_HashMap.table.read().unwrap().mp.capacity(), 8);

	println!("Finished.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn HashMap_basics() {
        let mut new_HashMap = HashMap::new(); //init with 2 buckets
        //input values

        new_HashMap.insert(1, 1);
        new_HashMap.insert(2, 5);
        new_HashMap.insert(12, 5);
        new_HashMap.insert(13, 7);
        new_HashMap.insert(0, 0);

        assert_eq!(new_HashMap.table.read().unwrap().mp.capacity(), 4); //should be 4 after you attempt the 5th insert

        new_HashMap.insert(20, 3);
        new_HashMap.insert(3, 2);
        new_HashMap.insert(4, 1);
        new_HashMap.insert(5, 5);

        new_HashMap.insert(20, 5); //repeated
        new_HashMap.insert(3, 8); //repeated
        assert_eq!(new_HashMap.table.read().unwrap().mp.capacity(), 8); //should be 8 after you attempt the 9th insert

        assert_eq!(new_HashMap.get(20).unwrap(), 5);
        assert_eq!(new_HashMap.get(12).unwrap(), 5);
        assert_eq!(new_HashMap.get(1).unwrap(), 1);
        assert_eq!(new_HashMap.get(0).unwrap(), 0);
        assert!(new_HashMap.get(3).unwrap() != 2); // test that it changed

        new_HashMap.resize(64);

        assert_eq!(new_HashMap.table.read().unwrap().mp.capacity(), 64); //make sure it is correct length

        // try the same assert_eqs
        assert_eq!(new_HashMap.get(20).unwrap(), 5);
        assert_eq!(new_HashMap.get(12).unwrap(), 5);
        assert_eq!(new_HashMap.get(1).unwrap(), 1);
        assert_eq!(new_HashMap.get(0).unwrap(), 0);
        assert!(new_HashMap.get(3).unwrap() != 2); // test that it changed
    }

    #[test]
    fn HashMap_concurr() {
        let mut new_HashMap = Arc::new(HashMap::new()); //init with 16 buckets                                                   // new_HashMap.mp[0].push((1,2));
        let mut threads = vec![];
        let nthreads = 10;
        for _ in 0..nthreads {
            let new_HashMap = new_HashMap.clone();

            threads.push(thread::spawn(move || {
                for _ in 1..1000 {
                    let mut rng = thread_rng();
                    let val = rng.gen_range(0, 256);
                    if val % 2 == 0 {
                        new_HashMap.insert(val, val);
                    } else {
                        let v = new_HashMap.get(val);
                        if (v != None) {
                            assert_eq!(v.unwrap(), val);
                        }
                    }
                    println!("here");
                }
            }));
        }
        for t in threads {
        	println!("here");
            t.join().unwrap();
        }
    }
}
