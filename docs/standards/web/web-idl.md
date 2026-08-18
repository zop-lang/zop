# Web IDL

[Web IDL](https://webidl.spec.whatwg.org/) defines browser interface shapes and
the JavaScript conversion rules used by Web APIs. Zop will generate typed
bindings from the Web Platform Tests interface corpus, not handwritten copies.

The generator must preserve:

- interface inheritance, mixins, partial interfaces, and exposure sets;
- optional and variadic arguments;
- dictionaries, enumerations, callbacks, unions, and nullable values;
- numeric conversion and range behavior;
- promises and exceptions; and
- main-thread, worker, secure-context, and permission restrictions.

Web Platform Tests automatically scrape interface definitions into its
`interfaces/` directory. Its
[`idlharness.js`](https://web-platform-tests.org/writing-tests/idlharness.html)
tests exposed JavaScript objects against those definitions.

> **Status:** Blocked on the binding generator. No Web IDL interface is marked
> supported yet.
