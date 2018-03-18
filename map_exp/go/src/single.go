package main

import (
	"sync"
	"fmt"
	"time"
	"math/rand"
)

var data = make(map[int]int)
var mutex = &sync.Mutex{}
var wg sync.WaitGroup

func main() {
	fmt.Println("numGoroutines numTrial accessType totalOps totalDur opsPerSecond")

	for trialNumber := 1; trialNumber <= 3; trialNumber++ {
		timeStart := time.Now()
		numOperations := 0

		for time.Now().Before(timeStart.Add((5 * time.Second))) {
			//just some random key values
			var constant = rand.Int() %256
			data[constant] = constant
			numOperations += 1
		}
		accessType := "w"
		fmt.Println(1, trialNumber, accessType, numOperations, time.Since(timeStart), float64(numOperations)/time.Since(timeStart).Seconds())
	}

	for trialNumber := 1; trialNumber <= 3; trialNumber++ {
		timeStart := time.Now()
		numOperations := 0

		for time.Now().Before(timeStart.Add((5 * time.Second))) {
			//just some random key values
			var constant = rand.Int() %256
			_ = data[constant]
			numOperations += 1
		}
		accessType := "r"
		fmt.Println(1, trialNumber, accessType, numOperations, time.Since(timeStart), float64(numOperations)/time.Since(timeStart).Seconds())
	}

	for trialNumber := 1; trialNumber <= 3; trialNumber++ {
		timeStart := time.Now()
		numOperations := 0

		for time.Now().Before(timeStart.Add((5 * time.Second))) {
			//just some random key values
			var constant = rand.Int() %256
			if constant % 2 == 0 {
				data[constant] = constant
				
			} else{
				_ = data[constant]
			}
			numOperations += 1	
		}
		accessType := "rw"
		fmt.Println(1, trialNumber, accessType, numOperations, time.Since(timeStart), float64(numOperations)/time.Since(timeStart).Seconds())
	}
}