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

func main() {
	rand.Seed(time.Now().UnixNano()) //generate seed
	var wg sync.WaitGroup
	

	var numGoroutines = 50
	wg.Add(numGoroutines*3) //reader, writer, deleter

	for i:=0; i < numGoroutines; i++ {
		defer wg.Done()
		go writer(i)
	}

	for i:=0; i < numGoroutines; i++ { //spawn 50 goroutines 
		defer wg.Done()
		go reader(i)
	}

	for i:=0; i < numGoroutines; i++ {
		defer wg.Done()
		go deleter(i)
	}
	wg.Wait() //wait for the goroutines to finish (they never will)
}

func writer (from int) {
	for true {
		mutex.Lock() //crashes if no mutex lock!
		var randKey = rand.Int() %500
		var randValue = rand.Int() %500
		data[randKey] = randValue
		fmt.Println("writer: ", "Inserting: ", randValue, " into: ", randKey)
		mutex.Unlock()
	}
	
}

func reader(from int) {
	for true {
		var randKey = rand.Int() %500
		mutex.Lock()
		if val, ok := data[randKey]; ok {
    		fmt.Println("reader: ", from, " Key: ", randKey, " Value: ", val)
		} else {
			fmt.Println("reader: ", from, " Key: ", randKey, " Value: ", "No Such Value!")
		}
		mutex.Unlock()
	}
}

func deleter(from int) {
	for true {
		var randKey = rand.Int() %500
		mutex.Lock() //crashes if no mutex lock!
		var val = data[randKey]
		delete (data, randKey)
		fmt.Println("deleter: ", from, " Deleted Key: ", randKey, " Deleted Value: ", val)
		mutex.Unlock()	
	}
	
}