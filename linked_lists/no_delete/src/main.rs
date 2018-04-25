#![allow(unused)]
// #[derive(Debug)]

extern crate rand;

use rand::{thread_rng, Rng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicPtr};
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::ptr;

#[derive(Debug)]
struct Node {
	data: (usize, Mutex<usize>),
	next: AtomicPtr<Node>,
	// next: *const Node, //not sure if const is right
}

#[derive(Debug)]
struct LinkedList {
	head: AtomicPtr<Node>,
}

impl LinkedList {
	fn new() -> Self {
		let mut ll = LinkedList {	
			head: AtomicPtr::new(ptr::null_mut()),
		};
		ll
	}

	fn insert(&mut self, value: (usize, usize)) {
		//Make new Node
		let mut new_node = Box::new (Node {
			data: (value.0, Mutex::new(value.1)), 
			next: AtomicPtr::new(ptr::null_mut()),
		});

		if self.head.get_mut().is_null() {
			self.head.compare_and_swap(ptr::null_mut(), Box::into_raw(new_node), Ordering::SeqCst);
		} else {
			let mut curr_node: &Node;
			let mut curr_ptr = &self.head; //not sure if needed atomic needed here

			//go until finds the NULL pointer
			while (!curr_ptr.load(Ordering::SeqCst).is_null()) {
				unsafe {
					curr_node = &*curr_ptr.load(Ordering::SeqCst);
				}
				curr_ptr = &curr_node.next;

				if curr_node.data.0 == value.0 {
					let mut change_value = curr_node.data.1.lock().unwrap();
					*change_value = value.1;
					return
				}
			}
			//insert at the new pointer
			curr_ptr.compare_and_swap(ptr::null_mut(), Box::into_raw(new_node), Ordering::SeqCst);

		}
	}

	fn print(&mut self) {
		println!("Printing List!");
		if self.head.get_mut().is_null() { //is the a way to get the non mut pointer?
		} else {
			let mut curr_node: &Node;
			let mut curr_ptr = &self.head; //not sure if needed atomic needed here
			// unsafe {
			// 	println!("{:?}", *curr_ptr.load(Ordering::SeqCst));	
			// }
			//go until finds the NULL pointer
			while (!curr_ptr.load(Ordering::SeqCst).is_null()) {
				unsafe {
					curr_node = &*curr_ptr.load(Ordering::SeqCst);
					println!("{:?}", curr_node);	
				}
				curr_ptr = &curr_node.next;
			}
		}
	}

	fn get(&mut self, key: usize) -> Option<usize> {
		if !self.head.get_mut().is_null() { //is the a way to get the non mut pointer?
			let mut curr_node: &Node;
			let mut curr_ptr = &self.head; //not sure if needed atomic needed here

			//go until finds the NULL pointer
			while (!curr_ptr.load(Ordering::SeqCst).is_null()) {
				unsafe {
					curr_node = &*curr_ptr.load(Ordering::SeqCst);
					if curr_node.data.0 == key {
						let value = curr_node.data.1.lock().unwrap(); //underlying value
						return Some(*value);
					}
				}
				curr_ptr = &curr_node.next;
			}
		}
		None
	}
}

struct Table {
    nbuckets: usize,
    map: Vec<RwLock<Vec<(usize, usize)>>>,
    nitems: AtomicUsize,
}

impl Table {
    fn new(num_of_buckets: usize) -> Self {
        let mut t = Table {
            nbuckets: num_of_buckets,
            map: Vec::with_capacity(num_of_buckets),
            nitems: AtomicUsize::new(0),
        };

        for _ in 0..num_of_buckets {
            t.map.push(RwLock::new(Vec::new()));
        }

        t
    }

    fn insert(&self, key: usize, value: usize) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        // println!("index: {}", index);
        let mut w = self.map[index].write().unwrap(); //give write access

        //push the key and value tuple into the map
        for &mut (k, ref mut v) in w.iter_mut() {
            if k == key {
                *v = value;
                // let acount = self.nitems;
                // replacing does *not* increase the number of items
                // self.nitems.fetch_add(1, Ordering::SeqCst);
                return;
            }
        }
        // let acount = self.nitems;
        self.nitems.fetch_add(1, Ordering::SeqCst);
        w.push((key, value));
    }

    fn get(&self, key: usize) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let r = self.map[index].read().unwrap();
        //search for key value and return Some(value), otherwise return None
        for &(k, v) in r.iter() {
            if k == key {
                return Some(v);
            }
        }
        None

        // self.map[key.rem(self.nbuckets)].iter().find(|&&(k,_)| k == key).map(|&(_,v)|v) //equivalent to the above search function
    }

    fn resize(&mut self, newsize: usize) {
        println!("resize: {}", newsize);
        let new = Table::new(newsize);

        for bucket in &self.map {
            let bucket = bucket.read().unwrap();
            for &(key, value) in bucket.iter() {
                new.insert(key, value);
            }
        }

        self.map = new.map;
        self.nitems = new.nitems;
        self.nbuckets = new.nbuckets;
    }
}

