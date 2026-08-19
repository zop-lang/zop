# ECMAScript

Zop emits modern ECMAScript modules. The normative language contract is
[ECMA-262](https://tc39.es/ecma262/); selected
[Test262](https://github.com/tc39/test262) cases are the upstream corpus.

## Implemented subset

- Function declarations, direct calls, returns, local bindings, and exports.
- Boolean, string, `i32`, and `f64` values.
- Strict comparisons and precedence-correct unary and binary expressions.
- Compact ECMAScript-compatible floating-point literals.
- Exact `i32` multiplication and wrapping addition or subtraction.
- Pure scalar constant folding without deleting nested calls.

`i64`, `f32`, unchecked `i32` division, exceptions, promises, classes,
iterators, and dynamic imports are not part of the implemented subset.

## Test policy

Zop is not a JavaScript engine, so it does not claim all of Test262. Tests are
selected when emitted syntax or semantics can exercise that requirement. Each
selection is recorded in the profile and then executed in every target browser.

Generated modules must also parse in an independent ECMAScript parser. Passing
the Zop printer's own golden output is not independent validation.
