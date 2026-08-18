# Testing

Zop is correct when every execution path agrees on language behavior.
Passing one backend or one happy-path program is not enough.

## Current suite

The bootstrap tests lexer layout and rejection rules, parser structure and
error recovery, scalar type checking, verified Multi-Level Intermediate
Representation (MLIR) emission, contextual numeric literals, direct calls,
local assignments, typed just-in-time (JIT) invocation, and native object
emission. It also covers deterministic ECMAScript emission, exact `i32`
representation, compact floating-point literals, constant folding,
effect-preserving elimination, malformed high-level intermediate representation
(HIR), and JavaScript target rejection. One test executes generated machine
code and proves `20 + 22 == 42`. The suite also proves that a `kn` kernel cannot
fall back to the central processing unit (CPU) or JavaScript backend.

The web conformance suite parses the machine-readable
[browser profile](../standards/web/README.md), rejects unsupported claims, bans
runtime source evaluation, and independently parses and validates the reference
WebGPU Shading Language fixture with Naga.

The semantic oracle, tensor corpus, ownership checker, effect system, and
graphics processing unit (GPU) execution suite below are release gates. They
are not implemented yet.

## Semantic oracle

A planned small interpreter will define the behavior of restricted MLIR
independently of Cranelift. Every supported CPU operation has one interpreter
case, one JIT case, and one ahead-of-time (AOT) case. Tests compare values,
output, and failure class.

The interpreter never runs because native compilation failed. It exists only
for tests and debugging.

## Language contract

Zop uses a built-in test declaration with an ordinary expression body:

```zop
test "binary search returns the first equal position"
    values = [1, 4, 4, 9]
    expect equal(binary_search(values, 4), 1)
```

The compiler owns discovery, the test name, source span, enclosing module, and
whether the declaration enters a test build. Assertions are library calls. This
keeps custom comparisons extensible without requiring Rust-style attributes or
Go-style naming conventions to identify tests.

A test has an implicit success result of `Unit` and may fail with `TestFailure`.
Its body follows ordinary ownership, effect, error, and task rules. Tests never
enter a production artifact or change production type checking.

Test declarations beside source may access private declarations in that module.
Files below a package's `tests/` target compile as external clients and can use
only its public application programming interface (API). This preserves the
useful white-box and black-box split shared by Rust and Go without requiring a
second package name.

> **Status:** Test declarations, the test manifest, and the Zop runner are not
> implemented. The Rust bootstrap suite remains the compiler's current harness.

## Capabilities and isolation

Pure tests declare no parameters. A test that needs files, time, entropy,
processes, or networking requests a `Test` capability:

```zop
test "flushes before close" t: Test
    file = try to t.temp.file()
    try to write_record t.io, file, "ready"
    expect equal(try to file.read_all(t.io), "ready")
```

`Test` owns a temporary directory, deterministic random seed, virtual clock,
captured input/output, cancellation scope, and only the host capabilities the
test declares. The runner reports the seed for every failure. Tests cannot
silently share a current directory, process environment, port, or global random
generator.

The default runner executes independent tests in parallel in one process.
Tests requiring traps, unsafe corruption checks, exclusive hardware, or process
state select process isolation through standard test metadata. Serial execution
is explicit metadata, never inferred from timing or a hidden lock.

## Discovery and runner protocol

`zop test` asks the compiler for a versioned test manifest. Every entry contains
the stable test identifier, display name, package and module, source span,
target, tags, required capabilities, isolation, timeout, and executable symbol.

The bundled runner consumes the same public protocol as third-party runners.
The compiler does not parse runner output, and alternate runners do not scrape
compiler internals. `zop test --list --format json-v1` exposes the selected
manifest for editors, build systems, and remote schedulers.

The default runner provides:

- package, path, name, tag, and target selection;
- deterministic sharding and parallel execution;
- per-test timeout, cancellation, and resource cleanup;
- captured output shown only on failure;
- terse human output and a versioned machine event stream; and
- exact reproduction by test identifier, target, and seed.

The normal success path is quiet. Failures name the violated expectation,
source span, values, captured effects, seed, and cleanup errors.

## Library boundary

<!-- markdownlint-disable MD013 -->

| Owner | Responsibility |
| --- | --- |
| Language | `test` declarations and their ordinary body semantics |
| Compiler | Discovery, manifest emission, test-only code elimination, coverage and fuzz instrumentation |
| `core.test` | `expect`, equality, failure, source location, and allocation-free assertions |
| `std.test` | `Test`, temporary resources, captured input/output, clocks, entropy, subprocesses, and process isolation |
| Toolchain runner | Selection, scheduling, sharding, timeouts, reporting, replay, and protocol compatibility |
| Official packages | Property generators, snapshots, rich matchers, fixtures, and domain integrations |
| Community | Alternate runners, reporters, orchestration, and specialized testing libraries |

