extern crate concache;
extern crate rand;

use concache::crossbeam::Map;

fn main() {
    let handle = Map::with_capacity(1024);
    handle.insert(1, 3);
    handle.remove(1);
}
