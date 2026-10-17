use crate::*;
pub fn handle_claim(ctx: Context<ManagePosition>) -> Result<()> {
    let a = ctx.accounts;
    a.position
        .settle(Clock::get()?.unix_timestamp, a.pool.end_timestamp)?;
    let amount = a.position.pending_rewards_base_units;
    require!(amount > 0, StakingError::ZeroAmount);
    a.position.pending_rewards_base_units = 0;
    let remaining = a
        .reward_vault
        .amount
        .checked_sub(amount)
        .ok_or(StakingError::InsufficientFunding)?;
    a.position.reserve(&mut a.pool, &a.character, remaining)?;
    let p = &a.pool;
    let seeds: &[&[u8]] = &[b"reward_pool", p.admin.as_ref(), p.mint.as_ref(), &[p.bump]];
    transfer(
        &a.token_program,
        a.reward_vault.to_account_info(),
        a.user_tokens.to_account_info(),
        p.to_account_info(),
        &a.mint,
        amount,
        &[seeds],
    )?;
    emit!(RewardsClaimed {
        pool: p.key(),
        owner: a.owner.key(),
        character: a.character.key(),
        amount
    });
    Ok(())
}
