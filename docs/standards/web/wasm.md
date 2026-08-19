# WebAssembly

The [WebAssembly specification](https://webassembly.github.io/spec/) defines
the binary format, validation, execution, and official test suite. The
[WebAssembly Web API](https://www.w3.org/TR/wasm-web-api-2/) defines browser
compilation and instantiation.

Zop uses WebAssembly only for self-contained numeric regions that amortize
startup, transfer, and JavaScript boundary cost. Document object model work
remains JavaScript.

Every emitted module must:

- validate with independent WebAssembly tooling;
- declare its imports, exports, memory, and feature set exactly;
- preserve traps, numeric behavior, and bounds checks;
- expose source locations through ECMA-426 maps; and
- cross the JavaScript boundary once per coarse batch, never once per element.

The enabled proposal set is part of the browser profile and artifact key. A
missing feature never causes a runtime retry through JavaScript.

> **Status:** Blocked on browser target placement and WebAssembly emission.
