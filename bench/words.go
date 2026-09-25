package main

import "fmt"

const words = 10000000

func main() {
	counts := make(map[string]int64)
	var seed uint64 = 42
	var total int64
	for i := 0; i < words; i++ {
		seed = seed*6364136223846793005 + 1442695040888963407
		k := (seed >> 33) % 50000
		word := fmt.Sprintf("w%d", k)
		counts[word]++
		total++
	}
	fmt.Println(len(counts), total)
}
