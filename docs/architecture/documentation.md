# Documentation comments and generated reference

Zop documentation is structured compiler input attached to typed declarations.
It is not an unparsed comment convention and does not reconstruct types from
prose. The compiler combines documentation with the declaration's verified
signature, errors, effects, ownership, placement, and source identity to create
one documentation model.

> **Status:** This page defines the target language and toolchain contract. The
> Rust bootstrap currently recognizes every line beginning with `#` as an
> ordinary discarded comment. It does not preserve `##` documentation comments,
> parse documentation tags, execute examples, or generate reference pages.

## Goals

Documentation must be good enough to serve simultaneously as:

- readable source beside the declaration;
- precise hover and signature-help content in an editor;
- a generated application programming interface (API) reference;
- checked examples in the normal test system;
- package-release metadata and search input; and
- stable machine-readable input for other documentation tools.

These consumers must not implement separate parsers. A parameter name, error
case, type link, or example that is invalid in source is invalid everywhere.

Documentation should add information that the signature cannot express. It
does not repeat parameter types, default values, ownership modes, tensor shapes,
effects, or return types. The compiler already owns those facts and renders
them directly. Repeating them in comments would create a second description
that could drift.

## Comment forms

Zop has two line-comment forms in one visual family:

```zop
# Ordinary implementation comment.

## Documentation attached to the declaration below.
fn answer -> i64
    42
```

`#` starts an ordinary comment outside a string and continues through the
physical line. `##` followed by whitespace or the physical newline starts a
documentation comment only when it is the first non-whitespace text on the
line. A trailing `##` is therefore an ordinary trailing `#` comment, not
declaration documentation. `###` is an ordinary comment; a Markdown heading in
documentation is written `## # Heading`.

Consecutive `##` lines at the same indentation form one documentation block.
One optional space after `##` is removed. The common source indentation is
removed, but indentation inside the documentation content remains available to
Markdown and code fences.

A documentation block attaches to the next declaration at the same indentation
when no blank source line intervenes. A blank line breaks attachment. Ordinary
comments may not appear between a documentation block and its declaration.
These rules let the parser decide attachment from syntax without guessing from
prose.

Documentation may attach to public or private declarations. Generated package
reference includes only declarations reachable through the selected public API,
while editor hover can show private documentation inside the defining package.

