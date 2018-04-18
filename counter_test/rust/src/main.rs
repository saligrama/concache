use std::thread;

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
    
    println!("counter: {}", counter);

}
