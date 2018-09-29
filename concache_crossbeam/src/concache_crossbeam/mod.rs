pub mod linked_list;

use self::linked_list::LinkedList;

use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::fmt;

pub struct ConcacheCrossbeam {
    bsize: usize,
    size: Arc<AtomicUsize>,
    mp: Arc<Vec<LinkedList>>
}

impl ConcacheCrossbeam {
    pub fn with_capacity (nbuckets : usize) -> Self {
        let mut v = Vec::with_capacity(nbuckets);

        for _i in 0..nbuckets {
            v.push(LinkedList::new());
        }

        let ret = ConcacheCrossbeam {
            bsize: nbuckets,
            size: Arc::new(AtomicUsize::new(0)),
            mp: Arc::new(v)
        };

        ret
    }

    pub fn size (&self) -> usize {
        return self.size.load(Ordering::SeqCst);
    }

    pub fn insert (&self, key : usize, value : usize) -> bool {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;
        if self.mp[ndx].insert((key, value)) {
            self.size.fetch_add(1, Ordering::SeqCst);
            return true;
        }
        false
    }

    pub fn get (&self, key : usize) -> Option<usize> {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        self.mp[ndx].get(key)
    }

    pub fn remove (&self, key : usize) -> bool {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        if self.mp[ndx].remove(key) {
            self.size.fetch_sub(1, Ordering::SeqCst);
            return true;
        }
        false
    }
}

impl Clone for ConcacheCrossbeam {
    fn clone (&self) -> Self {
        Self {
            bsize: self.bsize,
            size: Arc::clone(&self.size),
            mp: Arc::clone(&self.mp)
        }
    }
}

impl fmt::Display for ConcacheCrossbeam {
    fn fmt (&self, f : &mut fmt::Formatter) -> fmt::Result {
        let mut all = String::new();
        for i in 0..self.bsize {
            all.push_str(&(&self).mp[i].to_string());
        }
        let ret : String = all.chars().skip(0).take(all.len() - 2).collect();
        write!(f, "[{}]", ret)
    }
}
