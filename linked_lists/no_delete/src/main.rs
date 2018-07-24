#![allow(unused)]
// #[derive(Debug)]

extern crate rand;

use rand::{thread_rng, Rng};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicPtr, AtomicBool};
use std::sync::{Arc, RwLock, Mutex};
use std::thread;
use std::ptr;
use std::marker::PhantomData;
use std::mem;

#[derive(Debug)]
struct Node {
	data: (usize, Mutex<usize>),
	next: AtomicPtr<Node>,
    active: AtomicBool,
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

	fn insert(&self, key: usize, value: usize) -> Option<usize> {
        //Make new Node
        let mut new_node = Box::new (Node {
            data: (key, Mutex::new(value)), 
            next: AtomicPtr::new(ptr::null_mut()),
            active: AtomicBool::new(true),
        });
        
        loop {
            let mut curr_ptr = &self.head;
            let mut next_raw = curr_ptr.load(Ordering::SeqCst);
            while !next_raw.is_null() {
                let next_node = unsafe { &*next_raw };
                if key == next_node.data.0 {
                    //case where the key already exists
                    let mut change_val = next_node.data.1.lock().unwrap();
                    let node = Box::into_raw(new_node); //drop the node we created earlier to free the memory
                    let old_val = mem::replace(&mut *change_val, value); //exchange the values
                    return Some(old_val);
                }
                curr_ptr = &next_node.next;
                next_raw = curr_ptr.load(Ordering::SeqCst);
            }
            //case where the key doesn't already exist so we add it to the end
            //if CAS fails we want to loop again, keep going til it works
            let node = Box::into_raw(new_node);
            let cas_ret = curr_ptr.compare_and_swap(ptr::null_mut(), node, Ordering::SeqCst);
            if cas_ret == ptr::null_mut() {
                return None;
            } else {
                new_node = unsafe{ Box::from_raw(node) };
            }
        }
	}

	fn print(&self) {
		for node in self.iter() {
			println!("{:?}", node);
		}
	}

	fn get(&self, key: usize) -> Option<usize> {
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

    fn delete(&self, key: usize) -> Option<*mut Node> {
        //iterate through until we find the node to delete and then CAS it out
        let mut finished = false;


        while !finished {
            let mut curr_ptr = &self.head;
            let mut next_raw = curr_ptr.load(Ordering::SeqCst);            

            finished = true; //this is our last iteration through unless we encounter the key and fail the CAS, if this happens we try again
            while !next_raw.is_null() {
                let next_node = unsafe { &*next_raw };
                if key == next_node.data.0 {
                    //we check if the key is active or not, if it is active then we want to cas the active bool and make it inactive
                    if next_node.active.load(Ordering::SeqCst) {
                        //if the node is active we want to make it inactive
                        next_node.active.compare_and_swap(true, false, Ordering::SeqCst); //what if the cas fails
                        finished = false;
                        break; //we need to interate through the list again to "phsycailly" delete the element
                    } else {
                        //if the node is inactive we want to remove it
                        let cas_ret = curr_ptr.compare_and_swap(next_raw, next_node.next.load(Ordering::SeqCst), Ordering::SeqCst);
                        if cas_ret == next_raw {
                            return Some(next_raw);
                        } else {
                            return None;
                        }
                    }
                }
                curr_ptr = &next_node.next;
                next_raw = curr_ptr.load(Ordering::SeqCst);
            }
        }
        None
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

    fn insert(&self, key: usize, value: usize) -> Option<usize> {
        let check = self.nitems.load(Ordering::SeqCst);
        
        if check > 10 {
            panic!("too big!");  
        }
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let ret = self.map[index].insert(key, value);
        
        if ret == None {
            self.nitems.fetch_add(1, Ordering::SeqCst);    
        }

        ret
    }

    fn get(&self, key: usize) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        self.map[index].get(key)
    }

    fn resize(&mut self, newsize: usize) {
        let mut new_table = Table::new(newsize);

        for bucket in &self.map {
            for node in bucket.iter() {
                unsafe {
                    let k = node.data.0;
                    // assert_eq!(new_table.get(k), None);
                    let v = node.data.1.lock().unwrap();
                    // let v = *v;

                    new_table.insert(k, *v);
                    self.delete(k);
            	}
            }
        }

        self.map = new_table.map;
        self.nitems = new_table.nitems;
        self.nbuckets = new_table.nbuckets;
    }

    fn delete(&self, key: usize) -> Option<*mut Node> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let ret = self.map[index].delete(key);
        //if not None then subtract 1 from nitems

        if ret != None {
            self.nitems.fetch_sub(1, Ordering::SeqCst);
        }

        ret
    }
}

