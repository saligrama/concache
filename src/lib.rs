#![cfg_attr(feature = "bench", feature(test))]

extern crate crossbeam as cx;

#[cfg(any(feature = "bench", test))]
extern crate rand;
#[cfg(feature = "bench")]
extern crate test;

pub mod crossbeam;
pub mod manual;
