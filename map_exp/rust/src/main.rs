extern crate rand;
extern crate getopts;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::thread;
use std::env;
use std::time::{SystemTime, Duration};
use rand::{thread_rng, Rng};

use getopts::Options;

const TIME : u64 = 10;

fn handle (cache_map : &Arc<Mutex<HashMap<i32, i32>>>, mode : u32) -> u64 {
    let time = SystemTime::now();
    let end = Duration::new(TIME, 0);

    let mut ops : u64 = 0;
    let mut rng = thread_rng();
    while time.elapsed().unwrap().le(&end) {
        let mut cache_map = cache_map.lock().unwrap();

        if mode == 0 {
            cache_map.get(&rng.gen_range(0, 256));
        } else if mode == 1 {
            let key = rng.gen_range(0, 256);
            let val = rng.gen_range(0, 256);
            cache_map.insert(key, val);
        } else {
            if rng.gen_weighted_bool(2) == true {
                let key = rng.gen_range(0, 256);
                let val = rng.gen_range(0, 256);
                cache_map.insert(key, val);
            } else {
                let key = rng.gen_range(0, 256);
                cache_map.get(&key);
            }
        }
        ops += 1;
    }
    ops
}

fn main () {
    let args : Vec<String> = env::args().collect();
    let mut opts = Options::new();
    opts.optopt("m", "mode", "Benchmark mode (r for read-only, w for write-only, rw for mixed)", "MODE");
    opts.optopt("t", "threads", "Number of threads to run", "THREADS");
    let matches = opts.parse(&args[1..]).unwrap();

    let mode = match matches.opt_str("m").unwrap().as_ref() {
        "r" => 0,
        "w" => 1,
        "rw" => 2,
        _ => panic!()
    };

    let nthreads = matches.opt_str("t").unwrap().parse::<u32>().unwrap();

    let mut cache_map = HashMap::<i32, i32>::new();

    if mode == 0 {
        for i in 0..255 {
            cache_map.insert(i, 1);
        }
    }

    let cache_map = Arc::new(Mutex::new(cache_map));

    let ops = Arc::new(AtomicUsize::new(0));

    let mut threads = vec![];

    for _t in 0..nthreads {
        let cache_map = cache_map.clone();
        let ops = ops.clone();

        threads.push(thread::spawn(move || {
            let top = handle(&cache_map, mode);
            ops.fetch_add((top/TIME) as usize, Ordering::SeqCst);
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    println!("{}", ops.load(Ordering::SeqCst));
}