struct MapHandle {
    map: Arc<Hashmap>,
    epoch_counter: Arc<AtomicUsize>,
}

impl MapHandle {
    fn insert(&self, key: usize, value: usize) -> Option<usize> {
        //increment started before the operations begins
        self.epoch_counter.fetch_add(1, Ordering::SeqCst);
        let ret = self.map.insert(key, value);
        //increment finished after the operation ends
        self.epoch_counter.fetch_add(1, Ordering::SeqCst);

        ret
    }

    fn get(&self, key: usize) -> Option<usize> {
        //increment started before the operations begins
        self.epoch_counter.fetch_add(1, Ordering::SeqCst);
        let ret = self.map.get(key);
        //increment finished after the operation ends
        self.epoch_counter.fetch_add(1, Ordering::SeqCst);

        ret
    }

    fn delete(&self, key: usize) -> Option<usize> {
        self.epoch_counter.fetch_add(1, Ordering::SeqCst);
        //logical deletion aka cas
        let ret = self.map.delete(key);
        self.epoch_counter.fetch_add(1, Ordering::SeqCst);

        if ret == None {
            return None;
        }

        //epoch set up, load all of the values
        let mut started = Vec::new();
        let handles_map = self.map.handles.read().unwrap();
        for h in handles_map.iter() {
            started.push(h.load(Ordering::SeqCst));
        }
        for (i,h) in handles_map.iter().enumerate() {
            let mut check = h.load(Ordering::SeqCst);
            while (check <= started[i]) && (check%2 == 1) {
                println!("epoch is spinning");
                check = h.load(Ordering::SeqCst);
                //do nothing
            }
            //now finished is greater than or equal to started
        }

        // let ret = unsafe { &*ret.unwrap() };
        // let ret_val = ret.data.1.lock().unwrap().clone();

        //physical deletion, epoch has rolled over so we are safe to proceed with physical deletion
        let to_drop = ret.unwrap();
        // epoch rolled over, so we know we have exclusive access to the node
        let node = unsafe { Box::from_raw(to_drop) };
        let ret_val = node.data.1.into_inner().unwrap();
        
        return Some(ret_val);
    }
}

impl Clone for MapHandle {
    fn clone(&self) -> Self {
        let ret = Self {
            map: Arc::clone(&self.map),
            epoch_counter: Arc::new(AtomicUsize::new(0)),
        };

        let mut hashmap = &self.map;
        let mut handles_vec = hashmap.handles.write().unwrap();  //handles vector
        handles_vec.push(Arc::clone(&ret.epoch_counter));

        ret
    }
}

struct Hashmap {
    table: RwLock<Table>,
    handles: RwLock<Vec<Arc<AtomicUsize>>>, //(started, finished)
}

impl Hashmap {
    fn new() -> MapHandle {
        let new_hashmap = Hashmap {
            table: RwLock::new(Table::new(8)),
            handles: RwLock::new(Vec::new()),
        };
        let mut ret = MapHandle {
            map: Arc::new(new_hashmap),
            epoch_counter: Arc::new(AtomicUsize::new(0)),
        };

        //push the first maphandle into the epoch system
        let mut hashmap = Arc::clone(&ret.map);
        let mut handles_vec = hashmap.handles.write().unwrap();
        handles_vec.push(Arc::clone(&ret.epoch_counter));
        ret
    }

    fn insert(&self, key: usize, value: usize) -> Option<usize> {
        let inner_table = self.table.read().unwrap();
        // // check for resize
        let num_items = inner_table.nitems.load(Ordering::SeqCst);
        if (num_items / inner_table.nbuckets >= 3) { //threshold is 2
        	let resize_value: usize = inner_table.nbuckets * 2;
        	drop(inner_table); //let the resize function take the lock
        	self.resize(resize_value); //double the size
        } else {
            drop(inner_table); //force drop in case resize doesnt happen?    
        }

        let inner_table = self.table.read().unwrap();

        inner_table.insert(key, value)
    }

    fn get(&self, key: usize) -> Option<usize> {
        let inner_table = self.table.read().unwrap(); //need read access
        inner_table.get(key)
    }

    fn resize(&self, newsize: usize) {
        let mut inner_table = self.table.write().unwrap();
        if inner_table.map.capacity() != newsize {
        	inner_table.resize(newsize);
        }
    }

    fn delete(&self, key: usize) -> Option<*mut Node> {
        let inner_table = self.table.read().unwrap();
        inner_table.delete(key)
    }
}

