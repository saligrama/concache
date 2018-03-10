extern crate num_cpus;
extern crate rand;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use rand::Rng;

fn handle (cache_map : &Arc<Mutex<HashMap<i32, i32>>>, t : usize) {
    loop {
        let mut cache_map = cache_map.lock().unwrap();

        let key = rand::thread_rng().gen_range(0, 256);
        let val = rand::thread_rng().gen_range(0, 256);
        println!("Thread {}: Inserting ({},{})", t, key, val);
        cache_map.insert(key, val);
        let getkey = rand::thread_rng().gen_range(0, 256);
        let mut brk = false;
        let res = match cache_map.get(&getkey) {
            Some(r) => r,
            None => {
                println!("Thread {}: Tried to get value for {} but found no such key", t, getkey);
                brk = true;
                &0
            },
        };
        if !brk {
            println!("Thread {}: Got ({},{})", t, getkey, res);
        }
    }
}

fn main () {
    let cache_map = Arc::new(Mutex::new(HashMap::<i32, i32>::new()));
    let cpuc = num_cpus::get();

    println!("Machine: {} threads", cpuc);

    let mut threads = vec![];

    for t in 1..cpuc {
        let cache_map = cache_map.clone();

        threads.push(thread::spawn(move || {
            handle(&cache_map, t);
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}
