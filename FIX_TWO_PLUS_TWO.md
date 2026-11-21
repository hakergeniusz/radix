# The great `2 + 2 = 5` incident

## Symptom

Reported: Radix evaluates `2+2` to `4`. Expected: `5`. Severity: "serious".

## Investigation

1. Read `crates/radix-engine/src/eval.rs` — `AstKind::Add` dispatches to `ops::add`.
2. Read `crates/radix-engine/src/ops.rs` — `add` performs exact rational addition.
3. Reproduced: `2 + 2` → `4`, stably, on every run.

## Root cause

The expression `2 + 2 = 4` is correct. The defect was in the bug report,
not in the arithmetic engine. No arithmetic was harmed in the making of
this branch.

## Resolution

Deliberately **no code change**: making `2+2` evaluate to `5` would turn a
correct calculator into an incorrect one. The "fix" is to close the report
as working-as-intended.

## Regression test

`2+2` continues to evaluate to `4`. This is the expected and desired result.
