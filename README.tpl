# {{crate}}

[![Crates.io](https://img.shields.io/crates/v/concache.svg)](https://crates.io/crates/concache)
[![Documentation](https://docs.rs/concache/badge.svg)](https://docs.rs/concache/)
[![Build Status](https://travis-ci.org/saligrama/concache.svg?branch=master)](https://travis-ci.org/saligrama/concache)

{{readme}}

## Performance

We've run some benchmarks of concache against a standard Rust `HashMap`
protected by a [reader-writer
lock](https://doc.rust-lang.org/std/sync/struct.RwLock.html), as well as
against [chashmap](https://crates.io/crates/chashmap) — a crate which provides
"concurrent hash maps, based on bucket-level multi-reader locks". The
benchmarks were run using the binary in [benchmark/](benchmark/src/main.rs) on
a 40-core machine with Intel(R) Xeon(R) CPU E5-2660 v3 @ 2.60GHz CPUs.

The benchmark runs a number of reader and writer threads in tight loops, each
of which does a read or write to a random key in the map respectively. Results
for both uniform and skewed distributions are provided below. The benchmark
measures the average number of reads and writes per second as the number of
readers and writers increases.

Preliminary results show that `concache` performs well under contention.

![Read throughput](read-throughput.png)
![Write throughput](write-throughput.png)
