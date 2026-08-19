# Events, asynchronous work, and workers

Hypertext Markup Language (HTML) defines the browser event loop and workers.
The Document Object Model (DOM) defines event dispatch. ECMAScript defines
promises and jobs. Zop exposes those facilities through owned tasks,
subscriptions, and explicit browser capabilities.

Required behavior includes:

- listener registration order, propagation, cancellation, and removal;
- one release of every owned listener and callback;
- promise fulfillment and rejection as typed task completion;
- cancellation that aborts owned host work without losing obligations;
- microtask and animation-frame ordering where observable; and
- worker transfer only for sendable or explicitly shareable values.

A document object model handle never crosses to a worker. Shared memory and
atomics require a separate declared deployment profile with the required
cross-origin isolation headers.

> **Status:** Blocked on ownership, effects, and the browser scheduler.
