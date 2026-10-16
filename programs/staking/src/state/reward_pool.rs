//! Planned PDA: [b"reward_pool", stake_mint].
//! TODO: Store admin, stake/reward mints, mint decimals, PDA bumps, vault references,
//! total principal, reward liabilities, funding, and reward accounting policy.
//! Principal and rewards use separate vaults even when their mint is identical.
//! Reward duration, calculation basis, funding limits, and zero-stake behavior
//! must be specified before implementing accrual.
