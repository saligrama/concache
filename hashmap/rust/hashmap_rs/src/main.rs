#![allow(unused)]
// #[derive(Debug)]

extern crate rand;

use std::thread;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};	
use rand::{thread_rng, Rng};

struct Table {
	nbuckets: usize,
	map: Vec<RwLock<Vec<(usize,usize)>>>,
	nitems: AtomicUsize,
}

struct Hashmap {
	table: RwLock<Table>
}

impl Hashmap {
	fn new(num_of_buckets: usize) -> Self {
		let mut new_table = Table {nbuckets: num_of_buckets, map: Vec::with_capacity(num_of_buckets), nitems: AtomicUsize::new(0)};
		// new_hashmap.map.resize(num_of_buckets, Vec::new());
		for _ in 0..num_of_buckets {
			let new_rwlock_vec = RwLock::new(Vec::new());
			new_table.map.push(new_rwlock_vec);
		}

		let new_hashmap = Hashmap { table: RwLock::new(new_table) };
		
		new_hashmap
	}

	fn insert (&self, key: usize, value: usize) {
		let inner_table = self.table.read().unwrap();//need read access

		// check for resize
		// let num_item: usize = inner_table.nitems.load(Ordering::Relaxed);
		// if (num_item / inner_table.nbuckets >= 2) { //threshold is 2
		// 	let resize_value: usize = inner_table.nbuckets * 2;
		// 	self.resize(resize_value); //double the size
		// }

		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;
		let index = hash % inner_table.nbuckets;

		// println!("index: {}", index);
		let mut w = inner_table.map[index].write().unwrap(); //give write access
		let ref mut bucket = *w;

		//push the key and value tuple into the map
		for &mut (k,ref mut v) in &mut *bucket {
			if k == key {
				*v = value;
				// let acount = self.nitems;
				&inner_table.nitems.fetch_add(1, Ordering::SeqCst);
				return
			}
		}
		// let acount = self.nitems;
		&inner_table.nitems.fetch_add(1, Ordering::SeqCst);
		bucket.push((key, value));
	}

	fn get (&self, key: usize) -> Option<usize>  {
		let inner_table = self.table.read().unwrap();//need read access

		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;
		let index = hash % inner_table.nbuckets;

		let r = inner_table.map[index].read().unwrap();
		//search for key value and return Some(value), otherwise return None
		for &(k,v) in r.iter() {
			if k == key {
				return Some(v)
			}
		}
		None

		// self.map[key.rem(self.nbuckets)].iter().find(|&&(k,_)| k == key).map(|&(_,v)|v) //equivalent to the above search function
	}

	fn resize (&self, newsize: usize) {
		//check if resize is actually needed

		let mut inner_table = self.table.write().unwrap();

		println!("resize: {}", newsize);
		let new_hashmap = Hashmap::new(newsize);
		let new = &new_hashmap.table.write().unwrap();

		for bucket in inner_table.map.iter() {
			let w = bucket.write().unwrap(); //give write access

			for &(k, v) in &mut w.iter() {
				new_hashmap.insert(k, v);
			}
		}

		inner_table.map = new.map;
		inner_table.nitems = new.nitems;
		inner_table.nbuckets = new.nbuckets;
	}
}

fn main() {
	println!("Program Start!");
	let new_hashmap = Arc::new(Hashmap::new(16)); //init with 16 buckets

	let h = new_hashmap.clone();
	h.insert(1,1);
	h.insert(2,5);
	h.insert(12,5);
	h.insert(13,7);
	h.insert(0,0);
	h.insert(20,3);
	h.insert(3,2);
	h.insert(3,1);
	h.insert(20,5);

	let read = h.table.read().unwrap(); //let it read it
	println!("Before Resize {:?}", read.map);

	// new_hashmap.resize(64);

	// println!("After Resize {:?}", new_hashmap.map);
    println!("Program Done!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_basics () {
		let mut new_hashmap = Hashmap::new(2); //init with 16 buckets
		// new_hashmap.map[0].push((1,2)); //manually push

		//input values
		new_hashmap.insert(1,1);
		new_hashmap.insert(2,5);
		new_hashmap.insert(12,5);
		new_hashmap.insert(13,7);
		new_hashmap.insert(0,0);

		// assert_eq!(new_hashmap.map.capacity(), 4); //should be 4 after you attempt the 5th insert

		new_hashmap.insert(20,3);
		new_hashmap.insert(3,2);
		new_hashmap.insert(3,1);
		new_hashmap.insert(20,5);

		// assert_eq!(new_hashmap.map.capacity(), 8); //should be 8 after you attempt the 9th insert		

		assert_eq!(new_hashmap.get(20).unwrap(), 5);
		assert_eq!(new_hashmap.get(12).unwrap(), 5);
		assert_eq!(new_hashmap.get(1).unwrap(), 1);
		assert_eq!(new_hashmap.get(0).unwrap(), 0);
		assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed

		// new_hashmap.resize(64);

		// assert_eq!(new_hashmap.map.capacity(), 64); //make sure it is correct length

		//try the same assert_eqs
		assert_eq!(new_hashmap.get(20).unwrap(), 5);
		assert_eq!(new_hashmap.get(12).unwrap(), 5);
		assert_eq!(new_hashmap.get(1).unwrap(), 1);
		assert_eq!(new_hashmap.get(0).unwrap(), 0);
		assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed
    }

    #[test]
    fn hashmap_concurr () {
    	let mut new_hashmap = Arc::new(Hashmap::new(16)); //init with 16 buckets
		// new_hashmap.map[0].push((1,2));

		let mut threads = vec![];

		let nthreads = 10;
		for _ in 0..nthreads {
			let new_hashmap = new_hashmap.clone();

	        threads.push(thread::spawn(move || {
	        	for _ in 1..1000 {
		        	let mut rng = thread_rng();
		        	let val = rng.gen_range(0,256);
		        	if val % 2 == 0 {
		        		new_hashmap.insert(val, val);
		        	} else {
		        		let v = new_hashmap.get(val);
		        		if (v != None) {
		        			assert_eq!(v.unwrap(), val);	
		        		}
		        	}	
	        	}
	    	}));
		}

		for t in threads {
	        t.join().unwrap();
	    }
    }

}
