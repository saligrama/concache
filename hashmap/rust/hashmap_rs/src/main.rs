#![allow(unused)]
// #[derive(Debug)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Rem;

struct Hashmap {
	nbuckets: usize,
	map: Vec<Vec<(usize,usize)>>,
}

impl Hashmap {
	fn new() -> Self {
		let num_of_buckets: usize = 16;
		let mut new_hashmap = Hashmap {nbuckets: num_of_buckets, map: Vec::with_capacity(num_of_buckets)};
		new_hashmap.map.resize(num_of_buckets, Vec::new());

		new_hashmap
	}

	fn insert (&mut self, key: usize, value: usize) {

		//hasher to hash stuff
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;
		let index = hash % self.nbuckets;


		let ref mut bucket = self.map[index];

		//push the key and value tuple into the map
		for &mut (k,ref mut v) in &mut *bucket {
			if (k == key) {
				*v = value;
				return
			}
		}
		bucket.push((key, value));
	}

	fn get (&self, key: usize) -> Option<usize>  {
		//hash more stuff
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;

		//search for key value and return Some(value), otherwise return None
		for &(k,v) in &self.map[hash % self.nbuckets] {
			if (k == key) {
				return Some(v)
			}
		}
		None

		// self.map[key.rem(self.nbuckets)].iter().find(|&&(k,_)| k == key).map(|&(_,v)|v) //equivalent to the above search function
	}
}

fn main() {

	let mut new_hashmap = Hashmap::new();
	new_hashmap.map[0].push((1,2));

	new_hashmap.insert(1,1);
	new_hashmap.insert(2,5);
	new_hashmap.insert(3,2);
	new_hashmap.insert(3,1);

	println!("result: {:?}", new_hashmap.get(3));

	println!("{:?}", new_hashmap.map);
    println!("Program Done!");
}
