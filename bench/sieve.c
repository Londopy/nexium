#include <stdio.h>
#include <stdlib.h>
#define LIMIT 10000000
int main(void) {
    unsigned char* composite = calloc(LIMIT, 1);
    long count = 0;
    for (long i = 2; i < LIMIT; i++) {
        if (!composite[i]) {
            count++;
            for (long j = i * i; j < LIMIT; j += i) composite[j] = 1;
        }
    }
    printf("%ld\n", count);
    free(composite);
    return 0;
}
