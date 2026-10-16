# Critbit character staking

Structural scaffold only; no executable staking or Anchor account handlers yet.

| Character level | Total stake allocated | Reward rate |
| --- | ---: | ---: |
| Locked (0) | Below 1,000 tokens | 0% |
| 1 | 1,000 tokens | 5% |
| 2 | 5,000 tokens | 7.5% |
| 3 | 10,000 tokens | 10% |

Initial proposed behavior: thresholds apply to the total stake in each character
position. Reaching level 3 requires 10,000 total tokens, rather than 16,000.
Every character uses these tiers initially. Each wallet has one position per
character per reward pool, with separately allocated principal. Deposits unlock
or upgrade the selected character; withdrawals downgrade or lock it according
to its remaining allocation. Unlocks are therefore stake-backed, not permanent.
These lifecycle assumptions should be confirmed before implementation.

The program records each character's level on-chain for the game to read.
Game integration and custom integration tests remain empty boilerplate.

## Accounts

- Reward pool: authority, mints, principal/reward vaults, and reward funding.
- Character configuration: character ID, enabled flag, and three staking tiers.
- Character position: owner/character references, allocated stake, level,
  reward checkpoint, and pending rewards.

Rust structures are sketches, not final serialized Anchor account layouts.
Rates use basis points; token transfers and thresholds use mint base units.
Principal and funded reward tokens must remain separately accounted for.

## Decisions required before implementation

- Are the rates annual APR, or do they apply over a different period?
- Are rewards calculated on the entire allocated stake or capped at its tier threshold?
- Can wallets unlock every character, and what proves eligibility for a character?
- Are withdrawals immediate, or subject to a lock/cooldown?
- How are emissions bounded when available reward funding runs low?

Rewards must be settled before any stake or rate change. Admin reclaim may only
withdraw provable unused rewards after reserving earned obligations; principal
is never reclaimable by the admin.
