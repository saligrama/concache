#![feature(integer_atomics)]

use std::sync::{Mutex, RwLock, atomic::*};
use std::ptr;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::mem;

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
    mp: RwLock<Vec<LinkedList>>
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
            mp: RwLock::new(v)
        };

        ret
    }

    fn resize (&mut self, nbuckets : usize) {
        let new = Table::new(nbuckets);
        let mut v = (&self).mp.write().unwrap();
        let bsize = self.bsize.load(Ordering::SeqCst);
        for i in 0..bsize {
            let ll = &v[i];

            let mut ptr_cur = &ll.first;
            let mut ptr_raw = ptr_cur.load(Ordering::SeqCst);

            while !ptr_raw.is_null() {
                let mut node_cur = unsafe { &*ptr_raw };

                new.insert(node_cur.kv.0, *node_cur.kv.1.lock().unwrap());

                ptr_cur = &node_cur.next;
                ptr_raw = ptr_cur.load(Ordering::SeqCst);
            }
        }

        mem::replace(&mut v, (&new).mp.write().unwrap());
        self.bsize.compare_and_swap(bsize, nbuckets, Ordering::SeqCst);
    }

    fn insert (&self, key : usize, value : usize) {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize.load(Ordering::SeqCst);

        &self.mp.read().unwrap()[ndx].insert((key, value));
        self.size.fetch_add(1, Ordering::SeqCst);
    }

    fn get (&self, key : usize) -> Option<usize> {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize.load(Ordering::SeqCst);

        (&self).mp.read().unwrap()[ndx].get(key)
    }
}
