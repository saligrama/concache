#![allow(unused)]
// #[derive(Debug)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{RwLock};
use std::ops::Rem;

struct Hashmap {
	nbuckets: usize,
	map: Vec<RwLock<Vec<(usize,usize)>>>,
	nitems: usize,
}

impl Hashmap {
	fn new(num_of_buckets: usize) -> Self {
		let mut new_hashmap = Hashmap {nbuckets: num_of_buckets, map: Vec::with_capacity(num_of_buckets), nitems: 0};
		// new_hashmap.map.resize(num_of_buckets, Vec::new());
		for _ in 0..num_of_buckets {
			let new_rwlock_vec = RwLock::new(Vec::new());
			new_hashmap.map.push(new_rwlock_vec);
		}
		
		new_hashmap
	}

	fn insert (&mut self, key: usize, value: usize) {
		// check for resize
		if (self.nitems / self.nbuckets >= 2) { //threshold is 2
			let resize_value: usize = self.nbuckets * 2;
			self.resize(resize_value); //double the size
		}

		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;
		let index = hash % self.nbuckets;

		// println!("index: {}", index);
		let mut w = self.map[index].write().unwrap(); //give write access
		let ref mut bucket = *w;

		//push the key and value tuple into the map
		for &mut (k,ref mut v) in &mut *bucket {
			if (k == key) {
				*v = value;
				self.nitems += 1;
				return
			}
		}
		self.nitems += 1;
		bucket.push((key, value));
	}

	fn get (&self, key: usize) -> Option<usize>  {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;
		let index = hash % self.nbuckets;

		let mut r = self.map[index].read().unwrap();
		//search for key value and return Some(value), otherwise return None
		for &(k,v) in r.iter() {
			if (k == key) {
				return Some(v)
			}
		}
		None

		// self.map[key.rem(self.nbuckets)].iter().find(|&&(k,_)| k == key).map(|&(_,v)|v) //equivalent to the above search function
	}

	fn resize (&mut self, newsize: usize) {
		println!("resize: {}", newsize);
		let mut new_hashmap = Hashmap::new(newsize);

		for ref bucket in &self.map {
			let mut w = bucket.write().unwrap(); //give write access

			for &(k, v) in &mut w.iter() {
				new_hashmap.insert(k, v);
			}
		}
		self.map = new_hashmap.map;
		self.nbuckets = new_hashmap.nbuckets;
		self.nitems = new_hashmap.nitems;
	}
}

fn main() {
	println!("Program Start!");
	let mut new_hashmap = Hashmap::new(16); //init with 16 buckets
	// new_hashmap.map[0].push((1,2));

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

	new_hashmap.resize(64);

	println!("After Resize {:?}", new_hashmap.map);
    println!("Program Done!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]

    fn hashmap_works () {
		let mut new_hashmap = Hashmap::new(2); //init with 16 buckets
		// new_hashmap.map[0].push((1,2)); //manually push

		//input values
		new_hashmap.insert(1,1);
		new_hashmap.insert(2,5);
		new_hashmap.insert(12,5);
		new_hashmap.insert(13,7);
		new_hashmap.insert(0,0);

		assert_eq!(new_hashmap.map.capacity(), 4); //should be 4 after you attempt the 5th insert

		new_hashmap.insert(20,3);
		new_hashmap.insert(3,2);
		new_hashmap.insert(3,1);
		new_hashmap.insert(20,5);

		assert_eq!(new_hashmap.map.capacity(), 8); //should be 8 after you attempt the 9th insert		

		assert_eq!(new_hashmap.get(20).unwrap(), 5);
		assert_eq!(new_hashmap.get(12).unwrap(), 5);
		assert_eq!(new_hashmap.get(1).unwrap(), 1);
		assert_eq!(new_hashmap.get(0).unwrap(), 0);
		assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed

		new_hashmap.resize(64);

		assert_eq!(new_hashmap.map.capacity(), 64); //make sure it is correct length

		//try the same assert_eqs
		assert_eq!(new_hashmap.get(20).unwrap(), 5);
		assert_eq!(new_hashmap.get(12).unwrap(), 5);
		assert_eq!(new_hashmap.get(1).unwrap(), 1);
		assert_eq!(new_hashmap.get(0).unwrap(), 0);
		assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed
    }

}
