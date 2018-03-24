package main

import (
	"math/rand"
	"sync"
	"fmt"
	"time"
	"sync/atomic"
	"os"
	"strconv"
	"runtime"
	"runtime/pprof"
	"log"
	"flag"
)

var cpuprofile = flag.String("cpuprofile", "", "write cpu profile to `file`")

func main() {
	fmt.Println("Start Time:", time.Now())
    flag.Parse()
    if *cpuprofile != "" {
        f, err := os.Create(*cpuprofile)
        if err != nil {
            log.Fatal(err)
        }
        pprof.StartCPUProfile(f)
        defer pprof.StopCPUProfile()
    }

	// path, _ := strconv.Atoi(os.Args[0])
	first, _ := strconv.Atoi(os.Args[1])
	last, _ := strconv.Atoi(os.Args[2])
	numTrials, _ := strconv.Atoi(os.Args[3])

	length := 30
	fmt.Println("numGoroutines numTrials totalOps(r) opsPerSecond(r) totalDur(r)")
	for numGoroutines := first; numGoroutines <= last; numGoroutines++ {
		runtime.GOMAXPROCS(numGoroutines)
		for trialNumber := 1; trialNumber <= numTrials; trialNumber++ {
			val, dur := trial(numGoroutines, length, "r")
			fmt.Println(numGoroutines, trialNumber, val, float64(val)/dur.Seconds(), dur)
		}
	}

	fmt.Println("numGoroutines numTrials totalOps(w) opsPerSecond(w) totalDur(w)")
	for numGoroutines := first; numGoroutines <= last; numGoroutines++ {
		runtime.GOMAXPROCS(numGoroutines)
		for trialNumber := 1; trialNumber <= numTrials; trialNumber++ {
			val, dur := trial(numGoroutines, length, "w")
			fmt.Println(numGoroutines, trialNumber, val, float64(val)/dur.Seconds(), dur)
		}
	}

	fmt.Println("numGoroutines numTrials totalOps(rw) opsPerSecond(rw) totalDur(rw)")
	for numGoroutines := first; numGoroutines <= last; numGoroutines++ {
		runtime.GOMAXPROCS(numGoroutines)
		for trialNumber := 1; trialNumber <= numTrials; trialNumber++ {
			val, dur := trial(numGoroutines, length, "rw")
			fmt.Println(numGoroutines, trialNumber, val, float64(val)/dur.Seconds(), dur)
		}
	}
	fmt.Println("End Time:", time.Now())
	f.Close()
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
					} else if readWrite == "w" {
						mutex.Lock()
						data[constant] = constant
						mutex.Unlock()
						numOperations += 1
					} else if readWrite == "r" {
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