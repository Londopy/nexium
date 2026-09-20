/* A small vendored C library used by examples/cimport.nx (spec 17.1). */
#ifndef CVENDOR_H
#define CVENDOR_H
#include <stddef.h>
#include <stdint.h>

#define CVENDOR_VERSION 3
#define CVENDOR_NAME "cvendor"

typedef struct cv_point { double x; double y; } cv_point;

enum cv_mode { CV_FAST = 1, CV_SLOW = 2 };

/* a handle whose definition holds a function pointer: importable only as an
 * opaque type, used through pointers (the shape of Apple's FILE) */
typedef struct cv_counter { int (*step)(int); int value; } cv_counter;
cv_counter* cv_counter_new(int start);
int cv_counter_tick(cv_counter* c);
void cv_counter_free(cv_counter* c);

double cv_distance(cv_point a, cv_point b);
uint32_t cv_sum(const uint32_t* xs, size_t n);
int cv_mode_speed(int mode);

#endif
