pub const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;
pub const BASIS_POINTS_DENOMINATOR: u64 = 10_000;
pub const MAX_DURATION: u64 = 5 * SECONDS_PER_YEAR;
pub const CHARACTER_TIERS: [(u8, u64, u16); 3] =
    [(1, 1_000, 500), (2, 5_000, 750), (3, 10_000, 1_000)];
