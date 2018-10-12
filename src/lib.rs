//! This crate provides two implementations of fast, concurrent, shared hash maps.
//!
//! Both implementations provide lock-free implementations that use [lock-free linked list
//! buckets](https://www.microsoft.com/en-us/research/wp-content/uploads/2001/10/2001-disc.pdf).
//! Memory is safely destructed and reclaimed using either
//! [`crossbeam::epoch`](https://docs.rs/crossbeam-epoch/) or a manual _Quiescent-State-Based
//! Reclamation_ implementation. See the [`crossbeam`] and [`manual`] module documentations
//! respectively for further details.
//!
//! Table resizing is not yet supported in either implementation, but the map will also never fill
//! due to the linked implementation; instead, performance will decrease as the map is filled with
//! more keys.
//!
//! The crate was written by Aditya Saligrama and Andrew Shen while writing _A practical analysis
//! of Rust’s concurrency story_ as their 2018 project for [MIT
//! PRIMES](https://math.mit.edu/research/highschool/primes/program.php).

#![cfg_attr(feature = "bench", feature(test))]
#![deny(missing_docs)]

extern crate crossbeam as cx;

#[cfg(any(feature = "bench", test))]
extern crate rand;
#[cfg(feature = "bench")]
extern crate test;

pub mod crossbeam;
pub mod manual;