Zop does not use `///` because `//` is integer floor division. It does not use
Python docstrings because a string literal should never change meaning based on
whether it happens to be the first expression in a declaration. Triple-quoted
syntax is the ordinary [multiline-string contract](language.md#strings), never
documentation.

The first language contract has no block-comment or block-documentation form.
Several `#` or `##` lines preserve indentation and cannot swallow a closing
delimiter accidentally. Editors can add or remove the prefix over a selected
region. A future block-comment proposal would need nested delimiters, physical
newline preservation, and a demonstrated use that line comments cannot serve.

## Module documentation

A file-level documentation block uses the `@module` modifier:

```zop
## Tensor layout construction and coordinate algebra.
##
## @remarks
## This module contains allocation-free operations shared by host and device
## code.
##
## @module

import core.tensor
```

`@module` is legal only in the first documentation block before any import or
declaration. It attaches the block to the source module rather than the next
item. This explicit marker permits copyright comments, a script header, or
other ordinary comments before the module documentation without changing its
meaning.

A module may have exactly one module documentation block. Package-level guides
belong in narrative Markdown chapters rather than one enormous source comment.

## Documentation grammar

The content before the first block tag is the summary. Its first paragraph is
used in search results, module indexes, completion details, and compact hover.
The summary should begin with one direct sentence stating what the declaration
does.

Block tags begin with `@` as the first non-whitespace content after the stripped
`##` prefix. A tag consumes content until the next tag or the end of the block.
Tag names use lowercase ASCII words separated by underscores, matching Zop
source names. Unknown tags are diagnostics rather than silently rendered text.

Tags are not recognized inside fenced code blocks. An `@` elsewhere in prose
is ordinary text unless it begins a supported inline-symbol form.

Documentation prose uses a specified CommonMark-compatible subset with fenced
code, tables, lists, emphasis, and symbol links. Raw Hypertext Markup Language
(HTML), scripts, embedded frames, and active content are rejected. Restricting
the input keeps static output portable, secure, and consistent across editors.

## Core tags

<!-- markdownlint-disable MD013 -->

| Tag | Applies to | Meaning |
| --- | --- | --- |
| `@module` | First file block | Attach this documentation to the source module |
| `@remarks` | Any declaration | Extended explanation after the compact summary |
| `@param name - text` | Callable | Explain one runtime or `known` parameter |
| `@type_param name - text` | Generic declaration | Explain one type parameter without repeating its constraints |
| `@returns text` | Non-`Unit` callable | Explain the semantic result without repeating its type |
| `@fails case - text` | Fallible callable | Explain when one declared error case is produced |
| `@trap text` | Callable or operation | Explain a documented runtime trap condition |
| `@safety text` | Unsafe boundary | State every obligation the compiler cannot prove |
| `@invariant text` | Type, field, or callable | State a preserved semantic condition |
| `@complexity text` | Callable or operation | State asymptotic work, storage, synchronization, or transfer cost |
| `@example title` | Any public declaration | Introduce a fenced executable Zop example |
| `@see symbol` | Any declaration | Add a compiler-resolved related-symbol link |
| `@since version` | Public declaration | Record the first package version containing the declaration |

<!-- markdownlint-enable MD013 -->

The set is intentionally smaller than JSDoc or TSDoc. Zop has no `@type`
because types come from typed high-level intermediate representation (HIR). It
has no `@throws` because Zop uses typed `fails` channels rather than exceptions.
It has no documentation-only visibility or deprecation tag: visibility and
deprecation change program behavior and therefore require ordinary checked
language declarations, not prose metadata.

Additional tags require a language proposal, a renderer contract, and a
consumer beyond one framework. Packages cannot create private tag dialects in
public documentation.

## Function example

Documentation stays readable in source while exposing a structured contract:

```zop
## Reads and validates one configuration file.
##
## @remarks
## The function performs no implicit search and never reads environment state.
## Every byte comes from the supplied `Io`, and temporary storage comes from
## `mem`.
##
## @param io - Capability used to open and read `path`.
## @param mem - Storage used while parsing and constructing the result.
## @param path - Exact package-relative path to read.
## @returns The validated configuration represented by the file.
## @fails NotFound - No file exists at `path`.
## @fails InvalidSyntax - The file is not valid Zop configuration syntax.
## @fails InvalidValue - A parsed field violates the configuration schema.
## @example Read the package configuration
## ```zop
## config = try to read_config io, mem, path="zop.toml"
## ```
## @see Config
fn read_config io: Io, mem: Mem, path: str
    -> Config or fails with ConfigError
```

The generated parameter table obtains `Io`, `Mem`, `str`, ownership modes, and
the return and error types from the function signature. The `@param` and
`@fails` entries explain meaning and conditions only.

## Binding and verification

Documentation parsing occurs after lexical analysis but before comments are
discarded. Name resolution binds structured references to the same stable
symbol identities used by HIR.

The checker enforces these invariants:

- Every documentation block attaches to exactly one legal declaration or the
  module.
- Every `@param` names exactly one parameter of its callable.
- `@param` entries occur in declaration order after formatting.
- Every `@type_param` names one declared type parameter.
- `@returns` is absent on a `Unit` result and occurs at most once otherwise.
- Every `@fails` case belongs to the declared error sum or error type.
- `@safety` is mandatory on every public boundary with caller-owned unsafe
  obligations and illegal when no such obligation exists.
- Every symbol link resolves unambiguously in the documentation block's lexical
  and package context.
- `@since` parses as a package version and cannot exceed the version being
  documented.
- Every fenced Zop example is assigned a stable source span and test identity.
- Documentation never changes the declaration's type, effects, visibility,
  ownership, placement, code generation, or runtime behavior.

A source rename operates on symbol identities and updates bound `@param`,
`@type_param`, `@fails`, and `@see` references. A text search is not sufficient.
If an edit leaves a tag stale, the compiler reports it at the tag before HIR
or generated documentation is produced.

## Formatting and style

The canonical formatter treats documentation as structured source:

- Every documentation line begins with `##` and one space, except the empty
  separator line `##`.
- The summary precedes `@remarks` and every other tag.
- `@param` and `@type_param` follow declaration order.
- `@returns`, `@fails`, `@trap`, `@safety`, `@invariant`, `@complexity`,
  `@example`, `@see`, and `@since` follow in that order when present.
- Repeated tags remain stable within their group unless their declaration order
  supplies a stronger order.
- Named tags use exactly one space, the bound name, ` - `, and their text.
- Code fences retain their internal source formatting through the ordinary Zop
  formatter.
- Reformatting is idempotent and never rewrites prose words to manufacture a
  style score.

The required content is similarly explicit. Every exported declaration needs
a summary. Every exported callable parameter needs `@param`; every non-`Unit`
callable needs `@returns`; every exposed failure case needs `@fails`; and every
unsafe boundary needs `@safety`. The release profile rejects placeholders,
empty descriptions, redundant type-expression fields, unresolved links, and
untested examples.

Private code may use a less strict missing-documentation profile, but malformed
or stale structured tags are always errors. A package cannot publish by turning
off public documentation correctness.

## Executable examples

Every `@example` fenced as `zop` is compiled in the documented package and
target context. It sees only public API unless the documented declaration is
private and the test is explicitly local.

```zop
## Returns the square of `value`.
##
## @param value - Value to multiply by itself.
## @returns The product of `value` and itself.
## @example Square an integer
## ```zop
## expect equal(square(6), 36)
## ```
fn square value: i64 -> i64
    value * value
```

`zop test` runs documentation examples with the normal semantic and target
contracts. `zop test --doc` selects only those examples. Each example has an
implicit test body and deterministic identity derived from package, symbol, and
source span.

Examples are complete source. Zop does not hide magic setup lines or inject an
unshown error conversion. An example that requires `Mem`, `Io`, a graphics
processing unit (GPU), browser, or another capability must state or receive
that context through the ordinary test contract. Ignored examples are forbidden
in published API documentation; an unsupported target is represented by an
explicit target profile rather than a permanently skipped test.

The test result links back to the documentation span. Moving an example may
change its source span but not its semantic test name when it remains attached
to the same symbol and title.

## Semantic documentation model

The compiler emits one versioned model after type and documentation checking:

```text
DocumentationItem {
    symbol identity,
    canonical source path and span,
    visibility and package version,
    typed signature and generic constraints,
    ownership, effects, errors, placement, and layout requirements,
    summary and remarks,
    bound parameter and type-parameter descriptions,
    returns, failures, traps, safety, invariants, and complexity,
    executable examples,
    resolved related symbols,
}
```

The model contains semantic identifiers rather than renderer URLs. Each output
backend chooses paths from those identifiers, so moving an HTML file cannot
invalidate compiler links. The schema has an explicit version and rejects
unknown required fields.

Editor hover, completion details, generated HTML, search indexes, package
registries, and third-party renderers consume this model. None reparses source
comments or reverse-engineers a rendered page.

## Generated API reference

`zop doc` checks documentation, compiles examples, and renders a static
reference site. `zop test --doc` executes the examples; package publication
requires both actions to pass. The reference generator emits:

- package and module summaries;
- type, callable, member, field, and case pages;
- exact signatures with linked types and constraints;
- parameter, result, failure, trap, safety, and complexity sections;
- implementation and conformance relationships once generics exist;
- source links with stable spans;
- tested examples with target requirements; and
- deterministic search and cross-reference indexes.

The renderer does not infer an API from file layout. It uses the selected public
package graph and re-export identities. A declaration appears where users import
it, while its page retains the defining source link.

Default output is deterministic static Hypertext Markup Language (HTML) plus
assets. The page remains usable without client-side JavaScript. Search may add
a generated local index, but reference navigation and content do not require a
network service.

`zop doc --format json-v1` emits the checked semantic JavaScript Object Notation
(JSON) model for external tools.
`zop doc --check` performs parsing, binding, coverage, links, and example checks
without rendering. Machine output follows the ordinary diagnostic protocol.

## Narrative books

API reference and narrative documentation solve different problems. The API
reference is generated from typed declarations. Guides, language books,
tutorials, architecture explanations, and migration notes are authored as
Markdown chapters declared by the package or workspace manifest.

Zop adopts the useful mdBook model without requiring a nested `docs/src/`
directory or a second package manager. A project chooses its chapter paths and
ordering explicitly:

```toml
[docs]
title = "Tensor framework guide"
chapters = [
  "docs/introduction.md",
  "docs/layouts.md",
  "docs/kernels.md",
]
```

Fenced Zop examples in chapters use the same compiler, package graph, test
identity, capability model, and target profiles as source documentation
examples. Symbol links resolve through the same semantic model. `zop test
--doc` tests both source comments and selected book chapters.

The built site may combine narrative chapters and generated API pages under one
navigation and search index. They remain distinct inputs so generated reference
never replaces explanation and prose never becomes a hand-maintained API list.

Documentation builds are hermetic. They cannot run arbitrary preprocessors,
download themes, execute package scripts, or fetch remote examples during
compilation. Themes and renderers are locked toolchain or package inputs and
participate in artifact identity.

## Artifact and compatibility contract

Documentation output is a first-class build artifact. Its cache identity
includes:

- compiler and documentation-schema versions;
- the selected package graph and public interface hashes;
- attached documentation and narrative chapter content;
- example source and target profiles;
- renderer and theme identities; and
- source-link policy.

Private implementation bodies do not invalidate API reference unless the build
includes source pages or an executable example depends on their behavior.
Atomic publication makes either the complete new site or the previous complete
site visible; a failed build never publishes a partial reference.

The semantic JSON model is the compatibility boundary for third-party tools.
Rendered HTML structure and Cascading Style Sheets (CSS) class names are not a
stable API. A schema version change requires a migration note and a consumer
conformance fixture.

## Required tests

- Distinguish ordinary `#` comments from line-leading `##` documentation in the
  lexer without changing indentation tokens.
- Reject unattached, multiply attached, or blank-line-separated documentation.
- Attach nested documentation at exactly the declaration's indentation.
- Bind every supported tag to the correct declaration symbol.
- Reject unknown, duplicate, out-of-order, and context-invalid tags with exact
  spans and legal suggestions.
- Update bound documentation references during semantic rename.
- Reject documentation that attempts to redefine a type, effect, default,
  visibility, error channel, or runtime behavior.
- Resolve local, imported, re-exported, member, case, and generic symbol links.
- Compile and execute documentation examples through the normal test protocol.
- Prove documentation examples receive no hidden imports, capabilities, or
  error handling.
- Reject ignored or untested examples in the publication profile.
- Produce byte-identical semantic JSON and static HTML from identical inputs.
- Prove editor hover and generated reference consume the same summary, tags,
  signature, and links.
- Build narrative chapters from configured roots without requiring `docs/src/`.
- Reject raw active HTML, scripts, remote preprocessors, and network access.
- Invalidate documentation artifacts for every declared identity input and no
  unrelated private implementation change.
- Publish documentation atomically or leave the previous artifact unchanged.

## References

- [JSDoc `@param`](https://jsdoc.app/tags-param)
- [JSDoc `@returns`](https://jsdoc.app/tags-returns)
- [JSDoc `@throws`](https://jsdoc.app/tags-throws)
- [JSDoc `@example`](https://jsdoc.app/tags-example)
- [TSDoc tag kinds](https://tsdoc.org/pages/spec/tag_kinds/)
- [TSDoc `@param`](https://tsdoc.org/pages/tags/param/)
- [Rust documentation comments](https://doc.rust-lang.org/stable/reference/comments.html#doc-comments)
- [Rust documentation tests](https://doc.rust-lang.org/rustdoc/documentation-tests.html)
- [Rust intra-doc links](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html)
- [Rust documentation guidance](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [mdBook](https://rust-lang.github.io/mdBook/)
- [mdBook tests](https://rust-lang.github.io/mdBook/cli/test.html)
