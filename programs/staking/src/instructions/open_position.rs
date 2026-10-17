use crate::*;
#[derive(Accounts)]
pub struct OpenPosition<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    pub pool: Box<Account<'info, RewardPool>>,
    #[account(has_one = pool)]
    pub character: Box<Account<'info, CharacterConfig>>,
    #[account(init, payer = owner, space = 8 + CharacterPosition::INIT_SPACE, seeds = [b"position", pool.key().as_ref(), owner.key().as_ref(), character.key().as_ref()], bump)]
    pub position: Box<Account<'info, CharacterPosition>>,
    pub system_program: Program<'info, System>,
}
pub fn handle_open_position(ctx: Context<OpenPosition>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(
        now < ctx.accounts.pool.end_timestamp,
        StakingError::CampaignEnded
    );
    let a = ctx.accounts;
    a.position.set_inner(CharacterPosition {
        pool: a.pool.key(),
        owner: a.owner.key(),
        character: a.character.key(),
        staked_base_units: 0,
        level: 0,
        reward_rate_bps: 0,
        pending_rewards_base_units: 0,
        last_reward_timestamp: now,
        reward_remainder: 0,
        reserved_rewards: 0,
        bump: ctx.bumps.position,
    });
    Ok(())
}
