package main

import (
	"math/rand"
	"sync"
	"fmt"
	"time"
	"sync/atomic"
	// "reflect"
	"strconv"
	"os"
	"encoding/csv"
)

func main() {
	// var data = [][]string{{"Line1", "Hello Readers of"}, {"Line2", "golangcode.com"}}
	// fmt.Println(reflect.TypeOf(data))
	var data = [41][4]string{{}}
	data[0][0] = "Number Of Threads"
	data[0][1] = "Trial 1"
	data[0][2] = "Trial 2"
	data[0][3] = "Trial 3"
	fmt.Println(data)


	for numThreads := 1; numThreads < 41; numThreads++ {
		// var numWriters int = numThreads
		// var numReaders int = numThreads
		data[numThreads][0] = strconv.Itoa(numThreads)
		for trialNumber := 1; trialNumber <= 3; trialNumber++ {
			val := trial(numThreads, 30)
			data[numThreads][trialNumber] = strconv.FormatUint(val, 10)
			fmt.Println(numThreads, val)
		}
	}
	fmt.Println(data)
	fmt.Println(len(data))
	file, err := os.Create("result_rwlock.csv")
	if err != nil {
		fmt.Println("couldn't create file")
	}
    defer file.Close()

    writer := csv.NewWriter(file)
    defer writer.Flush()

    for i := 0; i < len(data); i++ {
	numberOfThreads := data[i][0]
	trial1 := data[i][1]
	trial2 := data[i][2]
	trial3 := data[i][3]
        writer.Write([]string{numberOfThreads, trial1, trial2, trial3})
    }
}

func trial (numThreads int, threadDuration int) uint64 {
	var data = make(map[int]int)
	var mutex = &sync.RWMutex{}
	var wg sync.WaitGroup
	var ops uint64

	rand.Seed(time.Now().UnixNano()) //generate seed

	wg.Add(numThreads) //reader, writer


	timeStart := time.Now()
	for i:=0; i < numThreads; i++ {
		go func() {
			defer wg.Done()
			var numOperations uint64 = 0

			for time.Now().Before(timeStart.Add((time.Duration(threadDuration) * time.Second))) {
				//just some random key/values
				for i := 0; i < 10000; i++ {
					var readOrWrite = rand.Int()%2 //read or write
					var randKey = rand.Int() %256 //generate key
					if readOrWrite % 2 == 0 {
						var randValue = rand.Int() %256
						mutex.Lock()
						data[randKey] = randValue
						mutex.Unlock()
					} else {
						mutex.RLock()
						_ = data[randKey]
						mutex.RUnlock()
					}
					numOperations += 1
				}
			}
			// fmt.Println("Number of Operations from Writer #", from, ": ", numOperations)
			atomic.AddUint64(&ops, numOperations)
		} ()
	}
	wg.Wait() //wait for the goroutines to finish

	opsFinal := atomic.LoadUint64(&ops)
    // fmt.Println("ops:", opsFinal)
    return opsFinal
}
