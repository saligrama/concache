#![cfg_attr(feature = "bench", feature(test))]

extern crate crossbeam as cx;

#[cfg(feature = "bench")]
extern crate rand;
#[cfg(feature = "bench")]
extern crate test;

pub mod crossbeam;
pub mod manual;
