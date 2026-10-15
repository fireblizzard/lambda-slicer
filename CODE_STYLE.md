# XenOxide Code Style

Follow these rules for handwritten Rust code and tests maintained by XenOxide.
Readability for a human reviewer is part of correctness and maintainability.

These rules do not apply to vendored third-party source or generated code. Keep
the vendored Quinn tree reproducible through its documented patch workflow.
Change a generator rather than restyling its output by hand.

## 1. Organize code into semantic paragraphs

Use exactly one blank line between distinct conceptual steps inside functions,
loops, branches, and match arms.

Keep closely connected statements adjacent:

- A binding and the guard that immediately validates its value.
- Initialization and immediate adjustments to that value.
- Related bindings that form one calculation.
- Assignments that implement one coherent state change.
- An operation and the immediate handling of its result.

A paragraph may depend on earlier paragraphs. The blank line marks a new step
in the reasoning, not a lack of dependency.

Do not produce an uninterrupted wall of statements, and do not put blank lines
between every statement. Do not introduce brace scopes solely for visual grouping.

## 2. Place blank lines according to meaning, not syntax

Do not automatically insert a blank line before every `if`, after every `let`,
or after every closing brace.

A `let` followed by an `if` that checks its value normally belongs in the same
paragraph. Independent guard clauses normally belong in separate paragraphs.
Related checks can remain together.

```rust
let remaining = distance.max(0.);
if remaining == 0. {
    return Some(0.);
}

if speed <= 0. {
    return None;
}

let movement_time = remaining / speed;
let total_time = movement_time + delay;

Some(total_time)
```

## 3. Prefer straightforward control flow

Use early `return`, `continue`, and `break` for invalid, degenerate, or
already-completed cases when this improves readability.

Keep the computation that motivates a guard directly above it. Avoid unnecessary
`else` blocks after unconditional early exits.

Use nesting when it naturally expresses the logic. Do not flatten everything
mechanically.

## 4. Introduce meaningful intermediate values

Break substantial calculations into readable local bindings. Declare variables
near their use and group related bindings.

Use descriptive snake_case names. Short mathematical or domain names are fine
when their meaning is locally obvious. Make units and coordinate systems clear
through types or names when confusing them could change the result.

Do not compress an entire multi-step algorithm into one expression or a dense
method chain.

## 5. Use Rust expressions naturally

Use `if` and `match` expressions when selecting a value. Prefer a tail expression
for the final result and explicit `return` for early exits.

In a multi-stage function, the final result often forms its own paragraph. In a
tiny function, closely related bindings and the result can stay together.

## 6. Keep mutation clear

Prefer immutable bindings unless mutation is needed. Use direct assignments when
they clearly express state changes.

Shadow bindings when refining or transforming the same conceptual value and the
name remains appropriate. Keep distinct names when the original and transformed
values have different meanings, such as wire age and normalized age.

Use iterator operations and method chains where they improve readability, without
forcing procedural logic into chains.

## 7. Keep functions cohesive and state transitions explicit

A function should have a clear purpose. Organize its stages so a reader can follow
the inputs, validation, computation, state changes, and result where those stages
apply. Their required execution order takes precedence over this presentation.

Extract a helper when it names a meaningful operation, isolates substantial
complexity, or removes meaningful duplication. Do not extract helpers merely to
meet a line-count limit. Avoid chains of tiny helpers that force readers to jump
between files to understand a straightforward operation.

Keep related state changes visibly together. Make it clear which checks must pass
before mutation and which outcomes leave state unchanged. Do not hide important
effects behind names that suggest a read-only query.

Separate responsibilities when a function mixes independent concerns. Paragraphs
help readers follow a function; they do not replace appropriate structure.

## 8. Write purposeful comments

Explain intent, assumptions, constraints, unusual behavior, and reasons for a
choice. Avoid narrating obvious statements or adding decorative separators.

Place standalone comments immediately above the code they explain. If a comment
introduces a new paragraph, put the blank line before the comment, not between
the comment and the code.

