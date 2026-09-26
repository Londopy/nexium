/* Nexium runtime. Embedded into every generated translation unit.
 *
 * Design constraints (specification section 4.1):
 *   S1  no initialization: every function here works from any thread with no setup.
 *   S2  no process-global state: the only static is a thread-local panic boundary,
 *       which is per-thread and per-translation-unit.
 *   S3  panics do not cross an export boundary: nx_panic longjmps to the nearest
 *       boundary when one is installed, and aborts the process otherwise.
 */
#ifndef NX_RT_H
#define NX_RT_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <string.h>
#include <stdio.h>
#include <stdlib.h>
/* NX_WASM: built for wasm32-wasi, the playground. The platform has no
   processes, sockets, terminal or setjmp; those parts fail with the error a
   program would see when the operating system refuses. */
#if defined(__wasi__) && !defined(NX_WASM)
#define NX_WASM 1
#endif
#if defined(NX_WASM)
typedef int jmp_buf[1];
#define setjmp(b) ((void)(b), 0)
#define longjmp(b, v) ((void)(b), (void)(v), abort())
#else
#include <setjmp.h>
#endif
#include <errno.h>
#include <sys/stat.h>
#include <math.h>
#include <time.h>

#if defined(_WIN32)
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <io.h>
#include <fcntl.h>
#include <direct.h>
#elif defined(NX_WASM)
#include <sys/time.h>
#include <unistd.h>
#include <dirent.h>
#include <fcntl.h>
extern char** environ;
#else
#include <sys/time.h>
#include <unistd.h>
#include <termios.h>
#include <poll.h>
#include <signal.h>
#include <spawn.h>
#include <sys/wait.h>
#include <dirent.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <sys/select.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <arpa/inet.h>
#include <netdb.h>
extern char** environ;
#if defined(__APPLE__)
#include <mach-o/dyld.h>
#endif
#if defined(__linux__)
#include <sys/syscall.h>
#endif
#endif

#if defined(_MSC_VER) && !defined(__clang__)
#define NX_THREAD_LOCAL __declspec(thread)
#define NX_NORETURN __declspec(noreturn)
#else
#define NX_THREAD_LOCAL _Thread_local
#define NX_NORETURN _Noreturn
#endif
#define NX_INLINE static inline
#if defined(_WIN32) && defined(NX_BUILD_SHARED)
#define NX_EXPORT __declspec(dllexport)
#elif defined(NX_BUILD_SHARED)
#define NX_EXPORT __attribute__((visibility("default")))
#else
#define NX_EXPORT
#endif
#define NX_UNUSED(x) (void)(x)

typedef __int128 nx_i128;
typedef unsigned __int128 nx_u128;
#define NX_I128_MAX ((nx_i128)((((nx_u128)1) << 127) - 1))
#define NX_I128_MIN ((nx_i128)(-NX_I128_MAX - 1))

/* ------------------------------------------------------------------ slices */
typedef struct nx_sl_u8 { uint8_t* ptr; size_t len; } nx_sl_u8;
struct nx_arena;
/* Growable containers remember the arena they were created in (NULL = the
 * root allocator), so a container created outside a `using arena` block keeps
 * its storage on the heap even when it grows inside the block. */
typedef struct nx_string { uint8_t* ptr; size_t len; size_t cap; struct nx_arena* ar; } nx_string;
typedef struct nx_rawlist { void* ptr; size_t len; size_t cap; struct nx_arena* ar; } nx_rawlist;

NX_INLINE nx_sl_u8 nx_lit(const char* s, size_t n) { nx_sl_u8 r; r.ptr = (uint8_t*)s; r.len = n; return r; }

/* --------------------------------------------------------------- allocator */
typedef struct nx_alloc {
    void* (*alloc)(void* state, size_t size, size_t align);
    void* (*realloc)(void* state, void* p, size_t old_size, size_t new_size, size_t align);
    void (*free)(void* state, void* p, size_t size);
    void* state;
} nx_alloc;

typedef struct nx_ctx {
    nx_alloc alloc;
    /* the root (non-arena) allocator, and the innermost arena in scope */
    nx_alloc base;
    struct nx_arena* arena;
    uint64_t rng;
    bool rng_seeded;
    int argc;
    char** argv;
    FILE* out;
    FILE* err;
    /* leak tracking (only maintained when built with -DNX_LEAK_CHECK) */
    size_t live_allocs;
    size_t live_bytes;
    size_t total_allocs;
    size_t peak_bytes;
    /* os.args(): built once, owned by the context */
    nx_sl_u8* args_cache;
    size_t args_len;
} nx_ctx;

/* ------------------------------------------------------------------ panics */
struct nx_tracker;
typedef struct nx_boundary {
    jmp_buf jb;
    char msg[256];
    char loc[128];
    /* the resources of the export call this boundary belongs to, or NULL */
    struct nx_tracker* track;
} nx_boundary;
/* files (kind 0), sockets (1) and held locks (2) register with the boundary's
   tracker as they are acquired and released; defined with the tracker below */
static void nx_track_handle(int kind, int64_t h, bool acquire);
static void nx_ctx_untrack(nx_ctx* c);

/* The runtime's state. In one C file (the default) each variable is static;
   a program compiled as several (`nx build` of a debug build, one C file per
   module: NX_RT_SHARED) shares one copy, which the unit with NX_RT_OWNER
   defines and the others declare. */
#if defined(NX_RT_SHARED) && !defined(NX_RT_OWNER)
#define NX_STATE extern
#define NX_STATE_INIT(v)
#elif defined(NX_RT_SHARED)
#define NX_STATE
#define NX_STATE_INIT(v) = v
#else
#define NX_STATE static
#define NX_STATE_INIT(v) = v
#endif

NX_STATE NX_THREAD_LOCAL nx_boundary* nx_tls_boundary NX_STATE_INIT(NULL);
NX_STATE NX_THREAD_LOCAL char nx_tls_last_panic[256];

NX_NORETURN NX_INLINE void nx_panic(const char* msg, const char* loc) {
    if (nx_tls_boundary) {
        nx_boundary* b = nx_tls_boundary;
        snprintf(b->msg, sizeof b->msg, "%s", msg);
        snprintf(b->loc, sizeof b->loc, "%s", loc ? loc : "");
        snprintf(nx_tls_last_panic, sizeof nx_tls_last_panic, "%s (at %s)", msg, loc ? loc : "?");
        longjmp(b->jb, 1);
    }
    fprintf(stderr, "panic: %s\n  at %s\n", msg, loc ? loc : "?");
    fflush(stderr);
    abort();
}

NX_NORETURN NX_INLINE void nx_panic_bounds(size_t i, size_t len, const char* loc) {
    char buf[128];
    snprintf(buf, sizeof buf, "index %zu out of bounds for length %zu", i, len);
    nx_panic(buf, loc);
}

/* pointer + offset that is defined for a null pointer: an empty slice has no
 * storage, and `NULL + 0` is undefined in C (UBSan traps it) */
#define nx_padd(p, n) ((n) ? (p) + (n) : (p))

NX_INLINE size_t nx_idx(size_t i, size_t len, const char* loc) {
    if (i >= len) nx_panic_bounds(i, len, loc);
    return i;
}

NX_INLINE void nx_slice_check(size_t start, size_t end, size_t len, const char* loc) {
    if (start > end || end > len) {
        char buf[128];
        snprintf(buf, sizeof buf, "slice %zu..%zu out of range for length %zu", start, end, len);
        nx_panic(buf, loc);
    }
}

NX_INLINE nx_i128 nx_cast_check(nx_i128 v, nx_i128 lo, nx_i128 hi, const char* loc) {
    if (v < lo || v > hi) nx_panic("value does not fit the target type", loc);
    return v;
}

NX_INLINE int64_t nx_f2i(double f, nx_i128 lo, nx_i128 hi, const char* loc) {
    if (!(f == f) || f < (double)lo || f > (double)hi) nx_panic("float to integer cast out of range", loc);
    return (int64_t)f;
}

/* ------------------------------------------------------- default allocator */
#ifdef NX_LEAK_CHECK
/* the tracking allocator keeps its counters in the context (no globals) */
NX_INLINE void nx_track(void* st, ptrdiff_t allocs, ptrdiff_t bytes) {
    struct nx_ctx* c = (struct nx_ctx*)st;
    if (!c) return;
    c->live_allocs = (size_t)((ptrdiff_t)c->live_allocs + allocs);
    c->live_bytes = (size_t)((ptrdiff_t)c->live_bytes + bytes);
    if (allocs > 0) c->total_allocs++;
    if (c->live_bytes > c->peak_bytes) c->peak_bytes = c->live_bytes;
}
#endif
NX_INLINE void* nx_malloc_alloc(void* st, size_t size, size_t align) {
    NX_UNUSED(st); NX_UNUSED(align);
    void* p = malloc(size ? size : 1);
    if (!p) nx_panic("out of memory", "allocator");
#ifdef NX_LEAK_CHECK
    nx_track(st, 1, (ptrdiff_t)size);
#endif
    return p;
}
NX_INLINE void* nx_malloc_realloc(void* st, void* p, size_t old_size, size_t new_size, size_t align) {
    NX_UNUSED(st); NX_UNUSED(old_size); NX_UNUSED(align);
    void* q = realloc(p, new_size ? new_size : 1);
    if (!q) nx_panic("out of memory", "allocator");
#ifdef NX_LEAK_CHECK
    nx_track(st, p ? 0 : 1, (ptrdiff_t)new_size - (ptrdiff_t)old_size);
#endif
    return q;
}
NX_INLINE void nx_malloc_free(void* st, void* p, size_t size) {
    NX_UNUSED(st); NX_UNUSED(size);
#ifdef NX_LEAK_CHECK
    if (p) nx_track(st, -1, -(ptrdiff_t)size);
#endif
    free(p);
}
NX_INLINE void nx_leak_report(struct nx_ctx* c) {
#ifdef NX_LEAK_CHECK
    fflush(stdout);
    if (c->live_allocs == 0) {
        fprintf(stderr, "leaks: none (%zu allocation(s), peak %zu bytes)\n", c->total_allocs, c->peak_bytes);
    } else {
        fprintf(stderr, "leaks: %zu allocation(s) still live at exit, %zu bytes (of %zu total, peak %zu bytes)\n", c->live_allocs, c->live_bytes, c->total_allocs, c->peak_bytes);
        fprintf(stderr, "       a live `ref class` cycle or a value moved into a container that was never released is the usual cause (spec 5.4: use `weak` at back edges)\n");
    }
#else
    NX_UNUSED(c);
#endif
}

NX_INLINE nx_ctx nx_default_ctx(int argc, char** argv) {
    nx_ctx c;
    c.alloc.alloc = nx_malloc_alloc;
    c.alloc.realloc = nx_malloc_realloc;
    c.alloc.free = nx_malloc_free;
    c.alloc.state = NULL;
    c.base = c.alloc;
    c.arena = NULL;
    c.rng = 0x9E3779B97F4A7C15ULL;
    c.rng_seeded = false;
    c.argc = argc;
    c.argv = argv;
    c.out = stdout;
    c.err = stderr;
    c.live_allocs = 0; c.live_bytes = 0; c.total_allocs = 0; c.peak_bytes = 0;
    c.args_cache = NULL; c.args_len = 0;
#if defined(_WIN32)
    /* byte-exact output on every platform: no CRLF translation */
    _setmode(_fileno(stdin), _O_BINARY);
    _setmode(_fileno(stdout), _O_BINARY);
    _setmode(_fileno(stderr), _O_BINARY);
#endif
    return c;
}
/* the architecture this program runs on; the driver chooses a CPU baseline by it */
NX_INLINE nx_sl_u8 nx_host_arch(void) {
#if defined(__x86_64__) || defined(_M_X64)
    return nx_lit("x86_64", 6);
#elif defined(__aarch64__) || defined(_M_ARM64)
    return nx_lit("aarch64", 7);
#elif defined(__i386__) || defined(_M_IX86)
    return nx_lit("x86", 3);
#elif defined(__arm__) || defined(_M_ARM)
    return nx_lit("arm", 3);
#elif defined(__riscv) && (__riscv_xlen == 64)
    return nx_lit("riscv64", 7);
#else
    return nx_lit("unknown", 7);
#endif
}
/* the operating system this program runs on, and the pointer width in bits:
   `@target()` is (os, arch, bits), a constant of the C build */
NX_INLINE nx_sl_u8 nx_host_os(void) {
#if defined(_WIN32)
    return nx_lit("windows", 7);
#elif defined(__APPLE__)
    return nx_lit("macos", 5);
#elif defined(__linux__)
    return nx_lit("linux", 5);
#elif defined(__FreeBSD__) || defined(__OpenBSD__) || defined(__NetBSD__)
    return nx_lit("bsd", 3);
#else
    return nx_lit("unknown", 7);
#endif
}
#define NX_PTR_BITS ((uint32_t)(sizeof(void*) * 8))
/* UTF-8 on the Windows console for the program's life (the console's own code
   page shows `é` as two symbols); the previous page comes back at exit */
#if defined(_WIN32)
NX_STATE UINT nx_prev_console_cp NX_STATE_INIT(0);
static void nx_console_restore(void) { if (nx_prev_console_cp) SetConsoleOutputCP(nx_prev_console_cp); }
#endif
NX_INLINE void nx_console_utf8(void) {
#if defined(_WIN32)
    UINT cur = GetConsoleOutputCP();
    if (cur != 0 && cur != 65001) { nx_prev_console_cp = cur; SetConsoleOutputCP(65001); atexit(nx_console_restore); }
#endif
}
/* the tracking allocator needs the context as its state; installed by entry points */
NX_INLINE void nx_ctx_track_self(nx_ctx* c) {
#ifdef NX_LEAK_CHECK
    if (c->alloc.alloc == nx_malloc_alloc) c->alloc.state = c;
    if (c->base.alloc == nx_malloc_alloc) c->base.state = c;
#else
    NX_UNUSED(c);
#endif
}

NX_INLINE void* nx_alloc_bytes(nx_ctx* c, size_t size, size_t align) { return c->alloc.alloc(c->alloc.state, size, align); }
/* A debug build fills storage with a fixed byte before freeing it, so a
 * view that outlived its storage (specification 5.6) reads garbage or
 * panics on its length instead of yielding the old contents by luck. */
NX_INLINE void nx_free_bytes(nx_ctx* c, void* p, size_t size) {
    if (!p) return;
#ifdef NX_MODE_DEBUG
    memset(p, 0xDD, size);
#endif
    c->alloc.free(c->alloc.state, p, size);
}
NX_INLINE void nx_ctx_release(nx_ctx* c) {
    if (c->args_cache) { nx_free_bytes(c, c->args_cache, (c->args_len ? c->args_len : 1) * sizeof(nx_sl_u8)); c->args_cache = NULL; }
}
NX_INLINE nx_sl_u8* nx_args(nx_ctx* c, size_t* n) {
    if (!c->args_cache) {
        size_t k = c->argc > 0 ? (size_t)c->argc : 0;
        c->args_cache = (nx_sl_u8*)nx_alloc_bytes(c, (k ? k : 1) * sizeof(nx_sl_u8), 8);
        for (size_t i = 0; i < k; i++) { c->args_cache[i].ptr = (uint8_t*)c->argv[i]; c->args_cache[i].len = strlen(c->argv[i]); }
        c->args_len = k;
    }
    *n = c->args_len;
    return c->args_cache;
}

/* ----------------------------------------------------------------- arena */
/* A bump allocator over chunks taken from the parent context. `free` is a
 * no-op; everything is released when the `using arena { }` block ends. */
typedef struct nx_arena_chunk { struct nx_arena_chunk* next; size_t cap; size_t used; } nx_arena_chunk;
typedef struct nx_arena { nx_ctx* parent; nx_arena_chunk* head; void* last; size_t last_size; } nx_arena;

NX_INLINE void* nx_arena_alloc(void* st, size_t size, size_t align) {
    nx_arena* a = (nx_arena*)st;
    if (align < 16) align = 16;
    size_t need = (size + align - 1) / align * align;
    nx_arena_chunk* ch = a->head;
    if (!ch || ch->used + need > ch->cap) {
        size_t cap = need > 65536 - sizeof(nx_arena_chunk) ? need + sizeof(nx_arena_chunk) : 65536;
        nx_arena_chunk* n = (nx_arena_chunk*)nx_alloc_bytes(a->parent, cap, 16);
        n->next = ch; n->cap = cap; n->used = (sizeof(nx_arena_chunk) + 15) / 16 * 16;
        a->head = ch = n;
    }
    void* p = (uint8_t*)ch + ch->used;
    ch->used += need;
    a->last = p; a->last_size = need;
    return p;
}
NX_INLINE bool nx_arena_owns(const nx_arena* a, const void* p) {
    for (const nx_arena_chunk* ch = a->head; ch; ch = ch->next)
        if ((const uint8_t*)p >= (const uint8_t*)ch && (const uint8_t*)p < (const uint8_t*)ch + ch->cap) return true;
    return false;
}
NX_INLINE void* nx_arena_realloc(void* st, void* p, size_t old_size, size_t new_size, size_t align) {
    nx_arena* a = (nx_arena*)st;
    if (p && !nx_arena_owns(a, p)) return a->parent->alloc.realloc(a->parent->alloc.state, p, old_size, new_size, align);
    if (p && p == a->last) {
        nx_arena_chunk* ch = a->head;
        size_t need = (new_size + 15) / 16 * 16;
        if (ch->used - a->last_size + need <= ch->cap) { ch->used = ch->used - a->last_size + need; a->last_size = need; return p; }
    }
    void* q = nx_arena_alloc(st, new_size, align);
    if (p && old_size) memcpy(q, p, old_size < new_size ? old_size : new_size);
    return q;
}
NX_INLINE void nx_arena_free(void* st, void* p, size_t size) {
    nx_arena* a = (nx_arena*)st;
    if (p && !nx_arena_owns(a, p)) a->parent->alloc.free(a->parent->alloc.state, p, size);
}
NX_INLINE nx_ctx nx_arena_begin(nx_ctx* parent, nx_arena* a) {
    a->parent = parent; a->head = NULL; a->last = NULL; a->last_size = 0;
    nx_ctx sub = *parent;
    sub.alloc.alloc = nx_arena_alloc; sub.alloc.realloc = nx_arena_realloc; sub.alloc.free = nx_arena_free; sub.alloc.state = a;
    sub.arena = a;
    return sub;
}
/* Container storage goes to the container's own arena, or to the root allocator. */
NX_INLINE void* nx_cont_alloc(nx_ctx* c, nx_arena* ar, size_t size, size_t align) {
    return ar ? nx_arena_alloc(ar, size, align) : c->base.alloc(c->base.state, size, align);
}
NX_INLINE void* nx_cont_realloc(nx_ctx* c, nx_arena* ar, void* p, size_t old_size, size_t new_size, size_t align) {
    return ar ? nx_arena_realloc(ar, p, old_size, new_size, align) : c->base.realloc(c->base.state, p, old_size, new_size, align);
}
NX_INLINE void nx_cont_free(nx_ctx* c, nx_arena* ar, void* p, size_t size) {
    if (p && !ar) c->base.free(c->base.state, p, size);
}
NX_INLINE void nx_arena_end(nx_arena* a) {
    nx_arena_chunk* ch = a->head;
    while (ch) { nx_arena_chunk* n = ch->next; nx_free_bytes(a->parent, ch, ch->cap); ch = n; }
    a->head = NULL;
}

/* ------------------------------------------------------------ parallel for */
typedef void (*nx_par_fn)(nx_ctx*, void*, size_t, size_t);
typedef struct nx_par_task { nx_ctx ctx; nx_par_fn f; void* env; size_t begin; size_t end; bool panicked; char msg[256]; char loc[128]; } nx_par_task;

NX_INLINE void nx_par_run(nx_par_task* t) {
    nx_boundary b;
    b.track = NULL;
    nx_boundary* prev = nx_tls_boundary;
    nx_tls_boundary = &b;
    if (setjmp(b.jb)) {
        t->panicked = true;
        snprintf(t->msg, sizeof t->msg, "%s", b.msg);
        snprintf(t->loc, sizeof t->loc, "%s", b.loc);
    } else {
        t->f(&t->ctx, t->env, t->begin, t->end);
    }
    nx_tls_boundary = prev;
}
#if defined(_WIN32)
static DWORD WINAPI nx_par_thread(LPVOID p) { nx_par_run((nx_par_task*)p); return 0; }
NX_INLINE size_t nx_hw_threads(void) { SYSTEM_INFO si; GetSystemInfo(&si); return si.dwNumberOfProcessors ? si.dwNumberOfProcessors : 1; }
#else
#include <pthread.h>
#if defined(NX_WASM)
/* wasm32-wasi has no threads: wasi-libc declares pthreads and defines none.
   Starting a thread fails, so the work runs in place where the parallel
   loop and `thread.spawn` already fall back to, and a lock has no one else
   to wait for. (The playground's compiler links against these: its driver
   compiles C on threads, which in the page it never does.) */
#define pthread_create(t, attr, f, arg) ((void)(t), (void)(attr), (void)(f), (void)(arg), EAGAIN)
#define pthread_join(t, r) ((void)(t), (void)(r), 0)
#define pthread_mutex_init(m, a) ((void)(m), (void)(a), 0)
#define pthread_mutex_destroy(m) ((void)(m), 0)
#define pthread_mutex_lock(m) ((void)(m), 0)
#define pthread_mutex_unlock(m) ((void)(m), 0)
#define pthread_cond_init(v, a) ((void)(v), (void)(a), 0)
#define pthread_cond_destroy(v) ((void)(v), 0)
#define pthread_cond_wait(v, m) ((void)(v), (void)(m), 0)
#define pthread_cond_timedwait(v, m, t) ((void)(v), (void)(m), (void)(t), ETIMEDOUT)
#define pthread_cond_signal(v) ((void)(v), 0)
#define pthread_cond_broadcast(v) ((void)(v), 0)
#endif
static void* nx_par_thread(void* p) { nx_par_run((nx_par_task*)p); return NULL; }
NX_INLINE size_t nx_hw_threads(void) { long n = sysconf(_SC_NPROCESSORS_ONLN); return n > 0 ? (size_t)n : 1; }
#endif

/* Runs f over [0, n) split across worker threads. Each worker gets its own
 * context (no shared allocator state); a panic in any worker is re-raised in
 * the calling thread after every worker has finished. */
NX_INLINE void nx_parallel_for(nx_ctx* c, size_t n, nx_par_fn f, void* env, const char* loc) {
    if (n == 0) return;
    size_t workers = nx_hw_threads();
    if (workers > 64) workers = 64;
    if (workers > n) workers = n;
    if (workers <= 1) { f(c, env, 0, n); return; }
    nx_par_task tasks[64];
    size_t chunk = (n + workers - 1) / workers;
#if defined(_WIN32)
    HANDLE handles[64];
#else
    pthread_t handles[64];
#endif
    for (size_t w = 0; w < workers; w++) {
        tasks[w].ctx = *c;
        tasks[w].ctx.live_allocs = 0; tasks[w].ctx.live_bytes = 0; tasks[w].ctx.total_allocs = 0; tasks[w].ctx.peak_bytes = 0;
        nx_ctx_track_self(&tasks[w].ctx);
        tasks[w].ctx.rng ^= (uint64_t)(w + 1) * 0x9E3779B97F4A7C15ULL;
        tasks[w].f = f; tasks[w].env = env; tasks[w].panicked = false;
        tasks[w].begin = w * chunk;
        tasks[w].end = (w + 1) * chunk < n ? (w + 1) * chunk : n;
#if defined(_WIN32)
        handles[w] = CreateThread(NULL, 0, nx_par_thread, &tasks[w], 0, NULL);
        if (!handles[w]) nx_par_run(&tasks[w]);
#else
        if (pthread_create(&handles[w], NULL, nx_par_thread, &tasks[w]) != 0) { nx_par_run(&tasks[w]); handles[w] = 0; }
#endif
    }
    for (size_t w = 0; w < workers; w++) {
#if defined(_WIN32)
        if (handles[w]) { WaitForSingleObject(handles[w], INFINITE); CloseHandle(handles[w]); }
#else
        if (handles[w]) pthread_join(handles[w], NULL);
#endif
        c->live_allocs += tasks[w].ctx.live_allocs;
        c->live_bytes += tasks[w].ctx.live_bytes;
        c->total_allocs += tasks[w].ctx.total_allocs;
    }
    for (size_t w = 0; w < workers; w++) {
        if (tasks[w].panicked) {
            char buf[400];
            snprintf(buf, sizeof buf, "%s (in a parallel worker at %s)", tasks[w].msg, tasks[w].loc);
            nx_panic(buf, loc);
        }
    }
}

/* --------------------------------------------------------------- lists */
NX_INLINE void nx_list_grow(nx_ctx* c, nx_rawlist* l, size_t elem, size_t align, size_t min_cap) {
    size_t cap = l->cap ? l->cap * 2 : 4;
    if (cap < min_cap) cap = min_cap;
    l->ptr = nx_cont_realloc(c, l->ar, l->ptr, l->cap * elem, cap * elem, align);
    l->cap = cap;
}
NX_INLINE void nx_list_free(nx_ctx* c, nx_rawlist* l, size_t elem) {
    nx_cont_free(c, l->ar, l->ptr, l->cap * elem);
    l->ptr = NULL; l->len = 0; l->cap = 0;
}
NX_INLINE void nx_list_reserve(nx_ctx* c, nx_rawlist* l, size_t elem, size_t align, size_t extra) {
    if (l->len + extra > l->cap) nx_list_grow(c, l, elem, align, l->len + extra);
}
NX_INLINE nx_rawlist nx_list_clone_raw(nx_ctx* c, const nx_rawlist* l, size_t elem, size_t align) {
    nx_rawlist r; r.ptr = NULL; r.len = 0; r.cap = 0; r.ar = c->arena;
    if (l->len) {
        r.ptr = nx_cont_alloc(c, r.ar, l->len * elem, align);
        memcpy(r.ptr, l->ptr, l->len * elem);
        r.len = l->len; r.cap = l->len;
    }
    return r;
}

/* -------------------------------------------------------------- strings */
NX_INLINE void nx_str_reserve(nx_ctx* c, nx_string* s, size_t extra) {
    if (s->len + extra > s->cap) nx_list_grow(c, (nx_rawlist*)s, 1, 1, s->len + extra);
}
NX_INLINE void nx_str_append(nx_ctx* c, nx_string* s, const uint8_t* p, size_t n) {
    nx_str_reserve(c, s, n);
    if (n) memcpy(s->ptr + s->len, p, n);
    s->len += n;
}
NX_INLINE nx_string nx_str_from(nx_ctx* c, nx_sl_u8 src) {
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    nx_str_append(c, &s, src.ptr, src.len);
    return s;
}
NX_INLINE void nx_str_free(nx_ctx* c, nx_string* s) { nx_list_free(c, (nx_rawlist*)s, 1); }
NX_INLINE nx_sl_u8 nx_str_slice(nx_string s) { nx_sl_u8 r; r.ptr = s.ptr; r.len = s.len; return r; }
NX_INLINE size_t nx_utf8_encode(uint32_t cp, uint8_t* out) {
    if (cp < 0x80) { out[0] = (uint8_t)cp; return 1; }
    if (cp < 0x800) { out[0] = (uint8_t)(0xC0 | (cp >> 6)); out[1] = (uint8_t)(0x80 | (cp & 0x3F)); return 2; }
    if (cp < 0x10000) { out[0] = (uint8_t)(0xE0 | (cp >> 12)); out[1] = (uint8_t)(0x80 | ((cp >> 6) & 0x3F)); out[2] = (uint8_t)(0x80 | (cp & 0x3F)); return 3; }
    out[0] = (uint8_t)(0xF0 | (cp >> 18)); out[1] = (uint8_t)(0x80 | ((cp >> 12) & 0x3F)); out[2] = (uint8_t)(0x80 | ((cp >> 6) & 0x3F)); out[3] = (uint8_t)(0x80 | (cp & 0x3F)); return 4;
}
NX_INLINE void nx_str_append_char(nx_ctx* c, nx_string* s, uint32_t cp) {
    uint8_t buf[4];
    size_t n = nx_utf8_encode(cp, buf);
    nx_str_append(c, s, buf, n);
}

/* Copy a slice's bytes. An empty slice may have a null pointer (an empty
   String has no storage), and memcpy may not be given one even to copy
   nothing (C11 7.24.1; glibc declares the arguments nonnull), so every copy
   out of a slice goes through here or checks the length itself. */
