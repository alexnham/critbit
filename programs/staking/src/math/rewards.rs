//! Character reward math scaffold; no accrual formula implemented yet.
//! Rates: level 1 = 500 bps, level 2 = 750 bps, level 3 = 1,000 bps.
//! TODO: Decide rate period (e.g. annual), reward basis, and compounding policy.
//! Settle the previous stake and level before every balance or tier transition.
//! Use checked wide integer intermediates and an explicit rounding policy.
