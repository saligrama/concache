// TODO look into adding a logger (envlogger)
// TODO delete concurrent memory reclamation
// TODO try crossbeam

#![feature(integer_atomics)]

extern crate rand;
extern crate crossbeam;

use std::sync::{Mutex, RwLock, atomic::*, Arc};
use std::ptr;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::thread;
use std::fmt;
use rand::thread_rng;

use crossbeam::epoch::{self, Atomic, Owned};

const AVG_PER_BIN_THRESH : usize = 4;

struct Node {
    kv: (usize, Mutex<usize>),
    active: AtomicBool,
    next: Atomic<Node>,
    prev: Atomic<Node>
}

struct LinkedList {
    size: AtomicUsize,
    first: Atomic<Node>,
}

struct Table {
    bsize: usize,
    mp: Vec<LinkedList>
}

struct HashMap {
    bsize: usize,
    size: AtomicUsize,
    table: RwLock<Table>
}

impl Node {
    fn new (k : usize, v : usize) -> Self {
        Node {
            kv: (k, Mutex::new(v)),
            active: AtomicBool::new(true),
            next: Atomic::null(),
            prev: Atomic::null()
        }
    }
}

impl LinkedList {
    fn new () -> Self {
        LinkedList {
            size: AtomicUsize::new(0),
            first: Atomic::null(),
        }
    }

    fn insert (&self, kv : (usize, usize)) -> bool {
        let guard = epoch::pin();

        // if self.first.load(Ordering::SeqCst, &guard).is_null() {
        //     // nonexistent first node
        //     self.first.compare_and_swap(ptr::null_mut(), Box::into_raw(ins), Ordering::SeqCst);
        // } else {
        //     let mut not_mutated = true;
        //     let p = Box::into_raw(ins);
        //
        //     while not_mutated {
        //         let mut node_cur : &Node;
        //         let mut ptr_cur = &self.first;
        //         let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //
        //         while !ptr_raw.is_null() {
        //             node_cur = unsafe { &*ptr_raw };
        //             ptr_cur = &node_cur.next;
        //             ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //
        //             // found same key
        //             if node_cur.kv.0 == kv.0 && node_cur.active.load(Ordering::SeqCst) == true {
        //                 let mut change = node_cur.kv.1.lock().unwrap();
        //                 *change = kv.1;
        //                 return false;
        //             }
        //         }
        //
        //         let ret = ptr_cur.compare_and_swap(ptr::null_mut(), p, Ordering::SeqCst);
        //         if ret == ptr::null_mut() {
        //             not_mutated = false;
        //         }
        //     }

        let mut node = &self.first;
        loop {
            match node.load(Ordering::Relaxed, &guard) {
                Some(k) => {
                    let mut raw = k.as_raw();
                    let mut cur = unsafe { &*raw };
                    if cur.kv.0 == kv.0 && cur.active.load(Ordering::Relaxed) {
                        let mut change = cur.kv.1.lock().unwrap();
                        *change = kv.1;
                        return false;
                    }
                    node = &k.next;
                },
                None => {
                    break;
                }
            };
        }

        // key does not exist
        let mut ins = Owned::new(Node::new(kv.0, kv.1));
        loop {
            let first = self.first.load(Ordering::Relaxed, &guard);
            ins.next.store_shared(first, Ordering::Relaxed);

            match self.first.cas_and_ref(first, ins, Ordering::Release, &guard) {
                Ok(_) => break,
                Err(owned) => ins = owned
            }
        }

        // update the prev reference of first.next to reform the doubly-linked list
        let first = self.first.load(Ordering::Relaxed, &guard);
        let k = first.unwrap().as_raw();
        let k_raw = unsafe { &*k };
        match k_raw.next.load(Ordering::Relaxed, &guard) {
            Some(next) => {
                let next_raw = unsafe { &*next.as_raw() };
                next_raw.prev.store_shared(first, Ordering::Relaxed);
            },
            None => {}
        }

        return true;
    }

    fn get (&self, key : usize) -> Option<usize> {
        // if !self.first.load(Ordering::SeqCst).is_null() {
        //
        //     let mut node_cur : &Node;
        //     let mut ptr_cur = &self.first;
        //     let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //
        //     while !ptr_raw.is_null() {
        //         node_cur = unsafe { &*ptr_raw };
        //         if node_cur.kv.0 == key {
        //             let active = node_cur.active.load(Ordering::SeqCst);
        //             if !active {
        //                 return None;
        //             }
        //             let value = node_cur.kv.1.lock().unwrap();
        //             return Some(*value);
        //         }
        //
        //         ptr_cur = &node_cur.next;
        //         ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //     }
        // }
        // None

        let guard = epoch::pin();
        let mut node = &self.first;
        loop {
            match node.load(Ordering::Relaxed, &guard) {
                Some(k) => {
                    let mut raw = k.as_raw();
                    let mut cur = unsafe { &*raw };
                    if cur.kv.0 == key && cur.active.load(Ordering::Relaxed) {
                        let value = cur.kv.1.lock().unwrap();
                        return Some(*value);
                    }
                    node = &k.next;
                },
                None => {
                    return None;
                }
            };
        }

    }