<!-- markdownlint-enable MD013 -->

Mocking, snapshot formats, browser fixtures, database fixtures, and large
matcher libraries do not enter `std`. They evolve too quickly and do not define
language interoperability.

## Fuzzing, properties, and benchmarks

Unit tests, fuzz targets, property tests, and benchmarks share discovery and
reporting, but they do not share measurement semantics.

`zop fuzz` is a first-party coverage-guided tool. Seed inputs always run under
`zop test`, and a minimized failure becomes a durable regression corpus entry.
The exact source declaration remains open until parameterized tests and the
serialization contract are designed.

`zop bench` owns warmup, iteration control, allocation counters, compiler
barriers, samples, and machine metadata. Benchmark bodies use ordinary library
setup and teardown. A benchmark never runs accidentally under `zop test`.

Property testing is an official package over ordinary parameterized tests. The
runner records generator seed and shrunk input but does not standardize one
generator algebra in the language.

## Rationale

Rust proves that compiler discovery plus colocated and external tests works,
but its attribute syntax and libtest coupling make alternate runners integrate
around a harness implementation. Go proves the value of one first-party command
covering tests, examples, benchmarks, coverage, and fuzzing, but naming
conventions are weaker than declarations. Zig supplies the clearest source
form: a test is visibly a test and disappears from ordinary builds. Swift
Testing demonstrates scalable tags, traits, parameterization, and parallel
execution as library metadata.

Zop takes the smallest stable composition: one language declaration, one
toolchain command, small assertion and capability libraries, and one public
runner protocol. More built-in syntax would freeze framework policy. Less
compiler involvement would fragment discovery, tooling, and test-only code
elimination.

## Test layers

<!-- markdownlint-disable MD013 -->

| Layer | Required proof |
| --- | --- |
| Lexer and parser | Layout, delimiters, precedence, spans, and errors |
| Type and ownership checker | Valid programs plus exact rejection spans |
| Compile-time values | Binding-time checks, purity, caching, and ABI erasure |
| Callables | Members, calls, captures, lifetimes, and dispatch |
| Errors | Typed channels, mandatory handling, propagation, and recovery |
| Typed frontend intermediate representation to MLIR | Verified MLIR and canonical golden output |
| MLIR lowering | No high-level operation survives the base boundary |
| MLIR to Cranelift intermediate representation (CLIF) | Verified CLIF and differential interpreter results |
| Native interface | 0-8 arguments, recursion, callbacks, and alignment |
| Input and output | Deterministic effects, flush, close, and cancellation |
| Concurrency | Task lifetimes, transfer safety, channels, scheduling, and races |
| Runtime | `Mem`, ownership, bounds, uninitialized reads, and faults |
| Autodiff | Activity, gradients, checkpointing, and numerical agreement |
| Graphics processor | Structural lowering plus execution on real hardware |
| Browser | JavaScript and WebAssembly validation, DOM behavior, interop, workers, and real-browser execution |
| Artifacts and cache | Identity, atomicity, restoration, corruption, and garbage collection |

<!-- markdownlint-enable MD013 -->

Correctness gates quality tests. Runtime speed, compile latency, memory use,
code size, and register pressure are measured only after the program passes the
semantic suite. The [performance contract](performance.md) defines reference
floors, structural operation counts, browser traces, benchmark hygiene, and
regression budgets.

## Conformance corpus

Small tests isolate one rule. Larger programs prove the rules compose. The
corpus grows from arithmetic and control flow into parsers, graph algorithms,
renderers, systems utilities, tensor libraries, and model workloads.

Every fixed bug adds the smallest program that expresses the violated language
invariant. Test names describe the invariant, not the historical failure.

## MLIR boundaries

Each MLIR layer has its own verifier tool. The CLIF-ready verifier does not
register high-level Zop dialects, so leaked operations fail at parse or
verification time. End-to-end tests remain separate from these structural
tests because a plausible IR dump does not prove executable behavior.

## References

- [Rust test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust test harness](https://doc.rust-lang.org/nightly/rustc/tests/index.html)
- [Go `testing`](https://pkg.go.dev/testing)
- [Go fuzzing](https://go.dev/doc/security/fuzz/)
- [Zig test declarations](https://ziglang.org/documentation/master/#Zig-Test)
- [Swift Testing](https://developer.apple.com/documentation/Testing)
