use crate::errors::AmmError;
use anchor_lang::prelude::*;
pub const BPS: u128 = 10_000;
pub fn down(a: u64, b: u64, d: u64) -> Result<u64> {
    require!(d > 0, AmmError::Math);
    u64::try_from(u128::from(a) * u128::from(b) / u128::from(d)).map_err(|_| error!(AmmError::Math))
}
pub fn up(a: u64, b: u64, d: u64) -> Result<u64> {
    require!(d > 0, AmmError::Math);
    let n = u128::from(a) * u128::from(b);
    u64::try_from(n.div_ceil(u128::from(d))).map_err(|_| error!(AmmError::Math))
}
pub fn reserves(balance: u64, fees: u64) -> Result<u64> {
    balance
        .checked_sub(fees)
        .ok_or_else(|| error!(AmmError::Math))
}
