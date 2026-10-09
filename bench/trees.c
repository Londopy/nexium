#include <stdio.h>
#include <stdlib.h>
#define MIN_DEPTH 4
#define MAX_DEPTH 17
typedef struct node { struct node *left, *right; } node;
static node* make(int depth) {
    node* n = malloc(sizeof *n);
    if (depth == 0) { n->left = NULL; n->right = NULL; }
    else { n->left = make(depth - 1); n->right = make(depth - 1); }
    return n;
}
static long count(const node* n) { return n->left ? 1 + count(n->left) + count(n->right) : 1; }
static void drop(node* n) { if (n->left) { drop(n->left); drop(n->right); } free(n); }
int main(void) {
    long trees = 0, nodes = 0;
    node* stretch = make(MAX_DEPTH + 1);
    nodes += count(stretch); trees++;
    drop(stretch);
    node* long_lived = make(MAX_DEPTH);
    for (int depth = MIN_DEPTH; depth <= MAX_DEPTH; depth += 2) {
        long iterations = 1L << (MAX_DEPTH - depth + MIN_DEPTH);
        for (long i = 0; i < iterations; i++) {
            node* t = make(depth);
            nodes += count(t); trees++;
            drop(t);
        }
    }
    nodes += count(long_lived); trees++;
    drop(long_lived);
    printf("%ld %ld\n", trees, nodes);
    return 0;
}
