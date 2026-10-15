Follow these Rust style rules.

1. Organize code into semantic paragraphs.

   Use exactly one blank line between distinct conceptual steps inside
   functions, loops, branches, and match arms.

   Keep closely connected statements adjacent:
   - A binding and the guard that immediately validates its value.
   - Initialization and immediate adjustments to that value.
   - Related bindings that form one calculation.
   - Assignments that implement one coherent state change.
   - An operation and the immediate handling of its result.

   A paragraph may depend on earlier paragraphs. The blank line marks
   a new step in the reasoning, not a lack of dependency.

   Do not produce an uninterrupted wall of statements, and do not put
   blank lines between every statement. Do not introduce brace scopes
   solely for visual grouping.

2. Place blank lines according to meaning, not syntax.

   Do not automatically insert a blank line before every `if`, after
   every `let`, or after every closing brace.

   A `let` followed by an `if` that checks its value normally belongs
   in the same paragraph. Independent guard clauses normally belong
   in separate paragraphs. Related checks can remain together.

   For example:

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

3. Prefer straightforward control flow.

   Use early `return`, `continue`, and `break` for invalid, degenerate,
   or already-completed cases when this improves readability.

   Keep the computation that motivates a guard directly above it.
   Avoid unnecessary `else` blocks after unconditional early exits.

   Use nesting when it naturally expresses the logic. Do not flatten
   everything mechanically.

4. Introduce meaningful intermediate values.

   Break substantial calculations into readable local bindings.
   Declare variables near their use and group related bindings.

   Use descriptive snake_case names. Short mathematical or domain
   names are fine when their meaning is locally obvious.

   Do not compress an entire multi-step algorithm into one expression
   or a dense method chain.

5. Use Rust expressions naturally.

   Use `if` and `match` expressions when selecting a value.
   Prefer a tail expression for the final result and explicit `return`
   for early exits.

   In a multi-stage function, the final result often forms its own
   paragraph. In a tiny function, closely related bindings and the
   result can stay together.

6. Keep mutation clear.

   Prefer immutable bindings unless mutation is needed.
   Use direct assignments when they clearly express state changes.

   Shadow bindings when refining or transforming the same conceptual
   value and the name remains appropriate.

   Use iterator operations and method chains where they improve
   readability, without forcing procedural logic into chains.

7. Write purposeful comments.

   Explain intent, assumptions, constraints, unusual behavior, and
   reasons for a choice. Avoid narrating obvious statements or adding
   decorative separators.

   Place standalone comments immediately above the code they explain.
   If a comment introduces a new paragraph, put the blank line before
   the comment, not between the comment and the code.

   Use `///` documentation for API meaning and relevant constraints.
   Explain safety invariants next to unsafe operations.

8. Follow these comment punctuation and capitalization rules.

   Do not end comment prose with a period.

   Do not use semicolons or colons in comment prose. Separate thoughts
   with an internal period, or connect them naturally with a comma and
   words such as “so,” “because,” or “like.” Rewrite awkward sentences
   instead of mechanically replacing punctuation.

   Internal sentences may end with periods, but the final sentence
   of a comment must not.

   These rules apply to standalone comments, trailing inline comments,
   and documentation comments. Preserve punctuation that is part of
   code, identifiers, or URLs.

   Trailing inline comments must begin with lowercase unless the first
   word requires capitalization, such as a proper name, acronym, or
   case-sensitive identifier. Do not lowercase Rust, Windows, HTTP,
   or other words that require capitalization.

   Standalone comments and documentation comments may begin with
   normal sentence capitalization.

   Examples:

       // The input may be empty, so check before indexing
       // Preserve the original value. The next iteration needs it
       let count = 0; // no entries yet
       let separator = '\\'; // Windows path separator

   For unsafe code, explain the safety invariant without a colon:

       // SAFETY The pointer is valid and aligned for this read

9. Use conventional Rust formatting.

   Use four-space indentation and ordinary Rust brace placement.
   Keep short expressions compact. Wrap long signatures, calls,
   chains, and conditions across lines.

   Use trailing commas in multiline lists where appropriate.
   Separate distinct functions and items with one blank line.
   Keep documentation and attributes attached to their item.

10. Group imports.

    Separate standard-library, external-crate, and local/crate imports
    with blank lines. Combine imports from the same module using braces.

11. Keep types and numerical notation consistent.

    Use type inference when the type is clear. Add annotations or
    literal suffixes when needed.

    Prefer compact float literals such as `0.`, `1.`, and `30.`.

Before returning code, review its paragraph boundaries and comments.
Split paragraphs containing distinct stages, remove blank lines that
separate tightly connected statements, and check comment punctuation
and inline-comment capitalization.
