# Web and browser target

Zop aims to let applications and frontend frameworks be written entirely in
Zop. The browser target emits an optimized ECMAScript module for host and
document object model (DOM) work. It adds WebAssembly compute islands or WebGPU
kernels only when the program needs them. Generated JavaScript is an artifact,
not application source.

> **Status:** This page is an aspirational target contract. A restricted scalar
> ECMAScript emitter exists. Browser IR, Web APIs, DOM lowering, source maps,
> WebAssembly compute islands, WebGPU kernels, and package output are not
> implemented.

The [web standards profile](../standards/web/README.md) is the compatibility
ledger for this architecture. It pins normative sources and upstream suites,
and a Rust test rejects unsupported `supported` claims.

## Platform reality

Browsers expose the DOM and other Web application programming interfaces
(APIs) through JavaScript host objects. WebAssembly core is a portable
instruction set with no ambient host access. It cannot manipulate the DOM
directly and must call an imported host function.

The WebAssembly Component Model can describe typed resources, strings, futures,
streams, and ECMAScript module integration. Direct Web Interface Definition
Language (Web IDL) bindings remain an evolving browser proposal. Zop does
not assume that future path is available today.

The baseline target therefore emits JavaScript directly for host work. A
WebAssembly island uses a generated typed bridge. A future Component Model
profile is a separate declared target. The compiler never silently switches
between these interfaces.

## Output

A browser application exported with `--out dist` produces a complete deployment
unit:

```text
dist/
├── app.mjs
├── app.d.ts
├── app.mjs.map
├── app.wasm      # only when a compute island exists
├── app.wasm.map  # only when app.wasm exists
└── kernels/      # only when WebGPU kernels exist
    └── <hash>.wgsl
```

The ECMAScript module contains the application and its direct Web API calls. If
a WebAssembly island exists, the module streams, instantiates, and wires it. The
package includes only operations and exports the application uses. It performs
no evaluation of generated source at runtime.

WebGPU kernels are emitted as content-hashed WebGPU Shading Language (WGSL)
modules because browsers accept WGSL source when creating shader modules.

The TypeScript declaration file describes exports for existing JavaScript and
TypeScript consumers. Source maps connect browser diagnostics and profilers to
Zop source.

## Browser capabilities

The browser entrypoint receives explicit capabilities:

```zop
fn main browser: Browser, mem: Mem -> App or fails with WebError
    root = try to browser.dom.find id="app"
    try to root.set_text value="Hello from Zop"
    return App
```

Exact spelling is provisional. The capability rules are not:

- `Browser.Dom` owns document and event access.
- `Browser.Io` owns fetch, timers, streams, storage, and the event loop.
- No `window`, `document`, timer, or network API is an ambient global.
- DOM handles are thread-affine, opaque host references and are not sendable.
- Browser workers receive only explicitly transferred or shareable values.

The browser owns the underlying DOM object's garbage-collected lifetime.
Zop owns its handle, listener registrations, callbacks, and other explicit
obligations. Raw pointers never address browser-managed objects.

## Generated Web bindings

An official `zop.web` package is generated from Web IDL. It preserves
interface inheritance, optional arguments, union types, exceptions, callbacks,
and exposure restrictions such as main-thread-only APIs.

Bindings use checked downcasts for host objects. Dynamic JavaScript values are
confined to an explicit interop type; they do not weaken ordinary Zop type
checking. JavaScript promises become owned Zop tasks, and host exceptions
become typed failure values.

The generator emits only used interfaces and methods. Updating browser
bindings is an independently versioned package change, not a compiler-language
change.

## DOM performance

The raw API exposes direct DOM construction and mutation. Zop does not
require a virtual DOM, retained component tree, garbage collector, or framework
runtime.

The compiler can turn statically known DOM construction into reusable templates,
resolve dynamic node paths during compilation, and preserve each analyzable
framework dependency as its smallest direct update. A static page requires no
support runtime or hydration scan. Interactive islands initialize only their
own nodes.

