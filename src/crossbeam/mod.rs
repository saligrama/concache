mod linked_list;

use self::linked_list::LinkedList;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::Hasher;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone)]
pub struct Map {
    bsize: usize,
    size: Arc<AtomicUsize>,
    mp: Arc<Vec<LinkedList>>,
}

impl Map {
    pub fn with_capacity(nbuckets: usize) -> Self {
        let mut v = Vec::with_capacity(nbuckets);

        for _i in 0..nbuckets {
            v.push(LinkedList::new());
        }

        Map {
            bsize: nbuckets,
            size: Arc::new(AtomicUsize::new(0)),
            mp: Arc::new(v),
        }
    }

    pub fn size(&self) -> usize {
        self.size.load(Ordering::SeqCst)
    }

    pub fn insert(&self, key: usize, value: usize) -> bool {
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

    pub fn get(&self, key: usize) -> Option<usize> {
        let mut hsh = DefaultHasher::new();
        hsh.write_usize(key);
        let h = hsh.finish() as usize;

        let ndx = h % self.bsize;

        self.mp[ndx].get(key)
    }

    pub fn remove(&self, key: usize) -> bool {
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

impl fmt::Debug for Map {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut all = String::new();
        for i in 0..self.bsize {
            // TODO: I'm _sure_ there's a better way to do this
            all.push_str(&format!("{:?}", &self.mp[i]));
        }
        let ret: String = all.chars().skip(0).take(all.len() - 2).collect();
        write!(f, "[{}]", ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
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
