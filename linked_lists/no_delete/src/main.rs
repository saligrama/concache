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

const OSC: Ordering = Ordering::SeqCst;

#[derive(Debug)]
struct Node {
	key: Option<usize>,
    val: Option<Mutex<usize>>,
	next: AtomicPtr<Node>,
    marked: AtomicBool,
}

impl Node {
    fn new(key: Option<usize>, val: Option<Mutex<usize>>) -> Node {
        Node {
            key: key,
            val: val,
            next: AtomicPtr::new(ptr::null_mut()),
            marked: AtomicBool::new(false),
        }
    }
}

#[derive(Debug)]
struct LinkedList {
	head: AtomicPtr<Node>,
}

impl LinkedList {
	fn new() -> Self {
        let head_node = Box::new(Node::new(None, None));
        let tail_node = Box::new(Node::new(None, None));

        println!("Linked List Created.");
		let mut ret = LinkedList {
            head: AtomicPtr::new(Box::into_raw(head_node)),
		};
        let hnode = unsafe{ &*ret.head.load(OSC) };
        hnode.next.compare_and_swap(ptr::null_mut(), Box::into_raw(tail_node), OSC);
        ret
	}

	fn insert(&self, key: usize, val: usize) -> Option<bool> {
        println!("here!");
        let mut new_node = Node::new(Some(key), Some(Mutex::new(val)));
        let mut left_node = Node::new(None, None);
        let mut right_node = Node::new(None, None);

        let left_node_ptr: AtomicPtr<Node> = AtomicPtr::new(ptr::null_mut());
        let right_node_ptr: AtomicPtr<Node> = AtomicPtr::new(ptr::null_mut());
        println!("left_node_ptr: {:?}", left_node_ptr);

        loop {
            println!("Searching.");
            self.search(key, &left_node_ptr, &right_node_ptr);
            
            println!("left_node_ptr loaded val: {:?}", left_node_ptr.load(OSC));
            println!("right_node_ptr loaded val: {:?}", right_node_ptr.load(OSC));

            // if (right_node.next.load(OSC) != ptr::null_mut()) && right_node.key == Some(key) {
            //     return Some(false);
            // }

            // let right_ptr = &mut right_node as *mut Node;
            // let left_ptr = &mut left_node as *mut Node;
            // let node_ptr = &mut new_node as *mut Node;
            // new_node.next = AtomicPtr::new(right_ptr);


            // let cas_ret = left_node.next.compare_and_swap(right_ptr, node_ptr, OSC);
            // if  cas_ret == right_ptr {
            //     return Some(true);
            // }
            return None;
        }

        None
	}

	fn print(&self) {
		// for node in self.iter() {
		// 	println!("{:?}", node);
		// }
	}

	fn get(&self, search_key: usize) -> Option<bool> {
        // let mut left_node = Node::new(None, None);
        // let mut right_node = Node::new(None, None);
        let left_node_ptr: AtomicPtr<Node> = AtomicPtr::new(ptr::null_mut());
        let right_node_ptr: AtomicPtr<Node> = AtomicPtr::new(ptr::null_mut());

        self.search(search_key, &left_node_ptr, &right_node_ptr);

        // println!("Left Node: {:?}", left_node);
        // println!("Right Node: {:?}", right_node);

        // if right_node.next.load(OSC) == ptr::null_mut() || (right_node.key != Some(search_key)) {
        //     return Some(false);
        // } else {
        //     return Some(true);
        // }
        None
	}

    fn delete(&self, key: usize) -> Option<*mut Node> {
        //iterate through until we find the node to delete and then CAS it out
        // let mut finished = false;


        // while !finished {
        //     let mut curr_ptr = &self.head;
        //     let mut next_raw = curr_ptr.load(OSC);            

        //     finished = true; //this is our last iteration through unless we encounter the key and fail the CAS, if this happens we try again
        //     while !next_raw.is_null() {
        //         let next_node = unsafe { &*next_raw };
        //         if key == next_node.data.0 {
        //             //we check if the key is active or not, if it is active then we want to cas the active bool and make it inactive
        //             if next_node.active.load(OSC) {
        //                 //if the node is active we want to make it inactive
        //                 next_node.active.compare_and_swap(true, false, OSC); //what if the cas fails
        //                 finished = false;
        //                 break; //we need to interate through the list again to "phsycailly" delete the element
        //             } else {
        //                 //if the node is inactive we want to remove it
        //                 let cas_ret = curr_ptr.compare_and_swap(next_raw, next_node.next.load(OSC), OSC);
        //                 if cas_ret == next_raw {
        //                     return Some(next_raw);
        //                 } else {
        //                     return None;
        //                 }
        //             }
        //         }
        //         curr_ptr = &next_node.next;
        //         next_raw = curr_ptr.load(OSC);
        //     }
        // }
        None
    }

