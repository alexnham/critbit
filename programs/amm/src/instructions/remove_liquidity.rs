use crate::*;
pub fn handle_remove_liquidity(
    ctx: Context<Liquidity>,
    shares: u64,
    min_a: u64,
    min_b: u64,
    deadline: i64,
) -> Result<()> {
    check_deadline(deadline)?;
    require!(shares > 0, AmmError::ZeroAmount);
    let a = ctx.accounts;
    let amount_a = math::fees::down(
        shares,
        math::fees::reserves(a.vault_a.amount, a.pool.protocol_fees_a)?,
        a.lp_mint.supply,
    )?;
    let amount_b = math::fees::down(
        shares,
        math::fees::reserves(a.vault_b.amount, a.pool.protocol_fees_b)?,
        a.lp_mint.supply,
    )?;
    require!(
        amount_a > 0 && amount_b > 0,
        AmmError::InsufficientLiquidity
    );
    require!(amount_a >= min_a && amount_b >= min_b, AmmError::Slippage);
    token::burn(
        CpiContext::new(
            a.token_program.key(),
            Burn {
                mint: a.lp_mint.to_account_info(),
                from: a.user_lp.to_account_info(),
                authority: a.user.to_account_info(),
            },
        ),
        shares,
    )?;
    let p = &a.pool;
    let seeds: &[&[u8]] = &[
        b"pool",
        p.config.as_ref(),
        p.mint_a.as_ref(),
        p.mint_b.as_ref(),
        &[p.bump],
    ];
    transfer(
        &a.token_program,
        a.vault_a.to_account_info(),
        a.user_a.to_account_info(),
        p.to_account_info(),
        &a.mint_a,
        amount_a,
        &[seeds],
    )?;
    transfer(
        &a.token_program,
        a.vault_b.to_account_info(),
        a.user_b.to_account_info(),
        p.to_account_info(),
        &a.mint_b,
        amount_b,
        &[seeds],
    )?;
    emit!(LiquidityChanged {
        pool: p.key(),
        user: a.user.key(),
        adding: false,
        amount_a,
        amount_b,
        shares
    });
    Ok(())
}
