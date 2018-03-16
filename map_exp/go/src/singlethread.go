package main

import (
	// "bufio"
	"sync"
	"fmt"
	"time"
	"math/rand"
)

var data = make(map[int]int)
var mutex = &sync.Mutex{}
var wg sync.WaitGroup

func main() {
	wg.Add(1)

	timeStart := time.Now()

	fmt.Println("Running. ")
	go writer(timeStart)
	wg.Wait()
}

func writer(timeStart time.Time) {
	defer wg.Done()
	numOperations := 0

	for time.Now().Before(timeStart.Add((30 * time.Second))) {
		//just some random key values
		var randKey = rand.Int() %256
		var randValue = rand.Int() %256
		data[randKey] = randValue
		numOperations += 1
	}
	fmt.Println("Number of Operations: ", numOperations)
}