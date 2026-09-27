/* setjmp and longjmp on WebAssembly: wasi-libc's runtime for them, which
 * zig 0.14 leaves out. `-mllvm -wasm-enable-sjlj` lowers the runtime's
 * setjmp and longjmp to these three calls: longjmp throws WebAssembly's
 * C_LONGJMP exception, and the frame that called setjmp knows its own by the
 * invocation it was made in. nx compiles this file beside a program built
 * for wasm32-wasi. It is a translation unit of its own on purpose: defined
 * in the one that calls setjmp, LLVM 19 lowers the calls to code that no
 * engine accepts. Weak, so a libc that has them wins.
 */
#include <stdint.h>
#include <stddef.h>

struct nx_wasm_jb { void* invocation; uint32_t label; struct { void* env; int val; } arg; };

__attribute__((weak)) void __wasm_setjmp(void* env, uint32_t label, void* invocation) {
    struct nx_wasm_jb* jb = (struct nx_wasm_jb*)env;
    if (label == 0 || invocation == NULL) __builtin_trap();
    jb->invocation = invocation;
    jb->label = label;
}

__attribute__((weak)) uint32_t __wasm_setjmp_test(void* env, void* invocation) {
    struct nx_wasm_jb* jb = (struct nx_wasm_jb*)env;
    if (jb->label == 0 || invocation == NULL) __builtin_trap();
    return jb->invocation == invocation ? jb->label : 0;
}

__attribute__((weak)) void __wasm_longjmp(void* env, int val) {
    struct nx_wasm_jb* jb = (struct nx_wasm_jb*)env;
    jb->arg.env = env;
    jb->arg.val = val == 0 ? 1 : val;
    __builtin_wasm_throw(1, &jb->arg);
}
