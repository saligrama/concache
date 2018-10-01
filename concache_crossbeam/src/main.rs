#![feature(test)]

extern crate concache_crossbeam;
extern crate test;
extern crate rand;

use rand::{thread_rng, Rng};
use concache_crossbeam::concache_crossbeam::ConcacheCrossbeam;
use test::Bencher;

fn main() {
    let handle = ConcacheCrossbeam::with_capacity(1024);
    handle.insert(1,3);
    handle.remove(1);
}

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