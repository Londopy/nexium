/* Vendored C for the section 14 case. */
#ifndef CSPEC_H
#define CSPEC_H
#include <stdint.h>
#include <stddef.h>
#define CSPEC_LIMIT 7
#define CSPEC_RATIO 0.5
#define CSPEC_TAG "tag"
typedef struct cs_pair { int32_t a; int32_t b; } cs_pair;
typedef int32_t cs_id;
enum cs_kind { CS_ONE = 1, CS_TWO = 2 };
typedef struct cs_opaque { void (*hook)(void); int n; } cs_opaque;
cs_pair cs_swap(cs_pair p);
int64_t cs_total(const int32_t* xs, size_t n);
cs_id cs_next(cs_id id);
int cs_kind_value(int k);
cs_opaque* cs_opaque_new(int n);
int cs_opaque_get(const cs_opaque* o);
void cs_opaque_free(cs_opaque* o);
int cs_count_args(int n, ...);
#endif
