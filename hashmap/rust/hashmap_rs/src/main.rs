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
	fn new(num_of_buckets: usize) -> Self {
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

	fn resize (&mut self, newsize: usize) -> Self {
		println!("resize: {}", newsize);
		let mut new_hashmap = Hashmap::new(newsize);
		for ref mut bucket in &self.map {
			for &(k, v) in &mut bucket.iter() {
				new_hashmap.insert(k, v);
			}
		}
		new_hashmap
	}
}

fn main() {
	println!("Program Start!");
	let mut new_hashmap = Hashmap::new(16); //init with 16 buckets
	new_hashmap.map[0].push((1,2));

	new_hashmap.insert(1,1);
	new_hashmap.insert(2,5);
	new_hashmap.insert(12,5);
	new_hashmap.insert(13,7);
	new_hashmap.insert(0,0);
	new_hashmap.insert(20,3);
	new_hashmap.insert(3,2);
	new_hashmap.insert(3,1);
	new_hashmap.insert(20,5);

	println!("Before Resize {:?}", new_hashmap.map);

	let mut new_hashmap = new_hashmap.resize(64);

	println!("After Resize {:?}", new_hashmap.map);
    println!("Program Done!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn hashmap_works () {
		let mut new_hashmap = Hashmap::new(16); //init with 16 buckets
		// new_hashmap.map[0].push((1,2)); //manually push

		//input values
		new_hashmap.insert(1,1);
		new_hashmap.insert(2,5);
		new_hashmap.insert(12,5);
		new_hashmap.insert(13,7);
		new_hashmap.insert(0,0);
		new_hashmap.insert(20,3);
		new_hashmap.insert(3,2);
		new_hashmap.insert(3,1);
		new_hashmap.insert(20,5);

		assert!(new_hashmap.get(20).unwrap() == 5);
		assert!(new_hashmap.get(12).unwrap() == 5);
		assert!(new_hashmap.get(1).unwrap() == 1);
		assert!(new_hashmap.get(0).unwrap() == 0);
		assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed

		let mut new_hashmap = new_hashmap.resize(64);

		assert!(new_hashmap.map.len() == 64); //make sure it is correct length

		//try the same asserts
		assert!(new_hashmap.get(20).unwrap() == 5);
		assert!(new_hashmap.get(12).unwrap() == 5);
		assert!(new_hashmap.get(1).unwrap() == 1);
		assert!(new_hashmap.get(0).unwrap() == 0);
		assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed
    }

}
