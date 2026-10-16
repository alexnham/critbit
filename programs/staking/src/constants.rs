//! Initial character tiers. Token thresholds are whole tokens, not base units.
//! Percentages use basis points; the reward period remains to be decided.

pub const BASIS_POINTS_DENOMINATOR: u64 = 10_000;
pub const CHARACTER_TIERS: [(u8, u64, u16); 3] = [
    (1, 1_000, 500),  // Level 1: 5%
    (2, 5_000, 750),  // Level 2: 7.5%
    (3, 10_000, 1_000), // Level 3: 10%
];
