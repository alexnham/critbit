use crate::{
    errors::AmmError,
    math::fees::{down, up},
};
use anchor_lang::prelude::*;
pub fn quote_swap(
    input: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u16,
    share_bps: u16,
) -> Result<(u64, u64)> {
    require!(input > 0, AmmError::ZeroAmount);
    require!(
        reserve_in > 0 && reserve_out > 0,
        AmmError::InsufficientLiquidity
    );
    require!(
        fee_bps < 10_000 && share_bps <= 10_000,
        AmmError::InvalidFee
    );
    let fee = up(input, fee_bps.into(), 10_000)?;
    let net = input.checked_sub(fee).ok_or(AmmError::Math)?;
    let denominator = reserve_in.checked_add(net).ok_or(AmmError::Math)?;
    let output = down(net, reserve_out, denominator)?;
    require!(
        output > 0 && output < reserve_out,
        AmmError::InsufficientLiquidity
    );
    Ok((output, down(fee, share_bps.into(), 10_000)?))
}
