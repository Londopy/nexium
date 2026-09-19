#include "cvendor.h"
#include <math.h>

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
