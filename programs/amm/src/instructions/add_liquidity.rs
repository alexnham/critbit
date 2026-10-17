use crate::*;
#[derive(Accounts)]
pub struct Liquidity<'info> {
    pub user: Signer<'info>,
    #[account(seeds = [b"pool", pool.config.as_ref(), pool.mint_a.as_ref(), pool.mint_b.as_ref()], bump = pool.bump, has_one = mint_a, has_one = mint_b, has_one = vault_a, has_one = vault_b, has_one = lp_mint)]
    pub pool: Box<Account<'info, Pool>>,
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint_a, token::authority = pool)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = pool)]
    pub vault_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, mint::authority = pool)]
    pub lp_mint: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint_a, token::authority = user)]
    pub user_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = user)]
    pub user_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = lp_mint, token::authority = user)]
    pub user_lp: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
pub fn handle_add_liquidity(
    ctx: Context<Liquidity>,
    shares: u64,
    max_a: u64,
    max_b: u64,
    deadline: i64,
) -> Result<()> {
    check_deadline(deadline)?;
    require!(shares > 0, AmmError::ZeroAmount);
    let a = ctx.accounts;
    let amount_a = math::fees::up(
        shares,
        math::fees::reserves(a.vault_a.amount, a.pool.protocol_fees_a)?,
        a.lp_mint.supply,
    )?;
    let amount_b = math::fees::up(
        shares,
        math::fees::reserves(a.vault_b.amount, a.pool.protocol_fees_b)?,
        a.lp_mint.supply,
    )?;
    require!(
        amount_a > 0 && amount_b > 0,
        AmmError::InsufficientLiquidity
    );
    require!(amount_a <= max_a && amount_b <= max_b, AmmError::Slippage);
    transfer(
        &a.token_program,
        a.user_a.to_account_info(),
        a.vault_a.to_account_info(),
        a.user.to_account_info(),
        &a.mint_a,
        amount_a,
        &[],
    )?;
    transfer(
        &a.token_program,
        a.user_b.to_account_info(),
        a.vault_b.to_account_info(),
        a.user.to_account_info(),
        &a.mint_b,
        amount_b,
        &[],
    )?;
    let p = &a.pool;
    let seeds: &[&[u8]] = &[
        b"pool",
        p.config.as_ref(),
        p.mint_a.as_ref(),
        p.mint_b.as_ref(),
        &[p.bump],
    ];
    token::mint_to(
        CpiContext::new(
            a.token_program.key(),
            MintTo {
                mint: a.lp_mint.to_account_info(),
                to: a.user_lp.to_account_info(),
                authority: p.to_account_info(),
            },
        )
        .with_signer(&[seeds]),
        shares,
    )?;
    emit!(LiquidityChanged {
        pool: p.key(),
        user: a.user.key(),
        adding: true,
        amount_a,
        amount_b,
        shares
    });
    Ok(())
}
