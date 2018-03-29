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
		let num_of_buckets = 16;
		let mut new_hashmap = Hashmap {nbuckets: 16, map: Vec::with_capacity(num_of_buckets)};
		new_hashmap.map.resize(num_of_buckets, Vec::new());
		return new_hashmap
	}

	fn insert (&mut self, key: usize, value: usize) {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;
		self.map[hash.rem(self.nbuckets)].push((hash, value));
		// println!("hasher.finish() {}", hasher.finish() % self.nbuckets);
		// println!("{}", self.nbuckets);
		println!("ins hash: {}", hash);
		println!("ins indx {:?}", hash.rem(self.nbuckets));
	}

	fn get (&self, key: usize) -> Option<usize>  {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let hash: usize = hasher.finish() as usize;

		println!("get hash: {}", hash);
		println!("get indx {:?}", hash.rem(self.nbuckets));
		// println!("Result of Search: {:?}", self.map[key.rem(self.nbuckets)].iter().find(|&&(k,_)| k == key));	

		self.map[key.rem(self.nbuckets)].iter().find(|&&(k,_)| k == key).map(|&(_,v)|v)
	}
}

fn main() {

	let mut new_hashmap = Hashmap::new();
	new_hashmap.map[0].push((1,2));
	
	new_hashmap.insert(3,1);
	println!("result: {:?}", new_hashmap.get(3));

	// let mut input = String::new();
 //    io::stdin().read_line(&mut input)


	println!("{:?}", new_hashmap.map);

    println!("Program Done!");
}