NX_INLINE void nx_bytes_copy(void* d, const void* s, size_t n) { if (n) memcpy(d, s, n); }
NX_INLINE bool nx_sl_eq(nx_sl_u8 a, nx_sl_u8 b) { return a.len == b.len && (a.len == 0 || memcmp(a.ptr, b.ptr, a.len) == 0); }
NX_INLINE int nx_sl_cmp(nx_sl_u8 a, nx_sl_u8 b) {
    size_t n = a.len < b.len ? a.len : b.len;
    int r = n ? memcmp(a.ptr, b.ptr, n) : 0;
    if (r) return r;
    return a.len < b.len ? -1 : (a.len > b.len ? 1 : 0);
}
NX_INLINE bool nx_sl_starts_with(nx_sl_u8 a, nx_sl_u8 p) { return a.len >= p.len && (p.len == 0 || memcmp(a.ptr, p.ptr, p.len) == 0); }
NX_INLINE bool nx_sl_ends_with(nx_sl_u8 a, nx_sl_u8 p) { return a.len >= p.len && (p.len == 0 || memcmp(a.ptr + a.len - p.len, p.ptr, p.len) == 0); }
NX_INLINE bool nx_sl_find(nx_sl_u8 a, nx_sl_u8 n, size_t* out) {
    if (n.len == 0) { *out = 0; return true; }
    if (a.len < n.len) return false;
    for (size_t i = 0; i + n.len <= a.len; i++) {
        if (a.ptr[i] == n.ptr[0] && memcmp(a.ptr + i, n.ptr, n.len) == 0) { *out = i; return true; }
    }
    return false;
}
NX_INLINE nx_sl_u8 nx_sl_trim(nx_sl_u8 a) {
    size_t s = 0, e = a.len;
    while (s < e && (a.ptr[s] == ' ' || a.ptr[s] == '\t' || a.ptr[s] == '\n' || a.ptr[s] == '\r')) s++;
    while (e > s && (a.ptr[e - 1] == ' ' || a.ptr[e - 1] == '\t' || a.ptr[e - 1] == '\n' || a.ptr[e - 1] == '\r')) e--;
    nx_sl_u8 r; r.ptr = nx_padd(a.ptr, s); r.len = e - s; return r;
}
NX_INLINE bool nx_sl_eq_ignore_case(nx_sl_u8 a, nx_sl_u8 b) {
    if (a.len != b.len) return false;
    for (size_t i = 0; i < a.len; i++) {
        uint8_t x = a.ptr[i], y = b.ptr[i];
        if (x >= 'A' && x <= 'Z') x += 32;
        if (y >= 'A' && y <= 'Z') y += 32;
        if (x != y) return false;
    }
    return true;
}
/* parse a decimal/hex integer; returns 0 ok, 1 invalid, 2 overflow */
NX_INLINE int nx_parse_int(nx_sl_u8 s, nx_i128 lo, nx_i128 hi, nx_i128* out) {
    size_t i = 0; bool neg = false;
    s = nx_sl_trim(s);
    if (s.len == 0) return 1;
    if (s.ptr[0] == '-') { neg = true; i = 1; } else if (s.ptr[0] == '+') { i = 1; }
    if (i >= s.len) return 1;
    nx_i128 v = 0; int base = 10;
    if (i + 1 < s.len && s.ptr[i] == '0' && (s.ptr[i + 1] == 'x' || s.ptr[i + 1] == 'X')) { base = 16; i += 2; }
    for (; i < s.len; i++) {
        uint8_t ch = s.ptr[i]; int d;
        if (ch == '_') continue;
        if (ch >= '0' && ch <= '9') d = ch - '0';
        else if (base == 16 && ch >= 'a' && ch <= 'f') d = ch - 'a' + 10;
        else if (base == 16 && ch >= 'A' && ch <= 'F') d = ch - 'A' + 10;
        else return 1;
        /* overflow of the accumulator itself: the range check below cannot see it */
        if (v > (NX_I128_MAX - d) / base) return 2;
        v = v * base + d;
    }
    if (neg) v = -v;
    if (v < lo || v > hi) return 2;
    *out = v;
    return 0;
}
/* nx_parse_int for an unsigned type, whose range can pass nx_i128's:
 * the same text is read, a minus sign only in range on zero */
NX_INLINE int nx_parse_uint(nx_sl_u8 s, nx_u128 hi, nx_u128* out) {
    size_t i = 0; bool neg = false;
    s = nx_sl_trim(s);
    if (s.len == 0) return 1;
    if (s.ptr[0] == '-') { neg = true; i = 1; } else if (s.ptr[0] == '+') { i = 1; }
    if (i >= s.len) return 1;
    nx_u128 v = 0; unsigned base = 10;
    if (i + 1 < s.len && s.ptr[i] == '0' && (s.ptr[i + 1] == 'x' || s.ptr[i + 1] == 'X')) { base = 16; i += 2; }
    for (; i < s.len; i++) {
        uint8_t ch = s.ptr[i]; unsigned d;
        if (ch == '_') continue;
        if (ch >= '0' && ch <= '9') d = ch - '0';
        else if (base == 16 && ch >= 'a' && ch <= 'f') d = ch - 'a' + 10;
        else if (base == 16 && ch >= 'A' && ch <= 'F') d = ch - 'A' + 10;
        else return 1;
        if (v > (~(nx_u128)0 - d) / base) return 2;
        v = v * base + d;
    }
    if ((neg && v != 0) || v > hi) return 2;
    *out = v;
    return 0;
}
NX_INLINE bool nx_parse_float(nx_sl_u8 s, double* out) {
    char buf[64];
    s = nx_sl_trim(s);
    if (s.len == 0 || s.len >= sizeof buf) return false;
    memcpy(buf, s.ptr, s.len); buf[s.len] = 0;
    char* end = NULL;
    double v = strtod(buf, &end);
    if (end != buf + s.len) return false;
    *out = v;
    return true;
}

/* -------------------------------------------------------------- formatting */
typedef struct nx_sink { nx_ctx* ctx; nx_string* str; FILE* f; char buf[512]; size_t n; } nx_sink;
NX_INLINE nx_sink nx_sink_file(nx_ctx* c, FILE* f) { nx_sink s; s.ctx = c; s.str = NULL; s.f = f; s.n = 0; return s; }
NX_INLINE nx_sink nx_sink_str(nx_ctx* c, nx_string* str) { nx_sink s; s.ctx = c; s.str = str; s.f = NULL; s.n = 0; return s; }
NX_INLINE void nx_sink_flush(nx_sink* s) { if (s->f && s->n) { fwrite(s->buf, 1, s->n, s->f); s->n = 0; } if (s->f) fflush(s->f); }
NX_INLINE void nx_w(nx_sink* s, const uint8_t* p, size_t n) {
    if (n == 0) return;
    if (s->str) { nx_str_append(s->ctx, s->str, p, n); return; }
    if (n > sizeof s->buf) { nx_sink_flush(s); fwrite(p, 1, n, s->f); return; }
    if (s->n + n > sizeof s->buf) { fwrite(s->buf, 1, s->n, s->f); s->n = 0; }
    memcpy(s->buf + s->n, p, n); s->n += n;
}
NX_INLINE void nx_w_cstr(nx_sink* s, const char* p) { nx_w(s, (const uint8_t*)p, strlen(p)); }
NX_INLINE void nx_w_sl(nx_sink* s, nx_sl_u8 v) { nx_w(s, v.ptr, v.len); }
NX_INLINE void nx_w_pad(nx_sink* s, const char* txt, size_t len, int width, bool left) {
    if (width > 0 && (size_t)width > len && !left) { for (size_t i = len; i < (size_t)width; i++) nx_w(s, (const uint8_t*)" ", 1); }
    nx_w(s, (const uint8_t*)txt, len);
    if (width > 0 && (size_t)width > len && left) { for (size_t i = len; i < (size_t)width; i++) nx_w(s, (const uint8_t*)" ", 1); }
}
/* base: 10, 16 (lower), 17 (upper), 2, 8; an unsigned value, u128's whole range */
NX_INLINE void nx_w_uint(nx_sink* s, nx_u128 u, int base, int width, bool left) {
    char buf[140]; size_t i = sizeof buf;
    int b = base == 17 ? 16 : base;
    const char* digits = base == 17 ? "0123456789ABCDEF" : "0123456789abcdef";
    if (u == 0) buf[--i] = '0';
    while (u) { buf[--i] = digits[u % b]; u /= b; }
    nx_w_pad(s, buf + i, sizeof buf - i, width, left);
}
/* base: 10, 16 (lower), 17 (upper), 2, 8 */
NX_INLINE void nx_w_int(nx_sink* s, nx_i128 v, int base, int width, bool left) {
    char buf[140]; size_t i = sizeof buf; bool neg = v < 0;
    nx_u128 u = neg ? (nx_u128)(-(v + 1)) + 1 : (nx_u128)v;
    int b = base == 17 ? 16 : base;
    const char* digits = base == 17 ? "0123456789ABCDEF" : "0123456789abcdef";
    if (u == 0) buf[--i] = '0';
    while (u) { buf[--i] = digits[u % b]; u /= b; }
    if (neg) buf[--i] = '-';
    nx_w_pad(s, buf + i, sizeof buf - i, width, left);
}
/* an f32 (`f32` set) is written as the shortest text that reads back as that f32 */
NX_INLINE void nx_w_float(nx_sink* s, double v, int prec, bool exp, int width, bool left, bool f32) {
    char buf[64];
    if (v != v) { snprintf(buf, sizeof buf, "nan"); }
    else if (isinf(v)) { snprintf(buf, sizeof buf, v > 0 ? "inf" : "-inf"); }
    else if (exp) { snprintf(buf, sizeof buf, "%.*e", prec < 0 ? 6 : prec, v); }
    else if (prec >= 0) { snprintf(buf, sizeof buf, "%.*f", prec, v); }
    else if (v == floor(v) && fabs(v) < 1e16) { snprintf(buf, sizeof buf, "%.1f", v); }
    else { /* the shortest text that reads back as the same value */
        int p = 1;
        for (; p <= 17; p++) {
            snprintf(buf, sizeof buf, "%.*g", p, v);
            if (f32 ? (float)strtod(buf, NULL) == (float)v : strtod(buf, NULL) == v) break;
        }
    }
    nx_w_pad(s, buf, strlen(buf), width, left);
}
NX_INLINE void nx_w_bool(nx_sink* s, bool b) { nx_w_cstr(s, b ? "true" : "false"); }
NX_INLINE void nx_w_char(nx_sink* s, uint32_t cp) { uint8_t buf[4]; size_t n = nx_utf8_encode(cp, buf); nx_w(s, buf, n); }

/* --------------------------------------------------------------- maps */
/* A map keeps its entries in the order their keys were first put: iterating
 * gives them in that order (putting a key again keeps its place; removing
 * one closes the gap), whatever the keys hash to, as the interpreter's maps
 * do. An index of entry numbers, open addressing with linear probing over a
 * power-of-two table, finds a key. Keys are hashed with SipHash-1-3 under a
 * key drawn once per process from the operating system's generator, so
 * someone who chooses a program's keys cannot choose ones that collide.
 * Keys and values are stored as raw bytes.
 * key_kind: 0 = inline bytes, 1 = nx_sl_u8 (content hashed, storage borrowed),
 *           2 = nx_string (content hashed, owned by the map). */
typedef struct nx_map {
    /* the entries, `used` of them written, room for `ecap`; `live` 0 for a
       removed one */
    uint8_t* keys; uint8_t* vals; uint8_t* live; uint64_t* hashes;
    /* the index: per slot 0 (empty), NX_MAP_GONE (a removed entry's), or
       an entry's number + 1; `filled` slots are not empty */
    size_t* index; size_t icap; size_t filled;
    size_t used; size_t ecap; size_t len; size_t ksize; size_t vsize; int key_kind;
    nx_arena* ar;
} nx_map;
#define NX_MAP_GONE ((size_t)-1)

NX_INLINE bool nx_os_random(uint8_t* p, size_t n);
NX_STATE uint64_t nx_map_seed[2];
NX_STATE int nx_map_seeded;
/* the process's hash key, drawn at the first use of a map: 0 not drawn,
   1 being drawn (another thread waits), 2 ready */
NX_INLINE void nx_map_seed_init(void) {
    if (__atomic_load_n(&nx_map_seeded, __ATOMIC_ACQUIRE) == 2) return;
    int expected = 0;
    if (__atomic_compare_exchange_n(&nx_map_seeded, &expected, 1, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) {
        uint64_t k[2] = { 0, 0 };
        if (!nx_os_random((uint8_t*)k, sizeof k)) {
            /* no generator: something no one outside can predict well (the
               time and two addresses; not clock(), which wasm32-wasi lacks) */
            k[0] = (uint64_t)time(NULL) * 0x9E3779B97F4A7C15ULL;
            k[1] = (uint64_t)(uintptr_t)&k ^ ((uint64_t)(uintptr_t)&nx_map_seeded << 16);
        }
        nx_map_seed[0] = k[0];
        nx_map_seed[1] = k[1];
        __atomic_store_n(&nx_map_seeded, 2, __ATOMIC_RELEASE);
        return;
    }
    while (__atomic_load_n(&nx_map_seeded, __ATOMIC_ACQUIRE) != 2) { }
}
#define NX_ROTL64(x, b) (uint64_t)(((x) << (b)) | ((x) >> (64 - (b))))
#define NX_SIPROUND do { \
    v0 += v1; v1 = NX_ROTL64(v1, 13); v1 ^= v0; v0 = NX_ROTL64(v0, 32); \
    v2 += v3; v3 = NX_ROTL64(v3, 16); v3 ^= v2; \
    v0 += v3; v3 = NX_ROTL64(v3, 21); v3 ^= v0; \
    v2 += v1; v1 = NX_ROTL64(v1, 17); v1 ^= v2; v2 = NX_ROTL64(v2, 32); } while (0)
/* SipHash-1-3 (one round per 8 bytes, three to finish), as Rust's HashMap */
NX_INLINE uint64_t nx_siphash13(uint64_t k0, uint64_t k1, const uint8_t* p, size_t n) {
    uint64_t v0 = 0x736f6d6570736575ULL ^ k0, v1 = 0x646f72616e646f6dULL ^ k1;
    uint64_t v2 = 0x6c7967656e657261ULL ^ k0, v3 = 0x7465646279746573ULL ^ k1;
    size_t end = n & ~(size_t)7;
    for (size_t i = 0; i < end; i += 8) {
        uint64_t m = 0;
        for (int b = 0; b < 8; b++) m |= (uint64_t)p[i + b] << (8 * b);
        v3 ^= m; NX_SIPROUND; v0 ^= m;
    }
    uint64_t last = (uint64_t)n << 56;
    for (size_t b = 0; b < (n & 7); b++) last |= (uint64_t)p[end + b] << (8 * b);
    v3 ^= last; NX_SIPROUND; v0 ^= last;
    v2 ^= 0xff; NX_SIPROUND; NX_SIPROUND; NX_SIPROUND;
    return v0 ^ v1 ^ v2 ^ v3;
}
NX_INLINE nx_sl_u8 nx_map_key_bytes(const nx_map* m, const void* key) {
    nx_sl_u8 r;
    if (m->key_kind == 0) { r.ptr = (uint8_t*)key; r.len = m->ksize; }
    else { const nx_sl_u8* s = (const nx_sl_u8*)key; r.ptr = s->ptr; r.len = s->len; }
    return r;
}
NX_INLINE uint64_t nx_map_hash(nx_sl_u8 kb) {
    nx_map_seed_init();
    return nx_siphash13(nx_map_seed[0], nx_map_seed[1], kb.ptr, kb.len);
}
NX_INLINE nx_map nx_map_new(nx_ctx* c, size_t ksize, size_t vsize, int key_kind) {
    nx_map m; memset(&m, 0, sizeof m); m.ksize = ksize; m.vsize = vsize; m.key_kind = key_kind; m.ar = c->arena; return m;
}
/* the slot of the entry holding `kb` (true, and its number in *entry), or
   where it would go: the first removed entry's slot on its way, else the
   empty slot that ends it. The index always has an empty slot. */
NX_INLINE bool nx_map_find(const nx_map* m, nx_sl_u8 kb, uint64_t h, size_t* slot, size_t* entry) {
    if (m->icap == 0) { *slot = 0; return false; }
    size_t mask = m->icap - 1, i = (size_t)h & mask, gone = NX_MAP_GONE;
    for (;;) {
        size_t v = m->index[i];
        if (v == 0) { *slot = gone != NX_MAP_GONE ? gone : i; return false; }
        if (v == NX_MAP_GONE) {
            if (gone == NX_MAP_GONE) gone = i;
        } else if (m->hashes[v - 1] == h && nx_sl_eq(nx_map_key_bytes(m, m->keys + (v - 1) * m->ksize), kb)) {
            *slot = i; *entry = v - 1; return true;
        }
        i = (i + 1) & mask;
    }
}
/* an index of `icap` slots over the live entries */
NX_INLINE void nx_map_reindex(nx_ctx* c, nx_map* m, size_t icap) {
    if (m->icap) nx_cont_free(c, m->ar, m->index, m->icap * sizeof(size_t));
    m->index = (size_t*)nx_cont_alloc(c, m->ar, icap * sizeof(size_t), _Alignof(size_t));
    memset(m->index, 0, icap * sizeof(size_t));
    m->icap = icap;
    m->filled = 0;
    size_t mask = icap - 1;
    for (size_t e = 0; e < m->used; e++) {
        if (!m->live[e]) continue;
        size_t i = (size_t)m->hashes[e] & mask;
        while (m->index[i] != 0) i = (i + 1) & mask;
        m->index[i] = e + 1;
        m->filled++;
    }
}
/* room for one more entry: the removed ones packed out, in order, when they
   are half of what is written, else twice the room */
NX_INLINE void nx_map_make_room(nx_ctx* c, nx_map* m) {
    size_t dead = m->used - m->len;
    if (dead > 0 && dead * 2 >= m->used) {
        size_t w = 0;
        for (size_t r = 0; r < m->used; r++) {
            if (!m->live[r]) continue;
            if (w != r) {
                memmove(m->keys + w * m->ksize, m->keys + r * m->ksize, m->ksize);
                if (m->vsize) memmove(m->vals + w * m->vsize, m->vals + r * m->vsize, m->vsize);
                m->hashes[w] = m->hashes[r];
                m->live[w] = 1;
            }
            w++;
        }
        for (size_t r = w; r < m->used; r++) m->live[r] = 0;
        m->used = w;
        nx_map_reindex(c, m, m->icap);
        return;
    }
    size_t ncap = m->ecap ? m->ecap * 2 : 8;
    size_t vs = m->vsize ? m->vsize : 1;
    uint8_t* keys = (uint8_t*)nx_cont_alloc(c, m->ar, ncap * m->ksize, 16);
    uint8_t* vals = (uint8_t*)nx_cont_alloc(c, m->ar, ncap * vs, 16);
    uint8_t* live = (uint8_t*)nx_cont_alloc(c, m->ar, ncap, 1);
    uint64_t* hashes = (uint64_t*)nx_cont_alloc(c, m->ar, ncap * sizeof(uint64_t), _Alignof(uint64_t));
    memset(live, 0, ncap);
    if (m->used) {
        memcpy(keys, m->keys, m->used * m->ksize);
        memcpy(vals, m->vals, m->used * vs);
        memcpy(live, m->live, m->used);
        memcpy(hashes, m->hashes, m->used * sizeof(uint64_t));
    }
    if (m->ecap) {
        nx_cont_free(c, m->ar, m->keys, m->ecap * m->ksize);
        nx_cont_free(c, m->ar, m->vals, m->ecap * vs);
        nx_cont_free(c, m->ar, m->live, m->ecap);
        nx_cont_free(c, m->ar, m->hashes, m->ecap * sizeof(uint64_t));
    }
    m->keys = keys; m->vals = vals; m->live = live; m->hashes = hashes; m->ecap = ncap;
}
/* a new entry at the end, for a key that is not in the map */
NX_INLINE void nx_map_append(nx_ctx* c, nx_map* m, const void* key, const void* val, nx_sl_u8 kb, uint64_t h) {
    if (m->used == m->ecap) nx_map_make_room(c, m);
    if ((m->filled + 1) * 4 > m->icap * 3) {
        size_t icap = m->icap ? m->icap : 8;
        while ((m->len + 1) * 2 > icap) icap *= 2;
        nx_map_reindex(c, m, icap);
    }
    size_t slot = 0, unused = 0;
    (void)nx_map_find(m, kb, h, &slot, &unused);
    size_t e = m->used++;
    memcpy(m->keys + e * m->ksize, key, m->ksize);
    if (m->vsize) memcpy(m->vals + e * m->vsize, val, m->vsize);
    m->hashes[e] = h;
    m->live[e] = 1;
    if (m->index[slot] == 0) m->filled++;
    m->index[slot] = e + 1;
    m->len++;
}
/* returns pointer to the existing value slot, or NULL */
NX_INLINE void* nx_map_get(const nx_map* m, const void* key) {
    if (m->len == 0) return NULL;
    nx_sl_u8 kb = nx_map_key_bytes(m, key);
    size_t slot, e;
    if (nx_map_find(m, kb, nx_map_hash(kb), &slot, &e)) return m->vals + e * m->vsize;
    return NULL;
}
/* returns true if an existing entry was replaced (the old key/value are returned in old_key/old_val for dropping) */
NX_INLINE bool nx_map_put(nx_ctx* c, nx_map* m, const void* key, const void* val, void* old_key, void* old_val) {
    nx_sl_u8 kb = nx_map_key_bytes(m, key);
    uint64_t h = nx_map_hash(kb);
    size_t slot, e;
    if (nx_map_find(m, kb, h, &slot, &e)) {
        if (old_key) memcpy(old_key, m->keys + e * m->ksize, m->ksize);
        if (old_val) memcpy(old_val, m->vals + e * m->vsize, m->vsize);
        memcpy(m->keys + e * m->ksize, key, m->ksize);
        if (m->vsize) memcpy(m->vals + e * m->vsize, val, m->vsize);
        return true;
    }
    nx_map_append(c, m, key, val, kb, h);
    return false;
}
NX_INLINE bool nx_map_remove(nx_map* m, const void* key, void* old_key, void* old_val) {
    if (m->len == 0) return false;
    nx_sl_u8 kb = nx_map_key_bytes(m, key);
    size_t slot, e;
    if (!nx_map_find(m, kb, nx_map_hash(kb), &slot, &e)) return false;
    if (old_key) memcpy(old_key, m->keys + e * m->ksize, m->ksize);
    if (old_val) memcpy(old_val, m->vals + e * m->vsize, m->vsize);
    m->live[e] = 0;
    m->index[slot] = NX_MAP_GONE;
    m->len--;
    if (m->len == 0) {
        /* the last one gone: start over in the same room */
        memset(m->live, 0, m->used);
        m->used = 0;
        memset(m->index, 0, m->icap * sizeof(size_t));
        m->filled = 0;
    }
    return true;
}
NX_INLINE void nx_map_free_storage(nx_ctx* c, nx_map* m) {
    if (m->ecap) {
        size_t vs = m->vsize ? m->vsize : 1;
        nx_cont_free(c, m->ar, m->keys, m->ecap * m->ksize);
        nx_cont_free(c, m->ar, m->vals, m->ecap * vs);
        nx_cont_free(c, m->ar, m->live, m->ecap);
        nx_cont_free(c, m->ar, m->hashes, m->ecap * sizeof(uint64_t));
    }
    if (m->icap) nx_cont_free(c, m->ar, m->index, m->icap * sizeof(size_t));
    m->keys = NULL; m->vals = NULL; m->live = NULL; m->hashes = NULL; m->index = NULL;
    m->ecap = 0; m->icap = 0; m->filled = 0; m->used = 0; m->len = 0;
}
/* iterate in insertion order: start with i = 0; returns false when done */
NX_INLINE bool nx_map_next(const nx_map* m, size_t* i, void** key, void** val) {
    while (*i < m->used) {
        size_t k = (*i)++;
        if (m->live[k]) { *key = m->keys + k * m->ksize; *val = m->vals + k * m->vsize; return true; }
    }
    return false;
}
NX_INLINE nx_map nx_map_clone_raw(nx_ctx* c, const nx_map* m) {
    nx_map n = nx_map_new(c, m->ksize, m->vsize, m->key_kind);
    if (m->ecap) {
        size_t vs = m->vsize ? m->vsize : 1;
        n.keys = (uint8_t*)nx_cont_alloc(c, n.ar, m->ecap * m->ksize, 16);
        n.vals = (uint8_t*)nx_cont_alloc(c, n.ar, m->ecap * vs, 16);
        n.live = (uint8_t*)nx_cont_alloc(c, n.ar, m->ecap, 1);
        n.hashes = (uint64_t*)nx_cont_alloc(c, n.ar, m->ecap * sizeof(uint64_t), _Alignof(uint64_t));
        memset(n.live, 0, m->ecap);
        if (m->used) {
            memcpy(n.keys, m->keys, m->used * m->ksize);
            memcpy(n.vals, m->vals, m->used * vs);
            memcpy(n.live, m->live, m->used);
            memcpy(n.hashes, m->hashes, m->used * sizeof(uint64_t));
        }
        n.ecap = m->ecap; n.used = m->used; n.len = m->len;
    }
    if (m->icap) {
        n.index = (size_t*)nx_cont_alloc(c, n.ar, m->icap * sizeof(size_t), _Alignof(size_t));
        memcpy(n.index, m->index, m->icap * sizeof(size_t));
        n.icap = m->icap; n.filled = m->filled;
    }
    return n;
}

/* ------------------------------------------------------ reference counting */
typedef struct nx_obj_header { size_t rc; size_t weak; } nx_obj_header;
NX_INLINE void* nx_retain(void* p) { if (p) ((nx_obj_header*)p)->rc++; return p; }
NX_INLINE void* nx_weak_new(void* p) { if (p) ((nx_obj_header*)p)->weak++; return p; }
/* returns true when the object is alive (and retains it) */
NX_INLINE bool nx_weak_upgrade(void* p) { if (p && ((nx_obj_header*)p)->rc > 0) { ((nx_obj_header*)p)->rc++; return true; } return false; }

/* --------------------------------------------------------------- binary */
NX_INLINE uint64_t nx_bits_read(const uint8_t* p, size_t bit, size_t n) {
    uint64_t v = 0;
    /* fast path: byte aligned */
    if ((bit & 7) == 0 && (n & 7) == 0) {
        const uint8_t* q = p + bit / 8;
        for (size_t i = 0; i < n / 8; i++) v = (v << 8) | q[i];
        return v;
    }
    for (size_t i = 0; i < n; i++) {
        size_t b = bit + i;
        v = (v << 1) | ((p[b >> 3] >> (7 - (b & 7))) & 1);
    }
    return v;
}
NX_INLINE void nx_bits_write(uint8_t* p, size_t bit, size_t n, uint64_t v) {
    if ((bit & 7) == 0 && (n & 7) == 0) {
        uint8_t* q = p + bit / 8;
        for (size_t i = 0; i < n / 8; i++) q[i] = (uint8_t)(v >> ((n / 8 - 1 - i) * 8));
        return;
    }
    for (size_t i = 0; i < n; i++) {
        size_t b = bit + i;
        uint8_t bitv = (uint8_t)((v >> (n - 1 - i)) & 1);
        if (bitv) p[b >> 3] |= (uint8_t)(0x80 >> (b & 7));
        else p[b >> 3] &= (uint8_t)~(0x80 >> (b & 7));
    }
}
NX_INLINE uint64_t nx_bswap(uint64_t v, size_t nbytes) {
    uint64_t r = 0;
    for (size_t i = 0; i < nbytes; i++) r |= ((v >> (i * 8)) & 0xff) << ((nbytes - 1 - i) * 8);
    return r;
}
NX_INLINE bool nx_is_little_endian(void) { uint16_t x = 1; return *(uint8_t*)&x == 1; }
NX_INLINE int64_t nx_sign_extend(uint64_t v, size_t n) {
    if (n >= 64) return (int64_t)v;
    uint64_t m = 1ULL << (n - 1);
    return (int64_t)((v ^ m) - m);
}

