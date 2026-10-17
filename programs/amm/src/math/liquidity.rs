use crate::errors::AmmError;
use anchor_lang::prelude::*;
pub const MINIMUM_LIQUIDITY: u64 = 1_000;
pub fn initial_shares(a: u64, b: u64) -> Result<u64> {
    require!(a > 0 && b > 0, AmmError::ZeroAmount);
    let n = u128::from(a) * u128::from(b);
    // Integer Newton iteration, rounded down.
    let mut x = n;
    let mut y = x / 2 + 1;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    let total = u64::try_from(x).map_err(|_| error!(AmmError::Math))?;
    require!(total > MINIMUM_LIQUIDITY, AmmError::InsufficientLiquidity);
    Ok(total - MINIMUM_LIQUIDITY)
}