    //lifetimes are screwing me over!
    fn search(&self, search_key: usize, left_node_ptr: &AtomicPtr<Node>, right_node_ptr: &AtomicPtr<Node>) {

        let left_node_next_ptr: AtomicPtr<Node> = AtomicPtr::new(ptr::null_mut());

        //search
        loop {
            let mut t = &self.head; //get ptr to the head
            let mut t_next = unsafe { &(&*t.load(OSC)).next }; //get the ptr to the next

            // Find the left node and right node
            loop {
                if unsafe { !(&*t.load(OSC)).marked.load(OSC) } {
                    left_node_ptr.store(t.load(OSC), OSC); //set the left node 
                    left_node_next_ptr.store(t_next.load(OSC), OSC); //set the left next node
                }
                t = unsafe{ &(&*t.load(OSC)).next };
                if unsafe { (&*t.load(OSC)).next.load(OSC) == ptr::null_mut() } { //test if t == self.tail, not sure if this is okay, but we cmp t.next with null ptr because tail is always null ptr
                    break;
                }
                t_next = unsafe { &(&*t.load(OSC)).next }; //we know its not the tail so we can go to it

                if unsafe { (&*t_next.load(OSC)).marked.load(OSC) || (&*t.load(OSC)).key == None || (&*t.load(OSC)).key.unwrap() < search_key } {
                    break;
                }
            }
            right_node_ptr.store(t.load(OSC), OSC);

            let mut cont = true;
            //Ckeck nodes are adjacent
            if unsafe { (&*left_node_next_ptr.load(OSC)).next.load(OSC) == (&*right_node_ptr.load(OSC)).next.load(OSC) } {
                let right_node_next_ptr = unsafe { &(&*right_node_ptr.load(OSC)).next };
                if unsafe { (&*right_node_ptr.load(OSC)).next.load(OSC) != ptr::null_mut() && (&*right_node_next_ptr.load(OSC)).marked.load(OSC) } {
                    cont = false;
                } else {
                    // println!("left_node b: {:?}", left_node);
                    // return Some((left_node, right_node));
                    return;
                }
            }

            //MISSING A CAS HERE, TODO ADD CAS
            if (cont) {
                //if we continue, then remove one or more marked nodes
                if unsafe { (&*right_node_ptr.load(OSC)).next.load(OSC) != ptr::null_mut() } {
                    //then search again
                } else {
                    // return Some((left_node, right_node));
                    return;
                }
            }
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

    fn insert(&self, key: usize, value: usize) -> Option<bool> {
        println!("hereb");
        let check = self.nitems.load(OSC);
        
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let ret = self.map[index].insert(key, value);
        
        if ret == None {
            self.nitems.fetch_add(1, OSC);    
        }

        ret
    }

    fn get(&self, key: usize) -> Option<bool> {
        println!("herec");
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        self.map[index].get(key)
    }

    // fn resize(&mut self, newsize: usize) {
    //     let mut new_table = Table::new(newsize);

    //     for bucket in &self.map {
    //         for node in bucket.iter() {
    //             unsafe {
    //                 let k = node.data.0;
    //                 // assert_eq!(new_table.get(k), None);
    //                 let v = node.data.1.lock().unwrap();
    //                 // let v = *v;

    //                 new_table.insert(k, *v);
    //                 self.delete(k);
    //         	}
    //         }
    //     }

    //     self.map = new_table.map;
    //     self.nitems = new_table.nitems;
    //     self.nbuckets = new_table.nbuckets;
    // }

    fn delete(&self, key: usize) -> Option<*mut Node> {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash: usize = hasher.finish() as usize;
        let index = hash % self.nbuckets;

        let ret = self.map[index].delete(key);
        //if not None then subtract 1 from nitems

        if ret != None {
            self.nitems.fetch_sub(1, OSC);
        }

        ret
    }
}

struct MapHandle {
    map: Arc<Hashmap>,
    epoch_counter: Arc<AtomicUsize>,
}

impl MapHandle {
    fn insert(&self, key: usize, value: usize) -> Option<bool> {
        //increment started before the operations begins
        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.insert(key, value);
        //increment finished after the operation ends
        self.epoch_counter.fetch_add(1, OSC);

        ret
    }

    fn get(&self, key: usize) -> Option<bool> {
        //increment started before the operations begins
        self.epoch_counter.fetch_add(1, OSC);
        let ret = self.map.get(key);
        //increment finished after the operation ends
        self.epoch_counter.fetch_add(1, OSC);

        ret
    }

    fn delete(&self, key: usize) -> Option<usize> {
        // self.epoch_counter.fetch_add(1, OSC);
        // //logical deletion aka cas
        // let ret = self.map.delete(key);
        // self.epoch_counter.fetch_add(1, OSC);

        // if ret == None {
        //     return None;
        // }

        // //epoch set up, load all of the values
        // let mut started = Vec::new();
        // let handles_map = self.map.handles.read().unwrap();
        // for h in handles_map.iter() {
        //     started.push(h.load(OSC));
        // }
        // for (i,h) in handles_map.iter().enumerate() {
        //     let mut check = h.load(OSC);
        //     while (check <= started[i]) && (check%2 == 1) {
        //         // println!("epoch is spinning");
        //         check = h.load(OSC);
        //         //do nothing
        //     }
        //     //now finished is greater than or equal to started
        // }

        // // let ret = unsafe { &*ret.unwrap() };
        // // let ret_val = ret.data.1.lock().unwrap().clone();

        // //physical deletion, epoch has rolled over so we are safe to proceed with physical deletion
        // let to_drop = ret.unwrap();
        // // epoch rolled over, so we know we have exclusive access to the node
        // let node = unsafe { Box::from_raw(to_drop) };
        // let ret_val = node.data.1.into_inner().unwrap();
        
        // return Some(ret_val);
        None
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

    fn insert(&self, key: usize, value: usize) -> Option<bool> {
        let inner_table = self.table.read().unwrap();
        // // // check for resize
        // let num_items = inner_table.nitems.load(OSC);
        // if (num_items / inner_table.nbuckets >= 3) { //threshold is 2
        // 	let resize_value: usize = inner_table.nbuckets * 2;
        // 	drop(inner_table); //let the resize function take the lock
        // 	self.resize(resize_value); //double the size
        // } else {
        //     drop(inner_table); //force drop in case resize doesnt happen?    
        // }

        let inner_table = self.table.read().unwrap();

        inner_table.insert(key, value)
    }

    fn get(&self, key: usize) -> Option<bool> {
        let inner_table = self.table.read().unwrap(); //need read access
        inner_table.get(key)
    }

    // fn resize(&self, newsize: usize) {
    //     let mut inner_table = self.table.write().unwrap();
    //     if inner_table.map.capacity() != newsize {
    //     	inner_table.resize(newsize);
    //     }
    // }

    fn delete(&self, key: usize) -> Option<*mut Node> {
        let inner_table = self.table.read().unwrap();
        inner_table.delete(key)
    }
}

fn main() {
    println!("Started.");

    let mut new_linked_list = LinkedList::new();

    println!("{:?}", new_linked_list.head.load(OSC));
    let next_node = unsafe { &*new_linked_list.head.load(OSC) };
    println!("{:?}", next_node.next.load(OSC));
    let next_node = unsafe { &*next_node.next.load(OSC) };
    println!("{:?}", next_node.next.load(OSC));
    

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
                // assert_eq!(new_handle.epoch_counter.load(OSC), num_iterations*2);
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
        assert_eq!(cln.table.read().unwrap().nitems.load(OSC), 9);
            

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
