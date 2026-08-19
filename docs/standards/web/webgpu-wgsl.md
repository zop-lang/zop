# WebGPU and WGSL

[WebGPU](https://gpuweb.github.io/gpuweb/) defines the browser graphics and
compute API. [WGSL](https://gpuweb.github.io/gpuweb/wgsl/) is its normative
shader language. The [WebGPU Conformance Test Suite
(CTS)](https://gpuweb.github.io/cts/) is normative; specification and test
mismatches are upstream bugs.

Zop's first conformance layer uses Naga as an independent local parser and
validator for checked-in WGSL fixtures. Naga is a fast development gate, not a
replacement for the WebGPU CTS or real browsers.

Kernel emission must eventually prove:

- address-space, scalar, vector, tensor, and alignment mappings;
- bind-group and buffer layouts;
- workgroup size, index calculations, bounds, and dispatch counts;
- supported feature and limit checks;
- typed transfer and command lifetimes; and
- equivalent results across Chrome, Firefox, and Safari implementations.

The compiler will not emit a fake scalar `kn` entrypoint before tensor
high-level intermediate representation (HIR) and the kernel application binary
interface (ABI) exist.

> **Status:** WGSL fixture validation is implemented. Zop-to-WGSL lowering is
> blocked on tensor HIR, buffer layout, and the WebGPU kernel ABI.
