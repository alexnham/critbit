# Program building

Structural scaffold for two Anchor programs: `amm` and `staking`.

- `programs/amm/src`: pool state, liquidity, swaps, fees, and math.
- `programs/staking/src`: Critbit character unlocks, tiered staking, funded rewards, claims, and reclaim. See `programs/staking/README.md`.
- `tests/amm` and `tests/staking`: program test placeholders.
- `tests/integration/custom.rs`: empty custom integration boilerplate.

This scaffold contains no business logic. Anchor/Cargo manifests, program IDs, dependencies, and deployment configuration will be added when initializing the buildable workspace.
# critbit
