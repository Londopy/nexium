#include <stdio.h>
#include <stdlib.h>
#define LIMIT 100000000
int main(void) {
    unsigned char* composite = calloc(LIMIT, 1);
    long long count = 0;
    for (long long i = 2; i < LIMIT; i++) {
        if (!composite[i]) {
            count++;
            for (long long j = i * i; j < LIMIT; j += i) composite[j] = 1;
        }
    }
    printf("%lld\n", count);
    free(composite);
    return 0;
}
