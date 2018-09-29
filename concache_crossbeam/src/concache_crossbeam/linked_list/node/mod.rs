use std::sync::{Mutex, atomic::AtomicBool};
use crossbeam::epoch::Atomic;

pub (super) struct Node {
    pub (super) kv: (usize, Mutex<usize>),
    pub (super) active: AtomicBool,
    pub (super) next: Atomic<Node>,
    pub (super) prev: Atomic<Node>
}

impl Node {
    pub fn new (k : usize, v : usize) -> Self {
        Node {
            kv: (k, Mutex::new(v)),
            active: AtomicBool::new(true),
            next: Atomic::null(),
            prev: Atomic::null()
        }
    }
}
