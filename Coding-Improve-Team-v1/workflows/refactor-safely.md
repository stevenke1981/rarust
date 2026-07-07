# Safe Refactor Workflow

## Goal

改善內部結構，但不改外部行為。

## Preconditions

- Existing behavior understood.
- Baseline validation available.
- Refactor scope is small.

## Steps

1. Run baseline tests.
2. Choose one code smell.
3. Refactor minimal code.
4. Run same tests.
5. Compare behavior.
6. Record risk.

## Forbidden

- Feature changes.
- Public API changes without approval.
- Broad formatting mixed with logic changes.
