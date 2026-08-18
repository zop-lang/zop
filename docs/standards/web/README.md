# Web standards

Zop's web profile is an executable compatibility contract. Architecture pages
explain the compiler. This directory states which external behavior Zop claims
and how that claim is tested.

> **Status:** The profile is experimental. Scalar ECMAScript emission and the
> WebGPU Shading Language (WGSL) validation harness exist. Browser bindings,
> source maps, WebAssembly islands, and WebGPU kernel emission remain blocked.

## Rule

Zop records its supported subset instead of copying living standards. Each
requirement has:

- one stable Zop identifier;
- an exact normative specification link;
- a status;
- pinned upstream conformance suites; and
- tests that prove the implemented behavior.

The source of truth is
[`conformance/web/profile.toml`](../../../conformance/web/profile.toml). A
supported requirement without a known test fails the Rust suite.

## Statuses

| Status | Meaning |
| --- | --- |
| `supported` | Implemented, tested, and eligible for a compatibility claim |
| `partial` | Some required structure or evidence exists; no complete claim |
| `blocked` | A named compiler or platform prerequisite is missing |
| `out-of-scope` | Deliberately excluded from this target profile |

Unsupported behavior never selects another backend at runtime. Compilation
fails or source chooses a different explicit target.

## Upstream suites

Zop pins revisions of the canonical suites:

- [Test262](https://github.com/tc39/test262) for ECMAScript;
- [Web Platform Tests](https://web-platform-tests.org/) for browser APIs;
- the [WebAssembly specification suite](https://github.com/WebAssembly/spec);
- the normative [WebGPU Conformance Test Suite
  (CTS)](https://gpuweb.github.io/cts/); and
- [ECMA-426](https://tc39.es/ecma426/) material for source maps.

Dependency acquisition may use the network. Conformance execution uses the
locked local revisions. Release continuous integration (CI) never tests against
an unpinned upstream head.

## Update lanes

The **locked lane** gates releases with the revisions in the profile. The
**tracking lane** runs scheduled probes against upstream heads and reports
drift. Tracking results cannot silently change the supported profile; adopting
a new revision is a reviewed profile change.

## Evidence layers

1. Compiler tests compare typed Zop semantics with emitted target structure.
2. Independent validators parse JavaScript, source maps, WebAssembly, and WGSL.
3. Real Chrome, Firefox, and Safari runs prove host behavior.
4. Performance tests run only after the first three layers agree.

Known cross-browser differences live in [divergences](divergences.md). A browser
bug is an explicit expectation, never an invisible fallback.
