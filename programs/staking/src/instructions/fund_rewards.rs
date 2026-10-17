use crate::*;
#[derive(Accounts)]
pub struct FundRewards<'info> {
    pub funder: Signer<'info>,
    #[account(has_one = mint, has_one = reward_vault)]
    pub pool: Box<Account<'info, RewardPool>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint, token::authority = funder)]
    pub source: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint, token::authority = pool)]
    pub reward_vault: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
pub fn handle_fund_rewards(ctx: Context<FundRewards>, amount: u64) -> Result<()> {
    require!(amount > 0, StakingError::ZeroAmount);
    let a = ctx.accounts;
    transfer(
        &a.token_program,
        a.source.to_account_info(),
        a.reward_vault.to_account_info(),
        a.funder.to_account_info(),
        &a.mint,
        amount,
        &[],
    )
}
