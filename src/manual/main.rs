extern crate concache;
extern crate rand;

use concache::manual::Map;
use rand::{thread_rng, Rng};

fn main() {
    println!("Started.");
    let handle = Map::with_capacity(8);

    for _ in 0..16 {
        let mut rng = thread_rng();
        let val = rng.gen_range(0, 128);
        let key = rng.gen_range(0, 128);
        println!("{:?}", val);
        println!("{:?}", key);
        handle.insert(key, val);
    }
    println!("Finished.");
}
