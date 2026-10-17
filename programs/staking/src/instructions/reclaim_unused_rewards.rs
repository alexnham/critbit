use crate::*;
#[derive(Accounts)]
pub struct ReclaimUnusedRewards<'info> {
    pub admin: Signer<'info>,
    #[account(seeds = [b"reward_pool", pool.admin.as_ref(), pool.mint.as_ref()], bump = pool.bump, has_one = admin, has_one = mint, has_one = reward_vault)]
    pub pool: Box<Account<'info, RewardPool>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint, token::authority = pool)]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint, token::authority = admin)]
    pub destination: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
pub fn handle_reclaim_unused_rewards(
    ctx: Context<ReclaimUnusedRewards>,
    amount: u64,
) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);
    let a = ctx.accounts;
    let free = a
        .reward_vault
        .amount
        .checked_sub(a.pool.reserved_rewards)
        .ok_or(StakingError::Math)?;
    require!(amount <= free, StakingError::InsufficientFunding);
    let p = &a.pool;
    let seeds: &[&[u8]] = &[b"reward_pool", p.admin.as_ref(), p.mint.as_ref(), &[p.bump]];
    transfer(
        &a.token_program,
        a.reward_vault.to_account_info(),
        a.destination.to_account_info(),
        p.to_account_info(),
        &a.mint,
        amount,
        &[seeds],
    )
}
