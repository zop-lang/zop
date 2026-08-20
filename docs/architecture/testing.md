# Testing

Zop is correct when every execution path agrees on language behavior.
Passing one backend or one happy-path program is not enough.

## Current suite

The bootstrap tests lexer layout and rejection rules, parser structure and
error recovery, scalar type checking, verified Multi-Level Intermediate
Representation (MLIR) emission, contextual numeric literals, direct calls,
forward signature resolution, direct and mutual recursive signature cycles,
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

## Table-driven tests

Go-like table tests use ordinary Zop data and loops rather than a second
parameterization grammar:

```zop
type DivisionCase
    name: str
    left: i64
    right: i64
    expected: i64

test "floor division table" t: Test
    cases = [
        DivisionCase name="positive", left=7, right=3, expected=2,
        DivisionCase name="negative dividend", left=-7, right=3, expected=-3,
        DivisionCase name="negative divisor", left=7, right=-3, expected=-3,
    ]

    for case in cases
        t.run case.name
            expect equal(case.left // case.right, case.expected)
```

`t.run` runs one named subcase inline and adds its name to failure output and
the stable test identifier. It does not hide a task, copy the table, or infer
parallel execution. A subcase that needs concurrency requests it explicitly
through the ordinary task contract.

A small unnamed table can destructure tuples directly:

```zop
test "absolute value table"
    cases = [(-2, 2), (0, 0), (3, 3)]

    for input, expected in cases
        expect equal(abs(input), expected)
```

The formatter keeps short cases on one line and applies the ordinary multiline
literal rules to larger cases. Table construction, ownership, inference, and
errors are normal language behavior, so editor and compiler tooling need no
test-only expression grammar.

## Documentation tests

Every fenced Zop example in structured `##` documentation or a configured
narrative chapter becomes an ordinary discovered test. `zop test` includes
them; `zop test --doc` selects only documentation examples.

The compiler supplies the example's package, public API, target, source span,
and stable symbol identity. It does not inject hidden imports, capabilities,
error handling, or setup lines. An example that cannot compile and execute
under its documented context fails the same release gate as a source test.