    fn remove (&self, key : usize) -> bool {
        // if !self.first.load(Ordering::SeqCst).is_null() {
        //
        //     let mut node_cur : &Node;
        //     let mut ptr_cur = &self.first;
        //     let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //
        //     while !ptr_raw.is_null() {
        //         node_cur = unsafe { &*ptr_raw };
        //         if node_cur.kv.0 == key {
        //             if node_cur.active.store(false, Ordering::SeqCst) {
        //                 return true;
        //             }
        //             return false;
        //         }
        //
        //         ptr_cur = &node_cur.next;
        //         ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //     }
        // }
        // false
        let guard = epoch::pin();
        let mut node = &self.first;
        loop {
            match node.load(Ordering::Relaxed, &guard) {
                Some(k) => {
                    let mut raw = k.as_raw();
                    let mut cur = unsafe { &*raw };
                    if cur.kv.0 == key && cur.active.load(Ordering::Relaxed) {
                        cur.active.store(false, Ordering::SeqCst);

                        let next = k.next.load(Ordering::Relaxed, &guard);
                        let prev = k.prev.load(Ordering::Relaxed, &guard);

                        node.cas_shared(Some(k), next, Ordering::Release);

                        let mut new_node = node.load(Ordering::Relaxed, &guard).unwrap();
                        let mut new_node_raw_cur = unsafe { &*new_node.as_raw() };

                        if new_node_raw_cur.prev.cas_shared(Some(k), prev, Ordering::Release) {
                            unsafe { guard.unlinked(k) };
                            return true;
                        }
                    }
                    node = &k.next;
                },
                None => {
                    // the node with key key didn't exist
                    return false;
                }
            };
        }
    }
}

impl fmt::Display for LinkedList {
    fn fmt (&self, f : &mut fmt::Formatter) -> fmt::Result {
        // let mut ret = String::new();
        // if !self.first.load(Ordering::Relaxed, &guard).is_null() {
        //
        //     let mut node_cur : &Node;
        //     let mut ptr_cur = &self.first;
        //     let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //
        //     while !ptr_raw.is_null() {
        //         node_cur = unsafe { &*ptr_raw };
        //         let active = node_cur.active.load(Ordering::SeqCst);
        //         // let active = true;
        //         if active {
        //             let key = node_cur.kv.0;
        //             println!("Taking lock for value");
        //             let value = node_cur.kv.1.lock().unwrap();
        //             println!("Took lock for value");
        //
        //             ret.push_str("(");
        //             ret.push_str(&key.to_string());
        //             ret.push_str(", ");
        //             ret.push_str(&value.to_string());
        //             ret.push_str("), ");
        //
        //             println!("Releasing lock for value");
        //         }
        //         ptr_cur = &node_cur.next;
        //         ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //     }
        // }
        //
        // write!(f, "{}", ret)

        let guard = epoch::pin();
        let mut ret = String::new();
        let mut node = &self.first;
        loop {
            match node.load(Ordering::Relaxed, &guard) {
                Some(k) => {
                    let mut raw = k.as_raw();
                    let mut cur = unsafe { &*raw };
                    if cur.active.load(Ordering::Relaxed) {
                        let key = cur.kv.0;
                        println!("Taking lock for value");
                        let value = cur.kv.1.lock().unwrap();
                        println!("Took lock for value");

                        ret.push_str("(");
                        ret.push_str(&key.to_string());
                        ret.push_str(", ");
                        ret.push_str(&value.to_string());
                        ret.push_str("), ");

                        println!("Releasing lock for value");
                    }
                    node = &k.next;
                },
                None => {
                    break;
                }
            };
        }

        write!(f, "{}", ret)
    }
}

impl Table {
    fn new (nbuckets : usize) -> Self {
        let mut v = Vec::with_capacity(nbuckets);

        for _i in 0..nbuckets {
            v.push(LinkedList::new());
        }

        let ret = Table {
            bsize: nbuckets,
            mp: v
        };

        ret
    }

