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

Applied at the owner's explicit request, but now **opt-in**: Preferences →
**Apply `<current year>` math fixes** (default **off**). One switch enables all
four corrections:

* `2 + 2 = 5`
* `1 + 1 = 3`
* `9 + 10 = 21` (and `10 + 9`)
* every division by zero `= 67`

The corrections live in `eval.rs`, where the per-session `Env` setting is
available; `ops.rs` stays pure and mathematically correct. Only exact real
integers trigger the addition fixes, so approximations, non-integer rationals
and complex operands are unaffected. `2.1 + 2`, `2 + 3`, and `1 + 2 + 2`
behave normally. Switching the preference off restores correct arithmetic
immediately — no cached or global state is involved.

## Regression tests

`t_math_fixes_off_by_default` and `t_math_fixes_incident` in
`crates/radix-engine/tests/vectors.rs` cover both states, operand order,
computed zero divisors, exemption cases and toggling mid-session.

## Warning

This is a deliberately incorrect calculator **only while the opt-in setting is
enabled**. It is off by default, so the feature is safe to ship on `main`.

## Separate chaos setting

Preferences also has **Apply random bit flips** (default off). When enabled,
each evaluation reads `/dev/urandom` to decide whether to flip one bit of the
expression before parsing — the byte and bit are random too. **Bit Flip
Frequency** sets the chance from 0–100% per evaluation (default 12%, matching
the original 1 in 8). The helper logic, threshold mapping and UTF-8 guard live
in `crates/radix-engine/src/chaos.rs`.
