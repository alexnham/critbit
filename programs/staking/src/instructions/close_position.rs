use crate::*;
#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub pool: Box<Account<'info, RewardPool>>,
    #[account(mut, close = owner, seeds = [b"position", pool.key().as_ref(), owner.key().as_ref(), position.character.as_ref()], bump = position.bump, has_one = pool, has_one = owner)]
    pub position: Box<Account<'info, CharacterPosition>>,
}
pub fn handle_close_position(ctx: Context<ClosePosition>) -> Result<()> {
    let a = ctx.accounts;
    a.position
        .settle(Clock::get()?.unix_timestamp, a.pool.end_timestamp)?;
    require!(
        a.position.staked_base_units == 0 && a.position.pending_rewards_base_units == 0,
        StakingError::PositionNotEmpty
    );
    // Closing explicitly forfeits less than one base unit of fractional dust.
    a.pool.reserved_rewards = a
        .pool
        .reserved_rewards
        .checked_sub(a.position.reserved_rewards)
        .ok_or(StakingError::Math)?;
    Ok(())
}
