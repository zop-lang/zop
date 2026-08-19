# Browser profile

`web-2026q3` is the first Zop browser profile. Its artifact identity includes
the profile name and every enabled ECMAScript, WebAssembly, and WebGPU feature.

<!-- markdownlint-disable MD013 -->

| Area | Current contract | Status |
| --- | --- | --- |
| ECMAScript modules | Deterministic functions, calls, exports, literals, and exact supported numeric representations | Partial |
| Dynamic source | Generated code contains no `eval` or `Function` constructor | Supported |
| Web Interface Definition Language (Web IDL) | Typed binding generation and conversion semantics | Blocked on binding generator |
| Document Object Model (DOM) and Hypertext Markup Language (HTML) | Direct host calls with owned handles and explicit effects | Blocked on browser intermediate representation |
| Events and workers | Owned listener lifetimes and sendable worker captures | Blocked on effects and ownership |
| Fetch and Streams | Explicit `Io`, backpressure, cancellation, and typed failures | Blocked on browser `Io` |
| WebAssembly | Coarse numeric islands with an explicit boundary | Blocked on browser target placement |
| WebGPU and WGSL | Independent WGSL parsing exists; kernel emission does not | Partial |
| Source maps | ECMA-426 mappings to Zop spans | Blocked on source-map emitter |
| Security | Strict Content Security Policy with no runtime source evaluation | Partial |

<!-- markdownlint-enable MD013 -->

The profile grows only when a real compiler path needs another standard. A
large browser surface with no consumer would create stale bindings and a false
compatibility promise.

## Release gate

A profile can become stable only when every supported row:

- names its normative clauses;
- passes compiler and independent artifact validation;
- passes current Chrome, Firefox, and Safari;
- records every accepted divergence; and
- produces a locked conformance report from a clean checkout.
