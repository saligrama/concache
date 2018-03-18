package main

import (
	"math/rand"
	"sync"
	"fmt"
	"time"
	"sync/atomic"
	"os"
)

func main() {
	fmt.Println("numGoroutines numTrial accessType totalOps totalDur opsPerSecond")
	for numGoroutines := 1; numGoroutines < 9; numGoroutines++ {
		for trialNumber := 1; trialNumber <= 3; trialNumber++ {
			if len(os.Args) == 2 {
				val, dur := trial(numGoroutines, 5, os.Args[1])
				fmt.Println(numGoroutines, trialNumber, os.Args[1], val, dur, float64(val)/dur.Seconds())
			} else {
				fmt.Println("Not proper number of argument given.")
			}
		}
	}
}

func trial (numGoroutines int, threadDuration int, readWrite string) (uint64, time.Duration) {
	var data = make(map[int]int)
	var mutex = &sync.RWMutex{}
	var wg sync.WaitGroup
	var ops uint64

	rand.Seed(time.Now().UnixNano()) //generate seed

	wg.Add(numGoroutines) //reader, writer


	timeStart := time.Now()

	for i:=0; i < numGoroutines; i++ {
		go func(from int) {
			defer wg.Done()
			var numOperations uint64 = 0

			for time.Now().Before(timeStart.Add((time.Duration(threadDuration) * time.Second))) {
				//just some random key/values
				for i := 0; i < 10000; i++ {
					var constant = rand.Int()%2 //read or write
					if readWrite == "rw" {
						fmt.Println("RW here")
						if constant % 2 == 0 {
							mutex.Lock()
							data[constant] = constant
							mutex.Unlock()
							numOperations += 1
						} else {
							mutex.RLock()
							_ = data[constant]
							mutex.RUnlock()
							numOperations += 1
						}
					} else if readWrite == "r" {
						mutex.Lock()
						data[constant] = constant
						mutex.Unlock()
						numOperations += 1
					} else if readWrite == "w" {
						mutex.RLock()
						_ = data[constant]
						mutex.RUnlock()
						numOperations += 1
					} else {
						fmt.Println("Not proper choice.")
						break
					}
					
				}
			}
			// fmt.Println("Number of Operations from Writer #", from, ": ", numOperations)
			atomic.AddUint64(&ops, numOperations)
		} (i)
	}
	wg.Wait() //wait for the goroutines to finish
	totalDuration := time.Since(timeStart)
	opsFinal := atomic.LoadUint64(&ops)
	
	// fmt.Println(opsFinal)

    return opsFinal, totalDuration
}
