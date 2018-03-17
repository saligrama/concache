package main

import (
	"math/rand"
	// "bufio"
	"sync"
	"fmt"
	"time"
)

var data = make(map[int]int)
var mutex = &sync.Mutex{}
var wg sync.WaitGroup

func main() {
	rand.Seed(time.Now().UnixNano()) //generate seed
	
	

	var numGoroutines = 50
	wg.Add(numGoroutines*3) //reader, writer, deleter

	for i:=0; i < numGoroutines; i++ {
		go writer(i)
	}

	for i:=0; i < numGoroutines; i++ { //spawn 50 goroutines 
		go reader(i)
	}

	for i:=0; i < numGoroutines; i++ {
		go deleter(i)
	}
	wg.Wait() //wait for the goroutines to finish (they never will)
}

func writer (from int) {
	defer wg.Done()
	// for true {
		var randKey = rand.Int() %500
		var randValue = rand.Int() %500

		mutex.Lock() //crashes if no mutex lock!
		data[randKey] = randValue
		mutex.Unlock()
		
		fmt.Println("writer: ", from, " Inserting: ", randValue, " into: ", randKey)
	// }
	
}

func reader(from int) {
	defer wg.Done()
	// for true {
		var randKey = rand.Int() %500
		mutex.Lock()
		if val, ok := data[randKey]; ok {
    		fmt.Println("reader: ", from, " Key: ", randKey, " Value: ", val)
		} else {
			fmt.Println("reader: ", from, " Key: ", randKey, " Value: ", "No Such Value!")
		}
		mutex.Unlock()
	// }
}

func deleter(from int) {
	defer wg.Done()
	// for true {
		var randKey = rand.Int() %500
		mutex.Lock() //crashes if no mutex lock!
		var val = data[randKey]
		delete (data, randKey)
		fmt.Println("deleter: ", from, " Deleted Key: ", randKey, " Deleted Value: ", val)
		mutex.Unlock()	
	// }
}