struct Hashmap {
    table: RwLock<Table>,
}

impl Hashmap {
    fn new(num_of_buckets: usize) -> Self {
        Hashmap {
            table: RwLock::new(Table::new(num_of_buckets)),
        }
    }

    fn insert(&self, key: usize, value: usize) {
        let inner_table = self.table.read().unwrap(); //need read access

        // // check for resize
        let num_item: usize = inner_table.nitems.load(Ordering::Relaxed);
        if (num_item / inner_table.nbuckets >= 2) { //threshold is 2
        	let resize_value: usize = inner_table.nbuckets * 2;

        	drop(inner_table); //let the resize function take the lock
        	self.resize(resize_value); //double the size
        }

        let inner_table = self.table.read().unwrap(); //need read access
        inner_table.insert(key, value);
    }

    fn get(&self, key: usize) -> Option<usize> {
        let inner_table = self.table.read().unwrap(); //need read access
        inner_table.get(key)
    }

    fn resize(&self, newsize: usize) {
    	// println!("HEREA!");
        let mut inner_table = self.table.write().unwrap();
        // println!("THERE!");


        println!("Made it here, with value: {}", newsize);

        // TODO: re-check if resize is actually needed
        if inner_table.map.capacity() != newsize {
        	inner_table.resize(newsize);
        }
    }
}

fn main() {
	let mut new_linked_list = LinkedList::new();
	println!("{:?}", new_linked_list);
	new_linked_list.insert((3, 2));
	new_linked_list.insert((3, 4));
	new_linked_list.insert((5, 8));
	new_linked_list.insert((4, 6));
	new_linked_list.print();

	assert_eq!(new_linked_list.get(3).unwrap(), 4);
	assert_eq!(new_linked_list.get(5).unwrap(), 8);
	assert_eq!(new_linked_list.get(2), None);
	println!("Finished.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_basics() {
        let mut new_hashmap = Hashmap::new(2); //init with 16 buckets
		// new_hashmap.map[0].push((1,2)); //manually push

        //input values
        new_hashmap.insert(1, 1);
        new_hashmap.insert(2, 5);
        new_hashmap.insert(12, 5);
        new_hashmap.insert(13, 7);
        new_hashmap.insert(0, 0);

        println!("testing for 4");
        assert_eq!(new_hashmap.table.read().unwrap().map.capacity(), 4); //should be 4 after you attempt the 5th insert

        new_hashmap.insert(20, 3);
        new_hashmap.insert(3, 2);
        new_hashmap.insert(4, 1);
        new_hashmap.insert(5, 5);

        new_hashmap.insert(20, 5); //repeated
        new_hashmap.insert(3, 8); //repeated
        println!("testing for 8");
        // assert_eq!(new_hashmap.table.read().unwrap().map.capacity(), 8); //should be 8 after you attempt the 9th insert

        assert_eq!(new_hashmap.get(20).unwrap(), 5);
        assert_eq!(new_hashmap.get(12).unwrap(), 5);
        assert_eq!(new_hashmap.get(1).unwrap(), 1);
        assert_eq!(new_hashmap.get(0).unwrap(), 0);
        assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed

        new_hashmap.resize(64);

        assert_eq!(new_hashmap.table.read().unwrap().map.capacity(), 64); //make sure it is correct length

        //try the same assert_eqs
        assert_eq!(new_hashmap.get(20).unwrap(), 5);
        assert_eq!(new_hashmap.get(12).unwrap(), 5);
        assert_eq!(new_hashmap.get(1).unwrap(), 1);
        assert_eq!(new_hashmap.get(0).unwrap(), 0);
        assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed
    }

    #[test]
    fn hashmap_concurr() {
        let mut new_hashmap = Arc::new(Hashmap::new(16)); //init with 16 buckets                                                   // new_hashmap.map[0].push((1,2));
        let mut threads = vec![];
        let nthreads = 10;
        for _ in 0..nthreads {
            let new_hashmap = new_hashmap.clone();

            threads.push(thread::spawn(move || {
                for _ in 1..1000 {
                    let mut rng = thread_rng();
                    let val = rng.gen_range(0, 256);
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
