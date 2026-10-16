//! Register a Critbit character with the configured level thresholds and rates.
//! TODO: Require pool admin and validate unique character ID and monotonic tiers.
//! Initial tiers: 1,000 / 5,000 / 10,000 whole tokens; 500 / 750 / 1,000 bps.
//! Convert whole-token thresholds to mint base units with checked arithmetic.
//! Existing positions must settle earned rewards before any rate changes apply.
