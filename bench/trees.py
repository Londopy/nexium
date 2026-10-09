MIN_DEPTH = 4
MAX_DEPTH = 17

class Node:
    def __init__(self, left, right):
        self.left = left
        self.right = right

def make(depth):
    if depth == 0:
        return Node(None, None)
    return Node(make(depth - 1), make(depth - 1))

def count(n):
    if n.left is None:
        return 1
    return 1 + count(n.left) + count(n.right)

def main():
    trees = 0
    nodes = 0
    stretch = make(MAX_DEPTH + 1)
    nodes += count(stretch)
    trees += 1
    del stretch
    long_lived = make(MAX_DEPTH)
    for depth in range(MIN_DEPTH, MAX_DEPTH + 1, 2):
        iterations = 1 << (MAX_DEPTH - depth + MIN_DEPTH)
        for _ in range(iterations):
            t = make(depth)
            nodes += count(t)
            trees += 1
    nodes += count(long_lived)
    trees += 1
    print(trees, nodes)

main()
