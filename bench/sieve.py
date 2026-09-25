LIMIT = 100000000

def main():
    composite = bytearray(LIMIT)
    count = 0
    for i in range(2, LIMIT):
        if not composite[i]:
            count += 1
            composite[i * i::i] = b"\x01" * len(range(i * i, LIMIT, i))
    print(count)

main()
