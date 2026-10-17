use crate::*;
pub fn handle_unstake(ctx: Context<ManagePosition>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);
    let a = ctx.accounts;
    a.position
        .settle(Clock::get()?.unix_timestamp, a.pool.end_timestamp)?;
    a.position.staked_base_units = a
        .position
        .staked_base_units
        .checked_sub(amount)
        .ok_or(StakingError::InsufficientStake)?;
    a.pool.total_staked = a
        .pool
        .total_staked
        .checked_sub(amount)
        .ok_or(StakingError::Math)?;
    a.position
        .reserve(&mut a.pool, &a.character, a.reward_vault.amount)?;
    let p = &a.pool;
    let seeds: &[&[u8]] = &[b"reward_pool", p.admin.as_ref(), p.mint.as_ref(), &[p.bump]];
    transfer(
        &a.token_program,
        a.stake_vault.to_account_info(),
        a.user_tokens.to_account_info(),
        p.to_account_info(),
        &a.mint,
        amount,
        &[seeds],
    )?;
    emit!(CharacterStakeChanged {
        pool: p.key(),
        owner: a.owner.key(),
        character: a.character.key(),
        stake: a.position.staked_base_units,
        level: a.position.level
    });
    Ok(())
}
