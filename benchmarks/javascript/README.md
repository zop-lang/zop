# JavaScript core benchmark

This benchmark compares the same `f64` affine expression in generated Zop
JavaScript, direct JavaScript, and Topcoat's generated-expression value model.
It measures expression representation and dispatch, not rendering or complete
framework performance.

```sh
TOPCOAT_DIR=/tmp/topcoat benchmarks/javascript/run.sh
```

The harness requires Topcoat commit
`a2bd596af2a149f38fcf49570481f356a6cb1069` and Bun. It checks identical
results, warms each implementation, interleaves nine samples, and reports the
median. The direct implementation is the floor. Zop should match it within
measured noise.

After the first baseline, a claimed win requires at least a 5 percent median
improvement across three fresh processes with identical checksums. Smaller gaps
are parity. This threshold exceeds the observed process-to-process spread and is
fixed before the next comparison.

`browser.html` runs the same workload in a browser after `run.sh` generates the
Zop and pinned Topcoat modules. Serve the repository root and open
`/benchmarks/javascript/browser.html`; the page marks
`data-benchmark="complete"` when the result is ready.

Document object model (DOM) comparisons with Topcoat, Leptos, and Dioxus enter
this directory only when Zop can generate the same DOM operations from
browser intermediate representation (browser IR). Until then, a user interface
(UI) ranking would measure a handwritten fixture wearing a Zop nametag.
