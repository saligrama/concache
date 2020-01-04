use cx::epoch::{self, Atomic, Owned};
use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

struct Node<K, V> {
    kv: (K, Atomic<V>),
    active: AtomicBool,
    next: Atomic<Node<K, V>>,
    prev: Atomic<Node<K, V>>,
}

impl<K, V> Node<K, V> {
    fn new(k: K, v: V) -> Self {
        Node {
            kv: (k, Atomic::new(v)),
            active: AtomicBool::new(true),
            next: Atomic::null(),
            prev: Atomic::null(),
        }
    }
}

pub(super) struct LinkedList<K, V> {
    first: Atomic<Node<K, V>>,
}

impl<K, V> Default for LinkedList<K, V> {
    fn default() -> Self {
        LinkedList {
            first: Atomic::null(),
        }
    }
}

impl<K, V> LinkedList<K, V>
where
    K: Eq,
    V: Copy,
{
    pub(super) fn insert(&self, kv: (K, V)) -> Option<*mut V> {
        let guard = epoch::pin();

        let mut node = &self.first;
        loop {
            let l = node.load(Ordering::SeqCst, &guard);
            match l {
                Some(k) => {
                    let raw = k.as_raw();
                    let cur = unsafe { &*raw };
                    if &cur.kv.0 == &kv.0 && cur.active.load(Ordering::SeqCst) {
                        // if let Some(old) = cur.kv.1.load(Ordering::SeqCst, &guard) {
                        //     unsafe { guard.unlinked(old); }
                        // }
                        let ins = Owned::new(kv.1);
                        let old = cur.kv.1.load(Ordering::SeqCst, &guard);
                        cur.kv.1.cas_and_ref(old, ins, Ordering::SeqCst, &guard);
                        return Some(old.unwrap().as_raw());
                    }
                    node = &k.next;

                    // key does not exist
                    if cur.next.load(Ordering::SeqCst, &guard).is_none() {
                        let ins = Owned::new(Node::new(kv.0, kv.1));
                        ins.prev.store_shared(l, Ordering::SeqCst);
                        cur.next.store_and_ref(ins, Ordering::SeqCst, &guard);
                        return None;
                    }
                }
                None => {
                    // first is null
                    let ins = Owned::new(Node::new(kv.0, kv.1));
                    self.first.store_and_ref(ins, Ordering::SeqCst, &guard);
                    return None;
                }
            };
        }
    }

    pub(super) fn get(&self, key: &K) -> Option<V> {
        let guard = epoch::pin();

        let mut node = &self.first;
        loop {
            match node.load(Ordering::SeqCst, &guard) {
                Some(k) => {
                    let raw = k.as_raw();
                    let cur = unsafe { &*raw };
                    if &cur.kv.0 == key && cur.active.load(Ordering::SeqCst) {
                        let value = cur.kv.1.load(Ordering::SeqCst, &guard).unwrap();
                        return Some(**value);
                    }
                    node = &k.next;
                }
                None => {
                    return None;
                }
            };
        }
    }

    pub(super) fn remove(&self, key: &K) -> bool {
        let guard = epoch::pin();

        let mut node = &self.first;
        loop {
            match node.load(Ordering::SeqCst, &guard) {
                Some(k) => {
                    let raw = k.as_raw();
                    let cur = unsafe { &*raw };
                    if &cur.kv.0 == key && cur.active.load(Ordering::SeqCst) {
                        cur.active.store(false, Ordering::SeqCst);

                        let next = k.next.load(Ordering::SeqCst, &guard);
                        let prev = k.prev.load(Ordering::SeqCst, &guard);

                        if next.is_some() {
                            if prev.is_some() {
                                let n = next.unwrap();
                                let p = prev.unwrap();

                                if !p.next.cas_shared(Some(k), next, Ordering::SeqCst) {
                                    return false;
                                }
                                if !n.prev.cas_shared(Some(k), next, Ordering::SeqCst) {
                                    return false;
                                }
                            } else {
                                let n = next.unwrap();
                                if !n.prev.cas_shared(Some(k), None, Ordering::SeqCst) {
                                    return false;
                                }
                                if !self.first.cas_shared(Some(k), next, Ordering::SeqCst) {
                                    return false;
                                }
                            }
                        } else {
                            if prev.is_some() {
                                let p = prev.unwrap();

                                if !p.next.cas_shared(Some(k), None, Ordering::SeqCst) {
                                    return false;
                                }
                            } else {
                                if !self.first.cas_shared(Some(k), next, Ordering::SeqCst) {
                                    return false;
                                }
                            }
                        }

                        unsafe { guard.unlinked(k) };
                        return true;
                    }
                    node = &k.next;
                }
                None => {
                    // the node with key key didn't exist
                    return false;
                }
            };
        }
    }
}

impl<K, V> fmt::Debug for LinkedList<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: use https://doc.rust-lang.org/std/fmt/struct.DebugList.html
        let guard = epoch::pin();

        let mut ret = String::new();
        let mut node = &self.first;
        while let Some(k) = node.load(Ordering::SeqCst, &guard) {
            let raw = k.as_raw();
            let cur = unsafe { &*raw };
            if cur.active.load(Ordering::SeqCst) {
                let key = &cur.kv.0;
                let value = cur.kv.1.load(Ordering::SeqCst, &guard).unwrap();

                ret.push_str("(");
                ret.push_str(&format!("{:?}", key));
                ret.push_str(", ");
                ret.push_str(&format!("{:?}", value));
                ret.push_str("), ");
            }
            node = &k.next;
        }

        write!(f, "{}", ret)
    }
}
