package main 

import (
	"fmt"
	"sync/atomic"
	"sync"
)

var counter = 0
var atomic_counter uint64 = 0

func main() {
	//Test Non-Atomic Counter
	num_threads := 10000
	var wg sync.WaitGroup
	wg.Add(num_threads)
	for i := 0; i < num_threads; i++ {
		go func(num_increments int) {
			defer wg.Done()
			for j := 0; j < num_increments; j++ {
				counter++
			}
		} (10000)
	}
	wg.Wait() //wait for Goroutines to finish

	//Test Atomic Counter

	wg.Add(num_threads)
	for i := 0; i < num_threads; i++ {
		go func(num_increments int) {
			defer wg.Done()
			for j := 0; j < num_increments; j++ {
				atomic.AddUint64(&atomic_counter, 1)
			}
		} (10000)
	}
	wg.Wait() //wait for Goroutines to finish
	fmt.Println ("Counter: ", counter)
	fmt.Println ("Atomic: ", atomic_counter)
}