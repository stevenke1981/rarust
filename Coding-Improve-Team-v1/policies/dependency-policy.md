# Dependency Policy

## Rule

Do not add dependencies unless necessary.

## Before Adding

- Check if standard library or existing dependency can solve it.
- Check license and maintenance risk.
- Record why it is needed.
- Update lockfile only when dependency actually changes.

## Forbidden

- Adding abandoned dependencies without reason.
- Adding heavy dependencies for tiny utility.
- Adding network-facing dependency without security review.
