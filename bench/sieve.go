package main

import "fmt"

const limit = 10000000

func main() {
	composite := make([]bool, limit)
	count := 0
	for i := 2; i < limit; i++ {
		if !composite[i] {
			count++
			for j := i * i; j < limit; j += i {
				composite[j] = true
			}
		}
	}
	fmt.Println(count)
}