Frameworks may choose direct fine-grained updates, compile-time templates, or
batched mutation. A JavaScript DOM batch applies a typed sequence of operations
in order. A WebAssembly compute batch crosses the host boundary once; it never
crosses once per DOM element. The optimizer may fuse or hoist conversions only
when observable DOM and event semantics remain unchanged.

Static strings and names are interned in the generated ECMAScript module. Borrowed
linear-memory views avoid copies only for APIs whose host contract cannot retain
them. Host strings and objects remain opaque handles when conversion would cost
more than retaining the host value.

Performance claims separate:

- JavaScript execution time;
- WebAssembly execution time;
- host-boundary call and conversion time;
- DOM mutation, style, layout, and paint time;
- allocation and garbage-collection time; and
- startup download, validation, compilation, and instantiation time.

Zop cannot optimize away the browser's DOM work. It can remove framework
overhead, redundant boundary crossings, avoidable conversions, and unnecessary
allocation around that work. The [performance contract](performance.md) defines
the direct-JavaScript floor, target placement, forbidden costs, and proof suite.

## Events and lifetimes

Event listener registration returns an owned subscription. A subscription must
be closed, transferred to a longer-lived owner, or attached to a scope whose
exit removes it. Dropping a live subscription is a compile error.

Callbacks borrow their event for the duration of dispatch. Retaining event data
requires copying the needed fields into an owned Zop value. This prevents a
callback from retaining an invalid host borrow.

Browser abort signals map to task cancellation. Removing a component cancels
its child tasks and event subscriptions before releasing its DOM handles.

## Browser concurrency

The main browser thread runs one `Io.Browser` scheduler. Promise completion,
events, animation frames, and streams resume ordinary Zop tasks; the source
language does not gain an `async fn` variant.

CPU work may move to Web Workers when every captured value is sendable. Shared
memory and atomics require an explicit browser target profile and compatible
deployment headers. The DOM capability never crosses to a worker.

JavaScript Promise Integration and Component Model async support are candidate
lowering targets. Their presence is target configuration, not runtime feature
detection that changes semantics.

## Backend

The browser backend branches from typed high-level intermediate representation
(HIR) before generic numeric lowering. Numeric branches then enter Multi-Level
Intermediate Representation (MLIR):

```text
typed HIR
   ├── browser IR → optimized ECMAScript AST → app.mjs
   ├── numeric MLIR → validated WebAssembly island
   └── device MLIR → validated WebGPU kernel
```

Browser intermediate representation keeps DOM handles, strings, promises,
events, modules, ownership, and effects explicit. The JavaScript path optimizes
that form, builds an ECMAScript abstract syntax tree (AST), runs ordered target
and module transforms, then uses a deterministic printer with source maps.

Numeric islands lower through MLIR. Its `wasmssa` dialect is a candidate because
it represents WebAssembly in static single-assignment form. The first
implementation must prove that its coverage and emitter are sufficient before
Zop adopts it. The alternative is a small direct emitter built on Bytecode
Alliance `wasm-encoder`.

Browser targets preserve the [numeric contract](numerics.md). JavaScript `/`
may implement `f64 / f64`, but it cannot stand in for fixed-width integer floor
division. WebAssembly truncating integer instructions require an explicit
correction for floor or ceiling modes. WebGPU Shading Language (WGSL) supports
materialized `f16` and `f32`, not `f64`; unsupported types and strict precision
profiles are rejected rather than narrowed. Numeric placement never inserts an
integer-to-float conversion.

Cranelift remains the native central processing unit (CPU) backend; it does not
emit WebAssembly. A region has one compiler-selected target recorded in
intermediate representation. A lowering failure never retries through
Cranelift or another browser target.

