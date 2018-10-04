#![feature(test)]

extern crate concache_crossbeam;
extern crate rand;
extern crate test;

use concache_crossbeam::concache_crossbeam::ConcacheCrossbeam;
use rand::{thread_rng, Rng};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use test::Bencher;

const OSC: Ordering = Ordering::SeqCst;

fn main() {
    let handle = ConcacheCrossbeam::with_capacity(1024);
    handle.insert(1, 3);
    handle.remove(1);
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn HashMap_concurr() {
//         let handle = ConcacheCrossbeam::with_capacity(8); //changed this,
//         let mut threads = vec![];
//         let nthreads = 5;
//         for _ in 0..nthreads {
//             let new_handle = handle.clone();

//             threads.push(thread::spawn(move || {
//                 let num_iterations = 100000;
//                 for _ in 0..num_iterations {
//                     let mut rng = thread_rng();
//                     let val = rng.gen_range(0, 128);
//                     let two = rng.gen_range(0, 3);

//                     if two % 3 == 0 {
//                         new_handle.insert(val, val);
//                     } else if two % 3 == 1 {
//                         let v = new_handle.get(val);
//                         if v.is_some() {
//                             assert_eq!(v.unwrap(), val);
//                         }
//                     } else {
//                         new_handle.remove(val);
//                     }
//                 }
//             }));
//         }
//         for t in threads {
//             t.join().unwrap();
//         }
//     }

//     #[test]
//     fn HashMap_handle_cloning() {
//         let mut handle = ConcacheCrossbeam::with_capacity(8);
//         assert_eq!(handle.get(1).unwrap(), 3);

//         //create a new handle
//         let new_handle = Arc::clone(&handle);
//         assert_eq!(new_handle.get(1).unwrap(), 3);
//         new_handle.insert(2, 5);

//         assert_eq!(handle.get(2).unwrap(), 5);
//     }

//     #[test]
//     fn HashMap_remove() {
//         let mut handle = ConcacheCrossbeam::with_capacity(8);
//         handle.insert(1, 3);
//         handle.insert(2, 5);
//         handle.insert(3, 8);
//         handle.insert(4, 3);
//         handle.insert(5, 4);
//         handle.insert(6, 5);
//         handle.insert(7, 3);
//         handle.insert(8, 3);
//         handle.insert(9, 3);
//         handle.insert(10, 3);
//         handle.insert(11, 3);
//         handle.insert(12, 3);
//         handle.insert(13, 3);
//         handle.insert(14, 3);
//         handle.insert(15, 3);
//         handle.insert(16, 3);
//         assert_eq!(handle.get(1).unwrap(), 3);
//         assert_eq!(handle.remove(1).unwrap(), 3);
//         assert_eq!(handle.get(1), None);
//         assert_eq!(handle.remove(2).unwrap(), 5);
//         assert_eq!(handle.remove(16).unwrap(), 3);
//         assert_eq!(handle.get(16), None);
//     }

//     #[test]
//     fn HashMap_basics() {
//         let mut new_HashMap = ConcacheCrossbeam::with_capacity(8); //init with 2 buckets
//                                               //input values
//         new_HashMap.insert(1, 1);
//         new_HashMap.insert(2, 5);
//         new_HashMap.insert(12, 5);
//         new_HashMap.insert(13, 7);
//         new_HashMap.insert(0, 0);

//         new_HashMap.insert(20, 3);
//         new_HashMap.insert(3, 2);
//         new_HashMap.insert(4, 1);

//         assert_eq!(new_HashMap.insert(20, 5).unwrap(), 3); //repeated
//         assert_eq!(new_HashMap.insert(3, 8).unwrap(), 2); //repeated
//         assert_eq!(new_HashMap.insert(5, 5), None); //repeated

//         new_HashMap.insert(3, 8); //repeated

//         assert_eq!(new_HashMap.get(20).unwrap(), 5);
//         assert_eq!(new_HashMap.get(12).unwrap(), 5);
//         assert_eq!(new_HashMap.get(1).unwrap(), 1);
//         assert_eq!(new_HashMap.get(0).unwrap(), 0);
//         assert!(new_HashMap.get(3).unwrap() != 2); // test that it changed

//         // try the same assert_eqs
//         assert_eq!(new_HashMap.get(20).unwrap(), 5);
//         assert_eq!(new_HashMap.get(12).unwrap(), 5);
//         assert_eq!(new_HashMap.get(1).unwrap(), 1);
//         assert_eq!(new_HashMap.get(0).unwrap(), 0);
//         assert!(new_HashMap.get(3).unwrap() != 2); // test that it changed
//     }
// }

//BENCHMARKS
#[inline]
fn getn(b: &mut Bencher, n: usize) {
    let handle = ConcacheCrossbeam::with_capacity(1024);
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
    let handle = ConcacheCrossbeam::with_capacity(1024);
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
    let handle = ConcacheCrossbeam::with_capacity(1024);
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
    let handle = ConcacheCrossbeam::with_capacity(1024);

    b.iter(|| {
        handle.insert(1, 0);
        handle.remove(1);
    })
}