/* ------------------------------------------------------------------ misc */
NX_INLINE int64_t nx_time_now_ms(void) {
#if defined(_WIN32)
    FILETIME ft; GetSystemTimeAsFileTime(&ft);
    uint64_t t = ((uint64_t)ft.dwHighDateTime << 32) | ft.dwLowDateTime;
    return (int64_t)(t / 10000) - 11644473600000LL;
#else
    struct timeval tv; gettimeofday(&tv, NULL);
    return (int64_t)tv.tv_sec * 1000 + tv.tv_usec / 1000;
#endif
}
/* minutes east of UTC of local time at the given instant (0 when unknown) */
NX_INLINE int64_t nx_time_utc_offset_min(int64_t epoch_ms) {
    time_t t = (time_t)(epoch_ms / 1000);
    struct tm loc, utc;
#if defined(_WIN32)
    if (localtime_s(&loc, &t) != 0 || gmtime_s(&utc, &t) != 0) return 0;
#else
    if (!localtime_r(&t, &loc) || !gmtime_r(&t, &utc)) return 0;
#endif
    int64_t lmin = ((int64_t)loc.tm_yday * 1440) + loc.tm_hour * 60 + loc.tm_min;
    int64_t umin = ((int64_t)utc.tm_yday * 1440) + utc.tm_hour * 60 + utc.tm_min;
    int64_t diff = lmin - umin;
    if (loc.tm_year != utc.tm_year) diff += loc.tm_year > utc.tm_year ? 365 * 1440 : -365 * 1440;
    return diff;
}
/* time.zone_rules: a zone of the IANA database as text, for std.time. The
   first line is the zone's name; each further line is a period, `start
   offset dst abbrev`: start in ms since the epoch (`-` for the first
   period), the offset in seconds east of UTC, 1 for daylight time.
   Windows has no zoneinfo files; its ICU (icu.dll, Windows 10 1903 and
   later) has the database, loaded at the first call so no program links
   it. An empty name is the system's zone. Changes are listed to 2200;
   after that the zone keeps its standard offset. Elsewhere std.time reads
   the zoneinfo files itself and this is NotFound. 0, or 1 for a zone ICU
   does not know or no ICU. */
#if defined(_WIN32)
typedef void* (*nx_ucal_open_fn)(const uint16_t*, int32_t, const char*, int32_t, int32_t*);
typedef void (*nx_ucal_close_fn)(void*);
typedef void (*nx_ucal_set_millis_fn)(void*, double, int32_t*);
typedef int32_t (*nx_ucal_get_fn)(const void*, int32_t, int32_t*);
typedef int8_t (*nx_ucal_transition_fn)(const void*, int32_t, double*, int32_t*);
typedef int32_t (*nx_ucal_display_fn)(const void*, int32_t, const char*, uint16_t*, int32_t, int32_t*);
typedef int32_t (*nx_ucal_default_fn)(uint16_t*, int32_t, int32_t*);
typedef int32_t (*nx_ucal_canonical_fn)(const uint16_t*, int32_t, uint16_t*, int32_t, int8_t*, int32_t*);
typedef struct nx_icu_fns {
    nx_ucal_open_fn open; nx_ucal_close_fn close; nx_ucal_set_millis_fn set_millis; nx_ucal_get_fn get;
    nx_ucal_transition_fn transition; nx_ucal_display_fn display; nx_ucal_default_fn default_zone;
    nx_ucal_canonical_fn canonical;
} nx_icu_fns;
NX_STATE nx_icu_fns nx_icu;
NX_STATE int nx_icu_state; /* 0 not loaded yet, 1 loaded, 2 missing */
NX_INLINE bool nx_icu_load(void) {
    int st = __atomic_load_n(&nx_icu_state, __ATOMIC_ACQUIRE);
    if (st != 0) return st == 1;
    nx_icu_fns f;
    memset(&f, 0, sizeof f);
    HMODULE m = LoadLibraryA("icu.dll");
    if (!m) m = LoadLibraryA("icuin.dll");
    if (m) {
        f.open = (nx_ucal_open_fn)(void*)GetProcAddress(m, "ucal_open");
        f.close = (nx_ucal_close_fn)(void*)GetProcAddress(m, "ucal_close");
        f.set_millis = (nx_ucal_set_millis_fn)(void*)GetProcAddress(m, "ucal_setMillis");
        f.get = (nx_ucal_get_fn)(void*)GetProcAddress(m, "ucal_get");
        f.transition = (nx_ucal_transition_fn)(void*)GetProcAddress(m, "ucal_getTimeZoneTransitionDate");
        f.display = (nx_ucal_display_fn)(void*)GetProcAddress(m, "ucal_getTimeZoneDisplayName");
        f.default_zone = (nx_ucal_default_fn)(void*)GetProcAddress(m, "ucal_getDefaultTimeZone");
        f.canonical = (nx_ucal_canonical_fn)(void*)GetProcAddress(m, "ucal_getCanonicalTimeZoneID");
    }
    bool ok = f.open && f.close && f.set_millis && f.get && f.transition && f.display && f.default_zone && f.canonical;
    /* threads that race here store the same pointers */
    if (ok) nx_icu = f;
    __atomic_store_n(&nx_icu_state, ok ? 1 : 2, __ATOMIC_RELEASE);
    return ok;
}
/* one period's line. The abbreviation is ICU's short English name where
   one of these Englishes has it (CET is British, IST Indian, AEST
   Australian); else the zoneinfo files' style, `+0530`, or `LMT` for the
   local mean time a place kept before it took a standard offset */
NX_INLINE void nx_icu_period(nx_ctx* c, nx_string* s, void* cal, bool first, double at, int32_t offset_ms, bool dst) {
    static const char* const locales[] = { "en_US", "en_GB", "en_IN", "en_AU" };
    uint16_t w[48];
    char line[128], abbr[48];
    size_t k = 0;
    for (size_t l = 0; l < sizeof locales / sizeof locales[0]; l++) {
        int32_t st = 0;
        /* 1 UCAL_SHORT_STANDARD, 3 UCAL_SHORT_DST */
        int32_t n = nx_icu.display(cal, dst ? 3 : 1, locales[l], w, 47, &st);
        k = 0;
        if (st > 0) continue;
        for (int32_t i = 0; i < n && k < sizeof abbr - 1; i++) abbr[k++] = w[i] > 32 && w[i] < 127 ? (char)w[i] : '_';
        /* `GMT+1` is no name, only an offset */
        if (k > 3 && memcmp(abbr, "GMT", 3) == 0 && (abbr[3] == '+' || abbr[3] == '-')) { k = 0; continue; }
        if (k > 0) break;
    }
    if (k == 0) {
        int32_t sec = offset_ms / 1000;
        if (sec % 60 != 0) {
            memcpy(abbr, "LMT", 3);
            k = 3;
        } else {
            int32_t a = sec < 0 ? -sec : sec;
            k = (size_t)snprintf(abbr, sizeof abbr, "%c%02d", sec < 0 ? '-' : '+', (int)(a / 3600));
            if (a % 3600 != 0) k += (size_t)snprintf(abbr + k, sizeof abbr - k, "%02d", (int)(a % 3600 / 60));
        }
    }
    abbr[k] = 0;
    int len = first
        ? snprintf(line, sizeof line, "- %ld %d %s\n", (long)(offset_ms / 1000), dst ? 1 : 0, abbr)
        : snprintf(line, sizeof line, "%lld %ld %d %s\n", (long long)at, (long)(offset_ms / 1000), dst ? 1 : 0, abbr);
    if (len > 0) nx_str_append(c, s, (const uint8_t*)line, (size_t)len);
}
#endif
NX_INLINE int32_t nx_time_zone_rules(nx_ctx* c, nx_sl_u8 name, nx_string* out) {
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
#if defined(_WIN32)
    if (!nx_icu_load()) return 1;
    uint16_t id[128];
    int32_t idlen = 0, st = 0;
    if (name.len == 0) {
        idlen = nx_icu.default_zone(id, 127, &st);
        if (st > 0 || idlen <= 0 || idlen >= 127) return 1;
    } else {
        if (name.len >= 127) return 1;
        for (size_t i = 0; i < name.len; i++) {
            if (name.ptr[i] < 33 || name.ptr[i] > 126) return 1;
            id[i] = name.ptr[i];
        }
        idlen = (int32_t)name.len;
        /* an unknown id would open as GMT: ask whether the database has it */
        uint16_t canon[128];
        int8_t system = 0;
        nx_icu.canonical(id, idlen, canon, 127, &system, &st);
        if (st > 0 || !system) return 1;
    }
    for (int32_t i = 0; i < idlen; i++) {
        uint8_t b = id[i] < 127 ? (uint8_t)id[i] : '?';
        nx_str_append(c, &s, &b, 1);
    }
    nx_str_append(c, &s, (const uint8_t*)"\n", 1);
    st = 0;
    /* 1 UCAL_GREGORIAN */
    void* cal = nx_icu.open(id, idlen, "en_US", 1, &st);
    if (st > 0 || !cal) { nx_str_free(c, &s); return 1; }
    /* from 1684, before every zone's first change, to 2200 */
    double at = -9.0e12;
    const double end = 7258118400000.0;
    bool first = true;
    for (int guard = 0; guard < 5000; guard++) {
        st = 0;
        nx_icu.set_millis(cal, at, &st);
        /* 15 UCAL_ZONE_OFFSET, 16 UCAL_DST_OFFSET, in ms */
        int32_t raw = nx_icu.get(cal, 15, &st);
        int32_t dst = nx_icu.get(cal, 16, &st);
        if (st > 0) break;
        nx_icu_period(c, &s, cal, first, at, raw + dst, dst != 0);
        first = false;
        double next = 0;
        st = 0;
        /* 0 UCAL_TZ_TRANSITION_NEXT */
        if (!nx_icu.transition(cal, 0, &next, &st) || st > 0 || next <= at) break;
        if (next >= end) {
            if (dst != 0) {
                nx_icu.set_millis(cal, next, &st);
                nx_icu_period(c, &s, cal, false, next, raw, false);
            }
            break;
        }
        at = next;
    }
    nx_icu.close(cal);
    *out = s;
    return 0;
#else
    (void)name; (void)s; (void)out;
    return 1;
#endif
}
NX_INLINE uint64_t nx_time_monotonic_ns(void) {
#if defined(_WIN32)
    LARGE_INTEGER f, c; QueryPerformanceFrequency(&f); QueryPerformanceCounter(&c);
    return (uint64_t)((double)c.QuadPart * 1e9 / (double)f.QuadPart);
#else
    struct timespec ts; clock_gettime(CLOCK_MONOTONIC, &ts);
    return (uint64_t)ts.tv_sec * 1000000000ULL + (uint64_t)ts.tv_nsec;
#endif
}
NX_INLINE void nx_sleep_ms(uint64_t ms) {
#if defined(_WIN32)
    Sleep((DWORD)ms);
#else
    usleep((useconds_t)(ms * 1000));
#endif
}
NX_INLINE int64_t nx_mono_ms(void) { return (int64_t)(nx_time_monotonic_ns() / 1000000u); }
/* what is left of `timeout_ms` since `start` (ms): -1 for no limit */
NX_INLINE int64_t nx_left_ms(int64_t start, int64_t timeout_ms) {
    if (timeout_ms < 0) return -1;
    int64_t left = timeout_ms - (nx_mono_ms() - start);
    return left > 0 ? left : 0;
}

/* ---- `nx bench` ---- */
/* A value the optimizer must assume is read: what a benchmark computes is
   kept, so the work that makes it is not removed. */
NX_INLINE void nx_bench_keep(const void* p) {
#if defined(__GNUC__) || defined(__clang__)
    __asm__ __volatile__("" : : "r"(p) : "memory");
#else
    static const void* volatile sink; sink = p;
#endif
}
typedef struct nx_bench_result { uint64_t iters; uint32_t samples; uint32_t err; double median_ns, min_ns, max_ns; } nx_bench_result;
NX_INLINE int nx_bench_cmp(const void* a, const void* b) { double x = *(const double*)a, y = *(const double*)b; return (x > y) - (x < y); }
#define NX_BENCH_SAMPLES 21
/* Calibrate (which is the warmup): double the iterations until one sample
   takes `sample_ns`; then time NX_BENCH_SAMPLES samples of that many and
   keep the median. A body slower than a sample gets fewer samples (at
   least 5), so a slow benchmark takes seconds rather than minutes. A body
   that returns an error stops it, the error in `err`. */
NX_INLINE void nx_bench_measure(nx_ctx* c, uint32_t (*f)(nx_ctx*), uint64_t sample_ns, nx_bench_result* r) {
    memset(r, 0, sizeof *r);
    uint64_t n = 1, dt = 0;
    for (;;) {
        uint64_t t0 = nx_time_monotonic_ns();
        for (uint64_t i = 0; i < n; i++) { uint32_t e = f(c); if (e) { r->err = e; return; } }
        dt = nx_time_monotonic_ns() - t0;
        if (dt >= sample_ns || n >= (1ULL << 40)) break;
        uint64_t next = dt > 0 ? (uint64_t)((double)n * (double)sample_ns / (double)dt * 1.2) : n * 10;
        if (next > n * 10) next = n * 10;
        if (next <= n) next = n + 1;
        n = next;
    }
    uint32_t samples = NX_BENCH_SAMPLES;
    if (n == 1 && dt > sample_ns) {
        uint64_t fit = (sample_ns * NX_BENCH_SAMPLES) / dt;
        samples = fit < 5 ? 5 : (uint32_t)fit;
        if (samples > NX_BENCH_SAMPLES) samples = NX_BENCH_SAMPLES;
    }
    double s[NX_BENCH_SAMPLES];
    for (uint32_t k = 0; k < samples; k++) {
        uint64_t t0 = nx_time_monotonic_ns();
        for (uint64_t i = 0; i < n; i++) { uint32_t e = f(c); if (e) { r->err = e; return; } }
        s[k] = (double)(nx_time_monotonic_ns() - t0) / (double)n;
    }
    qsort(s, samples, sizeof(double), nx_bench_cmp);
    r->iters = n; r->samples = samples;
    r->median_ns = s[samples / 2]; r->min_ns = s[0]; r->max_ns = s[samples - 1];
}
/* `ns` with three significant digits in the unit that suits it. */
NX_INLINE const char* nx_bench_time(double ns, char* buf, size_t cap) {
    const char* unit = "ns"; double v = ns;
    if (ns >= 1e9) { v = ns / 1e9; unit = "s"; }
    else if (ns >= 1e6) { v = ns / 1e6; unit = "ms"; }
    else if (ns >= 1e3) { v = ns / 1e3; unit = "\xc2\xb5s"; }
    snprintf(buf, cap, "%.3g %s", v, unit);
    return buf;
}
NX_INLINE uint64_t nx_rng_next(nx_ctx* c) {
    if (!c->rng_seeded) { c->rng ^= (uint64_t)nx_time_monotonic_ns(); c->rng_seeded = true; }
    /* splitmix64 */
    c->rng += 0x9E3779B97F4A7C15ULL;
    uint64_t z = c->rng;
    z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9ULL;
    z = (z ^ (z >> 27)) * 0x94D049BB133111EBULL;
    return z ^ (z >> 31);
}
NX_INLINE int64_t nx_random_int(nx_ctx* c, int64_t lo, int64_t hi, const char* loc) {
    if (hi < lo) nx_panic("random.int: upper bound is below the lower bound", loc);
    uint64_t span = (uint64_t)(hi - lo) + 1;
    if (span == 0) return (int64_t)nx_rng_next(c);
    return lo + (int64_t)(nx_rng_next(c) % span);
}
NX_INLINE double nx_random_float(nx_ctx* c) { return (double)(nx_rng_next(c) >> 11) * (1.0 / 9007199254740992.0); }

/* random.secure: n bytes from the operating system's secure generator, for
   keys, tokens and UUIDs; never the seeded generator above. Windows asks
   BCryptGenRandom, loaded from bcrypt.dll so no program links another
   library; Linux the getrandom system call, made directly so the glibc a
   program needs stays 2.17, or /dev/urandom on a kernel older than 3.17;
   macOS and the BSDs arc4random_buf; WASI getentropy. False when the source
   fails, and the bytes must not be used then. */
#if defined(_WIN32)
typedef LONG (WINAPI* nx_bcrypt_gen_random)(void*, unsigned char*, ULONG, ULONG);
#endif
NX_INLINE bool nx_os_random(uint8_t* p, size_t n) {
    if (n == 0) return true;
#if defined(_WIN32)
    static nx_bcrypt_gen_random gen = NULL;
    if (!gen) {
        HMODULE m = LoadLibraryA("bcrypt.dll");
        if (m) gen = (nx_bcrypt_gen_random)GetProcAddress(m, "BCryptGenRandom");
        if (!gen) return false;
    }
    while (n > 0) {
        ULONG chunk = n > 0x40000000u ? 0x40000000u : (ULONG)n;
        /* 2: BCRYPT_USE_SYSTEM_PREFERRED_RNG, no algorithm handle */
        if (gen(NULL, p, chunk, 2) != 0) return false;
        p += chunk; n -= chunk;
    }
    return true;
#elif defined(NX_WASM)
    while (n > 0) {
        size_t chunk = n > 256 ? 256 : n;
        if (getentropy(p, chunk) != 0) return false;
        p += chunk; n -= chunk;
    }
    return true;
#elif defined(__APPLE__) || defined(__FreeBSD__) || defined(__OpenBSD__) || defined(__NetBSD__) || defined(__DragonFly__)
    arc4random_buf(p, n);
    return true;
#else
#if defined(__linux__) && defined(SYS_getrandom)
    while (n > 0) {
        long r = syscall(SYS_getrandom, p, n, 0);
        if (r > 0) { p += (size_t)r; n -= (size_t)r; continue; }
        if (r < 0 && errno == EINTR) continue;
        if (r < 0 && errno == ENOSYS) break;
        return false;
    }
    if (n == 0) return true;
#endif
#ifndef O_CLOEXEC
#define O_CLOEXEC 0
#endif
    int fd = open("/dev/urandom", O_RDONLY | O_CLOEXEC);
    if (fd < 0) return false;
    /* a regular file planted in its place is not a generator */
    struct stat st;
    if (fstat(fd, &st) != 0 || !S_ISCHR(st.st_mode)) { close(fd); return false; }
    while (n > 0) {
        ssize_t r = read(fd, p, n);
        if (r > 0) { p += (size_t)r; n -= (size_t)r; continue; }
        if (r < 0 && errno == EINTR) continue;
        close(fd);
        return false;
    }
    close(fd);
    return true;
#endif
}

/* ---------------------------------------------------------- starting programs */
#if defined(_WIN32)
/* a command line CommandLineToArgvW takes apart into `argv` again; the
   program's slashes become backslashes, which CreateProcess wants there */
NX_INLINE void nx_win_cmdline(nx_ctx* c, const nx_sl_u8* argv, size_t argc, nx_string* cmd) {
    cmd->ptr = NULL; cmd->len = 0; cmd->cap = 0; cmd->ar = NULL;
    char prog[4096];
    for (size_t i = 0; i < argc; i++) {
        if (i) nx_str_append(c, cmd, (const uint8_t*)" ", 1);
        nx_sl_u8 a = argv[i];
        if (i == 0 && a.len < sizeof prog) {
            for (size_t j = 0; j < a.len; j++) prog[j] = a.ptr[j] == '/' ? '\\' : (char)a.ptr[j];
            a.ptr = (uint8_t*)prog;
        }
        bool quote = a.len == 0;
        for (size_t j = 0; j < a.len && !quote; j++) quote = a.ptr[j] == ' ' || a.ptr[j] == '\t' || a.ptr[j] == '"';
        if (quote) nx_str_append(c, cmd, (const uint8_t*)"\"", 1);
        size_t bs = 0;
        for (size_t j = 0; j < a.len; j++) {
            uint8_t ch = a.ptr[j];
            if (ch == '\\') { bs++; continue; }
            if (ch == '"') { for (size_t k = 0; k < bs * 2 + 1; k++) nx_str_append(c, cmd, (const uint8_t*)"\\", 1); bs = 0; nx_str_append(c, cmd, &ch, 1); continue; }
            for (size_t k = 0; k < bs; k++) nx_str_append(c, cmd, (const uint8_t*)"\\", 1);
            bs = 0;
            nx_str_append(c, cmd, &ch, 1);
        }
        for (size_t k = 0; k < bs * (quote ? 2 : 1); k++) nx_str_append(c, cmd, (const uint8_t*)"\\", 1);
        if (quote) nx_str_append(c, cmd, (const uint8_t*)"\"", 1);
    }
    nx_str_append(c, cmd, (const uint8_t*)"", 1); /* NUL */
}
/* this program's standard handle `which` as one a child can inherit, or
   NULL (the caller closes it once the child has started) */
NX_INLINE HANDLE nx_win_std_dup(DWORD which) {
    HANDLE h = GetStdHandle(which), d = NULL;
    if (!h || h == INVALID_HANDLE_VALUE) return NULL;
    if (!DuplicateHandle(GetCurrentProcess(), h, GetCurrentProcess(), &d, 0, TRUE, DUPLICATE_SAME_ACCESS)) return NULL;
    return d;
}
/* Start `cmd` in `cwd` (NULL: this program's) with `give` as its stdin,
   stdout and stderr, inheritable handles or NULL, and no other handle of
   this program's: a program started at the same time on another thread
   cannot hold these pipes open, nor this one theirs. 0 when it started,
   else CreateProcess's error. */
NX_INLINE DWORD nx_win_start(char* cmd, const char* cwd, HANDLE give[3], PROCESS_INFORMATION* pi) {
    STARTUPINFOEXA si;
    memset(&si, 0, sizeof si);
    si.StartupInfo.cb = sizeof si;
    si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    si.StartupInfo.hStdInput = give[0]; si.StartupInfo.hStdOutput = give[1]; si.StartupInfo.hStdError = give[2];
    /* each handle once; a console's pseudo handle needs no inheriting and cannot be listed */
    HANDLE list[3]; DWORD nl = 0;
    for (int i = 0; i < 3; i++) {
        HANDLE h = give[i];
        if (!h || h == INVALID_HANDLE_VALUE) continue;
        if ((((uintptr_t)h) & 3) == 3 && GetFileType(h) == FILE_TYPE_CHAR) continue;
        bool seen = false;
        for (DWORD j = 0; j < nl; j++) seen = seen || list[j] == h;
        if (!seen) list[nl++] = h;
    }
    LPPROC_THREAD_ATTRIBUTE_LIST attrs = NULL;
    DWORD flags = 0;
    if (nl > 0) {
        SIZE_T size = 0;
        InitializeProcThreadAttributeList(NULL, 1, 0, &size);
        attrs = (LPPROC_THREAD_ATTRIBUTE_LIST)malloc(size);
        if (attrs && InitializeProcThreadAttributeList(attrs, 1, 0, &size)) {
            if (UpdateProcThreadAttribute(attrs, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, list, nl * sizeof(HANDLE), NULL, NULL)) {
                si.lpAttributeList = attrs;
                flags = EXTENDED_STARTUPINFO_PRESENT;
            } else { DeleteProcThreadAttributeList(attrs); free(attrs); attrs = NULL; }
        } else { free(attrs); attrs = NULL; }
    }
    memset(pi, 0, sizeof *pi);
    BOOL ok = CreateProcessA(NULL, cmd, NULL, NULL, TRUE, flags, NULL, cwd, &si.StartupInfo, pi);
    DWORD err = ok ? 0 : GetLastError();
    if (attrs) { DeleteProcThreadAttributeList(attrs); free(attrs); }
    return err;
}
#elif !defined(NX_WASM)
/* a pipe no program inherits: a child gets its end as 0, 1 or 2 */
NX_INLINE int nx_pipe_cloexec(int p[2]) {
#if defined(__linux__) && defined(SYS_pipe2)
    if (syscall(SYS_pipe2, p, O_CLOEXEC) == 0) return 0;
    if (errno != ENOSYS) return -1;
#endif
    if (pipe(p) != 0) return -1;
    fcntl(p[0], F_SETFD, FD_CLOEXEC);
    fcntl(p[1], F_SETFD, FD_CLOEXEC);
    return 0;
}
/* In a child between fork and exec: make `fds[i]` its descriptor i (-1
   leaves i as it is), with only the calls that are safe there. */
NX_INLINE void nx_child_std(int fds[3]) {
    /* one already below 3 would be overwritten before its turn: move it up */
    for (int i = 0; i < 3; i++) {
        if (fds[i] >= 0 && fds[i] < 3 && fds[i] != i) fds[i] = fcntl(fds[i], F_DUPFD_CLOEXEC, 3);
    }
    for (int i = 0; i < 3; i++) {
        if (fds[i] < 0) continue;
        if (fds[i] == i) {
            int fl = fcntl(i, F_GETFD);
            if (fl >= 0) fcntl(i, F_SETFD, fl & ~FD_CLOEXEC);
        } else dup2(fds[i], i);
    }
}
/* Is there a file `prog` names (along PATH when it has no slash)? exec
   says EACCES both for a program that is there but cannot run and for a
   directory on PATH it may not search, which leaves a missing program
   looking like a forbidden one. */
NX_INLINE bool nx_prog_exists(const char* prog) {
    struct stat st;
    if (strchr(prog, '/')) return stat(prog, &st) == 0;
    const char* path = getenv("PATH");
    if (!path || !*path) path = "/bin:/usr/bin";
    char buf[4096];
    size_t pl = strlen(prog);
    for (;;) {
        const char* e = strchr(path, ':');
        size_t dl = e ? (size_t)(e - path) : strlen(path);
        /* an empty entry is the working directory */
        const char* dir = dl ? path : ".";
        if (!dl) dl = 1;
        if (dl + 1 + pl < sizeof buf) {
            memcpy(buf, dir, dl);
            buf[dl] = '/';
            memcpy(buf + dl + 1, prog, pl + 1);
            if (stat(buf, &st) == 0 && !S_ISDIR(st.st_mode)) return true;
        }
        if (!e) return false;
        path = e + 1;
    }
}
/* A write to a pipe whose reader is gone raises SIGPIPE, which would end
   the program: held off while a child's input is written, and one raised
   meanwhile is taken before it is let through (unless it was held already). */
NX_INLINE void nx_sigpipe_hold(sigset_t* old) {
    sigset_t s;
    sigemptyset(&s);
    sigaddset(&s, SIGPIPE);
    pthread_sigmask(SIG_BLOCK, &s, old);
}
NX_INLINE void nx_sigpipe_release(const sigset_t* old) {
    sigset_t s, pending;
    sigemptyset(&s);
    sigaddset(&s, SIGPIPE);
    sigpending(&pending);
    if (sigismember(&pending, SIGPIPE) && !sigismember(old, SIGPIPE)) { int sig; sigwait(&s, &sig); }
    pthread_sigmask(SIG_SETMASK, old, NULL);
}
#endif

/* process.run: spawn argv[0] with the given arguments (searching PATH), wait,
 * and return its exit code. False when the process could not be started. */
#if defined(NX_WASM)
NX_INLINE bool nx_run(nx_ctx* c, const nx_sl_u8* argv, size_t argc, int* code) {
    (void)c; (void)argv; (void)argc; (void)code;
    errno = ENOSYS;
    return false;
}
#else
NX_INLINE bool nx_run(nx_ctx* c, const nx_sl_u8* argv, size_t argc, int* code) {
    if (argc == 0) return false;
    fflush(stdout); fflush(stderr);
#if defined(_WIN32)
    nx_string cmd;
    nx_win_cmdline(c, argv, argc, &cmd);
    /* the child writes where this program does */
    static const DWORD std_ids[3] = { STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE };
    HANDLE give[3], dups[3];
    for (int i = 0; i < 3; i++) {
        dups[i] = nx_win_std_dup(std_ids[i]);
        give[i] = dups[i] ? dups[i] : GetStdHandle(std_ids[i]);
    }
    PROCESS_INFORMATION pi;
    DWORD err = nx_win_start((char*)cmd.ptr, NULL, give, &pi);
    for (int i = 0; i < 3; i++) if (dups[i]) CloseHandle(dups[i]);
    nx_str_free(c, &cmd);
    if (err) return false;
    WaitForSingleObject(pi.hProcess, INFINITE);
    DWORD ec = 1;
    GetExitCodeProcess(pi.hProcess, &ec);
    CloseHandle(pi.hProcess); CloseHandle(pi.hThread);
    *code = (int)ec;
    return true;
#else
    char** av = (char**)nx_alloc_bytes(c, (argc + 1) * sizeof(char*), 8);
    for (size_t i = 0; i < argc; i++) {
        av[i] = (char*)nx_alloc_bytes(c, argv[i].len + 1, 1);
        nx_bytes_copy(av[i], argv[i].ptr, argv[i].len); av[i][argv[i].len] = 0;
    }
    av[argc] = NULL;
    pid_t pid = 0;
    int rc = posix_spawnp(&pid, av[0], NULL, NULL, av, environ);
    for (size_t i = 0; i < argc; i++) nx_free_bytes(c, av[i], argv[i].len + 1);
    nx_free_bytes(c, av, (argc + 1) * sizeof(char*));
    if (rc != 0) return false;
    int st = 0;
    if (waitpid(pid, &st, 0) < 0) return false;
    *code = WIFEXITED(st) ? WEXITSTATUS(st) : 128 + (WIFSIGNALED(st) ? WTERMSIG(st) : 0);
    return true;
#endif
}
#endif
NX_INLINE bool nx_cpath(nx_sl_u8 path, char* buf, size_t cap) {
    if (path.len >= cap) return false;
    nx_bytes_copy(buf, path.ptr, path.len); buf[path.len] = 0;
    return true;
}
/* Run a program with its stdin fed from `input`, in `cwd` when given, and
   its stdout and stderr captured. The captured text is kept for
   nx_last_stdout / nx_last_stderr to hand over; each thread has its own. */