The [documentation contract](documentation.md#executable-examples) owns source
syntax and attachment. The test runner owns isolation, scheduling, execution,
and reporting. Generated documentation may display a result only after the
corresponding example passes for the published target profile.

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
| `std.test` | `Test`, named subcases, temporary resources, captured input/output, clocks, entropy, subprocesses, and process isolation |
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

`zop fuzz` is a first-party coverage-guided tool. A fuzz declaration names its
generated input types explicitly:

```zop
fuzz "parser never panics" input: bytes
    parse input
```

Seed inputs always run under `zop test`, and a minimized failure becomes a
durable regression corpus entry. The input types must have a canonical
serialization contract before they are legal fuzz inputs.

`zop bench` owns warmup, iteration control, allocation counters, compiler
barriers, samples, and machine metadata. Benchmark bodies use ordinary library
setup and teardown:

```zop
bench "dense matrix multiplication" b: Bench
    left = fixture_left()
    right = fixture_right()

    b.measure
        left @ right
```

A benchmark never runs accidentally under `zop test`.

Property testing is an official package over ordinary parameterized tests. The
runner records generator seed and shrunk input but does not standardize one
generator algebra in the language.

## Rationale

Rust proves that compiler discovery plus colocated and external tests works,
but its attribute syntax and libtest coupling make alternate runners integrate
around a harness implementation. Go proves the value of one first-party command
covering tests, examples, benchmarks, coverage, and fuzzing. Go's table-driven
tests also prove that ordinary data plus a loop scales better than a dedicated
parameterization expression; named `t.Run` subtests keep failures readable.
Go naming conventions are weaker than declarations. Zig supplies the clearest
source form: a test is visibly a test and disappears from ordinary builds.
Swift Testing demonstrates scalable tags, traits, parameterization, and
parallel execution as library metadata.

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
| Diagnostics | Stable codes, source labels, deterministic rendering, and machine output |
| Documentation | Attachment, tags, symbol links, coverage, examples, semantic model, and deterministic rendering |
| Formatter | Syntax preservation, idempotence, range stability, and command/editor parity |
| Language server | LSP transcripts, semantic tokens, navigation, rename, cancellation, invalidation, and latency |
| Type and ownership checker | Valid programs plus exact rejection spans |
| Compile-time values | Binding-time checks, purity, caching, and ABI erasure |
| Numeric semantics | Types, quotient modes, traps, conversions, precision profiles, and target parity |
| Tensor layouts | Engine, affine and composed Layout profiles, algebra laws, views, dynamic leaves, ABI, and visualization |
| Tensor indexing | Normalization, bounds, slices, ranks, origins, descriptors, zero-copy proof, and target parity |
| SIMD | Scalar equivalence, legality, reason codes, tails, MLIR, CLIF, target instructions, and crossover |
| Callables | Members, calls, captures, lifetimes, and dispatch |
| Errors | Typed channels, mandatory handling, propagation, and recovery |
| Typed frontend intermediate representation to MLIR | Verified MLIR and canonical golden output |
| MLIR lowering | No high-level operation survives the base boundary |
| MLIR to Cranelift intermediate representation (CLIF) | Verified CLIF and differential interpreter results |
| Native interface | 0-8 arguments, recursion, callbacks, and alignment |
| Input and output | Deterministic effects, flush, close, and cancellation |
| Concurrency | Task lifetimes, transfer safety, channels, scheduling, and races |
| Runtime | `Mem`, ownership, bounds, uninitialized reads, and faults |
| Graphics processor | Structural lowering plus execution on real hardware |
| Browser | JavaScript and WebAssembly validation, DOM behavior, interop, workers, and real-browser execution |
| Artifacts and cache | Identity, atomicity, restoration, corruption, and garbage collection |

<!-- markdownlint-enable MD013 -->

Correctness gates quality tests. Runtime speed, compile latency, memory use,
code size, and register pressure are measured only after the program passes the
semantic suite. The [performance contract](performance.md) defines reference
floors, structural operation counts, browser traces, benchmark hygiene, and
regression budgets.

## Compiler phase isolation

Each compiler phase has a narrow test entry point:

- lexer and parser tests require no standard library;
- name and signature resolution tests use a minimal checked package stub;
- ownership, effects, and other semantic passes consume typed HIR fixtures;
- each MLIR transformation consumes and verifies the smallest legal input
  dialect;
- elaboration tests stop after producing concrete verified HIR; and
- integration tests alone invoke complete code generation and execution.

Every rejection test states its expected diagnostic and source span. The test
fails on any missing or unexpected diagnostic. Structural IR checks use parsed
operations and types when available; textual goldens cover stable human output
rather than becoming the only semantic assertion.

Mojo 1.0 validates the practical value of a tiny parser test package, separate
parser, elaboration, transformation, and integration suites, and direct
single-pass tools. Zop adopts that isolation without making LLVM FileCheck text
matching its only assertion mechanism.

## Conformance corpus

Small tests isolate one rule. Larger programs prove the rules compose. The
corpus grows from arithmetic and control flow into parsers, graph algorithms,
renderers, systems utilities, tensor libraries, and model workloads.

The tensor-layout corpus triangulates PyCuTe, official CUTLASS CuTe behavior,
and the independent MIT-licensed
[`tensor-layouts`](https://github.com/jduprat/tensor-layouts/tree/d9f51a435c02eb600a05f72508e681bd33dadee9).
Reviewed fixtures remain checked-in data, not Python build dependencies.
Agreement between two Python implementations cannot override CUTLASS CuTe.

The [worked layout examples](layout-examples.md) record the first readable
surface of that corpus.

The corpus distinguishes iterable `zip` from CuTe `zipped_divide`. Iterable
tests cover shortest exhaustion, `strict=true`, static mismatch diagnostics,
dynamic mismatch traps, and left-to-right iterator evaluation. Layout tests
compare the pinned PyCuTe zipped tile and remainder profiles coordinate by
coordinate.

Layout evaluation and tensor access remain separate test layers. A layout may
evaluate an extended coordinate as pure algebra; a tensor must reject any
coordinate outside its logical shape or backing storage.

The [indexing corpus](indexing.md#required-tests) proves positive and negative
indices, omitted endpoints, positive and negative steps, empty views, rank
reduction, recoverable bounds, view origins, mutable aliasing, descriptor
erasure, zero element copies, clipped brackets, and named strict failure. Every
backend executes the same endpoint matrix rather than inheriting a helper's
local convention.

Broadcasting tests follow the [core trailing-axis contract](layouts.md#broadcasting-and-expansion).
They cover rank-zero scalars, missing leading axes, singleton expansion, zero
extents, symbolic equality, incompatible extents, in-place destination shape,
and zero-stride mutation rejection. Framework wrappers must produce the same
HIR as the core axis APIs.

Tensor-formatting golden tests cover dense values, strided views, broadcasts,
large-value summarization, floating-point stability, explicit per-call limits,
and write failures. Test snapshots consume the same formatter as `std.io.print`
so testing cannot invent a second representation.

The [numeric corpus](numerics.md#required-tests) exercises every integer width,
quotient mode, conversion boundary, floating-point profile, and special value.
It compares development and optimized builds plus interpreter, Cranelift,
JavaScript, WebAssembly, and device results. Ordinary operators trap on the same
integer inputs everywhere; fallible members return the same typed errors; and
explicit wrapping and saturating members return the same values.

Nonfinite tests distinguish IEEE values from failures. They cover NaN,
infinity, signed zero, subnormals, scalar and tensor classification,
flow-sensitive invalidation after `require_finite`, deterministic first-invalid
coordinates, and opt-in `--check-nonfinite` instrumentation. The ordinary suite
never enables instrumentation implicitly.

Scalar and tensor `min` and `max` tests cover one or several arguments,
contextual literals, rejected mixed named types, NaN propagation, signed zero,
infinities, every integer boundary, broadcasting, empty reductions, explicit
`initial`, and CPU/device parity.

Syntax tests accept scalar compound updates, including `/=`, `//=`, and `%=`;
reject `++`, `--`, and `=-` as operators; and prove `//` cannot begin a
comment. Update targets are evaluated once and remain unchanged when scalar
arithmetic traps before the write. Trapping tensor updates run in an isolated
execution domain and may write partial storage before the trap. The test proves
that no safe operation can observe that storage afterward; it does not demand
rollback.

String tests cover one-line escapes, interpreted triple quotes, raw triple
quotes, empty and blank lines, closing-delimiter indentation, mixed indentation,
embedded comment markers, invalid escapes, and unterminated delimiters. A
standalone string before a declaration never becomes documentation.

Device-fault tests run in a dedicated context on real hardware. A deliberately
trapping `kn` must fail its completion, publish no output, invalidate every
allocation in the context, reject later storage access with `DeviceLost`, and
permit recovery only through a newly created context and explicit upload. A
failed test context is never reused by another test.

Every fixed bug adds the smallest program that expresses the violated language
invariant. Test names describe the invariant, not the historical failure.

Table-test conformance proves product and tuple tables use ordinary inference,
ownership, iteration, and destructuring. Named `t.run` scopes produce stable
subtest identifiers, preserve source order, and attribute a failure to the
exact row without evaluating another row twice. Fuzz and benchmark declarations
enter their own runner modes and never execute under an ordinary production
build.

Proper-tail-call tests recurse beyond the platform stack limit through
self-recursive, mutually recursive, direct, and stored-callable paths. They
exercise success and error results, local destruction before transfer, debug
and optimized builds, the reference interpreter, and every supported CPU `fn`
target. A nearly identical non-tail recursion test must still exhibit ordinary
stack growth so the verifier cannot label every recursive call as tail.

SIMD conformance follows the [vectorization test matrix](simd.md#required-tests).
The scalar reference, MLIR interpreter, JIT, and AOT paths compare values and
failure classes at every tail length, alignment, first-match lane, and supported
Layout. Structural tests consume stable vectorization reason codes, verified
MLIR, and CLIF rather than inferring success from one benchmark. Pinned x86-64
and AArch64 assembly checks remain the final target proof.

The corpus keeps the ownership boundary executable. One shaped tensor fixture
must reach MLIR Vector through upstream structured vectorization. One ordered
`find_first` fixture must reach the same dialect through Zop's semantic pattern.
A required semantic schedule fails if scalar CLIF reaches Cranelift, even when
the result remains correct. Upstream pass changes may replace an implementation
only when report, IR, assembly, and performance gates remain unchanged.

Documentation and editor conformance follow the [tooling test
contract](tooling.md#required-tests). Golden tests compare command-line and
editor diagnostics, semantic token roles, documentation hover, canonical
formatting, and semantic rename. Protocol tests replay versioned Language
Server Protocol transcripts, including cancellation and stale-document races,
without requiring a graphical editor.

## MLIR boundaries

Each MLIR layer has its own verifier tool. The CLIF-ready verifier does not
register high-level Zop dialects, so leaked operations fail at parse or
verification time. End-to-end tests remain separate from these structural
tests because a plausible IR dump does not prove executable behavior.

## References

- [Rust test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust test harness](https://doc.rust-lang.org/nightly/rustc/tests/index.html)
- [Go `testing`](https://pkg.go.dev/testing)
- [Go table-driven tests](https://go.dev/wiki/TableDrivenTests)
- [Go fuzzing](https://go.dev/doc/security/fuzz/)
- [Zig test declarations](https://ziglang.org/documentation/master/#Zig-Test)
- [Swift Testing](https://developer.apple.com/documentation/Testing)
- [Rust documentation tests](https://doc.rust-lang.org/rustdoc/documentation-tests.html)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [MLIR Vector dialect](https://mlir.llvm.org/docs/Dialects/Vector/)
- [Mojo compiler test organization](https://github.com/modular/modular/blob/f66d4d522c34be0a961ffac3dbfc81e30f67942e/KGEN/docs/testing.md)
