#include "cspec.h"
#include <stdlib.h>
#include <stdarg.h>
cs_pair cs_swap(cs_pair p) { cs_pair r; r.a = p.b; r.b = p.a; return r; }
int64_t cs_total(const int32_t* xs, size_t n) { int64_t t = 0; for (size_t i = 0; i < n; i++) t += xs[i]; return t; }
cs_id cs_next(cs_id id) { return id + 1; }
int cs_kind_value(int k) { return k * 10; }
cs_opaque* cs_opaque_new(int n) { cs_opaque* o = malloc(sizeof *o); o->hook = 0; o->n = n; return o; }
int cs_opaque_get(const cs_opaque* o) { return o->n; }
void cs_opaque_free(cs_opaque* o) { free(o); }
int cs_count_args(int n, ...) { va_list ap; va_start(ap, n); int s = 0; for (int i = 0; i < n; i++) s += va_arg(ap, int); va_end(ap); return s; }
