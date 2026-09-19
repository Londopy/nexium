/* A small vendored C library used by examples/cimport.nx (spec 17.1). */
#ifndef CVENDOR_H
#define CVENDOR_H
#include <stddef.h>
#include <stdint.h>

#define CVENDOR_VERSION 3
#define CVENDOR_NAME "cvendor"

typedef struct cv_point { double x; double y; } cv_point;

enum cv_mode { CV_FAST = 1, CV_SLOW = 2 };

double cv_distance(cv_point a, cv_point b);
uint32_t cv_sum(const uint32_t* xs, size_t n);
int cv_mode_speed(int mode);

#endif
