package main

import (
	"math/rand"
	"sync"
	"fmt"
	"time"
)

var data = make(map[int]int)
var mutex = &sync.Mutex{}
var wg sync.WaitGroup

func main() {
	rand.Seed(time.Now().UnixNano()) //generate seed

	var numGoroutines = 1
	wg.Add(numGoroutines*2) //reader, writer


	startTime := time.Now()

	for i:=0; i < numGoroutines; i++ {
		go writer(i, startTime)
	}

	for i:=0; i < numGoroutines; i++ {
		go reader(i, startTime)
	}

	wg.Wait() //wait for the goroutines to finish (they never will)
}

func writer (from int, timeStart time.Time) {
	defer wg.Done()
	numOperations := 0

	for time.Now().Before(timeStart.Add((30 * time.Second))) {
		//just some random key values
		var randKey = rand.Int() %256
		var randValue = rand.Int() %256
		mutex.Lock()
		data[randKey] = randValue
		mutex.Unlock()
		numOperations += 1
	}
	fmt.Println("Number of Operations from Writer: ", numOperations)
}

func reader(from int, timeStart time.Time) {
	defer wg.Done()
	numOperations := 0

	for time.Now().Before(timeStart.Add((30 * time.Second))) {
		//just some random key
		var randKey = rand.Int() %256
		mutex.Lock()
		_ = data[randKey]
		mutex.Unlock()
		numOperations += 1
	}
	fmt.Println("Number of Operations from Reader: ", numOperations)
}