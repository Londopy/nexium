WORDS = 2000000

def main():
    counts = {}
    seed = 42
    total = 0
    for _ in range(WORDS):
        seed = (seed * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        k = (seed >> 33) % 50000
        word = "w%d" % k
        counts[word] = counts.get(word, 0) + 1
        total += 1
    print(len(counts), total)

main()
