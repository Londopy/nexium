#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#define WORDS 10000000
/* an open-addressing table: what a C program keeps at hand */
#define CAP 131072
typedef struct { char key[16]; long count; int used; } slot;
static slot table[CAP];
static uint64_t hash(const char* s) { uint64_t h = 1469598103934665603ULL; for (; *s; s++) { h ^= (unsigned char)*s; h *= 1099511628211ULL; } return h; }
int main(void) {
    uint64_t seed = 42;
    long total = 0, distinct = 0;
    char word[16];
    for (long i = 0; i < WORDS; i++) {
        seed = seed * 6364136223846793005ULL + 1442695040888963407ULL;
        uint64_t k = (seed >> 33) % 50000;
        snprintf(word, sizeof word, "w%llu", (unsigned long long)k);
        uint64_t h = hash(word) & (CAP - 1);
        while (table[h].used && strcmp(table[h].key, word) != 0) h = (h + 1) & (CAP - 1);
        if (!table[h].used) { table[h].used = 1; strcpy(table[h].key, word); distinct++; }
        table[h].count++;
        total++;
    }
    printf("%ld %ld\n", distinct, total);
    return 0;
}
