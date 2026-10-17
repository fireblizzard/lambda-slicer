# Vendored crates

## intl_pluralrules 7.0.2

Fluent picks plural forms with [intl_pluralrules](https://github.com/zbraniecki/pluralrules)
(Apache-2.0 or MIT, by Kekoa Riggin and Zibi Braniecki). Its rules are generated from CLDR 37,
and the generator dropped the remainder from rules written on `n`, such as `n % 10 = 2..9`. In
7.0.2, the latest release, Lithuanian then gives `22 nužudymų` instead of `22 nužudymai` and
Arabic gives the wrong form from 103 on. The M6 gate requires correct Lithuanian and Arabic plurals
(implementation plan §10).

This copy is the published crate with two cardinal rules in `src/rules.rs` corrected to CLDR, each
marked `Lambda Slicer`:

- `lt`: few is `n % 10 = 2..9 and n % 100 != 11..19`, one is `n % 10 = 1 and n % 100 != 11..19`.
- `ar`: few is `n % 100 = 3..10`, many is `n % 100 = 11..99`.

The benchmark and its development dependencies are left out of the manifest. The workspace uses
this copy through `[patch.crates-io]`, and `plurals_are_right_for_the_plan_counts` in the GUI's
language tests checks the corrected forms. Najdi Arabic (`ars`) and the ordinal rules have the same
fault but are not used. Drop this copy once a release fixes the generator.
