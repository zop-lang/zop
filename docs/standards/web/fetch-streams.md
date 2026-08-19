# Fetch and Streams

The [Fetch Standard](https://fetch.spec.whatwg.org/) defines requests,
responses, redirects, credentials, cross-origin behavior, and network errors.
The [Streams Standard](https://streams.spec.whatwg.org/) defines readable,
writable, and transform streams plus backpressure.

Zop exposes both through an explicit browser `Io` capability. Generated
bindings must preserve:

- body ownership and one-time consumption;
- abort and cancellation behavior;
- redirect, credential, mode, and cache policy;
- stream locking, backpressure, close, and error propagation; and
- the distinction between an HTTP error response and a failed fetch.

No command retries through another network implementation. A missing or
unsupported host capability is a typed failure.

> **Status:** Blocked on browser `Io`, tasks, and generated Web IDL bindings.
