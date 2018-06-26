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
use std::marker::PhantomData;


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

struct LinkedListIterator<'a> {
    current: *mut Node,
    marker: PhantomData<&'a ()>,
}

impl<'a> Iterator for LinkedListIterator<'a> {
    type Item = &'a Node;
    fn next(&mut self) -> Option<&'a Node> {
		if self.current.is_null() {
			return None
		}
		let node = unsafe{&*self.current};
		self.current = node.next.load(Ordering::SeqCst);
        Some(node)
    }
}

impl LinkedList {
	fn new() -> Self {
		LinkedList {	
			head: AtomicPtr::new(ptr::null_mut()),
		}
	}

	//might change to accet a just values instead because tuples is confusing
	fn insert(&self, value: (usize, usize)) -> usize {
		//Make new Node
		let mut new_node = Box::new (Node {
			data: (value.0, Mutex::new(value.1)), 
			next: AtomicPtr::new(ptr::null_mut()),
		});

		if self.head.load(Ordering::SeqCst).is_null() {
			self.head.compare_and_swap(ptr::null_mut(), Box::into_raw(new_node), Ordering::SeqCst);
		} else {
			let mut node_ptr = Box::into_raw(new_node);

			loop {
				let mut curr_node: &Node;
				let mut curr_ptr = &self.head; //not sure if needed atomic needed here
				let mut loaded_next_ptr = curr_ptr.load(Ordering::SeqCst);

				//go until finds the NULL pointer
				while !loaded_next_ptr.is_null() {
					unsafe {
						curr_node = &*loaded_next_ptr;
					}
					curr_ptr = &curr_node.next;
					loaded_next_ptr = curr_ptr.load(Ordering::SeqCst);

					if curr_node.data.0 == value.0 {
						let mut change_value = curr_node.data.1.lock().unwrap();
						*change_value = value.1;
						return 0;
					}
				}
				//insert at the new pointer
				curr_ptr.compare_and_swap(ptr::null_mut(), node_ptr, Ordering::SeqCst);
				return 1;
			}
		}
        return 1;
	}

	fn print(&self) {
		for node in self.iter() {
			println!("{:?}", node);
		}
	}

	fn get(&self, key: usize) -> Option<usize> {
		// self.print();
		for node in self.iter() {
			if node.data.0 == key {
				let value = node.data.1.lock().unwrap();
				return Some(*value);
			}
		}
		None
	}

    fn iter(&self) -> LinkedListIterator { 
    	LinkedListIterator { 
    		current: self.head.load(Ordering::SeqCst), 
    		marker: PhantomData,
    	}
    }
}

struct Table {
    nbuckets: usize,
    map: Vec<LinkedList>,
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
            t.map.push(LinkedList::new());
        }

        t
    }

    //changed to mut 
    fn insert(&self, key: usize, value: usize) {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let add_val = self.map[index].insert((key, value));
        //issue with insert, have it return number 0 or 1?
        self.nitems.fetch_add(add_val, Ordering::SeqCst);
    }

    fn get(&self, key: usize) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        self.map[index].get(key)
    }

    fn resize(&mut self, newsize: usize) {
        // println!("resize: {}", newsize);
        let mut new = Table::new(newsize);

        for bucket in &self.map {
            for node in bucket.iter() {
            	unsafe {
            		let k = (*node).data.0;
	            	let v = (*node).data.1.lock().unwrap();
	            	let v = *v;
	                new.insert(k, v);	
            	}
            	
            }
        }

        self.map = new.map;
        self.nitems = new.nitems;
        self.nbuckets = new.nbuckets;
        // println!("finished resize");
    }
}

struct MapHandle {
    map: Arc<Hashmap>,
    started: Arc<AtomicUsize>,
    finished: Arc<AtomicUsize>,
}

impl MapHandle {
    //is this really what I should be doing??
    fn insert(&self, key: usize, value: usize) {
        Arc::clone(&self.map).insert(key, value);
    }

    fn get(&self, key: usize) -> Option<usize> {
        Arc::clone(&self.map).get(key)   
    }
}

impl Clone for MapHandle {
    fn clone(&self) -> Self {
        let ret = Self {
            map: Arc::clone(&self.map),
            started: Arc::new(AtomicUsize::new(0)),
            finished: Arc::new(AtomicUsize::new(0)),    
        };

        let mut hashmap = Arc::clone(&self.map);
        let mut handles_vec = hashmap.handles.write().unwrap();  //handles vector
        handles_vec.push((Arc::clone(&ret.started), Arc::clone(&ret.finished)));

        ret
    }
}

struct Hashmap {
    table: RwLock<Table>,
    handles: RwLock<Vec<(Arc<AtomicUsize>, Arc<AtomicUsize>)>>, //(started, finished)
}