fn main() {
    println!("Started");
    let mut handle = Hashmap::new(); //changed this,
    let mut threads = vec![];
    let nthreads = 2;
    for id in 0..nthreads {
        let new_handle = handle.clone();

        threads.push(thread::spawn(move || {
            let num_iterations = 10000;
            let mut rng = thread_rng();
            for curr in 0..num_iterations {
                let val = rng.gen_range(0, 10);
                let two = rng.gen_range(0, 3);

                if two == 0 {
                    let v = new_handle.insert(val, val);
                    if (v != None) {
                        assert_eq!(v.unwrap(), val);
                    }
                } else if two == 1 {
                    let v = new_handle.get(val);
                    if (v != None) {
                        assert_eq!(v.unwrap(), val);
                    }
                } else {
                    let v = new_handle.delete(val);
                    if (v != None) {
                        assert_eq!(v.unwrap(), val);;
                    }
                }
            }
            // assert_eq!(new_handle.epoch_counter.load(Ordering::SeqCst), 2*num_iterations);
        }));
    }
    for t in threads {
        t.join().unwrap();
    }
	println!("Finished.");
}

#[cfg(test)]
mod tests {
    use super::*;
    /*
    the data produced is a bit strange because of the way I take mod to test only even values 
    are inserted so the end number of values should be n/2 (computer style) and the capacity 
    of the map should be equal to the greatest power of 2 less than n/2.
    */
    #[test]
    fn hashmap_concurr() {
        let mut handle = Hashmap::new(); //changed this,
        let mut threads = vec![];
        let nthreads = 5;
        // let handle = MapHandle::new(Arc::clone(&new_hashmap).table.read().unwrap());
        for _ in 0..nthreads {
            let new_handle = handle.clone();

            threads.push(thread::spawn(move || {
                let num_iterations = 100000;
                for _ in 0..num_iterations {
                    let mut rng = thread_rng();
                    let val = rng.gen_range(0, 128);
                    let two = rng.gen_range(0, 3);

                    if two % 3 == 0 {
                        new_handle.insert(val, val);
                    } else if two % 3 == 1 {
	                    let v = new_handle.get(val);
	                    if (v != None) {
	                        assert_eq!(v.unwrap(), val);
	                    }
                    } else {
                        new_handle.delete(val);
                    }
                }
                // assert_eq!(new_handle.epoch_counter.load(Ordering::SeqCst), num_iterations*2);
            }));
        }
        for t in threads {
            t.join().unwrap();
        }
    }

    #[test]
    fn hashmap_handle_cloning() {
        let mut handle = Arc::new(Hashmap::new()); //init with 16 bucket
        println!("{:?}", handle.epoch_counter);
        handle.insert(1,3);
        assert_eq!(handle.get(1).unwrap(), 3);

        //create a new handle
        let new_handle = Arc::clone(&handle);
        assert_eq!(new_handle.get(1).unwrap(), 3);
        new_handle.insert(2,5);

        assert_eq!(handle.get(2).unwrap(), 5);
    }

    #[test]
    fn hashmap_delete() {
        let mut handle = Hashmap::new();
        handle.insert(1,3);
        handle.insert(2,5);
        handle.insert(3,8);
        handle.insert(4,3);
        handle.insert(5,4);
        handle.insert(6,5);
        handle.insert(7,3);
        handle.insert(8,3);
        handle.insert(9,3);
        handle.insert(10,3);
        handle.insert(11,3);
        handle.insert(12,3);
        handle.insert(13,3);
        handle.insert(14,3);
        handle.insert(15,3);
        handle.insert(16,3);
        assert_eq!(handle.get(1).unwrap(), 3);
        assert_eq!(handle.delete(1).unwrap(), 3);
        assert_eq!(handle.get(1), None);
        assert_eq!(handle.delete(2).unwrap(), 5);
        assert_eq!(handle.delete(16).unwrap(), 3);
        assert_eq!(handle.get(16), None);
    }

    #[test]
    fn linkedlist_basics() {
        let mut new_linked_list = LinkedList::new();
        
        println!("{:?}", new_linked_list);
        new_linked_list.insert(3, 2);
        new_linked_list.insert(3, 4);
        new_linked_list.insert(5, 8);
        new_linked_list.insert(4, 6);
        new_linked_list.insert(1, 8);
        new_linked_list.insert(6, 6);
        new_linked_list.print();

        assert_eq!(new_linked_list.get(3).unwrap(), 4);
        assert_eq!(new_linked_list.get(5).unwrap(), 8);
        assert_eq!(new_linked_list.get(2), None);
    }

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

        assert_eq!(new_hashmap.insert(20, 5).unwrap(), 3); //repeated
        assert_eq!(new_hashmap.insert(3, 8).unwrap(), 2); //repeated
        assert_eq!(new_hashmap.insert(5, 5), None); //repeated

        let cln = Arc::clone(&new_hashmap.map);
        assert_eq!(cln.table.read().unwrap().nitems.load(Ordering::SeqCst), 9);
            

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
}