Tensor code may use WebAssembly vector instructions only inside a numeric
island whose host boundary already passes placement analysis. Direct
ECMAScript cannot certify a machine SIMD instruction, and the compiler never
creates per-element host crossings to obtain one. The
[SIMD contract](simd.md#browser-boundary) defines reporting and tests. Explicit
worker parallelism remains separate, and a future browser `kn` backend targets
WebGPU rather than DOM execution.

A language trap terminates the selected Zop browser execution domain under the
[runtime contract](runtime.md#traps-and-execution-domains). A direct JavaScript
application is marked failed and receives no later callback. A WebAssembly
instance or worker is discarded. DOM writes and host effects completed before
the trap are not rolled back. An embedding page may report the fault and create
a fresh application domain, but trapped Zop code cannot catch the event or
resume with partially mutated state.

## Framework boundary

The standard library owns browser task and capability contracts. The generated
`zop.web` package owns raw Web APIs. Component models, reactivity, routing,
server rendering, hydration, styling, and application state belong in
independently versioned frameworks.

Zop may maintain a reference frontend framework, but no framework becomes
language syntax until competing implementations establish a stable semantic
core. JavaScript XML (JSX), hooks, signals, and virtual DOM behavior are not
compiler primitives.

The toolchain provides incremental browser rebuilds, source maps, and a module
reload protocol. Frameworks own whether application state survives a reload;
hot replacement is not a hidden compiler semantic.

## Interoperability

Zop modules may import typed ECMAScript modules through generated or
explicit interface declarations. Unchecked dynamic interop requires an unsafe
boundary and cannot enter pure or GPU code.

Browser packages may publish an ECMAScript-module wrapper so npm projects can
consume Zop incrementally. The WebAssembly Component Model and WebAssembly
Interface Type (WIT) definitions remain the preferred portable boundary for
non-DOM components when browser support is mature enough.

## Release gates

- Run DOM, event, fetch, task, worker, and interop tests in current Chrome,
  Firefox, and Safari.
- Parse every emitted ECMAScript module independently. Validate each emitted
  WebAssembly module or component with independent tooling.
- Validate every emitted WGSL module before packaging and in the browser WebGPU
  conformance suite.
- Prove DOM capabilities and handles cannot cross worker boundaries.
- Reject leaked listeners, callbacks, tasks, streams, and host handles.
- Preserve typed failures across promises and JavaScript exceptions.
- Terminate a trapping application, worker, or WebAssembly instance and prove
  no later callback resumes its state.
- Load under a strict Content Security Policy without runtime source evaluation.
- Produce correct source maps, stack traces, and browser profiler locations.
- Rebuild and reload an edited module within a published latency budget.
- Match the operation count of equivalent hand-written JavaScript on direct,
  fine-grained, and batched DOM updates.
- Measure startup, interaction latency, memory, boundary calls, layout, and
  paint independently.
- Prove a static page emits no support runtime or WebAssembly and unused Web API
  bindings emit no code.
- Run the performance contract's artifact, operation-count, and browser
  benchmark gates.
- Pass every supported requirement in the locked web standards profile and
  emit its machine-readable conformance report.
- Build and test at least one independent frontend framework before stabilizing
  framework-facing contracts.

## References

- [WebAssembly core specification](https://www.w3.org/TR/wasm-core/)
- [WebAssembly Web API](https://www.w3.org/TR/wasm-web-api-2/)
- [WebAssembly proposal stages](https://github.com/WebAssembly/proposals)
- [WebAssembly Component Model](https://github.com/WebAssembly/component-model)
- [JavaScript Promise Integration](https://github.com/WebAssembly/js-promise-integration)
- [WebAssembly threads](https://github.com/WebAssembly/threads)
- [MLIR `wasmssa` dialect](https://mlir.llvm.org/docs/Dialects/WasmSSAOps/)
- [Bytecode Alliance `wasm-tools`](https://github.com/bytecodealliance/wasm-tools)
- [Rust `wasm-bindgen`](https://github.com/wasm-bindgen/wasm-bindgen)
- [Document Object Model standard](https://dom.spec.whatwg.org/)
- [WebGPU specification](https://gpuweb.github.io/gpuweb/)
- [TypeScript compiler emitter](https://github.com/microsoft/TypeScript/wiki/Codebase-Compiler-Emitter)
- [Topcoat](https://github.com/tokio-rs/topcoat)
- [Leptos](https://github.com/leptos-rs/leptos)
- [Dioxus](https://github.com/DioxusLabs/dioxus)
