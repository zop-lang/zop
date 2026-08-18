# Concurrency and parallelism

Zop combines lightweight tasks and channels with compile-time ownership and
structured lifetimes. Concurrent code should be as direct as Go while retaining
the data-race resistance and low-level control expected from a systems language.

> **Status:** This page defines target semantics. Task, channel, selection, and
> scheduler APIs are not implemented. Example spelling is provisional.

## Principles

- Child tasks belong to a lexical scope by default.
- Ownership transfer and shared access are checked at compile time.
- Communication is preferred, but locks, atomics, and threads remain available.
- Cancellation, failure, completion, and resource limits are explicit.
- Concurrency does not require a second `async fn` function kind.
- Concurrent execution and parallel execution remain distinct concepts.

## Task scopes

A `concurrent` block owns every task spawned inside it:

```zop
fn fetch_both io: Io, left_url: Url, right_url: Url
    -> (Response, Response) or fails with FetchGroupError
    concurrent io
        left = spawn fetch io, left_url
        right = spawn fetch io, right_url
        return try to await left, right
```

`FetchGroupError` is the function's domain error and can contain failures from
more than one child. The block lowers to an owned task group; the standard
library exposes that group directly for dynamic task collections.

The group cannot leave scope while a child remains live. Each child completes,
is canceled and joined, or transfers to an explicit longer-lived owner. A task
handle is an owned `Task[T, E]`; dropping an unresolved handle is a compile
error.

If one child fails, the group requests cancellation of its siblings, waits for
their cleanup, and returns every observed failure in stable spawn order. No
child failure is discarded. Cancellation is cooperative, may race with normal
completion, and has a typed result.

Detached work requires a process-owned supervisor with a documented error sink
and shutdown policy. `spawn` outside a task scope is illegal; there is no bare
fire-and-forget operation.

## Ownership across tasks

The compiler derives two structural properties:

- A **sendable** value may transfer ownership to another task or thread.
- A **shareable** value may be borrowed concurrently by multiple tasks.

These are compiler facts, not annotations users repeat on ordinary types. A
record is sendable or shareable when all of its fields are. Host handles, raw
pointers, thread-affine values, and non-atomic shared owners may prevent either
property.

Sending an owned value through a channel consumes the sender's binding. A
borrow may cross into a child only when the task scope cannot outlive the
borrow. A `mut` borrow remains exclusive across all tasks. Shared mutation
requires a channel, lock, atomic, or another type whose synchronization
contract makes the access legal.

Unsafe code may implement new synchronization primitives, but it must prove the
same sendable and shareable invariants as the standard library.

## Channels

Channels are typed values with separate sender and receiver endpoints.
Rendezvous channels have no buffer. Bounded channels declare a positive
capacity and apply backpressure when full.

The standard library does not provide an implicitly unbounded channel. A
package may implement one by requiring `Mem` and exposing its memory-growth and
failure policy.

Closing is directional. The final sender closes the value stream; receivers
drain buffered values and then observe closure. Dropping a receiver wakes
blocked senders with a typed failure. Values already transferred remain owned
by the channel until received or destroyed.

## Selection

Selection waits on channel operations, task completion, timers, and
cancellation in one expression. If several operations are ready, the runtime
uses documented fair rotation rather than permanent source-order priority.

A non-blocking branch is explicit. Repeating a non-blocking selection without
progress is diagnosed in debug builds and remains visible to performance tools.
The deterministic test scheduler controls readiness and selection order.

## Scheduling and suspension

`Io` supplies the scheduler. The selected implementation may use operating
system threads, an event loop, immediate deterministic completion, or a browser
event loop. There is no ambient executor.

Functions retain one `fn` spelling. The frontend records suspension as an
effect in high-level intermediate representation (HIR), so the compiler still
knows which calls may suspend. A blocking operation submitted to an evented
scheduler must use an explicit blocking boundary; it cannot stall an executor
thread silently.

Scheduling policy is not program semantics. Work stealing, queue layout,
stackless-frame representation, and persistent workers may change only when
the same conformance suite and cancellation behavior still pass.

## Threads, locks, and atomics

`std.sync` exposes operating-system threads, mutexes, read-write locks,
condition variables, semaphores, and atomics. Threads are owned resources and
must be joined or transferred to a supervisor.

Lock guards are scoped values. The compiler rejects a guard held across a
suspension point unless the lock type explicitly supports that operation.
Atomic operations name their memory ordering; convenience operations choose a
documented safe ordering rather than an ambient global default.

## Parallel computation

Tasks express independent work and latency hiding. Parallel loops, reductions,
tensor operations, and `kn` kernels express data-parallel computation. A pure
parallel operation may be partitioned and reordered only when its reduction and
floating-point contracts permit it.

The compiler may map parallel work to vector instructions, a central processing
unit (CPU) pool, or a graphics processing unit (GPU) target. That mapping is an
explicit target decision, not a semantic fallback after another backend fails.

## Browser boundary

The browser document object model (DOM) is main-thread-affine and not sendable.
Browser workers may receive sendable values and shared linear memory, but they
cannot receive a DOM capability. The [web target contract](web.md) defines this
boundary.

## Required tests

- Reject a task scope that exits with a live child or unresolved handle.
- Cancel and join sibling tasks after a child failure.
- Preserve every simultaneous child failure in stable spawn order.
- Reject non-sendable transfers and non-shareable concurrent borrows.
- Prove an owned channel send invalidates the sender's binding.
- Apply bounded-channel backpressure without allocating hidden storage.
- Wake blocked operations on closure, cancellation, and peer destruction.
- Prove fair selection and deterministic test-scheduler behavior.
- Reject suspension while holding an incompatible lock guard.
- Run the task suite on threaded, evented, deterministic, and browser runtimes.
- Race-test channels, task groups, cancellation, locks, and atomics.
- Verify optimized task hot paths and context-switch costs with benchmarks.

## References

- [Effective Go concurrency](https://go.dev/doc/effective_go#concurrency)
- [Rust `Send` and `Sync`](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)
- [Kotlin structured concurrency](https://kotlinlang.org/docs/coroutines-basics.html#coroutine-scope-and-structured-concurrency)
- [WebAssembly threads](https://github.com/WebAssembly/threads)
- [WebAssembly shared-everything threads](https://github.com/WebAssembly/shared-everything-threads)