NX_STATE NX_THREAD_LOCAL nx_string nx_cap_out, nx_cap_err;
NX_INLINE void nx_cap_reset(nx_ctx* c) {
    nx_str_free(c, &nx_cap_out); nx_str_free(c, &nx_cap_err);
    nx_cap_out.ptr = NULL; nx_cap_out.len = 0; nx_cap_out.cap = 0; nx_cap_out.ar = c->arena;
    nx_cap_err.ptr = NULL; nx_cap_err.len = 0; nx_cap_err.cap = 0; nx_cap_err.ar = c->arena;
}
NX_INLINE nx_string nx_last_stdout(nx_ctx* c) {
    nx_string s = nx_cap_out;
    nx_cap_out.ptr = NULL; nx_cap_out.len = 0; nx_cap_out.cap = 0; nx_cap_out.ar = c->arena;
    return s;
}
NX_INLINE nx_string nx_last_stderr(nx_ctx* c) {
    nx_string s = nx_cap_err;
    nx_cap_err.ptr = NULL; nx_cap_err.len = 0; nx_cap_err.cap = 0; nx_cap_err.ar = c->arena;
    return s;
}
#if defined(_WIN32)
NX_INLINE void nx_win_drain(nx_ctx* c, HANDLE h, nx_string* out) {
    char buf[65536]; DWORD n;
    while (ReadFile(h, buf, sizeof buf, &n, NULL) && n > 0) nx_str_append(c, out, (const uint8_t*)buf, n);
}
/* stderr is drained on a helper thread while this one drains stdout, so a child
   that fills one pipe before finishing the other cannot stall */
typedef struct { nx_ctx* c; HANDLE h; nx_string* out; } nx_win_drain_job;
static DWORD WINAPI nx_win_drain_thread(LPVOID p) {
    nx_win_drain_job* j = (nx_win_drain_job*)p;
    nx_win_drain(j->c, j->h, j->out);
    return 0;
}
/* the child's input, written on a helper thread while the output is
   drained, so a child that writes before it reads cannot stall; it stops
   when the child no longer reads, and closes the pipe */
typedef struct { HANDLE h; const uint8_t* p; size_t n; } nx_win_feed_job;
NX_INLINE void nx_win_feed(nx_win_feed_job* j) {
    size_t off = 0;
    while (off < j->n) {
        DWORD chunk = (DWORD)((j->n - off) > (1u << 30) ? (1u << 30) : (j->n - off));
        DWORD w = 0;
        if (!WriteFile(j->h, j->p + off, chunk, &w, NULL) || w == 0) break;
        off += w;
    }
    CloseHandle(j->h);
}
static DWORD WINAPI nx_win_feed_thread(LPVOID p) {
    nx_win_feed((nx_win_feed_job*)p);
    return 0;
}
#endif
#if defined(NX_WASM)
NX_INLINE bool nx_run_capture(nx_ctx* c, const nx_sl_u8* argv, size_t argc, nx_sl_u8 input, nx_sl_u8 cwd, int* code) {
    (void)c; (void)argv; (void)argc; (void)input; (void)cwd; (void)code;
    nx_cap_reset(c);
    errno = ENOSYS;
    return false;
}
#else
NX_INLINE bool nx_run_capture(nx_ctx* c, const nx_sl_u8* argv, size_t argc, nx_sl_u8 input, nx_sl_u8 cwd, int* code) {
    if (argc == 0) return false;
    fflush(stdout); fflush(stderr);
    nx_cap_reset(c);
    char dir[4096];
    const char* cwdp = NULL;
    if (cwd.len > 0) { if (!nx_cpath(cwd, dir, sizeof dir)) return false; cwdp = dir; }
#if defined(_WIN32)
    nx_string cmd;
    nx_win_cmdline(c, argv, argc, &cmd);
    SECURITY_ATTRIBUTES sa; sa.nLength = sizeof sa; sa.bInheritHandle = TRUE; sa.lpSecurityDescriptor = NULL;
    HANDLE in_r = NULL, in_w = NULL, out_r = NULL, out_w = NULL, err_r = NULL, err_w = NULL;
    if (!CreatePipe(&in_r, &in_w, &sa, 1 << 20) || !CreatePipe(&out_r, &out_w, &sa, 1 << 20) || !CreatePipe(&err_r, &err_w, &sa, 1 << 20)) { nx_str_free(c, &cmd); return false; }
    SetHandleInformation(in_w, HANDLE_FLAG_INHERIT, 0);
    SetHandleInformation(out_r, HANDLE_FLAG_INHERIT, 0);
    SetHandleInformation(err_r, HANDLE_FLAG_INHERIT, 0);
    HANDLE give[3] = { in_r, out_w, err_w };
    PROCESS_INFORMATION pi;
    DWORD err = nx_win_start((char*)cmd.ptr, cwdp, give, &pi);
    nx_str_free(c, &cmd);
    CloseHandle(in_r); CloseHandle(out_w); CloseHandle(err_w);
    if (err) { CloseHandle(in_w); CloseHandle(out_r); CloseHandle(err_r); return false; }
    /* the input goes in on one helper thread and stderr comes out on
       another while this one drains stdout: a child that writes before it
       reads, or fills one pipe before finishing the other, cannot stall */
    nx_win_feed_job feed; feed.h = in_w; feed.p = input.ptr; feed.n = input.len;
    HANDLE feeder = NULL;
    if (input.len > 0) feeder = CreateThread(NULL, 0, nx_win_feed_thread, &feed, 0, NULL);
    if (!feeder) nx_win_feed(&feed);
    nx_string err_buf; err_buf.ptr = NULL; err_buf.len = 0; err_buf.cap = 0; err_buf.ar = c->arena;
    nx_win_drain_job job; job.c = c; job.h = err_r; job.out = &err_buf;
    HANDLE drain = CreateThread(NULL, 0, nx_win_drain_thread, &job, 0, NULL);
    nx_win_drain(c, out_r, &nx_cap_out);
    if (drain) { WaitForSingleObject(drain, INFINITE); CloseHandle(drain); } else nx_win_drain(c, err_r, &err_buf);
    if (feeder) { WaitForSingleObject(feeder, INFINITE); CloseHandle(feeder); }
    nx_cap_err = err_buf;
    CloseHandle(out_r); CloseHandle(err_r);
    WaitForSingleObject(pi.hProcess, INFINITE);
    DWORD ec = 1;
    GetExitCodeProcess(pi.hProcess, &ec);
    CloseHandle(pi.hProcess); CloseHandle(pi.hThread);
    *code = (int)ec;
    return true;
#else
    char** av = (char**)nx_alloc_bytes(c, (argc + 1) * sizeof(char*), 8);
    for (size_t i = 0; i < argc; i++) {
        av[i] = (char*)nx_alloc_bytes(c, argv[i].len + 1, 1);
        nx_bytes_copy(av[i], argv[i].ptr, argv[i].len); av[i][argv[i].len] = 0;
    }
    av[argc] = NULL;
    int inp[2], outp[2], errp[2];
    if (nx_pipe_cloexec(inp) != 0 || nx_pipe_cloexec(outp) != 0 || nx_pipe_cloexec(errp) != 0) return false;
    pid_t pid = fork();
    if (pid < 0) return false;
    if (pid == 0) {
        int fds[3] = { inp[0], outp[1], errp[1] };
        nx_child_std(fds);
        if (cwdp && chdir(cwdp) != 0) _exit(126);
        execvp(av[0], av);
        _exit(127);
    }
    close(inp[0]); close(outp[1]); close(errp[1]);
    for (size_t i = 0; i < argc; i++) nx_free_bytes(c, av[i], argv[i].len + 1);
    nx_free_bytes(c, av, (argc + 1) * sizeof(char*));
    /* the input is written as the child takes it while both output pipes
       are drained as it fills them: a child that writes before it reads, or
       fills one pipe before finishing the other, cannot stall. A write to a
       child that stopped reading fails with EPIPE, and SIGPIPE is held off. */
    sigset_t old_set;
    nx_sigpipe_hold(&old_set);
    size_t fed = 0;
    if (input.len == 0) { close(inp[1]); inp[1] = -1; }
    else { int fl = fcntl(inp[1], F_GETFL); if (fl >= 0) fcntl(inp[1], F_SETFL, fl | O_NONBLOCK); }
    char buf[65536]; ssize_t n;
    struct pollfd pfd[3];
    pfd[0].fd = outp[0]; pfd[0].events = POLLIN;
    pfd[1].fd = errp[0]; pfd[1].events = POLLIN;
    pfd[2].fd = inp[1]; pfd[2].events = POLLOUT;
    int open_fds = 2;
    while (open_fds > 0 || pfd[2].fd >= 0) {
        if (poll(pfd, 3, -1) < 0) { if (errno == EINTR) continue; break; }
        for (int i = 0; i < 2; i++) {
            if (pfd[i].fd < 0 || !(pfd[i].revents & (POLLIN | POLLHUP | POLLERR))) continue;
            n = read(pfd[i].fd, buf, sizeof buf);
            if (n > 0) nx_str_append(c, i == 0 ? &nx_cap_out : &nx_cap_err, (const uint8_t*)buf, (size_t)n);
            else if (n == 0 || errno != EINTR) { pfd[i].fd = -1; open_fds--; }
        }
        if (pfd[2].fd >= 0 && (pfd[2].revents & (POLLOUT | POLLHUP | POLLERR))) {
            ssize_t w = write(pfd[2].fd, input.ptr + fed, input.len - fed);
            if (w > 0) fed += (size_t)w;
            /* EPIPE, or another failure: the child reads no more */
            else if (!(w < 0 && (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR))) fed = input.len;
            if (fed >= input.len) { close(pfd[2].fd); pfd[2].fd = -1; }
        }
    }
    if (pfd[2].fd >= 0) close(pfd[2].fd);
    nx_sigpipe_release(&old_set);
    close(outp[0]); close(errp[0]);
    int st = 0;
    if (waitpid(pid, &st, 0) < 0) return false;
    *code = WIFEXITED(st) ? WEXITSTATUS(st) : 128 + (WIFSIGNALED(st) ? WTERMSIG(st) : 0);
    return true;
#endif
}
#endif

/* --------------------------------------------------------- child processes */
/* process.spawn and the child_* calls: a program started with pipes to this
   one, written to and read from while it runs. Output that arrives while
   the caller waits for something else (its input to go in, the other
   stream, the exit) is kept until it is read, so the child never stalls
   on a full pipe while this program waits on it. A handle is 1 + its slot.
   Results: 0 ok, 1 no such program, 3 timed out, 4 any other failure (the
   codes of the socket calls). Timeouts in ms: negative waits for ever, 0
   looks without waiting. */
#define NX_MAX_CHILDREN 64
typedef struct { uint8_t* p; size_t len, cap, pos; } nx_cbuf;
typedef struct {
#if defined(_WIN32)
    HANDLE h;              /* this program's end, overlapped; NULL once closed */
    OVERLAPPED ov;
    bool busy;             /* a read or write is in flight */
    uint8_t* stage;        /* where an output pipe's reads land */
#else
    int fd;                /* -1 once closed */
#endif
    bool piped;
    bool eof;              /* an output pipe the child closed */
    nx_cbuf buf;           /* output read and not yet taken */
} nx_cpipe;
typedef struct {
    int state;             /* 0 free, 1 being set up or let go, 2 in use */
    int64_t pid;
#if defined(_WIN32)
    HANDLE proc;
    int32_t sent;          /* the signal child_signal ended it with */
#endif
    nx_cpipe in, out, err;
    bool done;             /* it ended and was waited for */
    int32_t code, sig;
} nx_child;
NX_STATE nx_child nx_children[NX_MAX_CHILDREN];
enum { NX_CH_WRITTEN, NX_CH_OUT, NX_CH_ERR, NX_CH_EXIT };