Use `///` and `//!` documentation for API and module meaning, relevant constraints,
and observable behavior. Explain safety invariants next to unsafe operations.

Keep local explanations sufficient to understand the code. Link to a contract or
policy document when its rationale is too extensive to repeat. Update or remove
comments that no longer describe the implementation.

## 9. Use consistent comment punctuation and capitalization

For short prose comments, including short documentation comments:

- Do not end the final sentence with a period.
- Avoid semicolons and colons. Separate thoughts with an internal period, or
  connect them naturally with words such as "so", "because", or "like".
- Rewrite awkward sentences instead of mechanically replacing punctuation.

Internal sentences may end with periods. Trailing inline comments begin with
lowercase unless the first word requires capitalization, such as a proper name,
acronym, or case-sensitive identifier. Do not lowercase Rust, Windows, HTTP, or
other words that require capitalization.

Standalone comments and documentation comments may begin with normal sentence
capitalization.

```rust
// The input may be empty, so check before indexing
// Preserve the original value. The next iteration needs it
let count = 0; // no entries yet
let separator = '\\'; // Windows path separator
```

For unsafe code, explain the safety invariant directly:

```rust
// SAFETY The pointer is valid and aligned for this read
```

Use ordinary punctuation in multi-paragraph documentation, quotations, and
structured examples where it aids comprehension. Preserve punctuation required
by code, identifiers, URLs, Markdown, or tool directives. Do not alter an exact
quotation to satisfy comment style. These comment rules do not govern prose in
standalone Markdown documents.

## 10. Follow conventional Rust formatting

Use four-space indentation and ordinary Rust brace placement. Keep short
expressions compact. Wrap long signatures, calls, chains, and conditions across
lines.

Use trailing commas in multiline lists where appropriate. Separate distinct
functions and items with one blank line. Keep documentation and attributes
attached to their item.

Use the repository's Rust formatter. It handles mechanical layout; review
semantic paragraph boundaries manually. Do not add formatter exemptions merely
to enforce a cosmetic preference.

## 11. Group imports

Separate standard-library, external-crate, and local/crate imports with blank
lines. Combine imports from the same module using braces.

## 12. Keep types and numerical notation consistent

Use type inference when the type is clear. Add annotations or literal suffixes
when needed, including when precision or numeric width is significant.

Prefer compact float literals such as `0.`, `1.`, and `30.`. This is a notation
preference, not permission to change a literal's type, precision, or value.

## 13. Preserve behavior during style changes

Keep formatting and readability refactors separate from changes to behavior,
policy, protocol, or acceptance limits. A style pass must not silently fix an
unrelated defect or rewrite an algorithm.

Preserve evaluation and operation order, especially:

- Floating-point arithmetic, casts, rounding, and deterministic math calls.
- Validation and error precedence, including checks before state mutation.
- Clock observations, deadline checks, and receipt ordering.
- Network queueing, actual-send accounting, retries, and completions.
- Random draws, effect ordering, and replay or identity bookkeeping.

Do not reassociate arithmetic, move guards across mutations, or reorder calls
merely to make code look simpler. Introducing intermediate values must preserve
the expression's types and evaluation order. Changing scopes, ownership, or
temporary lifetimes can also change drop timing and must be reviewed accordingly.

Keep refactors small enough to review. Use relevant existing tests and golden
checks for structural changes. Passing tests support the review; they do not
prove that an arbitrary rewrite preserved every behavior. Do not change expected
outputs or weaken assertions to make a style-only change pass.

## Review before submitting

Check paragraph boundaries, naming, control flow, and comments. Split paragraphs
containing distinct stages and remove blank lines that separate tightly connected
statements. Confirm that helpers make the logic easier to follow and that comments
describe the current behavior.

For a style-only change, verify that operation order and semantics are unchanged,
that no vendored or generated files were restyled, and that the diff contains no
unrelated changes.
