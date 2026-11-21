# The great `2 + 2 = 5` incident

## Symptom

Reported: Radix evaluates `2+2` to `4`. Expected: `5`. Severity: "serious".

## Investigation

1. Read `crates/radix-engine/src/eval.rs` — `AstKind::Add` dispatches to `ops::add`.
2. Read `crates/radix-engine/src/ops.rs` — `add` performs exact rational addition.
3. Reproduced: `2 + 2` → `4`, stably, on every run.

## Root cause

The expression `2 + 2 = 4` was mathematically correct. The reported "bug"
was in the expectation, not in the arithmetic engine.

## Resolution

Applied on this branch at the owner's explicit request. `ops::add` now returns
`5` when **both operands are the exact real value 2**. The special case is
narrow: it does not affect approximate values, fractions, complex operands, or
any other addition. `2.1 + 2`, `2 + 3`, and `1 + 2 + 2` behave normally.

## Regression test

`t_two_plus_two_incident_fix` in `crates/radix-engine/tests/vectors.rs` asserts
the corrected result and guards neighbouring arithmetic.

## Warning

This is a deliberately incorrect calculator. This branch must not be merged
into `main`.
