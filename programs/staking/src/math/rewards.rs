use crate::{constants::*, errors::StakingError};
use anchor_lang::prelude::*;
pub const DENOMINATOR: u128 = SECONDS_PER_YEAR as u128 * BASIS_POINTS_DENOMINATOR as u128;
fn numerator(stake: u64, rate: u16, seconds: u64, remainder: u128) -> Result<u128> {
    require!(remainder < DENOMINATOR, StakingError::Math);
    u128::from(stake)
        .checked_mul(rate.into())
        .and_then(|n| n.checked_mul(seconds.into()))
        .and_then(|n| n.checked_add(remainder))
        .ok_or_else(|| error!(StakingError::Math))
}
pub fn accrue(stake: u64, rate: u16, seconds: u64, remainder: u128) -> Result<(u64, u128)> {
    let n = numerator(stake, rate, seconds, remainder)?;
    Ok((
        u64::try_from(n / DENOMINATOR).map_err(|_| error!(StakingError::Math))?,
        n % DENOMINATOR,
    ))
}
pub fn reserve(stake: u64, rate: u16, seconds: u64, remainder: u128) -> Result<u64> {
    u64::try_from(numerator(stake, rate, seconds, remainder)?.div_ceil(DENOMINATOR))
        .map_err(|_| error!(StakingError::Math))
}
