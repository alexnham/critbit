use crate::*;
#[derive(Accounts)]
pub struct ManagePosition<'info> {
    pub owner: Signer<'info>,
    #[account(mut, seeds = [b"reward_pool", pool.admin.as_ref(), pool.mint.as_ref()], bump = pool.bump, has_one = mint, has_one = stake_vault, has_one = reward_vault)]
    pub pool: Box<Account<'info, RewardPool>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(has_one = pool)]
    pub character: Box<Account<'info, CharacterConfig>>,
    #[account(mut, seeds = [b"position", pool.key().as_ref(), owner.key().as_ref(), character.key().as_ref()], bump = position.bump, has_one = owner, has_one = pool, has_one = character)]
    pub position: Box<Account<'info, CharacterPosition>>,
    #[account(mut, token::mint = mint, token::authority = pool)]
    pub stake_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint, token::authority = pool)]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint, token::authority = owner)]
    pub user_tokens: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
pub fn handle_stake(ctx: Context<ManagePosition>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);
    let now = Clock::get()?.unix_timestamp;
    let a = ctx.accounts;
    require!(now < a.pool.end_timestamp, StakingError::CampaignEnded);
    a.position.settle(now, a.pool.end_timestamp)?;
    a.position.staked_base_units = a
        .position
        .staked_base_units
        .checked_add(amount)
        .ok_or(StakingError::Math)?;
    a.pool.total_staked = a
        .pool
        .total_staked
        .checked_add(amount)
        .ok_or(StakingError::Math)?;
    a.position
        .reserve(&mut a.pool, &a.character, a.reward_vault.amount)?;
    transfer(
        &a.token_program,
        a.user_tokens.to_account_info(),
        a.stake_vault.to_account_info(),
        a.owner.to_account_info(),
        &a.mint,
        amount,
        &[],
    )?;
    emit!(CharacterStakeChanged {
        pool: a.pool.key(),
        owner: a.owner.key(),
        character: a.character.key(),
        stake: a.position.staked_base_units,
        level: a.position.level
    });
    Ok(())
}
