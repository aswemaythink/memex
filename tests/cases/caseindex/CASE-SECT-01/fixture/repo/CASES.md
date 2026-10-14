# Section cases

## Stability
- `SECT-001` Class names are stable across runs and platforms.
  Hash input is the canonical atom string, never object identity.
  Evidence: `src/hash.rs`