impl Hashmap {
    fn new() -> MapHandle {
        let new_hashmap = Hashmap {
            table: RwLock::new(Table::new(8)),
            handles: RwLock::new(Vec::new()),
        };
        MapHandle {
            map: Arc::new(new_hashmap),
            started: Arc::new(AtomicUsize::new(0)),
            finished: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn insert(&self, key: usize, value: usize) {
        let inner_table = self.table.read().unwrap(); //need read access
        // // check for resize
        let num_item: usize = inner_table.nitems.load(Ordering::SeqCst);
        if (num_item / inner_table.nbuckets >= 2) { //threshold is 2
        	let resize_value: usize = inner_table.nbuckets * 2;
        	drop(inner_table); //let the resize function take the lock
        	self.resize(resize_value); //double the size
        } else {
            drop(inner_table); //force drop in case resize doesnt happen?    
        }
        let inner_table = self.table.read().unwrap(); //need read access
        inner_table.insert(key, value);
    }

    fn get(&self, key: usize) -> Option<usize> {
        let inner_table = self.table.read().unwrap(); //need read access
        inner_table.get(key)
    }

    fn resize(&self, newsize: usize) {
        let mut inner_table = self.table.write().unwrap();
        // TODO: re-check if resize is actually needed
        if inner_table.map.capacity() != newsize {
        	inner_table.resize(newsize);
        }
    }

    // fn delete(&self, key: usize)
}

fn main() {
    println!("Started");
	println!("Finished.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashmap_basics() {
        let mut new_hashmap = Hashmap::new(); //init with 2 buckets
        //input values
        new_hashmap.insert(1, 1);
        new_hashmap.insert(2, 5);
        new_hashmap.insert(12, 5);
        new_hashmap.insert(13, 7);
        new_hashmap.insert(0, 0);

        // assert_eq!(new_hashmap.table.read().unwrap().map.capacity(), 4); //should be 4 after you attempt the 5th insert

        new_hashmap.insert(20, 3);
        new_hashmap.insert(3, 2);
        new_hashmap.insert(4, 1);
        new_hashmap.insert(5, 5);

        new_hashmap.insert(20, 5); //repeated
        new_hashmap.insert(3, 8); //repeated
        // assert_eq!(new_hashmap.table.read().unwrap().map.capacity(), 8); //should be 8 after you attempt the 9th insert

        assert_eq!(new_hashmap.get(20).unwrap(), 5);
        assert_eq!(new_hashmap.get(12).unwrap(), 5);
        assert_eq!(new_hashmap.get(1).unwrap(), 1);
        assert_eq!(new_hashmap.get(0).unwrap(), 0);
        assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed

        // new_hashmap.resize(64);

        // assert_eq!(new_hashmap.table.read().unwrap().map.capacity(), 64); //make sure it is correct length

        // try the same assert_eqs
        assert_eq!(new_hashmap.get(20).unwrap(), 5);
        assert_eq!(new_hashmap.get(12).unwrap(), 5);
        assert_eq!(new_hashmap.get(1).unwrap(), 1);
        assert_eq!(new_hashmap.get(0).unwrap(), 0);
        assert!(new_hashmap.get(3).unwrap() != 2); // test that it changed
    }

    /*
    the data produced is a bit strange because of the way I take mod to test only even values 
    are inserted so the end number of values should be n/2 (computer style) and the capacity 
    of the map should be equal to the greatest power of 2 less than n/2.
    */
    #[test]
    fn hashmap_concurr() {
        let mut handle = Arc::new(Hashmap::new()); //changed this,
        let mut threads = vec![];
        let nthreads = 5;
        // let handle = MapHandle::new(Arc::clone(&new_hashmap).table.read().unwrap());
        for _ in 0..nthreads {
            let new_handle = handle.clone();
            // println!("numitems at start {:?}", new_handle.table.write().unwrap().nitems);

            threads.push(thread::spawn(move || {
                for _ in 1..10000 {
                    let mut rng = thread_rng();
                    let val = rng.gen_range(0, 128);
                    let two = rng.gen_range(0, 2);

                    if two % 2 == 0 {
                        new_handle.insert(val, val);
                    } else {
	                    let v = new_handle.get(val);
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

    #[test]
    fn hashmap_handle_cloning() {
        let mut handle = Arc::new(Hashmap::new()); //init with 16 bucket
        println!("{:?}", handle.started);
        println!("{:?}", handle.finished);
        handle.insert(1,3);
        assert_eq!(handle.get(1).unwrap(), 3);

        //create a new handle
        let new_handle = Arc::clone(&handle);
        assert_eq!(new_handle.get(1).unwrap(), 3);
        new_handle.insert(2,5);

        assert_eq!(handle.get(2).unwrap(), 5);
    }


    #[test]
    fn linkedlist_basics() {
        let mut new_linked_list = LinkedList::new();
        
        println!("{:?}", new_linked_list);
        new_linked_list.insert((3, 2));
        new_linked_list.insert((3, 4));
        new_linked_list.insert((5, 8));
        new_linked_list.insert((4, 6));
        new_linked_list.insert((1, 8));
        new_linked_list.insert((6, 6));
        new_linked_list.print();

        assert_eq!(new_linked_list.get(3).unwrap(), 4);
        assert_eq!(new_linked_list.get(5).unwrap(), 8);
        assert_eq!(new_linked_list.get(2), None);

    }
}
