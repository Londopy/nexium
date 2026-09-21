# Threads and parallel loops

Four tools, from the one that needs no thinking to the one that needs the
most: `for parallel` for data that splits into independent pieces, threads
with a result, channels between threads, and a mutex around shared state.
Plus `using arena`, which is about memory rather than threads but keeps
them company in the fast paths.

{{include topo/code/concurrency.nx}}

{{output topo/code/concurrency.expected}}

## `for parallel`

```nexium
    for parallel _slot, i in counts[..] {
        ...
        out[i] = found
    }
```

A `for parallel` loop runs its body across a pool of worker threads, one
chunk of the index range each. The rules that make this safe are checked by
the compiler: the body may not have the `shared_mutable` effect (no
globals, no locks), may not `return` or `break`, and writes its results
through a mutable slice indexed by `i`, so that every iteration touches its
own slot and nothing else. A panic in a worker is re-raised in the caller
once all workers finish. Counting primes in sixteen ranges is the shape of
most of these loops: split, compute, write to slot `i`, sum at the end.

The loop itself carries the `blocks` effect (it joins its workers), and
that is all the caller sees.

## Threads with results

```nexium
    var a = thread.spawn(Range, u64, sum_range, Range{ .from = 0, .to = 500000 })
    println("sum {}", .{a.join() + b.join()})
```

`std.thread.spawn(T, R, f, arg)` runs `f(&mut arg)` on a new thread; the
`Thread(T, R)` owns the argument until `join` returns the result. Types come
first because the function is generic; the argument is a value you give
away (`own`), so the thread's data is its own and nothing is shared by
accident. A panic inside the thread surfaces from `join`.

`thread.run` is the same for functions without a result (a `Worker(T)`),
and `t.arg()` after `join` reads what the function left in its argument.

## Channels

```nexium
    var ch = thread.channel(String)
    var producer = thread.run(Producer, produce, Producer{ .out = &mut ch, .n = 5 })
    while true {
        let msg = ch.recv() orelse break
```

A `Channel(T)` is a queue with a lock inside. `send` moves a value in;
`recv` waits for one and returns `null` once the channel is closed and
drained, which is the loop's exit; `try_recv` does not wait. Channels are
values that threads share by pointer, so the owner joins every thread that
uses one before it goes out of scope, and calls `free` when done. The
producer here sends five `String`s and closes; the ownership of each
message moves from the producer to the consumer through the channel, so
neither side clones anything.

## Mutexes

```nexium
    let n = counter.lock()
    n.* += 1
    counter.unlock()
```

`Mutex(T)` wraps a value; `lock` returns a `*mut T` that is valid until
`unlock`. Locking has the `blocks` and `shared_mutable` effects, which is
why a `for parallel` body cannot do it, and why the thread pool and the
mutex are two different tools: the loop for when the work splits cleanly,
the mutex for when it genuinely does not.

## Arenas

```nexium
    using arena {
        for i in 0..1000 {
            let s = format("item number {}", .{i})
            lengths.append(s.len)
        }
    }
```

`using arena { }` swaps a bump allocator in for the block: every value
created inside comes from one growing region, individual releases are
no-ops, and the whole region is freed at once when the block ends. A
thousand temporary strings cost one allocation. Containers created *outside*
the block (`lengths`) keep using the heap when they grow inside it, so
collecting results into an outer list is safe; values created inside must
not be kept past the end, and since 1.2 the compiler warns when one is
(rule V5: assigned outward, appended to an outer container, returned).
`@escape(s)` is the way out when a value has to leave: a copy made by the
allocator outside the block. It is the allocation strategy for a parse, a
frame, a request: work that has a clear end.

## Threads and the rest of the language

Data races are not memory-safety violations in Nexium's promise (the
specification says so, in section 13), and the type system does not prevent
them: it makes them visible. A function that touches shared state has
`shared_mutable`; one that starts a thread has `nondeterministic`; a `for
parallel` body cannot have the first at all. The roadmap's 1.4 adds scoped
threads (joined when their block ends, so a thread cannot outlive the data
it was given) and `select` over channels.

Next: [a network service](17-networking.html).
