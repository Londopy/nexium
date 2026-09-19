#include "cvendor.h"
#include <math.h>
#include <stdlib.h>

static int cv_add_two(int v) { return v + 2; }

cv_counter* cv_counter_new(int start) {
    cv_counter* c = (cv_counter*)malloc(sizeof *c);
    c->step = cv_add_two;
    c->value = start;
    return c;
}

int cv_counter_tick(cv_counter* c) {
    c->value = c->step(c->value);
    return c->value;
}

void cv_counter_free(cv_counter* c) { free(c); }

double cv_distance(cv_point a, cv_point b) {
    double dx = a.x - b.x, dy = a.y - b.y;
    return sqrt(dx * dx + dy * dy);
}

uint32_t cv_sum(const uint32_t* xs, size_t n) {
    uint32_t s = 0;
    for (size_t i = 0; i < n; i++) s += xs[i];
    return s;
}

int cv_mode_speed(int mode) {
    return mode == CV_FAST ? 100 : 10;
}