    fn resize (&mut self, nbuckets : usize) {
        // let guard = epoch::pin;
        // let new = Table::new(nbuckets);
        // for i in 0..self.bsize {
        //     let ll = &self.mp[i];
        //
        //     let mut ptr_cur = &ll.first;
        //     let mut ptr_raw = ptr_cur.load(Ordering::Relaxed, &guard);
        //
        //     while !ptr_raw.is_null() {
        //         let mut node_cur = unsafe { &*ptr_raw };
        //
        //         new.insert(node_cur.kv.0, *node_cur.kv.1.lock().unwrap());
        //
        //         ptr_cur = &node_cur.next;
        //         ptr_raw = ptr_cur.load(Ordering::SeqCst);
        //     }
        // }
        //
        // self.mp = new.mp;
        // self.bsize = nbuckets;

        let guard = epoch::pin();

        let new = Table::new(nbuckets);
        for i in 0..self.bsize {

            let ll = &self.mp[i];
            let mut node = &ll.first;
            loop {
                match node.load(Ordering::Relaxed, &guard) {
                    Some(k) => {
                        let mut raw = k.as_raw();
                        let mut cur = unsafe { &*raw };
                        if cur.active.load(Ordering::Relaxed) {
                            new.insert(cur.kv.0, *cur.kv.1.lock().unwrap());
                        }
                        node = &k.next;
                    },
                    None => {
                        break;
                    }
                };
            }
        }
    }

    fn insert (&self, key : usize, value : usize) -> bool {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        self.mp[ndx].insert((key, value))
    }

    fn get (&self, key : usize) -> Option<usize> {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        self.mp[ndx].get(key)
    }

    fn remove (&self, key : usize) -> bool {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        self.mp[ndx].remove(key)
    }
}

impl fmt::Display for Table {
    fn fmt (&self, f : &mut fmt::Formatter) -> fmt::Result {
        let mut all = String::new();
        for i in 0..self.bsize {
            all.push_str(&(&self).mp[i].to_string());
        }
        let ret : String = all.chars().skip(0).take(all.len() - 2).collect();
        write!(f, "[{}]", ret)
    }
}

impl HashMap {
    fn new () -> Self {
        HashMap {
            bsize: 1,
            size: AtomicUsize::new(0),
            table: RwLock::new(Table::new(1))
        }
    }

    fn insert (&self, key : usize, val : usize) {
        let size = self.size.load(Ordering::SeqCst);
        if size / self.bsize > AVG_PER_BIN_THRESH {
            self.resize();
        }

        let t = self.table.read().unwrap();
        if t.insert(key, val) {
            self.size.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn get (&self, key : usize) -> Option<usize> {
        println!("Taking read lock for get");
        let t = (&self).table.read().unwrap();
        println!("Took read lock for get");
        let ret = t.get(key);
        println!("Releasing read lock for get");
        ret
    }

    fn remove (&self, key : usize) {
        println!("Taking read lock for remove");
        let t = (&self).table.read().unwrap();
        println!("Took read lock for remove");
        if t.remove(key) {
            self.size.fetch_sub(1, Ordering::SeqCst);
        }
        println!("Releasing read lock for remove");
    }

    fn resize (&self) {
        // TODO make sure we don't over-resize
        println!("Taking write lock");
        let mut t = (&self).table.write().unwrap();
        println!("Took write lock");
        t.resize(self.bsize * 2);
        println!("Releasing write lock");
    }

    fn size (&self) -> usize {
        self.size.load(Ordering::SeqCst)
    }
}

impl fmt::Display for HashMap {
    fn fmt (&self, f : &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", (&self).table.read().unwrap().to_string())
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
    new_HashMap.insert(12, 7);
    new_HashMap.insert(0, 0);

    println!("testing for 4");
    println!("{}", new_HashMap.to_string());
    assert_eq!(new_HashMap.size(), 4); //should be 4 after you attempt the 5th insert

    new_HashMap.insert(20, 3);
    new_HashMap.insert(3, 2);
    new_HashMap.insert(4, 1);
    new_HashMap.insert(5, 5);

    new_HashMap.insert(20, 5); //repeated
    new_HashMap.insert(3, 8); //repeated
    println!("testing for 8");
    assert_eq!(new_HashMap.size(), 8);

    new_HashMap.remove(20);
    println!("{} {}", new_HashMap.to_string(), new_HashMap.size());

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

        assert_eq!(new_HashMap.size(), 4); //should be 4 after you attempt the 5th insert

        new_HashMap.insert(20, 3);
        new_HashMap.insert(3, 2);
        new_HashMap.insert(4, 1);
        new_HashMap.insert(5, 5);

        new_HashMap.insert(20, 5); //repeated
        new_HashMap.insert(3, 8); //repeated
        assert_eq!(new_HashMap.size(), 8); //should be 8 after you attempt the 9th insert

        assert_eq!(new_HashMap.get(20).unwrap(), 5);
        assert_eq!(new_HashMap.get(12).unwrap(), 5);
        assert_eq!(new_HashMap.get(1).unwrap(), 1);
        assert_eq!(new_HashMap.get(0).unwrap(), 0);
        assert!(new_HashMap.get(3).unwrap() != 2); // test that it changed

        new_HashMap.resize();

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