NX_INLINE nx_child* nx_child_at(int64_t h) {
    if (h < 1 || h > NX_MAX_CHILDREN) return NULL;
    nx_child* ch = &nx_children[h - 1];
    return __atomic_load_n(&ch->state, __ATOMIC_ACQUIRE) == 2 ? ch : NULL;
}
NX_INLINE int64_t nx_child_pid(int64_t h) {
    nx_child* ch = nx_child_at(h);
    return ch ? ch->pid : -1;
}
#if defined(NX_WASM)
NX_INLINE int32_t nx_child_spawn(nx_ctx* c, const nx_sl_u8* argv, size_t argc, nx_sl_u8 cwd, int64_t flags, int64_t* out) {
    (void)c; (void)argv; (void)argc; (void)cwd; (void)flags; (void)out;
    return 4;
}
NX_INLINE int32_t nx_child_write(int64_t h, nx_sl_u8 data, int64_t timeout_ms) { (void)h; (void)data; (void)timeout_ms; return 4; }
NX_INLINE int32_t nx_child_read(nx_ctx* c, int64_t h, int64_t stream, size_t n, int64_t timeout_ms, nx_string* out) {
    (void)c; (void)h; (void)stream; (void)n; (void)timeout_ms; (void)out;
    return 4;
}
NX_INLINE int32_t nx_child_wait(int64_t h, int64_t timeout_ms, int64_t* status) { (void)h; (void)timeout_ms; (void)status; return 4; }
NX_INLINE int32_t nx_child_signal(int64_t h, int32_t sig) { (void)h; (void)sig; return 4; }
NX_INLINE void nx_child_close_input(int64_t h) { (void)h; }
NX_INLINE void nx_child_close(int64_t h) { (void)h; }
NX_INLINE void nx_trap_signals(void) { }
NX_INLINE int32_t nx_next_signal(int64_t timeout_ms) { (void)timeout_ms; return 0; }
#else
NX_INLINE int nx_child_claim(void) {
    for (int i = 0; i < NX_MAX_CHILDREN; i++) {
        int expected = 0;
        if (__atomic_compare_exchange_n(&nx_children[i].state, &expected, 1, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) return i;
    }
    return -1;
}
NX_INLINE void nx_child_reset(nx_child* ch) {
    ch->pid = 0; ch->done = false; ch->code = 0; ch->sig = 0;
#if defined(_WIN32)
    ch->proc = NULL; ch->sent = 0;
#endif
    nx_cpipe* pipes[3] = { &ch->in, &ch->out, &ch->err };
    for (int i = 0; i < 3; i++) {
        memset(pipes[i], 0, sizeof *pipes[i]);
#if !defined(_WIN32)
        pipes[i]->fd = -1;
#endif
    }
}
NX_INLINE void nx_cbuf_add(nx_cbuf* b, const uint8_t* p, size_t n) {
    if (b->pos == b->len) { b->pos = 0; b->len = 0; }
    else if (b->pos > 0 && b->len + n > b->cap) {
        memmove(b->p, b->p + b->pos, b->len - b->pos);
        b->len -= b->pos; b->pos = 0;
    }
    if (b->len + n > b->cap) {
        size_t cap = b->cap ? b->cap : 65536;
        while (cap < b->len + n) cap *= 2;
        uint8_t* q = (uint8_t*)realloc(b->p, cap);
        if (!q) nx_panic("out of memory keeping a child's output", "process");
        b->p = q; b->cap = cap;
    }
    memcpy(b->p + b->len, p, n);
    b->len += n;
}
NX_INLINE bool nx_cpipe_ready(const nx_cpipe* pp) { return pp->buf.pos < pp->buf.len || pp->eof; }
#if defined(_WIN32)
/* A pipe whose far end a child gets: this program's end overlapped, so
   reads, writes and the exit can be waited for together and with a
   timeout (an anonymous pipe cannot be); the child's an ordinary one. */
NX_STATE volatile LONG nx_pipe_serial;
#ifndef PIPE_REJECT_REMOTE_CLIENTS
#define PIPE_REJECT_REMOTE_CLIENTS 0x00000008
#endif
NX_INLINE bool nx_win_pipe(bool child_reads, HANDLE* ours, HANDLE* theirs) {
    char name[96];
    snprintf(name, sizeof name, "\\\\.\\pipe\\nx-%lu-%ld", (unsigned long)GetCurrentProcessId(), (long)InterlockedIncrement(&nx_pipe_serial));
    DWORD mode = (child_reads ? PIPE_ACCESS_OUTBOUND : PIPE_ACCESS_INBOUND) | FILE_FLAG_OVERLAPPED | FILE_FLAG_FIRST_PIPE_INSTANCE;
    HANDLE s = CreateNamedPipeA(name, mode, PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS, 1, 65536, 65536, 0, NULL);
    if (s == INVALID_HANDLE_VALUE) return false;
    SECURITY_ATTRIBUTES sa; sa.nLength = sizeof sa; sa.bInheritHandle = TRUE; sa.lpSecurityDescriptor = NULL;
    HANDLE t = CreateFileA(name, child_reads ? GENERIC_READ : GENERIC_WRITE, 0, &sa, OPEN_EXISTING, 0, NULL);
    if (t == INVALID_HANDLE_VALUE) { CloseHandle(s); return false; }
    *ours = s; *theirs = t;
    return true;
}
NX_INLINE void nx_cpipe_close(nx_cpipe* pp) {
    if (!pp->h) return;
    if (pp->busy) {
        DWORD x = 0;
        CancelIoEx(pp->h, &pp->ov);
        GetOverlappedResult(pp->h, &pp->ov, &x, TRUE);
        pp->busy = false;
    }
    CloseHandle(pp->h);
    pp->h = NULL;
}
/* keep a read waiting on an output pipe; what comes at once is kept */
NX_INLINE void nx_cpipe_post(nx_cpipe* pp) {
    while (pp->h && !pp->busy) {
        DWORD got = 0;
        ResetEvent(pp->ov.hEvent);
        if (!ReadFile(pp->h, pp->stage, 65536, NULL, &pp->ov)) {
            if (GetLastError() == ERROR_IO_PENDING) { pp->busy = true; return; }
            /* ERROR_BROKEN_PIPE: the child closed its end */
            nx_cpipe_close(pp);
            pp->eof = true;
            return;
        }
        if (GetOverlappedResult(pp->h, &pp->ov, &got, FALSE) && got > 0) nx_cbuf_add(&pp->buf, pp->stage, got);
    }
}
/* take a read that completed */
NX_INLINE void nx_cpipe_finish(nx_cpipe* pp) {
    if (!pp->busy) return;
    DWORD got = 0;
    if (GetOverlappedResult(pp->h, &pp->ov, &got, FALSE)) {
        pp->busy = false;
        if (got > 0) nx_cbuf_add(&pp->buf, pp->stage, got);
        return;
    }
    if (GetLastError() == ERROR_IO_INCOMPLETE) return;
    pp->busy = false;
    nx_cpipe_close(pp);
    pp->eof = true;
}
NX_INLINE bool nx_child_reap(nx_child* ch, bool block) {
    if (ch->done) return true;
    if (WaitForSingleObject(ch->proc, block ? INFINITE : 0) != WAIT_OBJECT_0) return false;
    DWORD ec = 1;
    GetExitCodeProcess(ch->proc, &ec);
    ch->done = true;
    ch->code = (int32_t)ec;
    ch->sig = ch->sent && ec == (DWORD)(128 + ch->sent) ? ch->sent : 0;
    return true;
}
#else
NX_INLINE void nx_cpipe_close(nx_cpipe* pp) {
    if (pp->fd >= 0) { close(pp->fd); pp->fd = -1; }
}
/* one read from an output pipe the poll found ready */
NX_INLINE void nx_cpipe_pull(nx_cpipe* pp) {
    uint8_t buf[65536];
    ssize_t n = read(pp->fd, buf, sizeof buf);
    if (n > 0) { nx_cbuf_add(&pp->buf, buf, (size_t)n); return; }
    if (n < 0 && (errno == EINTR || errno == EAGAIN || errno == EWOULDBLOCK)) return;
    nx_cpipe_close(pp);
    pp->eof = true;
}
NX_INLINE bool nx_child_reap(nx_child* ch, bool block) {
    if (ch->done) return true;
    int st = 0;
    pid_t r;
    do { r = waitpid((pid_t)ch->pid, &st, block ? 0 : WNOHANG); } while (r < 0 && errno == EINTR);
    if (r == 0) return false;
    ch->done = true;
    if (r < 0) { ch->code = -1; ch->sig = 0; return true; } /* waited for elsewhere */
    if (WIFSIGNALED(st)) { ch->sig = WTERMSIG(st); ch->code = 128 + ch->sig; }
    else { ch->sig = 0; ch->code = WIFEXITED(st) ? WEXITSTATUS(st) : -1; }
    return true;
}
#endif
NX_INLINE bool nx_child_holds(nx_child* ch, int until, size_t n, const size_t* off) {
    if (until == NX_CH_WRITTEN) return *off >= n;
    if (until == NX_CH_OUT) return nx_cpipe_ready(&ch->out);
    if (until == NX_CH_ERR) return nx_cpipe_ready(&ch->err);
    return nx_child_reap(ch, false);
}
/* Serve the child's pipes until `until` holds or `timeout_ms` passes:
   output that arrives is kept, and for NX_CH_WRITTEN `src[*off..n]` goes in
   as the child takes it. 0 when it held, 3 on timeout, 4 when the input
   broke (the child reads no more) or waiting failed. */
static int32_t nx_child_serve(nx_child* ch, int until, const uint8_t* src, size_t n, size_t* off, int64_t timeout_ms) {
    int64_t start = nx_mono_ms();
#if defined(_WIN32)
    int32_t result = 3;
    for (;;) {
        nx_cpipe_post(&ch->out);
        nx_cpipe_post(&ch->err);
        if (until == NX_CH_WRITTEN && !ch->in.busy && *off < n) {
            if (!ch->in.h) { result = 4; break; }
            DWORD chunk = (DWORD)(n - *off > (1u << 30) ? (1u << 30) : n - *off), w = 0;
            ResetEvent(ch->in.ov.hEvent);
            if (WriteFile(ch->in.h, src + *off, chunk, NULL, &ch->in.ov)) {
                if (GetOverlappedResult(ch->in.h, &ch->in.ov, &w, FALSE)) *off += w;
                continue;
            }
            /* ERROR_NO_DATA or ERROR_BROKEN_PIPE: the child reads no more */
            if (GetLastError() != ERROR_IO_PENDING) { result = 4; break; }
            ch->in.busy = true;
        }
        if (nx_child_holds(ch, until, n, off)) { result = 0; break; }
        int64_t left = nx_left_ms(start, timeout_ms);
        HANDLE ev[4];
        DWORD k = 0;
        if (ch->out.busy) ev[k++] = ch->out.ov.hEvent;
        if (ch->err.busy) ev[k++] = ch->err.ov.hEvent;
        if (ch->in.busy) ev[k++] = ch->in.ov.hEvent;
        if (until == NX_CH_EXIT) ev[k++] = ch->proc;
        if (k == 0) { result = 4; break; } /* nothing could change: a stream not piped */
        DWORD r = WaitForMultipleObjects(k, ev, FALSE, left < 0 ? INFINITE : (DWORD)(left > 0x7fffffff ? 0x7fffffff : left));
        if (r == WAIT_FAILED) { result = 4; break; }
        nx_cpipe_finish(&ch->out);
        nx_cpipe_finish(&ch->err);
        if (ch->in.busy) {
            DWORD w = 0;
            if (GetOverlappedResult(ch->in.h, &ch->in.ov, &w, FALSE)) { ch->in.busy = false; *off += w; }
            else if (GetLastError() != ERROR_IO_INCOMPLETE) { ch->in.busy = false; result = 4; break; }
        }
        if (nx_child_holds(ch, until, n, off)) { result = 0; break; }
        if (r == WAIT_TIMEOUT && nx_left_ms(start, timeout_ms) == 0) { result = 3; break; }
    }
    /* a write still in flight reads the caller's bytes: it is called off before they go */
    if (ch->in.busy) {
        DWORD w = 0;
        CancelIoEx(ch->in.h, &ch->in.ov);
        if (GetOverlappedResult(ch->in.h, &ch->in.ov, &w, TRUE)) *off += w;
        ch->in.busy = false;
    }
    return result;
#else
    int nap = 1;
    for (;;) {
        if (nx_child_holds(ch, until, n, off)) return 0;
        int64_t left = nx_left_ms(start, timeout_ms);
        struct pollfd p[3];
        int np = 0, io = -1, ie = -1, ii = -1;
        if (ch->out.fd >= 0) { p[np].fd = ch->out.fd; p[np].events = POLLIN; p[np].revents = 0; io = np++; }
        if (ch->err.fd >= 0) { p[np].fd = ch->err.fd; p[np].events = POLLIN; p[np].revents = 0; ie = np++; }
        if (until == NX_CH_WRITTEN) {
            if (ch->in.fd < 0) return 4;
            p[np].fd = ch->in.fd; p[np].events = POLLOUT; p[np].revents = 0; ii = np++;
        }
        int wait = left < 0 ? -1 : left > 1000000000 ? 1000000000 : (int)left;
        if (until == NX_CH_EXIT) {
            /* an exit cannot be polled for: with pipes to watch, look again
               every 20 ms; with none, wait for it, or nap a little longer each time */
            if (np == 0) {
                if (wait < 0) { nx_child_reap(ch, true); continue; }
                if (nap < wait) wait = nap;
                if (nap < 32) nap *= 2;
            } else if (wait < 0 || wait > 20) wait = 20;
        } else if (np == 0) return 4; /* nothing could change: a stream not piped */
        int r = 0;
        if (np == 0) nx_sleep_ms((uint64_t)wait);
        else r = poll(p, (nfds_t)np, wait);
        if (r < 0 && errno != EINTR) return 4;
        if (r > 0) {
            if (io >= 0 && p[io].revents) nx_cpipe_pull(&ch->out);
            if (ie >= 0 && p[ie].revents) nx_cpipe_pull(&ch->err);
            if (ii >= 0 && p[ii].revents) {
                ssize_t w = write(ch->in.fd, src + *off, n - *off);
                if (w > 0) *off += (size_t)w;
                /* EPIPE: the child reads no more */
                else if (w < 0 && errno != EAGAIN && errno != EWOULDBLOCK && errno != EINTR) return 4;
            }
        }
        if (!nx_child_holds(ch, until, n, off) && nx_left_ms(start, timeout_ms) == 0) return 3;
    }
#endif
}
/* Start argv[0] (found on PATH) in `cwd` when given. `flags` says where each
   stream goes, two bits each (stdin, then stdout, then stderr): 0 this
   program's, 1 a pipe, 2 nowhere (the null device); 3 for stderr: into
   stdout. `*out` gets the handle. */
NX_INLINE int32_t nx_child_spawn(nx_ctx* c, const nx_sl_u8* argv, size_t argc, nx_sl_u8 cwd, int64_t flags, int64_t* out) {
    if (argc == 0) return 4;
    char dir[4096];
    const char* cwdp = NULL;
    if (cwd.len > 0) { if (!nx_cpath(cwd, dir, sizeof dir)) return 4; cwdp = dir; }
    int mode[3] = { (int)(flags & 3), (int)((flags >> 2) & 3), (int)((flags >> 4) & 3) };
    if (mode[0] == 3 || mode[1] == 3) return 4;
    int slot = nx_child_claim();
    if (slot < 0) return 4;
    nx_child* ch = &nx_children[slot];
    nx_child_reset(ch);
    nx_cpipe* pipes[3] = { &ch->in, &ch->out, &ch->err };
    fflush(stdout); fflush(stderr);
#if defined(_WIN32)
    static const DWORD std_ids[3] = { STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE };
    SECURITY_ATTRIBUTES sa; sa.nLength = sizeof sa; sa.bInheritHandle = TRUE; sa.lpSecurityDescriptor = NULL;
    HANDLE ours[3] = { NULL, NULL, NULL }, give[3] = { NULL, NULL, NULL }, shut[3] = { NULL, NULL, NULL };
    HANDLE nul = NULL;
    int nshut = 0;
    bool ok = true;
    for (int i = 0; i < 3 && ok; i++) {
        if (mode[i] == 1) {
            ok = nx_win_pipe(i == 0, &ours[i], &give[i]);
            if (ok) shut[nshut++] = give[i];
        } else if (mode[i] == 2) {
            if (!nul) {
                nul = CreateFileA("NUL", GENERIC_READ | GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, &sa, OPEN_EXISTING, 0, NULL);
                if (nul == INVALID_HANDLE_VALUE) { nul = NULL; ok = false; break; }
                shut[nshut++] = nul;
            }
            give[i] = nul;
        } else if (mode[i] == 3) give[i] = give[1];
        else {
            HANDLE d = nx_win_std_dup(std_ids[i]);
            if (d) { give[i] = d; shut[nshut++] = d; } else give[i] = GetStdHandle(std_ids[i]);
        }
    }
    DWORD err = 0;
    PROCESS_INFORMATION pi;
    if (ok) {
        nx_string cmd;
        nx_win_cmdline(c, argv, argc, &cmd);
        err = nx_win_start((char*)cmd.ptr, cwdp, give, &pi);
        nx_str_free(c, &cmd);
    }
    for (int i = 0; i < nshut; i++) CloseHandle(shut[i]);
    if (!ok || err) {
        for (int i = 0; i < 3; i++) if (ours[i]) CloseHandle(ours[i]);
        __atomic_store_n(&ch->state, 0, __ATOMIC_RELEASE);
        return err == ERROR_FILE_NOT_FOUND || err == ERROR_PATH_NOT_FOUND ? 1 : 4;
    }
    CloseHandle(pi.hThread);
    ch->proc = pi.hProcess;
    ch->pid = (int64_t)pi.dwProcessId;
    for (int i = 0; i < 3; i++) {
        if (!ours[i]) continue;
        nx_cpipe* pp = pipes[i];
        pp->h = ours[i];
        pp->piped = true;
        pp->ov.hEvent = CreateEventA(NULL, TRUE, FALSE, NULL);
        if (i > 0) pp->stage = (uint8_t*)malloc(65536);
    }
#else
    char** av = (char**)nx_alloc_bytes(c, (argc + 1) * sizeof(char*), 8);
    for (size_t i = 0; i < argc; i++) {
        av[i] = (char*)nx_alloc_bytes(c, argv[i].len + 1, 1);
        nx_bytes_copy(av[i], argv[i].ptr, argv[i].len); av[i][argv[i].len] = 0;
    }
    av[argc] = NULL;
    int ends[3][2] = { { -1, -1 }, { -1, -1 }, { -1, -1 } };
    int nul = -1, fail[2] = { -1, -1 };
    bool ok = nx_pipe_cloexec(fail) == 0;
    for (int i = 0; i < 3 && ok; i++) {
        if (mode[i] == 1) ok = nx_pipe_cloexec(ends[i]) == 0;
        else if (mode[i] == 2 && nul < 0) { nul = open("/dev/null", O_RDWR | O_CLOEXEC); ok = nul >= 0; }
    }
    pid_t pid = ok ? fork() : -1;
    if (pid == 0) {
        /* the child: its end of each pipe, the null device, or this program's */
        int fds[3];
        for (int i = 0; i < 3; i++) {
            if (mode[i] == 1) fds[i] = i == 0 ? ends[0][0] : ends[i][1];
            else if (mode[i] == 2) fds[i] = nul;
            else if (mode[i] == 3) fds[i] = fds[1] >= 0 ? fds[1] : 1;
            else fds[i] = -1;
        }
        nx_child_std(fds);
        int why[2] = { 0, 0 };
        if (cwdp && chdir(cwdp) != 0) { why[0] = 1; why[1] = errno; }
        else {
            /* what this program held back or caught, the child starts without */
            sigset_t none;
            sigemptyset(&none);
            sigprocmask(SIG_SETMASK, &none, NULL);
            execvp(av[0], av);
            why[1] = errno;
        }
        /* the fail pipe closes when exec succeeds; failing, it says why */
        ssize_t w = write(fail[1], why, sizeof why);
        (void)w;
        _exit(127);
    }
    /* the child's ends are its own now */
    if (ends[0][0] >= 0) close(ends[0][0]);
    if (ends[1][1] >= 0) close(ends[1][1]);
    if (ends[2][1] >= 0) close(ends[2][1]);
    if (nul >= 0) close(nul);
    if (fail[1] >= 0) close(fail[1]);
    int why[2] = { 0, 0 };
    ssize_t got = 0;
    if (pid > 0) {
        do { got = read(fail[0], why, sizeof why); } while (got < 0 && errno == EINTR);
    }
    if (fail[0] >= 0) close(fail[0]);
    bool failed = pid <= 0 || got == (ssize_t)sizeof why;
    /* not found: exec found no such file anywhere it looked */
    bool missing = failed && pid > 0 && why[0] == 0 && (why[1] == ENOENT || why[1] == ENOTDIR || why[1] == EACCES) && !nx_prog_exists(av[0]);
    for (size_t i = 0; i < argc; i++) nx_free_bytes(c, av[i], argv[i].len + 1);
    nx_free_bytes(c, av, (argc + 1) * sizeof(char*));
    if (failed) {
        if (pid > 0) { int st; while (waitpid(pid, &st, 0) < 0 && errno == EINTR) { } }
        if (ends[0][1] >= 0) close(ends[0][1]);
        if (ends[1][0] >= 0) close(ends[1][0]);
        if (ends[2][0] >= 0) close(ends[2][0]);
        __atomic_store_n(&ch->state, 0, __ATOMIC_RELEASE);
        return missing ? 1 : 4;
    }
    ch->pid = (int64_t)pid;
    ch->in.fd = ends[0][1];
    ch->out.fd = ends[1][0];
    ch->err.fd = ends[2][0];
    for (int i = 0; i < 3; i++) pipes[i]->piped = mode[i] == 1;
    /* the input goes in as the child takes it: a write never blocks */
    if (ch->in.fd >= 0) { int fl = fcntl(ch->in.fd, F_GETFL); if (fl >= 0) fcntl(ch->in.fd, F_SETFL, fl | O_NONBLOCK); }
#endif
    __atomic_store_n(&ch->state, 2, __ATOMIC_RELEASE);
    *out = slot + 1;
    return 0;
}
/* Write all of `data` as the child takes it, keeping its output meanwhile. */
NX_INLINE int32_t nx_child_write(int64_t h, nx_sl_u8 data, int64_t timeout_ms) {
    nx_child* ch = nx_child_at(h);
    if (!ch || !ch->in.piped) return 4;
    size_t off = 0;
#if defined(_WIN32)
    return nx_child_serve(ch, NX_CH_WRITTEN, data.ptr, data.len, &off, timeout_ms);
#else
    sigset_t old;
    nx_sigpipe_hold(&old);
    int32_t r = nx_child_serve(ch, NX_CH_WRITTEN, data.ptr, data.len, &off, timeout_ms);
    nx_sigpipe_release(&old);
    return r;
#endif
}
/* the end of the child's input */
NX_INLINE void nx_child_close_input(int64_t h) {
    nx_child* ch = nx_child_at(h);
    if (ch) nx_cpipe_close(&ch->in);
}
/* Up to `n` bytes the child wrote to `stream` (1 stdout, 2 stderr), waiting
   for some; empty at its end. */
NX_INLINE int32_t nx_child_read(nx_ctx* c, int64_t h, int64_t stream, size_t n, int64_t timeout_ms, nx_string* out) {
    nx_child* ch = nx_child_at(h);
    if (!ch || (stream != 1 && stream != 2)) return 4;
    nx_cpipe* pp = stream == 2 ? &ch->err : &ch->out;
    if (!pp->piped) return 4;
    size_t none = 0;
    int32_t r = nx_child_serve(ch, stream == 2 ? NX_CH_ERR : NX_CH_OUT, NULL, 0, &none, timeout_ms);
    if (r) return r;
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    size_t have = pp->buf.len - pp->buf.pos;
    if (have > n) have = n;
    if (have > 0) { nx_str_append(c, &s, pp->buf.p + pp->buf.pos, have); pp->buf.pos += have; }
    *out = s;
    return 0;
}
/* Wait for the child to end, keeping its output meanwhile; `*status` is
   its exit code in the low 32 bits and the signal that ended it above. */
NX_INLINE int32_t nx_child_wait(int64_t h, int64_t timeout_ms, int64_t* status) {
    nx_child* ch = nx_child_at(h);
    if (!ch) return 4;
    size_t none = 0;
    int32_t r = nx_child_serve(ch, NX_CH_EXIT, NULL, 0, &none, timeout_ms);
    if (r) return r;
    *status = ((int64_t)ch->sig << 32) | (int64_t)(uint32_t)ch->code;
    return 0;
}
/* Send the child a signal; on Windows, which has none, any but 0 ends it
   with exit code 128 + sig, and child_wait reports the signal. Nothing
   happens to a child that already ended. */
NX_INLINE int32_t nx_child_signal(int64_t h, int32_t sig) {
    nx_child* ch = nx_child_at(h);
    if (!ch || sig < 0 || sig > 64) return 4;
    if (ch->done) return 0;
#if defined(_WIN32)
    if (sig == 0 || WaitForSingleObject(ch->proc, 0) == WAIT_OBJECT_0) return 0;
    ch->sent = sig;
    if (!TerminateProcess(ch->proc, (UINT)(128 + sig))) return WaitForSingleObject(ch->proc, 0) == WAIT_OBJECT_0 ? 0 : 4;
    return 0;
#else
    return kill((pid_t)ch->pid, sig) == 0 || errno == ESRCH ? 0 : 4;
#endif
}
/* Let the child go: its pipes close and what was not read is dropped. A
   program still running goes on (and is not waited for). */
NX_INLINE void nx_child_close(int64_t h) {
    nx_child* ch = nx_child_at(h);
    if (!ch) return;
    int expected = 2;
    if (!__atomic_compare_exchange_n(&ch->state, &expected, 1, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) return;
    nx_cpipe* pipes[3] = { &ch->in, &ch->out, &ch->err };
    for (int i = 0; i < 3; i++) {
        nx_cpipe* pp = pipes[i];
        nx_cpipe_close(pp);
        free(pp->buf.p);
        pp->buf.p = NULL;
#if defined(_WIN32)
        if (pp->ov.hEvent) CloseHandle(pp->ov.hEvent);
        free(pp->stage);
#endif
    }
    nx_child_reap(ch, false); /* one that ended leaves nothing behind */
#if defined(_WIN32)
    CloseHandle(ch->proc);
#endif
    __atomic_store_n(&ch->state, 0, __ATOMIC_RELEASE);
}

/* process.trap_signals and next_signal: SIGINT, SIGTERM and SIGHUP (on
   Windows Ctrl-C, Ctrl-Break, the console closing, logoff and shutdown)
   no longer end the program; each is queued, and next_signal takes them
   in order. */
NX_STATE int nx_sig_trapped;
#if defined(_WIN32)
NX_STATE HANDLE nx_sig_sem;
NX_STATE volatile LONG nx_sig_head, nx_sig_tail;
NX_STATE volatile LONG nx_sig_ring[64];
static BOOL WINAPI nx_sig_console(DWORD kind) {
    LONG sig = kind == CTRL_C_EVENT ? 2 : kind == CTRL_BREAK_EVENT ? 21 : kind == CTRL_CLOSE_EVENT ? 1 : 15;
    LONG t = InterlockedIncrement(&nx_sig_tail) - 1;
    nx_sig_ring[t & 63] = sig;
    ReleaseSemaphore(nx_sig_sem, 1, NULL);
    /* the console closing, logoff and shutdown end the program once this
       returns: it does not, so the program has until the system's limit
       (a few seconds) to finish */
    if (kind == CTRL_CLOSE_EVENT || kind == CTRL_LOGOFF_EVENT || kind == CTRL_SHUTDOWN_EVENT) Sleep(INFINITE);
    return TRUE;
}
NX_INLINE void nx_trap_signals(void) {
    int expected = 0;
    if (!__atomic_compare_exchange_n(&nx_sig_trapped, &expected, 1, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) return;
    nx_sig_sem = CreateSemaphoreA(NULL, 0, 64, NULL);
    SetConsoleCtrlHandler(nx_sig_console, TRUE);
}
NX_INLINE int32_t nx_next_signal(int64_t timeout_ms) {
    if (!nx_sig_sem) return 0;
    DWORD r = WaitForSingleObject(nx_sig_sem, timeout_ms < 0 ? INFINITE : (DWORD)(timeout_ms > 0x7fffffff ? 0x7fffffff : timeout_ms));
    if (r != WAIT_OBJECT_0) return 0;
    LONG hd = InterlockedIncrement(&nx_sig_head) - 1;
    return (int32_t)nx_sig_ring[hd & 63];
}
#else
/* the handler writes the signal's number to a pipe (all a handler may
   safely do), and next_signal reads it back */
NX_STATE int nx_sig_rd NX_STATE_INIT(-1);
NX_STATE int nx_sig_wr NX_STATE_INIT(-1);
static void nx_sig_caught(int sig) {
    int saved = errno;
    unsigned char b = (unsigned char)sig;
    ssize_t w = write(nx_sig_wr, &b, 1);
    (void)w;
    errno = saved;
}
NX_INLINE void nx_trap_signals(void) {
    int expected = 0;
    if (!__atomic_compare_exchange_n(&nx_sig_trapped, &expected, 1, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) return;
    int p[2];
    if (nx_pipe_cloexec(p) != 0) return;
    fcntl(p[0], F_SETFL, fcntl(p[0], F_GETFL) | O_NONBLOCK);
    fcntl(p[1], F_SETFL, fcntl(p[1], F_GETFL) | O_NONBLOCK);
    nx_sig_wr = p[1];
    __atomic_store_n(&nx_sig_rd, p[0], __ATOMIC_RELEASE);
    struct sigaction sa;
    memset(&sa, 0, sizeof sa);
    sa.sa_handler = nx_sig_caught;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_RESTART;
    sigaction(SIGINT, &sa, NULL);
    sigaction(SIGTERM, &sa, NULL);
    sigaction(SIGHUP, &sa, NULL);
}
NX_INLINE int32_t nx_next_signal(int64_t timeout_ms) {
    int fd = __atomic_load_n(&nx_sig_rd, __ATOMIC_ACQUIRE);
    if (fd < 0) return 0;
    int64_t start = nx_mono_ms();
    for (;;) {
        unsigned char b = 0;
        if (read(fd, &b, 1) == 1) return (int32_t)b;
        int64_t left = nx_left_ms(start, timeout_ms);
        if (left == 0) return 0;
        struct pollfd p;
        p.fd = fd; p.events = POLLIN; p.revents = 0;
        if (poll(&p, 1, left < 0 ? -1 : left > 1000000000 ? 1000000000 : (int)left) < 0 && errno != EINTR) return 0;
    }
}
#endif
#endif

NX_INLINE bool nx_read_file(nx_ctx* c, nx_sl_u8 path, nx_string* out) {
    char p[4096];
    if (path.len >= sizeof p) return false;
    nx_bytes_copy(p, path.ptr, path.len); p[path.len] = 0;
    FILE* f = fopen(p, "rb");
    if (!f) return false;
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    uint8_t buf[65536];
    size_t n;
    while ((n = fread(buf, 1, sizeof buf, f)) > 0) nx_str_append(c, &s, buf, n);
    fclose(f);
    *out = s;
    return true;
}
NX_INLINE bool nx_write_file(nx_sl_u8 path, nx_sl_u8 data) {
    char p[4096];
    if (path.len >= sizeof p) return false;
    nx_bytes_copy(p, path.ptr, path.len); p[path.len] = 0;
    FILE* f = fopen(p, "wb");
    if (!f) return false;
    size_t w = data.len ? fwrite(data.ptr, 1, data.len, f) : 0;
    fclose(f);
    return w == data.len;
}
NX_INLINE bool nx_append_file(nx_sl_u8 path, nx_sl_u8 data) {
    char p[4096];
    if (path.len >= sizeof p) return false;
    nx_bytes_copy(p, path.ptr, path.len); p[path.len] = 0;
    FILE* f = fopen(p, "ab");
    if (!f) return false;
    size_t w = data.len ? fwrite(data.ptr, 1, data.len, f) : 0;
    fclose(f);
    return w == data.len;
}

/* ------------------------------------------------------------- file system */
/* Results: 0 ok, 1 not found, 2 any other failure. */
NX_INLINE int32_t nx_fs_errcode(void) { return errno == ENOENT ? 1 : 2; }
/* 0 = nothing there, 1 = file (or anything not a directory), 2 = directory */
NX_INLINE int32_t nx_fs_kind(nx_sl_u8 path) {
    char p[4096];
    if (!nx_cpath(path, p, sizeof p)) return 0;
#if defined(_WIN32)
    DWORD a = GetFileAttributesA(p);
    if (a == INVALID_FILE_ATTRIBUTES) return 0;
    return (a & FILE_ATTRIBUTE_DIRECTORY) ? 2 : 1;
#else
    struct stat st;
    if (stat(p, &st) != 0) return 0;
    return S_ISDIR(st.st_mode) ? 2 : 1;
#endif
}
NX_INLINE int32_t nx_fs_stat(nx_sl_u8 path, int64_t* size, int64_t* mtime_ms) {
    char p[4096];
    if (!nx_cpath(path, p, sizeof p)) return 2;
#if defined(_WIN32)
    struct _stat64 st;
    if (_stat64(p, &st) != 0) return nx_fs_errcode();
#else
    struct stat st;
    if (stat(p, &st) != 0) return nx_fs_errcode();
#endif
    *size = (int64_t)st.st_size;
    *mtime_ms = (int64_t)st.st_mtime * 1000;
    return 0;
}
NX_INLINE int32_t nx_fs_mkdir(nx_sl_u8 path) {
    char p[4096];
    if (!nx_cpath(path, p, sizeof p)) return 2;
#if defined(_WIN32)
    if (_mkdir(p) == 0 || errno == EEXIST) return 0;
#else
    if (mkdir(p, 0777) == 0 || errno == EEXIST) return 0;
#endif
    return nx_fs_errcode();
}
NX_INLINE int32_t nx_fs_remove_file(nx_sl_u8 path) {
    char p[4096];
    if (!nx_cpath(path, p, sizeof p)) return 2;
    if (remove(p) == 0) return 0;
#if defined(_WIN32)
    /* a read-only file (every object in a git checkout) refuses `remove` on
       Windows; asking to delete it is asking to clear that bit first */
    if (errno == EACCES && _chmod(p, _S_IWRITE) == 0 && remove(p) == 0) return 0;
#endif
    return nx_fs_errcode();
}
NX_INLINE int32_t nx_fs_remove_dir(nx_sl_u8 path) {
    char p[4096];
    if (!nx_cpath(path, p, sizeof p)) return 2;
#if defined(_WIN32)
    return _rmdir(p) == 0 ? 0 : nx_fs_errcode();
#else
    return rmdir(p) == 0 ? 0 : nx_fs_errcode();
#endif
}
NX_INLINE int32_t nx_fs_rename(nx_sl_u8 from, nx_sl_u8 to) {
    char p[4096], q[4096];
    if (!nx_cpath(from, p, sizeof p) || !nx_cpath(to, q, sizeof q)) return 2;
#if defined(_WIN32)
    if (MoveFileExA(p, q, MOVEFILE_REPLACE_EXISTING)) return 0;
    return GetLastError() == ERROR_FILE_NOT_FOUND || GetLastError() == ERROR_PATH_NOT_FOUND ? 1 : 2;
#else
    return rename(p, q) == 0 ? 0 : nx_fs_errcode();
#endif
}
NX_INLINE void nx_fs_push_name(nx_ctx* c, nx_rawlist* l, const char* name) {
    if (strcmp(name, ".") == 0 || strcmp(name, "..") == 0) return;
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    nx_str_append(c, &s, (const uint8_t*)name, strlen(name));
    if (l->len == l->cap) nx_list_grow(c, l, sizeof(nx_string), _Alignof(nx_string), l->len + 1);
    ((nx_string*)l->ptr)[l->len++] = s;
}
/* the entries of a directory, unsorted, without `.` and `..` */
NX_INLINE int32_t nx_fs_list_dir(nx_ctx* c, nx_sl_u8 path, nx_rawlist* out) {
    char p[4096];
    if (!nx_cpath(path, p, sizeof p)) return 2;
    nx_rawlist l; l.ptr = NULL; l.len = 0; l.cap = 0; l.ar = c->arena;
#if defined(_WIN32)
    char pat[4200];
    snprintf(pat, sizeof pat, "%s\\*", p);
    WIN32_FIND_DATAA fd;
    HANDLE h = FindFirstFileA(pat, &fd);
    if (h == INVALID_HANDLE_VALUE) {
        DWORD e = GetLastError();
        return e == ERROR_FILE_NOT_FOUND || e == ERROR_PATH_NOT_FOUND ? 1 : 2;
    }
    do { nx_fs_push_name(c, &l, fd.cFileName); } while (FindNextFileA(h, &fd));
    FindClose(h);
#else
    DIR* d = opendir(p);
    if (!d) return nx_fs_errcode();
    struct dirent* e;
    while ((e = readdir(d)) != NULL) nx_fs_push_name(c, &l, e->d_name);
    closedir(d);
#endif
    *out = l;
    return 0;
}
NX_INLINE bool nx_fs_cwd(nx_ctx* c, nx_string* out) {
    char buf[4096];
    size_t n;
#if defined(_WIN32)
    DWORD r = GetCurrentDirectoryA(sizeof buf, buf);
    if (r == 0 || r >= sizeof buf) return false;
    n = (size_t)r;
#else
    if (!getcwd(buf, sizeof buf)) return false;
    n = strlen(buf);
#endif
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    nx_str_append(c, &s, (const uint8_t*)buf, n);
    *out = s;
    return true;
}
/* the path of the running executable; empty when the platform will not say */
NX_INLINE nx_string nx_exe_path(nx_ctx* c) {
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    char buf[4096];
    size_t n = 0;
#if defined(_WIN32)
    DWORD r = GetModuleFileNameA(NULL, buf, (DWORD)sizeof buf);
    if (r == 0 || r >= sizeof buf) return s;
    n = (size_t)r;
#elif defined(__APPLE__)
    uint32_t size = (uint32_t)sizeof buf;
    if (_NSGetExecutablePath(buf, &size) != 0) return s;
    n = strlen(buf);
#else
    ssize_t r = readlink("/proc/self/exe", buf, sizeof buf - 1);
    if (r <= 0) return s;
    n = (size_t)r;
#endif
    nx_str_append(c, &s, (const uint8_t*)buf, n);
    return s;
}
NX_INLINE nx_string nx_fs_temp_dir(nx_ctx* c) {
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
#if defined(_WIN32)
    char buf[MAX_PATH + 2];
    DWORD n = GetTempPathA(sizeof buf, buf);
    if (n > 0 && n < sizeof buf) {
        if (buf[n - 1] == '\\' || buf[n - 1] == '/') n--;
        nx_str_append(c, &s, (const uint8_t*)buf, n);
    }
#else
    const char* t = getenv("TMPDIR");
    if (!t || !*t) t = "/tmp";
    size_t n = strlen(t);
    if (n > 1 && t[n - 1] == '/') n--;
    nx_str_append(c, &s, (const uint8_t*)t, n);
#endif
    return s;
}

/* ------------------------------------------------------------ file handles */
/* 1 = stdin, 2 = stdout, 3 = stderr; opened files get 4 and up. */
#define NX_MAX_FILES 64
NX_STATE FILE* nx_files[NX_MAX_FILES];
NX_INLINE FILE* nx_fh(int64_t h) {
    if (h == 1) return stdin;
    if (h == 2) return stdout;
    if (h == 3) return stderr;
    if (h < 4 || h >= NX_MAX_FILES + 4) return NULL;
    return nx_files[h - 4];
}
/* a handle, or -1 when the path does not exist, -2 on any other failure */
NX_INLINE int64_t nx_file_open(nx_sl_u8 path, nx_sl_u8 mode) {
    char p[4096], m[8];
    if (!nx_cpath(path, p, sizeof p) || mode.len == 0 || mode.len > 3) return -2;
    memcpy(m, mode.ptr, mode.len); m[mode.len] = 'b'; m[mode.len + 1] = 0;
    FILE* f = fopen(p, m);
    if (!f) return errno == ENOENT ? -1 : -2;
    for (int i = 0; i < NX_MAX_FILES; i++) {
        if (!nx_files[i]) { nx_files[i] = f; nx_track_handle(0, i + 4, true); return i + 4; }
    }
    fclose(f);
    return -2;
}
/* stdin is read at the descriptor level, so a pipe or a terminal hands over what it
   has instead of waiting for a full buffer the way fread does; every stdin reader in
   the runtime consumes from this one buffer */
NX_STATE uint8_t nx_stdin_buf[65536];
NX_STATE size_t nx_stdin_pos, nx_stdin_len;
NX_INLINE bool nx_stdin_fill(void) {
    if (nx_stdin_pos < nx_stdin_len) return true;
#if defined(_WIN32)
    int n = _read(0, nx_stdin_buf, (unsigned)sizeof nx_stdin_buf);
#else
    ssize_t n;
    do { n = read(0, nx_stdin_buf, sizeof nx_stdin_buf); } while (n < 0 && errno == EINTR);
#endif
    if (n <= 0) return false;
    nx_stdin_pos = 0; nx_stdin_len = (size_t)n;
    return true;
}
/* up to n bytes; an empty result means end of input */
NX_INLINE bool nx_file_read(nx_ctx* c, int64_t h, size_t n, nx_string* out) {
    FILE* f = nx_fh(h);
    if (!f) return false;
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    if (n > 0 && h == 1) {
        if (nx_stdin_fill()) {
            size_t have = nx_stdin_len - nx_stdin_pos;
            if (have > n) have = n;
            nx_list_grow(c, (nx_rawlist*)&s, 1, 1, have);
            memcpy(s.ptr, nx_stdin_buf + nx_stdin_pos, have);
            s.len = have;
            nx_stdin_pos += have;
        }
    } else if (n > 0) {
        nx_list_grow(c, (nx_rawlist*)&s, 1, 1, n);
        s.len = fread(s.ptr, 1, n, f);
        if (s.len == 0 && ferror(f)) return false;
    }
    *out = s;
    return true;
}
NX_INLINE bool nx_file_write(int64_t h, nx_sl_u8 data) {
    FILE* f = nx_fh(h);
    if (!f) return false;
    return data.len == 0 || fwrite(data.ptr, 1, data.len, f) == data.len;
}
NX_INLINE bool nx_file_flush(int64_t h) {
    FILE* f = nx_fh(h);
    return f && fflush(f) == 0;
}
NX_INLINE bool nx_file_close(int64_t h) {
    if (h >= 1 && h <= 3) return true;
    FILE* f = nx_fh(h);
    if (!f) return false;
    nx_files[h - 4] = NULL;
    nx_track_handle(0, h, false);
    return fclose(f) == 0;
}
/* set a variable in this process's environment (and its children's); an
   empty value removes it */
NX_INLINE void nx_set_env(nx_sl_u8 name, nx_sl_u8 value) {
    char n[256], v[4096];
    if (name.len == 0 || name.len >= sizeof n || value.len >= sizeof v) return;
    memcpy(n, name.ptr, name.len); n[name.len] = 0;
    nx_bytes_copy(v, value.ptr, value.len); v[value.len] = 0;
#if defined(_WIN32)
    _putenv_s(n, v);
#else
    if (value.len == 0) unsetenv(n); else setenv(n, v, 1);
#endif
}
/* is the handle (1 stdin, 2 stdout, 3 stderr) a terminal? */
NX_INLINE bool nx_is_terminal(int64_t h) {
    int fd = h == 1 ? 0 : h == 2 ? 1 : h == 3 ? 2 : -1;
    if (fd < 0) return false;
#if defined(_WIN32)
    return _isatty(fd) != 0;
#else
    return isatty(fd) != 0;
#endif
}
/* every environment variable as "NAME=value" */
NX_INLINE void nx_environ(nx_ctx* c, nx_rawlist* out) {
    nx_rawlist l; l.ptr = NULL; l.len = 0; l.cap = 0; l.ar = c->arena;
#if defined(_WIN32)
    char* env = GetEnvironmentStringsA();
    if (env) {
        for (char* p = env; *p; p += strlen(p) + 1) {
            if (*p == '=') continue; /* per-drive working directories */
            nx_fs_push_name(c, &l, p);
        }
        FreeEnvironmentStringsA(env);
    }
#else
    for (char** e = environ; e && *e; e++) nx_fs_push_name(c, &l, *e);
#endif
    *out = l;
}

/* ------------------------------------------------------------------ sockets */
/* Handles are the OS socket numbers. Result codes: 0 ok, 1 not found (name
   lookup), 2 connection refused, 3 timed out, 4 any other failure. */
#if defined(_WIN32)
typedef SOCKET nx_sock;
#define NX_BAD_SOCK INVALID_SOCKET
#define nx_closesock closesocket
NX_INLINE void nx_net_init(void) {
    static int done = 0;
    if (!done) { WSADATA w; WSAStartup(MAKEWORD(2, 2), &w); done = 1; }
}
NX_INLINE int32_t nx_net_code(void) {
    int e = WSAGetLastError();
    if (e == WSAECONNREFUSED) return 2;
    if (e == WSAETIMEDOUT || e == WSAEWOULDBLOCK) return 3;
    return 4;
}
NX_INLINE void nx_net_blocking(nx_sock s, bool on) { u_long mode = on ? 0 : 1; ioctlsocket(s, FIONBIO, &mode); }
NX_INLINE bool nx_net_in_progress(void) { return WSAGetLastError() == WSAEWOULDBLOCK; }
#elif defined(NX_WASM)
typedef int nx_sock;
#define NX_BAD_SOCK (-1)
#define nx_closesock(s) ((void)(s), 0)
NX_INLINE void nx_net_init(void) {}
#else
typedef int nx_sock;
#define NX_BAD_SOCK (-1)
#define nx_closesock close
NX_INLINE void nx_net_init(void) {}
NX_INLINE int32_t nx_net_code(void) {
    if (errno == ECONNREFUSED) return 2;
    if (errno == ETIMEDOUT || errno == EAGAIN || errno == EWOULDBLOCK) return 3;
    return 4;
}
NX_INLINE void nx_net_blocking(nx_sock s, bool on) {
    int fl = fcntl(s, F_GETFL, 0);
    if (fl >= 0) fcntl(s, F_SETFL, on ? (fl & ~O_NONBLOCK) : (fl | O_NONBLOCK));
}
NX_INLINE bool nx_net_in_progress(void) { return errno == EINPROGRESS || errno == EINTR; }
#endif
/* A send to a connection the peer has reset fails with EPIPE; without
   these it raises SIGPIPE first, which ends the program. Linux and the BSDs
   take MSG_NOSIGNAL on each send, macOS SO_NOSIGPIPE on the socket. */
#if defined(MSG_NOSIGNAL)
#define NX_SEND_FLAGS MSG_NOSIGNAL
#else
#define NX_SEND_FLAGS 0
#endif
#if defined(NX_WASM)
/* no sockets in the playground: every call fails as "any other failure" */
NX_INLINE int32_t nx_tcp_connect(nx_sl_u8 host, uint16_t port, int64_t timeout_ms, int64_t* out) { (void)host; (void)port; (void)timeout_ms; (void)out; return 4; }
NX_INLINE int32_t nx_tcp_listen(nx_sl_u8 host, uint16_t port, int64_t* out) { (void)host; (void)port; (void)out; return 4; }
NX_INLINE int32_t nx_tcp_accept(int64_t l, int64_t timeout_ms, int64_t* out) { (void)l; (void)timeout_ms; (void)out; return 4; }
NX_INLINE int32_t nx_net_send(int64_t h, nx_sl_u8 data) { (void)h; (void)data; return 4; }
NX_INLINE int32_t nx_net_recv(nx_ctx* c, int64_t h, size_t n, int64_t timeout_ms, nx_string* out) { (void)c; (void)h; (void)n; (void)timeout_ms; (void)out; return 4; }
NX_INLINE int32_t nx_net_close(int64_t h) { (void)h; return 4; }
NX_INLINE int32_t nx_net_name(nx_ctx* c, int64_t h, bool local, nx_string* out) { (void)c; (void)h; (void)local; (void)out; return 4; }
NX_INLINE int32_t nx_net_resolve(nx_ctx* c, nx_sl_u8 host, nx_rawlist* out) { (void)c; (void)host; (void)out; return 4; }
NX_INLINE int32_t nx_udp_bind(nx_sl_u8 host, uint16_t port, int64_t* out) { (void)host; (void)port; (void)out; return 4; }
NX_INLINE int32_t nx_udp_send_to(int64_t h, nx_sl_u8 host, uint16_t port, nx_sl_u8 data) { (void)h; (void)host; (void)port; (void)data; return 4; }
NX_INLINE int32_t nx_udp_recv_from(nx_ctx* c, int64_t h, size_t n, int64_t timeout_ms, nx_string* out) { (void)c; (void)h; (void)n; (void)timeout_ms; (void)out; return 4; }
NX_INLINE nx_string nx_net_last_peer(nx_ctx* c) { nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena; return s; }
#else
NX_STATE char nx_net_peer_buf[128];

NX_INLINE struct addrinfo* nx_net_lookup(nx_sl_u8 host, uint16_t port, int socktype, bool passive) {
    char h[256], p[8];
    if (host.len >= sizeof h) return NULL;
    nx_bytes_copy(h, host.ptr, host.len); h[host.len] = 0;
    snprintf(p, sizeof p, "%u", (unsigned)port);
    struct addrinfo hints;
    memset(&hints, 0, sizeof hints);
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = socktype;
    if (passive) hints.ai_flags = AI_PASSIVE;
    struct addrinfo* res = NULL;
    nx_net_init();
    if (getaddrinfo(host.len ? h : NULL, p, &hints, &res) != 0) return NULL;
    return res;
}
NX_INLINE void nx_net_set_timeout(nx_sock s, int64_t ms) {
#if defined(_WIN32)
    DWORD t = (DWORD)(ms < 0 ? 0 : ms);
    setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, (const char*)&t, sizeof t);
#else
    struct timeval tv;
    tv.tv_sec = (time_t)(ms < 0 ? 0 : ms / 1000);
    tv.tv_usec = (suseconds_t)(ms < 0 ? 0 : (ms % 1000) * 1000);
    setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, (const char*)&tv, sizeof tv);
#endif
}
/* wait until the socket is readable (or writable); false on timeout. For a
   pending connect the exception set is watched too: Winsock reports a
   refused connection there rather than as writable. */
NX_INLINE bool nx_net_wait(nx_sock s, bool write, int64_t ms) {
    fd_set fds, exc;
    FD_ZERO(&fds);
    FD_SET(s, &fds);
    FD_ZERO(&exc);
    FD_SET(s, &exc);
    struct timeval tv;
    tv.tv_sec = (long)(ms / 1000);
    tv.tv_usec = (long)((ms % 1000) * 1000);
    int r = select((int)(s + 1), write ? NULL : &fds, write ? &fds : NULL, write ? &exc : NULL, ms > 0 ? &tv : NULL);
    return r > 0;
}
/* what every connected TCP socket gets: no Nagle delay, and no SIGPIPE */
NX_INLINE void nx_tcp_ready(nx_sock s) {
    int one = 1;
    setsockopt(s, IPPROTO_TCP, TCP_NODELAY, (const char*)&one, sizeof one);
#if defined(SO_NOSIGPIPE)
    setsockopt(s, SOL_SOCKET, SO_NOSIGPIPE, (const char*)&one, sizeof one);
#endif
}
NX_INLINE int32_t nx_tcp_connect(nx_sl_u8 host, uint16_t port, int64_t timeout_ms, int64_t* out) {
    struct addrinfo* res = nx_net_lookup(host, port, SOCK_STREAM, false);
    if (!res) return 1;
    int32_t code = 4;
    for (struct addrinfo* ai = res; ai; ai = ai->ai_next) {
        nx_sock s = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (s == NX_BAD_SOCK) continue;
        bool ok;
        if (timeout_ms > 0) {
            nx_net_blocking(s, false);
            int r = connect(s, ai->ai_addr, (int)ai->ai_addrlen);
            ok = r == 0;
            if (!ok && !nx_net_in_progress()) {
                /* it failed at once (no route, say): the socket then
                   selects as writable with no error pending, so waiting
                   would take it for connected */
                code = nx_net_code();
            } else if (!ok) {
                if (nx_net_wait(s, true, timeout_ms)) {
                    int err = 0; socklen_t len = sizeof err;
                    getsockopt(s, SOL_SOCKET, SO_ERROR, (char*)&err, &len);
#if defined(_WIN32)
                    if (err == 0) { fd_set ex; FD_ZERO(&ex); FD_SET(s, &ex); struct timeval z = {0, 0}; if (select((int)(s + 1), NULL, NULL, &ex, &z) > 0) err = WSAECONNREFUSED; }
#endif
                    ok = err == 0;
                    if (!ok) {
#if defined(_WIN32)
                        WSASetLastError(err);
#else
                        errno = err;
#endif
                        code = nx_net_code();
                    }
                } else {
                    code = 3;
                }
            }
            nx_net_blocking(s, true);
        } else {
            ok = connect(s, ai->ai_addr, (int)ai->ai_addrlen) == 0;
            if (!ok) code = nx_net_code();
        }
        if (ok) {
            nx_tcp_ready(s);
            *out = (int64_t)s; nx_track_handle(1, *out, true);
            freeaddrinfo(res);
            return 0;
        }
        nx_closesock(s);
    }
    freeaddrinfo(res);
    return code;
}
NX_INLINE int32_t nx_tcp_listen(nx_sl_u8 host, uint16_t port, int64_t* out) {
    struct addrinfo* res = nx_net_lookup(host, port, SOCK_STREAM, true);
    if (!res) return 1;
    int32_t code = 4;
    for (struct addrinfo* ai = res; ai; ai = ai->ai_next) {
        nx_sock s = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (s == NX_BAD_SOCK) continue;
        int one = 1;
        setsockopt(s, SOL_SOCKET, SO_REUSEADDR, (const char*)&one, sizeof one);
        if (bind(s, ai->ai_addr, (int)ai->ai_addrlen) == 0 && listen(s, 64) == 0) {
            *out = (int64_t)s; nx_track_handle(1, *out, true);
            freeaddrinfo(res);
            return 0;
        }
        code = nx_net_code();
        nx_closesock(s);
    }
    freeaddrinfo(res);
    return code;
}
NX_INLINE int32_t nx_tcp_accept(int64_t l, int64_t timeout_ms, int64_t* out) {
    nx_sock ls = (nx_sock)l;
    if (timeout_ms > 0 && !nx_net_wait(ls, false, timeout_ms)) return 3;
    nx_sock s = accept(ls, NULL, NULL);
    if (s == NX_BAD_SOCK) return nx_net_code();
    nx_tcp_ready(s);
    *out = (int64_t)s; nx_track_handle(1, *out, true);
    return 0;
}
NX_INLINE int32_t nx_net_send(int64_t h, nx_sl_u8 data) {
    nx_sock s = (nx_sock)h;
    size_t sent = 0;
    while (sent < data.len) {
        int n = (int)send(s, (const char*)data.ptr + sent, (int)(data.len - sent), NX_SEND_FLAGS);
        if (n <= 0) return nx_net_code();
        sent += (size_t)n;
    }
    return 0;
}
NX_INLINE int32_t nx_net_recv(nx_ctx* c, int64_t h, size_t n, int64_t timeout_ms, nx_string* out) {
    nx_sock s = (nx_sock)h;
    if (timeout_ms > 0 && !nx_net_wait(s, false, timeout_ms)) return 3;
    nx_string str; str.ptr = NULL; str.len = 0; str.cap = 0; str.ar = c->arena;
    if (n == 0) { *out = str; return 0; }
    nx_list_grow(c, (nx_rawlist*)&str, 1, 1, n);
    int got = (int)recv(s, (char*)str.ptr, (int)n, 0);
    if (got < 0) return nx_net_code();
    str.len = (size_t)got;
    *out = str;
    return 0;
}
NX_INLINE int32_t nx_net_close(int64_t h) {
    nx_track_handle(1, h, false);
    return nx_closesock((nx_sock)h) == 0 ? 0 : 4;
}
NX_INLINE void nx_net_format_addr(struct sockaddr* sa, socklen_t len, char* buf, size_t cap) {
    char host[96], serv[16];
    if (getnameinfo(sa, len, host, sizeof host, serv, sizeof serv, NI_NUMERICHOST | NI_NUMERICSERV) != 0) { buf[0] = 0; return; }
    if (sa->sa_family == AF_INET6) snprintf(buf, cap, "[%s]:%s", host, serv);
    else snprintf(buf, cap, "%s:%s", host, serv);
}
NX_INLINE int32_t nx_net_name(nx_ctx* c, int64_t h, bool local, nx_string* out) {
    struct sockaddr_storage ss;
    socklen_t len = sizeof ss;
    int r = local ? getsockname((nx_sock)h, (struct sockaddr*)&ss, &len) : getpeername((nx_sock)h, (struct sockaddr*)&ss, &len);
    if (r != 0) return 4;
    char buf[128];
    nx_net_format_addr((struct sockaddr*)&ss, len, buf, sizeof buf);
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    nx_str_append(c, &s, (const uint8_t*)buf, strlen(buf));
    *out = s;
    return 0;
}
NX_INLINE int32_t nx_net_resolve(nx_ctx* c, nx_sl_u8 host, nx_rawlist* out) {
    struct addrinfo* res = nx_net_lookup(host, 0, SOCK_STREAM, false);
    if (!res) return 1;
    nx_rawlist l; l.ptr = NULL; l.len = 0; l.cap = 0; l.ar = c->arena;
    for (struct addrinfo* ai = res; ai; ai = ai->ai_next) {
        char hostbuf[96];
        if (getnameinfo(ai->ai_addr, (socklen_t)ai->ai_addrlen, hostbuf, sizeof hostbuf, NULL, 0, NI_NUMERICHOST) == 0) {
            bool dup = false;
            for (size_t i = 0; i < l.len; i++) {
                nx_string* e = &((nx_string*)l.ptr)[i];
                if (e->len == strlen(hostbuf) && memcmp(e->ptr, hostbuf, e->len) == 0) dup = true;
            }
            if (!dup) nx_fs_push_name(c, &l, hostbuf);
        }
    }
    freeaddrinfo(res);
    *out = l;
    return 0;
}
NX_INLINE int32_t nx_udp_bind(nx_sl_u8 host, uint16_t port, int64_t* out) {
    struct addrinfo* res = nx_net_lookup(host, port, SOCK_DGRAM, true);
    if (!res) return 1;
    int32_t code = 4;
    for (struct addrinfo* ai = res; ai; ai = ai->ai_next) {
        nx_sock s = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (s == NX_BAD_SOCK) continue;
        if (bind(s, ai->ai_addr, (int)ai->ai_addrlen) == 0) {
            *out = (int64_t)s; nx_track_handle(1, *out, true);
            freeaddrinfo(res);
            return 0;
        }
        code = nx_net_code();
        nx_closesock(s);
    }
    freeaddrinfo(res);
    return code;
}
NX_INLINE int32_t nx_udp_send_to(int64_t h, nx_sl_u8 host, uint16_t port, nx_sl_u8 data) {
    struct addrinfo* res = nx_net_lookup(host, port, SOCK_DGRAM, false);
    if (!res) return 1;
    int n = (int)sendto((nx_sock)h, (const char*)data.ptr, (int)data.len, 0, res->ai_addr, (int)res->ai_addrlen);
    freeaddrinfo(res);
    return n < 0 ? nx_net_code() : 0;
}
NX_INLINE int32_t nx_udp_recv_from(nx_ctx* c, int64_t h, size_t n, int64_t timeout_ms, nx_string* out) {
    nx_sock s = (nx_sock)h;
    if (timeout_ms > 0 && !nx_net_wait(s, false, timeout_ms)) return 3;
    nx_string str; str.ptr = NULL; str.len = 0; str.cap = 0; str.ar = c->arena;
    if (n == 0) n = 1;
    nx_list_grow(c, (nx_rawlist*)&str, 1, 1, n);
    struct sockaddr_storage ss;
    socklen_t len = sizeof ss;
    int got = (int)recvfrom(s, (char*)str.ptr, (int)n, 0, (struct sockaddr*)&ss, &len);
    if (got < 0) return nx_net_code();
    str.len = (size_t)got;
    nx_net_format_addr((struct sockaddr*)&ss, len, nx_net_peer_buf, sizeof nx_net_peer_buf);
    *out = str;
    return 0;
}
NX_INLINE nx_string nx_net_last_peer(nx_ctx* c) {
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    nx_str_append(c, &s, (const uint8_t*)nx_net_peer_buf, strlen(nx_net_peer_buf));
    return s;
}
#endif

/* ------------------------------------------------------------------ TLS */
/* net.tls_*: TLS over a TCP connection by the platform's own library,
   loaded when first used so no program links it: SChannel on Windows,
   Security.framework on macOS, OpenSSL (libssl 3 or 1.1) elsewhere. The
   server's certificate is checked against the system's roots and the host
   name. Handles are 1 + a slot; the result codes are the socket calls'
   (1 no such host, 2 refused, 3 timed out, 4 any other failure), and
   net.tls_problem says in words what went wrong on this thread. */
#if defined(_WIN32)
#ifndef SECURITY_WIN32
#define SECURITY_WIN32
#endif
#include <security.h>
#include <schannel.h>
#elif !defined(NX_WASM)
#include <dlfcn.h>
#endif
#define NX_MAX_TLS 256
typedef struct {
    int state;               /* 0 free, 1 being set up or let go, 2 in use */
    nx_sock sock;
    char host[256];
    bool ended;              /* the connection is over */
    bool clean;              /* ... and ended with close_notify */
    int64_t timeout_ms;      /* what a send may wait (0: no limit) */
    uint8_t* plain;          /* decrypted and not yet taken */
    size_t plain_len, plain_pos, plain_cap;
#if defined(_WIN32)
    CredHandle cred;
    CtxtHandle ctx;
    bool have_cred, have_ctx;
    SecPkgContext_StreamSizes sizes;
    uint8_t* raw;            /* received and not yet decrypted */
    size_t raw_len, raw_cap;
#else
    void* ssl;               /* OpenSSL's SSL*, or an SSLContextRef */
    int64_t deadline;        /* for Security.framework's callbacks (0: none) */
#endif
} nx_tls;
NX_STATE nx_tls nx_tlss[NX_MAX_TLS];
NX_STATE NX_THREAD_LOCAL char nx_tls_why[256];

NX_INLINE void nx_tls_say(const char* what) { snprintf(nx_tls_why, sizeof nx_tls_why, "%s", what); }
NX_INLINE nx_string nx_tls_problem(nx_ctx* c) {
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    nx_str_append(c, &s, (const uint8_t*)nx_tls_why, strlen(nx_tls_why));
    return s;
}
NX_INLINE nx_tls* nx_tls_at(int64_t h) {
    if (h < 1 || h > NX_MAX_TLS) return NULL;
    nx_tls* t = &nx_tlss[h - 1];
    return __atomic_load_n(&t->state, __ATOMIC_ACQUIRE) == 2 ? t : NULL;
}
/* a deadline `ms` from now (0: none), and what is left of one: 0 for no
   deadline, -1 when it has passed */
NX_INLINE int64_t nx_deadline(int64_t ms) { return ms > 0 ? nx_mono_ms() + ms : 0; }
NX_INLINE int64_t nx_until(int64_t deadline) {
    if (deadline == 0) return 0;
    int64_t left = deadline - nx_mono_ms();
    return left > 0 ? left : -1;
}
#if defined(NX_WASM)
NX_INLINE bool nx_tls_available(void) { return false; }
NX_INLINE int32_t nx_tls_connect(nx_sl_u8 host, uint16_t port, int64_t timeout_ms, int64_t* out) {
    (void)host; (void)port; (void)timeout_ms; (void)out;
    nx_tls_say("no TLS in WebAssembly");
    return 4;
}
NX_INLINE int32_t nx_tls_send(int64_t h, nx_sl_u8 data) { (void)h; (void)data; return 4; }
NX_INLINE int32_t nx_tls_recv(nx_ctx* c, int64_t h, size_t n, int64_t timeout_ms, nx_string* out) {
    (void)c; (void)h; (void)n; (void)timeout_ms; (void)out;
    return 4;
}
NX_INLINE bool nx_tls_truncated(int64_t h) { (void)h; return false; }
NX_INLINE void nx_tls_close(int64_t h) { (void)h; }
#else
NX_INLINE void nx_tls_keep(nx_tls* t, const uint8_t* p, size_t n) {
    if (t->plain_pos == t->plain_len) { t->plain_pos = 0; t->plain_len = 0; }
    if (t->plain_len + n > t->plain_cap) {
        size_t cap = t->plain_cap ? t->plain_cap : 16384;
        while (cap < t->plain_len + n) cap *= 2;
        uint8_t* q = (uint8_t*)realloc(t->plain, cap);
        if (!q) nx_panic("out of memory keeping TLS data", "net.tls_recv");
        t->plain = q;
        t->plain_cap = cap;
    }
    memcpy(t->plain + t->plain_len, p, n);
    t->plain_len += n;
}
/* a send that would have blocked, on a socket that does not */
NX_INLINE bool nx_tls_again(void) {
#if defined(_WIN32)
    return WSAGetLastError() == WSAEWOULDBLOCK;
#else
    return errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR;
#endif
}
/* all of `n` bytes to the socket, waiting for room until the deadline */
NX_INLINE int32_t nx_tls_put(nx_sock s, const uint8_t* p, size_t n, int64_t deadline) {
    size_t sent = 0;
    while (sent < n) {
        int64_t left = nx_until(deadline);
        if (left < 0) return 3;
        if (!nx_net_wait(s, true, left)) return 3;
        int k = (int)send(s, (const char*)p + sent, (int)(n - sent), NX_SEND_FLAGS);
        if (k < 0 && !nx_tls_again()) return nx_net_code();
        if (k > 0) sent += (size_t)k;
    }
    return 0;
}
NX_INLINE void nx_tls_reset(nx_tls* t) {
    t->sock = NX_BAD_SOCK;
    t->host[0] = 0;
    t->ended = false;
    t->clean = false;
    t->timeout_ms = 0;
    t->plain = NULL;
    t->plain_len = 0; t->plain_pos = 0; t->plain_cap = 0;
#if defined(_WIN32)
    t->have_cred = false;
    t->have_ctx = false;
    memset(&t->sizes, 0, sizeof t->sizes);
    t->raw = NULL;
    t->raw_len = 0; t->raw_cap = 0;
#else
    t->ssl = NULL;
    t->deadline = 0;
#endif
}
NX_INLINE int nx_tls_claim(void) {
    for (int i = 0; i < NX_MAX_TLS; i++) {
        int expected = 0;
        if (__atomic_compare_exchange_n(&nx_tlss[i].state, &expected, 1, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) {
            nx_tls_reset(&nx_tlss[i]);
            return i;
        }
    }
    return -1;
}

#if defined(_WIN32)
/* ----- SChannel, through the function table secur32.dll hands out */
NX_STATE PSecurityFunctionTableA nx_sspi;
NX_STATE int nx_sspi_state;
NX_INLINE bool nx_tls_available(void) {
    int st = __atomic_load_n(&nx_sspi_state, __ATOMIC_ACQUIRE);
    if (st == 0) {
        HMODULE m = LoadLibraryA("secur32.dll");
        INIT_SECURITY_INTERFACE_A init = m ? (INIT_SECURITY_INTERFACE_A)GetProcAddress(m, "InitSecurityInterfaceA") : NULL;
        nx_sspi = init ? init() : NULL;
        st = nx_sspi ? 1 : 2;
        __atomic_store_n(&nx_sspi_state, st, __ATOMIC_RELEASE);
    }
    return st == 1;
}
#define NX_ISC_FLAGS (ISC_REQ_SEQUENCE_DETECT | ISC_REQ_REPLAY_DETECT | ISC_REQ_CONFIDENTIALITY | ISC_REQ_EXTENDED_ERROR | ISC_REQ_ALLOCATE_MEMORY | ISC_REQ_STREAM)
NX_INLINE void nx_tls_say_status(SECURITY_STATUS st) {
    const char* what = "the TLS handshake failed";
    if (st == SEC_E_UNTRUSTED_ROOT || st == CERT_E_UNTRUSTEDROOT || st == CERT_E_CHAINING) what = "the server's certificate is not signed by a root this system trusts";
    else if (st == SEC_E_WRONG_PRINCIPAL || st == CERT_E_CN_NO_MATCH) what = "the server's certificate is for another name";
    else if (st == SEC_E_CERT_EXPIRED || st == CERT_E_EXPIRED) what = "the server's certificate has expired";
    else if (st == CRYPT_E_REVOKED) what = "the server's certificate is revoked";
    else if (st == SEC_E_ILLEGAL_MESSAGE) what = "the server sent an alert, or something that is not TLS";
    else if (st == SEC_E_ALGORITHM_MISMATCH) what = "the server and this system have no cipher in common";
    else if (st == SEC_E_DECRYPT_FAILURE || st == SEC_E_MESSAGE_ALTERED) what = "a TLS record did not decrypt";
    snprintf(nx_tls_why, sizeof nx_tls_why, "%s (SChannel 0x%08lX)", what, (unsigned long)st);
}
/* room for `more` bytes after what `raw` holds */
NX_INLINE void nx_tls_room(nx_tls* t, size_t more) {
    if (t->raw_len + more <= t->raw_cap) return;
    size_t cap = t->raw_cap ? t->raw_cap : 32768;
    while (cap < t->raw_len + more) cap *= 2;
    uint8_t* q = (uint8_t*)realloc(t->raw, cap);
    if (!q) nx_panic("out of memory in TLS", "net.tls");
    t->raw = q;
    t->raw_cap = cap;
}
/* more ciphertext: 0, 3 on timeout, 5 when the peer closed, else a socket code */
NX_INLINE int32_t nx_tls_pull(nx_tls* t, int64_t deadline) {
    int64_t left = nx_until(deadline);
    if (left < 0) return 3;
    if (!nx_net_wait(t->sock, false, left)) return 3;
    nx_tls_room(t, 16384);
    int k = (int)recv(t->sock, (char*)t->raw + t->raw_len, 16384, 0);
    if (k == 0) return 5;
    if (k < 0) return nx_net_code();
    t->raw_len += (size_t)k;
    return 0;
}
NX_INLINE int32_t nx_tls_token(nx_tls* t, SecBuffer* b, int64_t deadline) {
    int32_t r = 0;
    if (b->pvBuffer && b->cbBuffer > 0) r = nx_tls_put(t->sock, (const uint8_t*)b->pvBuffer, b->cbBuffer, deadline);
    if (b->pvBuffer) nx_sspi->FreeContextBuffer(b->pvBuffer);
    b->pvBuffer = NULL;
    b->cbBuffer = 0;
    return r;
}
NX_INLINE int32_t nx_tls_lost(int32_t r) {
    if (r == 3) nx_tls_say("the TLS handshake took longer than allowed");
    else nx_tls_say("the server closed the connection during the TLS handshake");
    return r == 5 ? 4 : r;
}
/* Step the handshake over what `raw` holds until it completes: from the
   start, or for a message after it (DecryptMessage's SEC_I_RENEGOTIATE). */
NX_INLINE int32_t nx_tls_steps(nx_tls* t, int64_t deadline) {
    SECURITY_STATUS st;
    TimeStamp ts;
    ULONG got = 0;
    if (!t->have_ctx) {
        SecBuffer out = { 0, SECBUFFER_TOKEN, NULL };
        SecBufferDesc od = { SECBUFFER_VERSION, 1, &out };
        st = nx_sspi->InitializeSecurityContextA(&t->cred, NULL, (SEC_CHAR*)t->host, NX_ISC_FLAGS, 0, 0, NULL, 0, &t->ctx, &od, &got, &ts);
        if (st != SEC_I_CONTINUE_NEEDED) { nx_tls_say_status(st); return 4; }
        t->have_ctx = true;
        int32_t r = nx_tls_token(t, &out, deadline);
        if (r) return nx_tls_lost(r);
    }
    for (;;) {
        if (t->raw_len == 0) {
            int32_t r = nx_tls_pull(t, deadline);
            if (r) return nx_tls_lost(r);
        }
        SecBuffer in[2] = { { (ULONG)t->raw_len, SECBUFFER_TOKEN, t->raw }, { 0, SECBUFFER_EMPTY, NULL } };
        SecBufferDesc id = { SECBUFFER_VERSION, 2, in };
        SecBuffer out = { 0, SECBUFFER_TOKEN, NULL };
        SecBufferDesc od = { SECBUFFER_VERSION, 1, &out };
        st = nx_sspi->InitializeSecurityContextA(&t->cred, &t->ctx, (SEC_CHAR*)t->host, NX_ISC_FLAGS, 0, 0, &id, 0, NULL, &od, &got, &ts);
        if (st == SEC_E_INCOMPLETE_MESSAGE) {
            int32_t r = nx_tls_pull(t, deadline);
            if (r) return nx_tls_lost(r);
            continue;
        }
        /* a token to send even on failure: the alert that says why */
        int32_t sent = nx_tls_token(t, &out, deadline);
        if (st != SEC_E_OK && st != SEC_I_CONTINUE_NEEDED && st != SEC_I_INCOMPLETE_CREDENTIALS) {
            nx_tls_say_status(st);
            return 4;
        }
        /* keep what this step did not read */
        if (in[1].BufferType == SECBUFFER_EXTRA && in[1].cbBuffer > 0) {
            memmove(t->raw, t->raw + (t->raw_len - in[1].cbBuffer), in[1].cbBuffer);
            t->raw_len = in[1].cbBuffer;
        } else {
            t->raw_len = 0;
        }
        if (sent) return nx_tls_lost(sent);
        if (st == SEC_E_OK) return 0;
    }
}
NX_INLINE int32_t nx_tls_open(nx_tls* t, int64_t deadline) {
    SCHANNEL_CRED sc;
    memset(&sc, 0, sizeof sc);
    sc.dwVersion = SCHANNEL_CRED_VERSION;
    sc.dwFlags = SCH_CRED_AUTO_CRED_VALIDATION | SCH_CRED_NO_DEFAULT_CREDS | SCH_USE_STRONG_CRYPTO;
    TimeStamp ts;
    SECURITY_STATUS st = nx_sspi->AcquireCredentialsHandleA(NULL, (SEC_CHAR*)UNISP_NAME_A, SECPKG_CRED_OUTBOUND, NULL, &sc, NULL, NULL, &t->cred, &ts);
    if (st != SEC_E_OK) { nx_tls_say_status(st); return 4; }
    t->have_cred = true;
    int32_t r = nx_tls_steps(t, deadline);
    if (r) return r;
    st = nx_sspi->QueryContextAttributesA(&t->ctx, SECPKG_ATTR_STREAM_SIZES, &t->sizes);
    if (st != SEC_E_OK) { nx_tls_say_status(st); return 4; }
    return 0;
}
NX_INLINE int32_t nx_tls_write(nx_tls* t, const uint8_t* p, size_t n, int64_t deadline) {
    size_t hdr = t->sizes.cbHeader, tail = t->sizes.cbTrailer, max = t->sizes.cbMaximumMessage;
    uint8_t* buf = (uint8_t*)malloc(hdr + max + tail);
    if (!buf) nx_panic("out of memory in TLS", "net.tls_send");
    int32_t r = 0;
    while (n > 0) {
        size_t k = n < max ? n : max;
        memcpy(buf + hdr, p, k);
        SecBuffer b[4] = { { (ULONG)hdr, SECBUFFER_STREAM_HEADER, buf }, { (ULONG)k, SECBUFFER_DATA, buf + hdr }, { (ULONG)tail, SECBUFFER_STREAM_TRAILER, buf + hdr + k }, { 0, SECBUFFER_EMPTY, NULL } };
        SecBufferDesc d = { SECBUFFER_VERSION, 4, b };
        SECURITY_STATUS st = nx_sspi->EncryptMessage(&t->ctx, 0, &d, 0);
        if (st != SEC_E_OK) { nx_tls_say_status(st); r = 4; break; }
        r = nx_tls_put(t->sock, buf, b[0].cbBuffer + b[1].cbBuffer + b[2].cbBuffer, deadline);
        if (r) { nx_tls_say(r == 3 ? "a TLS send took longer than allowed" : "the connection failed while sending"); break; }
        p += k;
        n -= k;
    }
    free(buf);
    return r;
}
/* decrypt until there is something to read or the connection is over */
NX_INLINE int32_t nx_tls_fill(nx_tls* t, int64_t deadline) {
    while (t->plain_pos >= t->plain_len && !t->ended) {
        if (t->raw_len > 0) {
            SecBuffer b[4] = { { (ULONG)t->raw_len, SECBUFFER_DATA, t->raw }, { 0, SECBUFFER_EMPTY, NULL }, { 0, SECBUFFER_EMPTY, NULL }, { 0, SECBUFFER_EMPTY, NULL } };
            SecBufferDesc d = { SECBUFFER_VERSION, 4, b };
            SECURITY_STATUS st = nx_sspi->DecryptMessage(&t->ctx, &d, 0, NULL);
            if (st == SEC_E_OK || st == SEC_I_RENEGOTIATE || st == SEC_I_CONTEXT_EXPIRED) {
                SecBuffer* data = NULL;
                SecBuffer* extra = NULL;
                for (int i = 1; i < 4; i++) {
                    if (b[i].BufferType == SECBUFFER_DATA) data = &b[i];
                    if (b[i].BufferType == SECBUFFER_EXTRA) extra = &b[i];
                }
                /* the data sits in `raw`: taken before the rest moves over it */
                if (data && data->cbBuffer > 0) nx_tls_keep(t, (const uint8_t*)data->pvBuffer, data->cbBuffer);
                if (extra && extra->cbBuffer > 0) {
                    memmove(t->raw, t->raw + (t->raw_len - extra->cbBuffer), extra->cbBuffer);
                    t->raw_len = extra->cbBuffer;
                } else {
                    t->raw_len = 0;
                }
                if (st == SEC_I_CONTEXT_EXPIRED) {
                    t->ended = true;
                    t->clean = true;
                } else if (st == SEC_I_RENEGOTIATE) {
                    int32_t r = nx_tls_steps(t, deadline);
                    if (r) return r;
                }
                continue;
            }
            if (st != SEC_E_INCOMPLETE_MESSAGE) { nx_tls_say_status(st); return 4; }
        }
        int32_t r = nx_tls_pull(t, deadline);
        if (r == 5) { t->ended = true; break; }
        if (r) return r;
    }
    return 0;
}
/* close_notify, when the connection still runs, and the handles let go */
NX_INLINE void nx_tls_shut(nx_tls* t) {
    if (t->have_ctx && !t->ended) {
        DWORD kind = SCHANNEL_SHUTDOWN;
        SecBuffer b = { sizeof kind, SECBUFFER_TOKEN, &kind };
        SecBufferDesc d = { SECBUFFER_VERSION, 1, &b };
        if (nx_sspi->ApplyControlToken(&t->ctx, &d) == SEC_E_OK) {
            SecBuffer out = { 0, SECBUFFER_TOKEN, NULL };
            SecBufferDesc od = { SECBUFFER_VERSION, 1, &out };
            ULONG got = 0;
            TimeStamp ts;
            nx_sspi->InitializeSecurityContextA(&t->cred, &t->ctx, (SEC_CHAR*)t->host, NX_ISC_FLAGS, 0, 0, NULL, 0, NULL, &od, &got, &ts);
            nx_tls_token(t, &out, nx_deadline(1000));
        }
    }
    if (t->have_ctx) nx_sspi->DeleteSecurityContext(&t->ctx);
    if (t->have_cred) nx_sspi->FreeCredentialsHandle(&t->cred);
    free(t->raw);
    t->raw = NULL;
}
#elif defined(__APPLE__)
/* ----- Security.framework's Secure Transport, found with dlsym */
typedef int32_t nx_osstatus;
typedef nx_osstatus (*nx_st_readfn)(const void*, void*, size_t*);
typedef nx_osstatus (*nx_st_writefn)(const void*, const void*, size_t*);
typedef struct {
    void* (*SSLCreateContext)(const void*, int, int);
    nx_osstatus (*SSLSetIOFuncs)(void*, nx_st_readfn, nx_st_writefn);
    nx_osstatus (*SSLSetConnection)(void*, const void*);
    nx_osstatus (*SSLSetPeerDomainName)(void*, const char*, size_t);
    nx_osstatus (*SSLHandshake)(void*);
    nx_osstatus (*SSLWrite)(void*, const void*, size_t, size_t*);
    nx_osstatus (*SSLRead)(void*, void*, size_t, size_t*);
    nx_osstatus (*SSLClose)(void*);
    void (*CFRelease)(const void*);
} nx_sectrans;
NX_STATE nx_sectrans nx_st;
NX_STATE int nx_st_state;
#define NX_ST_WOULD_BLOCK (-9803)
#define NX_ST_CLOSED_GRACEFUL (-9805)
#define NX_ST_CLOSED_ABORT (-9806)
#define NX_ST_CLOSED_NO_NOTIFY (-9816)
NX_INLINE bool nx_tls_available(void) {
    int st = __atomic_load_n(&nx_st_state, __ATOMIC_ACQUIRE);
    if (st == 0) {
        void* sec = dlopen("/System/Library/Frameworks/Security.framework/Security", RTLD_LAZY);
        void* cf = dlopen("/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation", RTLD_LAZY);
        bool ok = sec && cf;
#define NX_ST_SYM(lib, f) if (ok) { *(void**)&nx_st.f = dlsym(lib, #f); ok = nx_st.f != NULL; }
        NX_ST_SYM(sec, SSLCreateContext)
        NX_ST_SYM(sec, SSLSetIOFuncs)
        NX_ST_SYM(sec, SSLSetConnection)
        NX_ST_SYM(sec, SSLSetPeerDomainName)
        NX_ST_SYM(sec, SSLHandshake)
        NX_ST_SYM(sec, SSLWrite)
        NX_ST_SYM(sec, SSLRead)
        NX_ST_SYM(sec, SSLClose)
        NX_ST_SYM(cf, CFRelease)
#undef NX_ST_SYM
        st = ok ? 1 : 2;
        __atomic_store_n(&nx_st_state, st, __ATOMIC_RELEASE);
    }
    return st == 1;
}
/* The socket is non-blocking: a read takes what has come and says
   "would block" for the rest, so SSLRead gives what it decrypted without
   waiting to fill its buffer; the callers wait for the socket. */
static nx_osstatus nx_st_read(const void* conn, void* data, size_t* len) {
    nx_tls* t = (nx_tls*)conn;
    size_t want = *len, got = 0;
    while (got < want) {
        ssize_t k = recv(t->sock, (char*)data + got, want - got, 0);
        if (k > 0) { got += (size_t)k; continue; }
        if (k == 0) { *len = got; return NX_ST_CLOSED_NO_NOTIFY; }
        if (errno == EINTR) continue;
        *len = got;
        return errno == EAGAIN || errno == EWOULDBLOCK ? NX_ST_WOULD_BLOCK : NX_ST_CLOSED_ABORT;
    }
    *len = got;
    return 0;
}
static nx_osstatus nx_st_write(const void* conn, const void* data, size_t* len) {
    nx_tls* t = (nx_tls*)conn;
    int32_t r = nx_tls_put(t->sock, (const uint8_t*)data, *len, t->deadline);
    if (r) { *len = 0; return NX_ST_CLOSED_ABORT; }
    return 0;
}
NX_INLINE void nx_tls_say_status(nx_osstatus st) {
    const char* what = "the TLS handshake failed";
    if (st == -9807 || st == -9812 || st == -9813) what = "the server's certificate is not signed by a root this system trusts";
    else if (st == -9843) what = "the server's certificate is for another name";
    else if (st == -9814 || st == -9815) what = "the server's certificate has expired or is not valid yet";
    else if (st == -9808) what = "the server's certificate is bad";
    else if (st == -9824) what = "the server refused the handshake";
    else if (st == NX_ST_CLOSED_ABORT || st == NX_ST_CLOSED_NO_NOTIFY) what = "the server closed the connection during the TLS handshake";
    snprintf(nx_tls_why, sizeof nx_tls_why, "%s (Secure Transport %d)", what, (int)st);
}
/* wait for the socket to have something to read; false once the deadline passed */
NX_INLINE bool nx_tls_await(nx_tls* t, int64_t deadline) {
    int64_t left = nx_until(deadline);
    return left >= 0 && nx_net_wait(t->sock, false, left);
}
NX_INLINE int32_t nx_tls_open(nx_tls* t, int64_t deadline) {
    void* ctx = nx_st.SSLCreateContext(NULL, 1, 0);
    if (!ctx) { nx_tls_say("Secure Transport would not make a context"); return 4; }
    t->ssl = ctx;
    nx_st.SSLSetIOFuncs(ctx, nx_st_read, nx_st_write);
    nx_st.SSLSetConnection(ctx, t);
    nx_st.SSLSetPeerDomainName(ctx, t->host, strlen(t->host));
    nx_net_blocking(t->sock, false);
    t->deadline = deadline;
    for (;;) {
        nx_osstatus st = nx_st.SSLHandshake(ctx);
        if (st == 0) return 0;
        if (st == NX_ST_WOULD_BLOCK) {
            if (!nx_tls_await(t, deadline)) { nx_tls_say("the TLS handshake took longer than allowed"); return 3; }
            continue;
        }
        nx_tls_say_status(st);
        return 4;
    }
}
NX_INLINE int32_t nx_tls_write(nx_tls* t, const uint8_t* p, size_t n, int64_t deadline) {
    t->deadline = deadline;
    while (n > 0) {
        size_t done = 0;
        nx_osstatus st = nx_st.SSLWrite(t->ssl, p, n, &done);
        p += done;
        n -= done;
        if (st == 0) continue;
        if (st == NX_ST_WOULD_BLOCK) {
            if (!nx_tls_await(t, deadline)) { nx_tls_say("a TLS send took longer than allowed"); return 3; }
            continue;
        }
        nx_tls_say("the connection failed while sending");
        return 4;
    }
    return 0;
}
NX_INLINE int32_t nx_tls_fill(nx_tls* t, int64_t deadline) {
    uint8_t buf[16384];
    t->deadline = deadline;
    while (t->plain_pos >= t->plain_len && !t->ended) {
        size_t got = 0;
        nx_osstatus st = nx_st.SSLRead(t->ssl, buf, sizeof buf, &got);
        if (got > 0) nx_tls_keep(t, buf, got);
        if (st == 0) continue;
        if (st == NX_ST_WOULD_BLOCK) {
            if (got > 0) continue;
            if (!nx_tls_await(t, deadline)) return 3;
            continue;
        }
        if (st == NX_ST_CLOSED_GRACEFUL) { t->ended = true; t->clean = true; break; }
        if (st == NX_ST_CLOSED_NO_NOTIFY || st == NX_ST_CLOSED_ABORT) { t->ended = true; break; }
        nx_tls_say_status(st);
        return 4;
    }
    return 0;
}
NX_INLINE void nx_tls_shut(nx_tls* t) {
    if (!t->ssl) return;
    if (!t->ended) {
        t->deadline = nx_deadline(1000);
        nx_st.SSLClose(t->ssl);
    }
    nx_st.CFRelease(t->ssl);
    t->ssl = NULL;
}
#else
/* ----- OpenSSL's libssl, loaded with dlopen */
typedef struct {
    int (*OPENSSL_init_ssl)(uint64_t, const void*);
    const void* (*TLS_client_method)(void);
    void* (*SSL_CTX_new)(const void*);
    int (*SSL_CTX_set_default_verify_paths)(void*);
    void (*SSL_CTX_set_verify)(void*, int, void*);
    void* (*SSL_new)(void*);
    int (*SSL_set_fd)(void*, int);
    long (*SSL_ctrl)(void*, int, long, void*);
    int (*SSL_set1_host)(void*, const char*);
    int (*SSL_connect)(void*);
    int (*SSL_read)(void*, void*, int);
    int (*SSL_write)(void*, const void*, int);
    int (*SSL_shutdown)(void*);
    int (*SSL_get_error)(const void*, int);
    long (*SSL_get_verify_result)(const void*);
    void (*SSL_free)(void*);
    unsigned long (*ERR_get_error)(void);
    void (*ERR_clear_error)(void);
    void (*ERR_error_string_n)(unsigned long, char*, size_t);
    const char* (*X509_verify_cert_error_string)(long);
} nx_openssl;
NX_STATE nx_openssl nx_ossl;
NX_STATE void* nx_ossl_ctx;
NX_STATE int nx_ossl_state;
NX_STATE bool nx_ossl_3;
#define NX_SSL_WANT_READ 2
#define NX_SSL_WANT_WRITE 3
#define NX_SSL_ERROR_SSL 1
#define NX_SSL_ERROR_SYSCALL 5
#define NX_SSL_ZERO_RETURN 6
NX_INLINE bool nx_tls_available(void) {
    int expected = 0;
    if (__atomic_compare_exchange_n(&nx_ossl_state, &expected, 3, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) {
        static const char* names[] = { "libssl.so.3", "libssl.so.1.1", "libssl.so" };
        void* h = NULL;
        for (int i = 0; i < 3 && !h; i++) {
            h = dlopen(names[i], RTLD_NOW | RTLD_GLOBAL);
            if (h && i == 0) nx_ossl_3 = true;
        }
        bool ok = h != NULL;
#define NX_OSSL_SYM(f) if (ok) { *(void**)&nx_ossl.f = dlsym(h, #f); ok = nx_ossl.f != NULL; }
        NX_OSSL_SYM(OPENSSL_init_ssl)
        NX_OSSL_SYM(TLS_client_method)
        NX_OSSL_SYM(SSL_CTX_new)
        NX_OSSL_SYM(SSL_CTX_set_default_verify_paths)
        NX_OSSL_SYM(SSL_CTX_set_verify)
        NX_OSSL_SYM(SSL_new)
        NX_OSSL_SYM(SSL_set_fd)
        NX_OSSL_SYM(SSL_ctrl)
        NX_OSSL_SYM(SSL_set1_host)
        NX_OSSL_SYM(SSL_connect)
        NX_OSSL_SYM(SSL_read)
        NX_OSSL_SYM(SSL_write)
        NX_OSSL_SYM(SSL_shutdown)
        NX_OSSL_SYM(SSL_get_error)
        NX_OSSL_SYM(SSL_get_verify_result)
        NX_OSSL_SYM(SSL_free)
        NX_OSSL_SYM(ERR_get_error)
        NX_OSSL_SYM(ERR_clear_error)
        NX_OSSL_SYM(ERR_error_string_n)
        NX_OSSL_SYM(X509_verify_cert_error_string)
#undef NX_OSSL_SYM
        if (ok) {
            nx_ossl.OPENSSL_init_ssl(0, NULL);
            nx_ossl_ctx = nx_ossl.SSL_CTX_new(nx_ossl.TLS_client_method());
            ok = nx_ossl_ctx != NULL && nx_ossl.SSL_CTX_set_default_verify_paths(nx_ossl_ctx) == 1;
            /* SSL_VERIFY_PEER: a certificate that does not check out ends the handshake */
            if (ok) nx_ossl.SSL_CTX_set_verify(nx_ossl_ctx, 1, NULL);
        }
        __atomic_store_n(&nx_ossl_state, ok ? 1 : 2, __ATOMIC_RELEASE);
    }
    int st;
    while ((st = __atomic_load_n(&nx_ossl_state, __ATOMIC_ACQUIRE)) == 3) nx_sleep_ms(1);
    return st == 1;
}
NX_INLINE void nx_tls_say_error(const char* what) {
    unsigned long e = nx_ossl.ERR_get_error();
    if (e == 0) { nx_tls_say(what); return; }
    char buf[160];
    nx_ossl.ERR_error_string_n(e, buf, sizeof buf);
    snprintf(nx_tls_why, sizeof nx_tls_why, "%s (%s)", what, buf);
}
/* wait for what OpenSSL asked for; false once the deadline passed */
NX_INLINE bool nx_tls_await(nx_tls* t, int want, int64_t deadline) {
    int64_t left = nx_until(deadline);
    return left >= 0 && nx_net_wait(t->sock, want == NX_SSL_WANT_WRITE, left);
}
NX_INLINE int32_t nx_tls_open(nx_tls* t, int64_t deadline) {
    void* ssl = nx_ossl.SSL_new(nx_ossl_ctx);
    if (!ssl) { nx_tls_say_error("OpenSSL would not make a connection"); return 4; }
    t->ssl = ssl;
    nx_ossl.SSL_set_fd(ssl, (int)t->sock);
    /* SSL_set_tlsext_host_name: the name for the server to pick a certificate by */
    nx_ossl.SSL_ctrl(ssl, 55, 0, t->host);
    /* and the name the certificate must carry */
    nx_ossl.SSL_set1_host(ssl, t->host);
    nx_net_blocking(t->sock, false);
    nx_ossl.ERR_clear_error();
    for (;;) {
        int r = nx_ossl.SSL_connect(ssl);
        if (r == 1) return 0;
        int e = nx_ossl.SSL_get_error(ssl, r);
        if (e == NX_SSL_WANT_READ || e == NX_SSL_WANT_WRITE) {
            if (!nx_tls_await(t, e, deadline)) { nx_tls_say("the TLS handshake took longer than allowed"); return 3; }
            continue;
        }
        long v = nx_ossl.SSL_get_verify_result(ssl);
        if (v != 0) {
            snprintf(nx_tls_why, sizeof nx_tls_why, "the server's certificate does not check out: %s", nx_ossl.X509_verify_cert_error_string(v));
        } else if (e == NX_SSL_ERROR_SYSCALL) {
            nx_tls_say_error("the server closed the connection during the TLS handshake");
        } else {
            nx_tls_say_error("the TLS handshake failed");
        }
        return 4;
    }
}
NX_INLINE int32_t nx_tls_write(nx_tls* t, const uint8_t* p, size_t n, int64_t deadline) {
    sigset_t old;
    nx_sigpipe_hold(&old);
    int32_t code = 0;
    while (n > 0) {
        int k = n > (1u << 30) ? (1 << 30) : (int)n;
        nx_ossl.ERR_clear_error();
        int r = nx_ossl.SSL_write(t->ssl, p, k);
        if (r > 0) { p += r; n -= (size_t)r; continue; }
        int e = nx_ossl.SSL_get_error(t->ssl, r);
        if (e == NX_SSL_WANT_READ || e == NX_SSL_WANT_WRITE) {
            if (nx_tls_await(t, e, deadline)) continue;
            nx_tls_say("a TLS send took longer than allowed");
            code = 3;
            break;
        }
        nx_tls_say_error("the connection failed while sending");
        code = 4;
        break;
    }
    nx_sigpipe_release(&old);
    return code;
}
NX_INLINE int32_t nx_tls_fill(nx_tls* t, int64_t deadline) {
    uint8_t buf[16384];
    while (t->plain_pos >= t->plain_len && !t->ended) {
        nx_ossl.ERR_clear_error();
        int r = nx_ossl.SSL_read(t->ssl, buf, (int)sizeof buf);
        if (r > 0) { nx_tls_keep(t, buf, (size_t)r); continue; }
        int e = nx_ossl.SSL_get_error(t->ssl, r);
        if (e == NX_SSL_ZERO_RETURN) { t->ended = true; t->clean = true; break; }
        if (e == NX_SSL_WANT_READ || e == NX_SSL_WANT_WRITE) {
            if (!nx_tls_await(t, e, deadline)) return 3;
            continue;
        }
        /* the peer went away without close_notify: OpenSSL 1.1 says so
           with an empty error queue, 3 with UNEXPECTED_EOF_WHILE_READING */
        unsigned long err = nx_ossl.ERR_get_error();
        if ((e == NX_SSL_ERROR_SYSCALL && err == 0) || (nx_ossl_3 && (err & 0x7FFFFF) == 294)) { t->ended = true; break; }
        char text[160];
        nx_ossl.ERR_error_string_n(err, text, sizeof text);
        snprintf(nx_tls_why, sizeof nx_tls_why, "a TLS record did not decrypt (%s)", text);
        return 4;
    }
    return 0;
}
NX_INLINE void nx_tls_shut(nx_tls* t) {
    if (!t->ssl) return;
    if (!t->ended) {
        sigset_t old;
        nx_sigpipe_hold(&old);
        nx_ossl.SSL_shutdown(t->ssl);
        nx_sigpipe_release(&old);
    }
    nx_ossl.SSL_free(t->ssl);
    t->ssl = NULL;
}
#endif

/* the slot let go: the platform's part first, then the socket */
NX_INLINE void nx_tls_release(nx_tls* t) {
    nx_tls_shut(t);
    free(t->plain);
    t->plain = NULL;
    if (t->sock != NX_BAD_SOCK) nx_net_close((int64_t)t->sock);
    t->sock = NX_BAD_SOCK;
    __atomic_store_n(&t->state, 0, __ATOMIC_RELEASE);
}
NX_INLINE int32_t nx_tls_connect(nx_sl_u8 host, uint16_t port, int64_t timeout_ms, int64_t* out) {
    nx_tls_why[0] = 0;
    if (!nx_tls_available()) {
        nx_tls_say("this system has no TLS library to load (OpenSSL's libssl)");
        return 4;
    }
    if (host.len == 0 || host.len >= sizeof nx_tlss[0].host) { nx_tls_say("TLS needs a host name"); return 4; }
    int64_t deadline = nx_deadline(timeout_ms);
    int64_t sock = 0;
    int32_t r = nx_tcp_connect(host, port, timeout_ms, &sock);
    if (r) {
        nx_tls_say(r == 1 ? "no such host" : r == 2 ? "the connection was refused" : r == 3 ? "the connection took longer than allowed" : "the connection failed");
        return r;
    }
    int slot = nx_tls_claim();
    if (slot < 0) {
        nx_net_close(sock);
        nx_tls_say("too many TLS connections are open");
        return 4;
    }
    nx_tls* t = &nx_tlss[slot];
    t->sock = (nx_sock)sock;
    memcpy(t->host, host.ptr, host.len);
    t->host[host.len] = 0;
    t->timeout_ms = timeout_ms;
    r = nx_tls_open(t, deadline);
    if (r) {
        /* no close_notify for a handshake that did not finish */
        t->ended = true;
        nx_tls_release(t);
        return r;
    }
    __atomic_store_n(&t->state, 2, __ATOMIC_RELEASE);
    *out = slot + 1;
    return 0;
}
NX_INLINE int32_t nx_tls_send(int64_t h, nx_sl_u8 data) {
    nx_tls* t = nx_tls_at(h);
    if (!t) { nx_tls_say("not an open TLS connection"); return 4; }
    if (t->ended) { nx_tls_say("the connection is over"); return 4; }
    return nx_tls_write(t, data.ptr, data.len, nx_deadline(t->timeout_ms));
}
/* Up to `n` bytes of what the server sent, waiting at most `timeout_ms`
   (0: no limit) for some; empty at the end of the connection. */
NX_INLINE int32_t nx_tls_recv(nx_ctx* c, int64_t h, size_t n, int64_t timeout_ms, nx_string* out) {
    nx_tls* t = nx_tls_at(h);
    if (!t) { nx_tls_say("not an open TLS connection"); return 4; }
    int32_t r = nx_tls_fill(t, nx_deadline(timeout_ms));
    if (r) return r;
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    size_t have = t->plain_len - t->plain_pos;
    if (have > n) have = n;
    if (have > 0) {
        nx_str_append(c, &s, t->plain + t->plain_pos, have);
        t->plain_pos += have;
    }
    *out = s;
    return 0;
}
/* Did the connection end without close_notify, so that what came may be cut short? */
NX_INLINE bool nx_tls_truncated(int64_t h) {
    nx_tls* t = nx_tls_at(h);
    return t && t->ended && !t->clean;
}
NX_INLINE void nx_tls_close(int64_t h) {
    nx_tls* t = nx_tls_at(h);
    if (!t) return;
    int expected = 2;
    if (!__atomic_compare_exchange_n(&t->state, &expected, 1, false, __ATOMIC_ACQ_REL, __ATOMIC_ACQUIRE)) return;
    nx_tls_release(t);
}
#endif

/* ------------------------------------------------------------- threads */
/* A spawned thread runs a Nexium function value `fn(*mut X)` with its own
   context; a panic inside it is re-raised by the joiner. Handles are
   pointers to the task record, freed by join. */
typedef struct nx_thread_task {
    nx_ctx ctx;
    void* fnp;
    void* env;
    void* arg;
    bool panicked;
    bool started;
    char msg[256];
    char loc[256];
#if defined(_WIN32)
    HANDLE h;
#else
    pthread_t h;
#endif
} nx_thread_task;
static void nx_thread_run(nx_thread_task* t) {
    nx_boundary b;
    b.track = NULL;
    nx_boundary* prev = nx_tls_boundary;
    nx_tls_boundary = &b;
    if (setjmp(b.jb)) {
        t->panicked = true;
        snprintf(t->msg, sizeof t->msg, "%s", b.msg);
        snprintf(t->loc, sizeof t->loc, "%s", b.loc);
    } else {
        ((void (*)(nx_ctx*, void*, void*))t->fnp)(&t->ctx, t->env, t->arg);
    }
    nx_tls_boundary = prev;
}
#if defined(_WIN32)
static DWORD WINAPI nx_thread_entry(LPVOID p) { nx_thread_run((nx_thread_task*)p); return 0; }
#else
static void* nx_thread_entry(void* p) { nx_thread_run((nx_thread_task*)p); return NULL; }
#endif
NX_INLINE int64_t nx_thread_start(nx_ctx* c, void* fnp, void* env, void* arg) {
    nx_thread_task* t = (nx_thread_task*)malloc(sizeof *t);
    if (!t) nx_panic("out of memory starting a thread", "thread.start");
    t->ctx = *c;
    nx_ctx_untrack(&t->ctx);
    t->ctx.live_allocs = 0; t->ctx.live_bytes = 0; t->ctx.total_allocs = 0; t->ctx.peak_bytes = 0;
    nx_ctx_track_self(&t->ctx);
    t->ctx.rng ^= (uint64_t)(uintptr_t)t * 0x9E3779B97F4A7C15ULL;
    t->fnp = fnp; t->env = env; t->arg = arg;
    t->panicked = false; t->started = true;
#if defined(_WIN32)
    t->h = CreateThread(NULL, 0, nx_thread_entry, t, 0, NULL);
    if (!t->h) { t->started = false; nx_thread_run(t); }
#else
    if (pthread_create(&t->h, NULL, nx_thread_entry, t) != 0) { t->started = false; nx_thread_run(t); }
#endif
    return (int64_t)(intptr_t)t;
}
/* wait for a thread and free its record; true, with what it said in `msg`,
   when it panicked */
NX_INLINE bool nx_thread_wait(int64_t h, char* msg, size_t cap) {
    nx_thread_task* t = (nx_thread_task*)(intptr_t)h;
    if (!t) return false;
    if (t->started) {
#if defined(_WIN32)
        WaitForSingleObject(t->h, INFINITE);
        CloseHandle(t->h);
#else
        pthread_join(t->h, NULL);
#endif
    }
    bool panicked = t->panicked;
    if (panicked) snprintf(msg, cap, "in a thread: %s (at %s)", t->msg, t->loc);
    free(t);
    return panicked;
}
NX_INLINE void nx_thread_join(int64_t h, const char* loc) {
    char msg[600];
    if (nx_thread_wait(h, msg, sizeof msg)) nx_panic(msg, loc);
}
/* thread.join_all: every thread is waited for before the first panic among
   them is re-raised, so none runs on with storage the panic releases */
NX_INLINE void nx_thread_join_all(const int64_t* hs, size_t n, const char* loc) {
    char msg[600], first[600];
    bool panicked = false;
    for (size_t i = 0; i < n; i++) {
        if (nx_thread_wait(hs[i], msg, sizeof msg) && !panicked) {
            panicked = true;
            memcpy(first, msg, sizeof first);
        }
    }
    if (panicked) nx_panic(first, loc);
}
/* mutexes and condition variables, as heap handles */
#if defined(_WIN32)
NX_INLINE int64_t nx_mutex_new(void) { CRITICAL_SECTION* m = (CRITICAL_SECTION*)malloc(sizeof *m); InitializeCriticalSection(m); return (int64_t)(intptr_t)m; }
NX_INLINE void nx_mutex_lock_raw(int64_t m) { EnterCriticalSection((CRITICAL_SECTION*)(intptr_t)m); }
NX_INLINE void nx_mutex_unlock_raw(int64_t m) { LeaveCriticalSection((CRITICAL_SECTION*)(intptr_t)m); }
NX_INLINE void nx_mutex_free(int64_t m) { DeleteCriticalSection((CRITICAL_SECTION*)(intptr_t)m); free((void*)(intptr_t)m); }
NX_INLINE int64_t nx_cond_new(void) { CONDITION_VARIABLE* cv = (CONDITION_VARIABLE*)malloc(sizeof *cv); InitializeConditionVariable(cv); return (int64_t)(intptr_t)cv; }
NX_INLINE void nx_cond_wait(int64_t cv, int64_t m) { SleepConditionVariableCS((CONDITION_VARIABLE*)(intptr_t)cv, (CRITICAL_SECTION*)(intptr_t)m, INFINITE); }
NX_INLINE void nx_cond_signal(int64_t cv) { WakeConditionVariable((CONDITION_VARIABLE*)(intptr_t)cv); }
NX_INLINE void nx_cond_broadcast(int64_t cv) { WakeAllConditionVariable((CONDITION_VARIABLE*)(intptr_t)cv); }
NX_INLINE void nx_cond_free(int64_t cv) { free((void*)(intptr_t)cv); }
#else
NX_INLINE int64_t nx_mutex_new(void) { pthread_mutex_t* m = (pthread_mutex_t*)malloc(sizeof *m); pthread_mutex_init(m, NULL); return (int64_t)(intptr_t)m; }
NX_INLINE void nx_mutex_lock_raw(int64_t m) { pthread_mutex_lock((pthread_mutex_t*)(intptr_t)m); }
NX_INLINE void nx_mutex_unlock_raw(int64_t m) { pthread_mutex_unlock((pthread_mutex_t*)(intptr_t)m); }
NX_INLINE void nx_mutex_free(int64_t m) { pthread_mutex_destroy((pthread_mutex_t*)(intptr_t)m); free((void*)(intptr_t)m); }
NX_INLINE int64_t nx_cond_new(void) { pthread_cond_t* cv = (pthread_cond_t*)malloc(sizeof *cv); pthread_cond_init(cv, NULL); return (int64_t)(intptr_t)cv; }
NX_INLINE void nx_cond_wait(int64_t cv, int64_t m) { pthread_cond_wait((pthread_cond_t*)(intptr_t)cv, (pthread_mutex_t*)(intptr_t)m); }
NX_INLINE void nx_cond_signal(int64_t cv) { pthread_cond_signal((pthread_cond_t*)(intptr_t)cv); }
NX_INLINE void nx_cond_broadcast(int64_t cv) { pthread_cond_broadcast((pthread_cond_t*)(intptr_t)cv); }
NX_INLINE void nx_cond_free(int64_t cv) { pthread_cond_destroy((pthread_cond_t*)(intptr_t)cv); free((void*)(intptr_t)cv); }
#endif
/* a lock taken inside an export call registers with the call's tracker (the
   tracker's own lock uses the raw pair, which registers nothing) */
NX_INLINE void nx_mutex_lock(int64_t m) { nx_mutex_lock_raw(m); nx_track_handle(2, m, true); }
NX_INLINE void nx_mutex_unlock(int64_t m) { nx_track_handle(2, m, false); nx_mutex_unlock_raw(m); }
/* sync.wait_for: sync.wait for at most `ms` (negative: for ever); false when
   the time ran out. Like sync.wait it may also return with nothing
   signalled, so the caller checks its condition again. */
NX_INLINE bool nx_cond_wait_for(int64_t cv, int64_t m, int64_t ms) {
    if (ms < 0) { nx_cond_wait(cv, m); return true; }
#if defined(_WIN32)
    return SleepConditionVariableCS((CONDITION_VARIABLE*)(intptr_t)cv, (CRITICAL_SECTION*)(intptr_t)m, ms > 0x7ffffffe ? 0x7ffffffe : (DWORD)ms) != 0;
#else
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    ts.tv_sec += (time_t)(ms / 1000);
    ts.tv_nsec += (long)(ms % 1000) * 1000000L;
    if (ts.tv_nsec >= 1000000000L) { ts.tv_sec += 1; ts.tv_nsec -= 1000000000L; }
    return pthread_cond_timedwait((pthread_cond_t*)(intptr_t)cv, (pthread_mutex_t*)(intptr_t)m, &ts) == 0;
#endif
}

/* A bell: rung by any thread, waited for by one, a ring before the wait
   kept until it. std.thread's select waits on one while the channels it
   watches ring it. */
typedef struct { int64_t m, cv; bool rung; } nx_bell;
NX_INLINE int64_t nx_bell_new(void) {
    nx_bell* b = (nx_bell*)malloc(sizeof *b);
    if (!b) nx_panic("out of memory making a bell", "sync.bell_new");
    b->m = nx_mutex_new();
    b->cv = nx_cond_new();
    b->rung = false;
    return (int64_t)(intptr_t)b;
}
NX_INLINE void nx_bell_ring(int64_t h) {
    nx_bell* b = (nx_bell*)(intptr_t)h;
    nx_mutex_lock_raw(b->m);
    b->rung = true;
    nx_cond_signal(b->cv);
    nx_mutex_unlock_raw(b->m);
}
/* true when it rang (and it is quiet again), false when `ms` passed first
   (negative: waits for ever) */
NX_INLINE bool nx_bell_wait(int64_t h, int64_t ms) {
    nx_bell* b = (nx_bell*)(intptr_t)h;
    int64_t start = nx_mono_ms();
    nx_mutex_lock_raw(b->m);
    while (!b->rung) {
        int64_t left = nx_left_ms(start, ms);
        if (left == 0) break;
        nx_cond_wait_for(b->cv, b->m, left);
    }
    bool rang = b->rung;
    b->rung = false;
    nx_mutex_unlock_raw(b->m);
    return rang;
}
NX_INLINE void nx_bell_free(int64_t h) {
    nx_bell* b = (nx_bell*)(intptr_t)h;
    if (!b) return;
    nx_cond_free(b->cv);
    nx_mutex_free(b->m);
    free(b);
}

/* sync.atomic_*: an i64 read and changed whole by any thread, sequentially
   consistent */
NX_INLINE int64_t nx_atomic_load(const int64_t* p) { return __atomic_load_n(p, __ATOMIC_SEQ_CST); }
NX_INLINE void nx_atomic_store(int64_t* p, int64_t v) { __atomic_store_n(p, v, __ATOMIC_SEQ_CST); }
NX_INLINE int64_t nx_atomic_add(int64_t* p, int64_t v) { return __atomic_fetch_add(p, v, __ATOMIC_SEQ_CST); }
NX_INLINE int64_t nx_atomic_swap(int64_t* p, int64_t v) { return __atomic_exchange_n(p, v, __ATOMIC_SEQ_CST); }
NX_INLINE bool nx_atomic_cas(int64_t* p, int64_t expected, int64_t desired) {
    return __atomic_compare_exchange_n(p, &expected, desired, false, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);
}

/* ---------------------------------------------------- raw terminal input */
/* `io.raw_mode(true)`: the console gives bytes as they are typed, without
 * echo, with VT sequences in (arrow keys) and out (colours); false restores
 * what was there, and so does exit. The REPL's line editor lives on this.
 * `io.read_key()` is one byte from the same buffer `io.read_line()` reads,
 * `io.pending_input()` how many are buffered (an escape sequence arrives
 * whole). */
#if defined(_WIN32)
NX_STATE DWORD nx_saved_in_mode, nx_saved_out_mode;
NX_STATE bool nx_raw_saved;
static void nx_raw_restore(void) {
    if (!nx_raw_saved) return;
    SetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), nx_saved_in_mode);
    SetConsoleMode(GetStdHandle(STD_OUTPUT_HANDLE), nx_saved_out_mode);
}
NX_INLINE bool nx_raw_mode(bool on) {
    HANDLE hin = GetStdHandle(STD_INPUT_HANDLE), hout = GetStdHandle(STD_OUTPUT_HANDLE);
    if (!on) { nx_raw_restore(); return true; }
    DWORD im, om;
    if (!GetConsoleMode(hin, &im) || !GetConsoleMode(hout, &om)) return false;
    if (!nx_raw_saved) { nx_saved_in_mode = im; nx_saved_out_mode = om; nx_raw_saved = true; atexit(nx_raw_restore); }
    DWORD nim = (im & ~(DWORD)(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT)) | ENABLE_VIRTUAL_TERMINAL_INPUT;
    if (!SetConsoleMode(hin, nim)) return false;
    SetConsoleMode(hout, om | ENABLE_VIRTUAL_TERMINAL_PROCESSING | ENABLE_PROCESSED_OUTPUT);
    return true;
}
#elif defined(NX_WASM)
NX_INLINE bool nx_raw_mode(bool on) { (void)on; return false; }
#else
NX_STATE struct termios nx_saved_termios;
NX_STATE bool nx_raw_saved;
static void nx_raw_restore(void) { if (nx_raw_saved) tcsetattr(0, TCSANOW, &nx_saved_termios); }
NX_INLINE bool nx_raw_mode(bool on) {
    if (!on) { nx_raw_restore(); return true; }
    struct termios t;
    if (tcgetattr(0, &t) != 0) return false;
    if (!nx_raw_saved) { nx_saved_termios = t; nx_raw_saved = true; atexit(nx_raw_restore); }
    t.c_lflag &= ~(tcflag_t)(ICANON | ECHO | ISIG);
    t.c_iflag &= ~(tcflag_t)(ICRNL);
    t.c_cc[VMIN] = 1;
    t.c_cc[VTIME] = 0;
    return tcsetattr(0, TCSANOW, &t) == 0;
}
#endif
NX_INLINE bool nx_read_key(int64_t* out) {
    if (!nx_stdin_fill()) return false;
    *out = (int64_t)nx_stdin_buf[nx_stdin_pos++];
    return true;
}
NX_INLINE int64_t nx_pending_input(void) { return (int64_t)(nx_stdin_len - nx_stdin_pos); }

NX_INLINE bool nx_read_line(nx_ctx* c, nx_string* out) {
    nx_string s; s.ptr = NULL; s.len = 0; s.cap = 0; s.ar = c->arena;
    bool any = false;
    while (nx_stdin_fill()) {
        uint8_t b = nx_stdin_buf[nx_stdin_pos++];
#if defined(_WIN32)
        /* a console in binary mode passes Ctrl-Z through; keep it as end of input */
        if (b == 0x1A && !any) return false;
#endif
        any = true;
        if (b == '\n') break;
        nx_str_append(c, &s, &b, 1);
    }
    if (!any) return false;
    if (s.len && s.ptr[s.len - 1] == '\r') s.len--;
    *out = s;
    return true;
}

/* ----------------------------------------------------- checked arithmetic */
#define NX_INT_OPS(N, T, UT, MIN, MAX) \
    NX_INLINE T nx_add_##N(T a, T b, const char* loc) { T r; if (__builtin_add_overflow(a, b, &r)) nx_panic("integer overflow in `+`", loc); return r; } \
    NX_INLINE T nx_sub_##N(T a, T b, const char* loc) { T r; if (__builtin_sub_overflow(a, b, &r)) nx_panic("integer overflow in `-`", loc); return r; } \
    NX_INLINE T nx_mul_##N(T a, T b, const char* loc) { T r; if (__builtin_mul_overflow(a, b, &r)) nx_panic("integer overflow in `*`", loc); return r; } \
    NX_INLINE T nx_div_##N(T a, T b, const char* loc) { if (b == 0) nx_panic("division by zero", loc); if ((T)(MIN) < 0 && a == (T)(MIN) && b == (T)-1) nx_panic("integer overflow in `/`", loc); return a / b; } \
    NX_INLINE T nx_rem_##N(T a, T b, const char* loc) { if (b == 0) nx_panic("remainder by zero", loc); if ((T)(MIN) < 0 && a == (T)(MIN) && b == (T)-1) return 0; return a % b; } \
    NX_INLINE T nx_neg_##N(T a, const char* loc) { if ((T)(MIN) < 0 && a == (T)(MIN)) nx_panic("integer overflow in negation", loc); return (T)(-a); } \
    NX_INLINE T nx_addw_##N(T a, T b) { return (T)((UT)a + (UT)b); } \
    NX_INLINE T nx_subw_##N(T a, T b) { return (T)((UT)a - (UT)b); } \
    NX_INLINE T nx_mulw_##N(T a, T b) { return (T)((UT)a * (UT)b); } \
    NX_INLINE T nx_adds_##N(T a, T b) { T r; if (__builtin_add_overflow(a, b, &r)) return (b > 0) ? (T)(MAX) : (T)(MIN); return r; } \
    NX_INLINE T nx_subs_##N(T a, T b) { T r; if (__builtin_sub_overflow(a, b, &r)) return (b > 0) ? (T)(MIN) : (T)(MAX); return r; } \
    NX_INLINE T nx_muls_##N(T a, T b) { T r; if (__builtin_mul_overflow(a, b, &r)) return ((a < 0) != (b < 0)) ? (T)(MIN) : (T)(MAX); return r; } \
    NX_INLINE T nx_shl_##N(T a, uint32_t b, const char* loc) { if (b >= sizeof(T) * 8) nx_panic("shift amount exceeds the bit width", loc); return (T)((UT)a << b); } \
    NX_INLINE T nx_shr_##N(T a, uint32_t b, const char* loc) { if (b >= sizeof(T) * 8) nx_panic("shift amount exceeds the bit width", loc); return (T)(a >> b); } \
    NX_INLINE T nx_abs_##N(T a, const char* loc) { if ((T)(MIN) < 0 && a == (T)(MIN)) nx_panic("integer overflow in abs", loc); return a < 0 ? (T)(-a) : a; }

NX_INT_OPS(i8, int8_t, uint8_t, INT8_MIN, INT8_MAX)
NX_INT_OPS(i16, int16_t, uint16_t, INT16_MIN, INT16_MAX)
NX_INT_OPS(i32, int32_t, uint32_t, INT32_MIN, INT32_MAX)
NX_INT_OPS(i64, int64_t, uint64_t, INT64_MIN, INT64_MAX)
NX_INT_OPS(u8, uint8_t, uint8_t, 0, UINT8_MAX)
NX_INT_OPS(u16, uint16_t, uint16_t, 0, UINT16_MAX)
NX_INT_OPS(u32, uint32_t, uint32_t, 0, UINT32_MAX)
NX_INT_OPS(u64, uint64_t, uint64_t, 0, UINT64_MAX)
NX_INT_OPS(isize, intptr_t, uintptr_t, INTPTR_MIN, INTPTR_MAX)
NX_INT_OPS(usize, size_t, size_t, 0, SIZE_MAX)
NX_INT_OPS(i128, nx_i128, nx_u128, NX_I128_MIN, NX_I128_MAX)
NX_INT_OPS(u128, nx_u128, nx_u128, 0, (~(nx_u128)0))

/* Generated locals are named `<name>_<n>`. macOS's <mach/.../thread_status.h>
 * (reached through the system headers above) defines object-like macros of
 * that shape (`#define ts_32 uts.ts_32`), which would rewrite a local such as
 * `ts_32`; the generated code never needs them. */
#undef ts_32
#undef ts_64
#undef es_32
#undef es_64
#undef fs_32
#undef fs_64
#undef ds_32
#undef ds_64
#undef ns_32
#undef ns_64
#undef ss_32
#undef ss_64
#undef cs_32
#undef cs_64

#endif /* NX_RT_H */

/* ---------------------------------------------------- main on a big stack */
/* `artifact cli { stack = "1G" }`: the generated main runs the program on a
 * thread reserving that much stack, so a recursion deeper than the platform's
 * default (a megabyte on some Windows toolchains, eight on Linux and macOS)
 * gets the room it declared. The reservation is address space; pages are
 * committed as the program reaches them. When the thread cannot be created
 * the program runs on the default stack. A 32-bit process caps it at 256 MB. */
typedef struct nx_stack_call { void (*f)(void*); void* arg; } nx_stack_call;
#if defined(_WIN32)
#ifndef STACK_SIZE_PARAM_IS_A_RESERVATION
#define STACK_SIZE_PARAM_IS_A_RESERVATION 0x00010000
#endif
static DWORD WINAPI nx_stack_entry(LPVOID p) { nx_stack_call* c = (nx_stack_call*)p; c->f(c->arg); return 0; }
#else
static void* nx_stack_entry(void* p) { nx_stack_call* c = (nx_stack_call*)p; c->f(c->arg); return NULL; }
#endif
NX_INLINE void nx_run_on_stack(uint64_t bytes, void (*f)(void*), void* arg) {
    nx_stack_call c; c.f = f; c.arg = arg;
    if (sizeof(void*) < 8 && bytes > (uint64_t)256 * 1024 * 1024) bytes = (uint64_t)256 * 1024 * 1024;
#if defined(_WIN32)
    HANDLE h = CreateThread(NULL, (SIZE_T)bytes, nx_stack_entry, &c, STACK_SIZE_PARAM_IS_A_RESERVATION, NULL);
    if (h) { WaitForSingleObject(h, INFINITE); CloseHandle(h); return; }
#elif defined(NX_WASM)
    (void)nx_stack_entry;
#else
    pthread_attr_t attr; pthread_t t;
    if (pthread_attr_init(&attr) == 0) {
        bool ok = pthread_attr_setstacksize(&attr, (size_t)bytes) == 0 && pthread_create(&t, &attr, nx_stack_entry, &c) == 0;
        pthread_attr_destroy(&attr);
        if (ok) { pthread_join(t, NULL); return; }
    }
#endif
    f(arg);
}

/* ------------------------------------------- what an export call acquired */
/* S3 promises that a panic never crosses an export boundary; this is the
 * other half: a panic caught at the boundary releases everything the call
 * acquired, so a call that keeps failing does not grow. The wrapper of every
 * export installs a tracker for the call's context: its allocator records
 * every live allocation (arena chunks included, since arenas allocate from the
 * base allocator), and the file, socket and lock functions register their
 * handles through the thread's boundary. A `for parallel` body copies the
 * context to other threads, so the tables are behind a lock. On the panic
 * path the wrapper releases every entry; on the normal path only the tables
 * go, since the code released what it owned. Threads and parallel tasks that
 * panic on their own keep leaking what they allocated: what a thread allocates
 * can escape through `shared_mutable`, and freeing it would be worse.
 * Exports cannot return heap values or reach globals (S1, S2), so nothing
 * allocated during a panicked call is reachable afterwards. */
typedef struct nx_tracker {
    nx_alloc parent;
    void** slots; size_t cap; size_t used; size_t live;
    int64_t* files; size_t nfiles; size_t files_cap;
    int64_t* socks; size_t nsocks; size_t socks_cap;
    int64_t* locks; size_t nlocks; size_t locks_cap;
    int64_t mutex;
} nx_tracker;
#define NX_TR_DEAD ((void*)(uintptr_t)1)
static size_t nx_tr_hash(void* p) { uintptr_t x = (uintptr_t)p; x ^= x >> 17; x *= (uintptr_t)0x9E3779B97F4A7C15ULL; x ^= x >> 29; return (size_t)x; }
static void nx_tr_rebuild(nx_tracker* t, size_t ncap) {
    void** ns = (void**)calloc(ncap, sizeof(void*));
    if (!ns) return;
    for (size_t i = 0; i < t->cap; i++) {
        void* p = t->slots[i];
        if (!p || p == NX_TR_DEAD) continue;
        size_t j = nx_tr_hash(p) & (ncap - 1);
        while (ns[j]) j = (j + 1) & (ncap - 1);
        ns[j] = p;
    }
    free(t->slots);
    t->slots = ns; t->cap = ncap; t->used = t->live;
}
static void nx_tr_add(nx_tracker* t, void* p) {
    if (!p) return;
    nx_mutex_lock_raw(t->mutex);
    if ((t->used + 1) * 2 > t->cap) nx_tr_rebuild(t, t->cap == 0 ? 256 : (t->live * 4 > t->cap ? t->cap * 2 : t->cap));
    if (t->cap) {
        size_t mask = t->cap - 1, j = nx_tr_hash(p) & mask;
        while (t->slots[j] && t->slots[j] != NX_TR_DEAD) j = (j + 1) & mask;
        if (!t->slots[j]) t->used++;
        t->slots[j] = p; t->live++;
    }
    nx_mutex_unlock_raw(t->mutex);
}
static void nx_tr_remove(nx_tracker* t, void* p) {
    if (!p || !t->cap) return;
    nx_mutex_lock_raw(t->mutex);
    size_t mask = t->cap - 1, j = nx_tr_hash(p) & mask;
    while (t->slots[j]) {
        if (t->slots[j] == p) { t->slots[j] = NX_TR_DEAD; t->live--; break; }
        j = (j + 1) & mask;
    }
    nx_mutex_unlock_raw(t->mutex);
}
static void* nx_tr_alloc(void* st, size_t size, size_t align) { nx_tracker* t = (nx_tracker*)st; void* p = t->parent.alloc(t->parent.state, size, align); nx_tr_add(t, p); return p; }
static void* nx_tr_realloc(void* st, void* p, size_t old_size, size_t new_size, size_t align) { nx_tracker* t = (nx_tracker*)st; nx_tr_remove(t, p); void* q = t->parent.realloc(t->parent.state, p, old_size, new_size, align); nx_tr_add(t, q); return q; }
static void nx_tr_free(void* st, void* p, size_t size) { nx_tracker* t = (nx_tracker*)st; nx_tr_remove(t, p); t->parent.free(t->parent.state, p, size); }
static void nx_tr_list_set(int64_t** xs, size_t* n, size_t* cap, int64_t h, bool acquire) {
    if (acquire) {
        if (*n == *cap) { size_t nc = *cap ? *cap * 2 : 8; int64_t* g = (int64_t*)realloc(*xs, nc * sizeof(int64_t)); if (!g) return; *xs = g; *cap = nc; }
        (*xs)[(*n)++] = h;
    } else {
        for (size_t i = *n; i-- > 0;) { if ((*xs)[i] == h) { (*xs)[i] = (*xs)[*n - 1]; (*n)--; return; } }
    }
}
static void nx_track_handle(int kind, int64_t h, bool acquire) {
    nx_boundary* b = nx_tls_boundary;
    if (!b || !b->track) return;
    nx_tracker* t = b->track;
    nx_mutex_lock_raw(t->mutex);
    if (kind == 0) nx_tr_list_set(&t->files, &t->nfiles, &t->files_cap, h, acquire);
    else if (kind == 1) nx_tr_list_set(&t->socks, &t->nsocks, &t->socks_cap, h, acquire);
    else nx_tr_list_set(&t->locks, &t->nlocks, &t->locks_cap, h, acquire);
    nx_mutex_unlock_raw(t->mutex);
}
/* a thread started inside an export call may outlive the call, and what it
   allocates can escape through `shared_mutable`: it allocates untracked */
static void nx_ctx_untrack(nx_ctx* c) {
    if (c->alloc.alloc == nx_tr_alloc) c->alloc = ((nx_tracker*)c->alloc.state)->parent;
    if (c->base.alloc == nx_tr_alloc) c->base = ((nx_tracker*)c->base.state)->parent;
}
NX_INLINE void nx_export_enter(nx_ctx* c, nx_boundary* b, nx_tracker* t) {
    memset(t, 0, sizeof *t);
    t->parent = c->alloc;
    t->mutex = nx_mutex_new();
    c->alloc.alloc = nx_tr_alloc; c->alloc.realloc = nx_tr_realloc; c->alloc.free = nx_tr_free; c->alloc.state = t;
    c->base = c->alloc;
    b->track = t;
}
NX_INLINE void nx_export_leave(nx_ctx* c, nx_boundary* b, nx_tracker* t, bool panicked) {
    b->track = NULL; /* the releases below must not register themselves */
    if (panicked) {
        for (size_t i = t->nlocks; i-- > 0;) nx_mutex_unlock_raw(t->locks[i]);
        for (size_t i = 0; i < t->nsocks; i++) nx_closesock((nx_sock)t->socks[i]);
        for (size_t i = 0; i < t->nfiles; i++) nx_file_close(t->files[i]);
        for (size_t i = 0; i < t->cap; i++) { void* p = t->slots[i]; if (p && p != NX_TR_DEAD) t->parent.free(t->parent.state, p, 0); }
    }
    free(t->slots); free(t->files); free(t->socks); free(t->locks);
    nx_mutex_free(t->mutex);
    c->alloc = t->parent;
    c->base = t->parent;
}
