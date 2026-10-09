package main

import "fmt"

const minDepth = 4
const maxDepth = 17

type node struct{ left, right *node }

func build(depth int) *node {
	if depth == 0 {
		return &node{}
	}
	return &node{build(depth - 1), build(depth - 1)}
}

func count(n *node) int64 {
	if n.left == nil {
		return 1
	}
	return 1 + count(n.left) + count(n.right)
}

func main() {
	var trees, nodes int64
	{
		stretch := build(maxDepth + 1)
		nodes += count(stretch)
		trees++
	}
	longLived := build(maxDepth)
	for depth := minDepth; depth <= maxDepth; depth += 2 {
		iterations := 1 << (maxDepth - depth + minDepth)
		for i := 0; i < iterations; i++ {
			t := build(depth)
			nodes += count(t)
			trees++
		}
	}
	nodes += count(longLived)
	trees++
	fmt.Println(trees, nodes)
}
