use std::thread;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;


fn main() {
	let counter : usize = 0;

    let mut threads = vec![];

    for _i in 1..10000 {
    	let ref mut counter = counter.clone();
    	threads.push(thread::spawn(move || {
	    	for _j in 1..10000 {
	    		*counter += 1;
	    	}
	    	println!("counter: {}", counter);
    	}));
    }
    for t in threads {
        t.join().unwrap();
    }

 	// let mut threads = vec![];

  //   let atomic_counter = Arc::new(AtomicUsize::new(0));
  //   for _i in 0..10000 {
  //   	let atomic_counter = atomic_counter.clone();
  //   	threads.push(thread::spawn(move || {
	 //    	for _j in 0..10000 {
	 //    		atomic_counter.fetch_add(1, Ordering::SeqCst);
	 //    	}
	 //    	println!("atomic_counter: {}", atomic_counter.load(Ordering::SeqCst));
  //   	}));
  //   }
  //   for t in threads {
  //       t.join().unwrap();
  //   }
    
  //   println!("atomic_counter: {}", atomic_counter.load(Ordering::SeqCst));

}
