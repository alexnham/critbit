use crate::*;
#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(init, payer = admin, space = 8 + RewardPool::INIT_SPACE, seeds = [b"reward_pool", admin.key().as_ref(), mint.key().as_ref()], bump)]
    pub pool: Box<Account<'info, RewardPool>>,
    #[account(init, payer = admin, seeds = [b"stake_vault", pool.key().as_ref()], bump, token::mint = mint, token::authority = pool)]
    pub stake_vault: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = admin, seeds = [b"reward_vault", pool.key().as_ref()], bump, token::mint = mint, token::authority = pool)]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
pub fn handle_initialize_pool(ctx: Context<InitializePool>, duration_seconds: u64) -> Result<()> {
    require!(
        duration_seconds > 0
            && duration_seconds <= constants::MAX_DURATION
            && ctx.accounts.mint.decimals <= 9,
        StakingError::InvalidConfiguration
    );
    let a = ctx.accounts;
    a.pool.set_inner(RewardPool {
        admin: a.admin.key(),
        mint: a.mint.key(),
        stake_vault: a.stake_vault.key(),
        reward_vault: a.reward_vault.key(),
        end_timestamp: Clock::get()?
            .unix_timestamp
            .checked_add(duration_seconds as i64)
            .ok_or(StakingError::Math)?,
        total_staked: 0,
        reserved_rewards: 0,
        bump: ctx.bumps.pool,
    });
    Ok(())
}
