# Critbit character staking

A user allocates tokens to a registered character to unlock its level. Each wallet
has one position per character per pool. All characters initially use the same
immutable tiers: 1,000 tokens at 5%, 5,000 at 7.5%, and 10,000 at 10%.
Thresholds are total allocations, so level 3 needs 10,000 tokens, not 16,000.
Below 1,000 tokens the position is locked and earns no new rewards.

## Implemented reward policy

Rates are annual simple APR using a 365-day year, applied to the entire allocated
balance at the current tier. Rewards are denominated in the same token as stake.
Fractional base-unit rewards carry across stakes, withdrawals, and claims. Users
must explicitly stake claimed rewards to compound them. These economic defaults
can be reviewed before deployment.

Each pool sets an immutable reward end timestamp at initialization, with a duration
between one second and five years. Accrual stops at that timestamp. There is one
pool per admin and mint in this initial version; renewal or additional campaigns
under the same authority would require an explicit versioned campaign design.

Every stake reserves its maximum remaining reward through campaign end. The
instruction fails atomically if the reward vault cannot cover all positions'
accrued and future obligations. Upgrades reserve the increased rate for the
remaining time only. Withdrawals release unused future reservations. Claims reduce
both the reward balance and liability. Unclaimed earned rewards remain reserved
indefinitely, including after campaign end. Admin reclaim can only take surplus.

## Instructions

1. `initialize_pool(duration_seconds)` creates principal and reward vaults for an
   SPL Token mint with up to nine decimals.
2. `register_character(character_id)` registers immutable tiers; only pool admin
   can register characters. Character IDs are `u32` values selected by the game.
3. `fund_rewards(amount)` transfers reward funding; anyone may fund a pool.
4. `open_position()` creates a user's position for a registered character.
5. `stake(amount)` deposits tokens and updates the unlocked level; no deposits
   are accepted after campaign end.
6. `unstake(amount)` settles rewards at the old level, withdraws principal, and
   updates the level. Withdrawals are immediate, including after campaign end.
7. `claim()` pays all whole base units of earned rewards without changing stake.
8. `reclaim_unused_rewards(amount)` returns unreserved rewards to pool admin.
9. `close_position()` returns rent to the owner once principal and payable rewards
   are zero. Closing explicitly forfeits fractional dust smaller than one base unit.

The client can compose opening a position and staking in one transaction. All
amounts passed to instructions are base units. A six-decimal mint requires
1,000,000,000 base units to reach level 1.

## Accounts and access

- Reward pool PDA: `["reward_pool", admin, mint]` owns both vaults and records
  total principal and reserved rewards.
- Character PDA: `["character", pool, character_id_le_u32]` records its fixed tiers.
- Position PDA: `["position", pool, owner, character]` records principal, level,
  pending rewards, fractional remainder, timestamp, and reserved rewards.
- Vault PDAs: `["stake_vault", pool]` and `["reward_vault", pool]`.

Any wallet may unlock any registered character by allocating its own stake. There
is no NFT ownership requirement or character scarcity rule. The on-chain position
is the source of truth for the game, but game integration remains unimplemented.
The pool admin cannot spend principal. Mint freeze authorities and program upgrade
authorities retain their normal Solana powers and are separate from pool admin.
