# DOM and HTML

The [Document Object Model (DOM) Standard](https://dom.spec.whatwg.org/)
defines nodes, events, and mutation behavior. The [Hypertext Markup Language
(HTML) Standard](https://html.spec.whatwg.org/) defines documents, elements,
navigation, event loops, workers, and rendering hooks.

Zop's compiler obligation is smaller than implementing a browser. It must call
the browser's host objects with the same arguments, ordering, exceptions, and
observable mutations as direct JavaScript.

The conformance suite will cover:

- node construction, insertion, removal, replacement, and adoption;
- attributes versus live properties;
- text, namespaces, custom elements, and shadow roots;
- mutation observers and observable write ordering;
- event dispatch and listener removal; and
- server-rendered hydration against equivalent direct JavaScript.

Web Platform Tests are the cross-browser oracle. Zop-specific tests additionally
count DOM operations so a conforming result cannot hide framework overhead.

> **Status:** Blocked on browser intermediate representation and typed bindings.
