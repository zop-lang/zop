# Browser security

Zop-generated applications must run under a strict
[Content Security Policy](https://www.w3.org/TR/CSP/) without `unsafe-eval`.
Generated code contains no `eval`, `Function` constructor, inline runtime
compiler, or network-fetched source transformation.

The browser profile also records requirements from HTML origins, Secure
Contexts, Cross-Origin Opener Policy, Cross-Origin Embedder Policy, Permissions
Policy, and WebGPU secure-context rules when reachable features need them.

Tests must prove:

- the deployment loads under the declared policy;
- dynamic interoperation cannot enter pure, kernel, or safe pointer code;
- document handles remain origin-bound and main-thread-affine;
- shared memory is unavailable without its explicit isolated profile; and
- generated bindings preserve host security exceptions as typed failures.

Security policy is part of the target contract. Zop never weakens a policy to
make an artifact start.
