use crate::*;
#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(seeds = [b"config", config.admin.as_ref()], bump = config.bump, has_one = admin)]
    pub config: Box<Account<'info, Config>>,
    pub admin: Signer<'info>,
    pub mint_a: Box<Account<'info, Mint>>,
    #[account(constraint = mint_a.key() < mint_b.key() @ AmmError::InvalidMints)]
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(init, payer = creator, space = 8 + Pool::INIT_SPACE, seeds = [b"pool", config.key().as_ref(), mint_a.key().as_ref(), mint_b.key().as_ref()], bump)]
    pub pool: Box<Account<'info, Pool>>,
    #[account(init, payer = creator, seeds = [b"vault_a", pool.key().as_ref()], bump, token::mint = mint_a, token::authority = pool)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = creator, seeds = [b"vault_b", pool.key().as_ref()], bump, token::mint = mint_b, token::authority = pool)]
    pub vault_b: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = creator, seeds = [b"lp_mint", pool.key().as_ref()], bump, mint::decimals = 9, mint::authority = pool)]
    pub lp_mint: Box<Account<'info, Mint>>,
    #[account(init, payer = creator, seeds = [b"locked_lp", pool.key().as_ref()], bump, token::mint = lp_mint, token::authority = pool)]
    pub locked_lp: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = creator, seeds = [b"initial_lp", pool.key().as_ref(), creator.key().as_ref()], bump, token::mint = lp_mint, token::authority = creator)]
    pub creator_lp: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_a, token::authority = creator)]
    pub source_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = creator)]
    pub source_b: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
pub fn handle_create_pool(
    ctx: Context<CreatePool>,
    amount_a: u64,
    amount_b: u64,
    min_shares: u64,
    deadline: i64,
) -> Result<()> {
    check_deadline(deadline)?;
    let shares = math::liquidity::initial_shares(amount_a, amount_b)?;
    require!(shares >= min_shares, AmmError::Slippage);
    let a = ctx.accounts;
    a.pool.set_inner(Pool {
        config: a.config.key(),
        mint_a: a.mint_a.key(),
        mint_b: a.mint_b.key(),
        vault_a: a.vault_a.key(),
        vault_b: a.vault_b.key(),
        lp_mint: a.lp_mint.key(),
        protocol_fees_a: 0,
        protocol_fees_b: 0,
        bump: ctx.bumps.pool,
    });
    transfer(
        &a.token_program,
        a.source_a.to_account_info(),
        a.vault_a.to_account_info(),
        a.creator.to_account_info(),
        &a.mint_a,
        amount_a,
        &[],
    )?;
    transfer(
        &a.token_program,
        a.source_b.to_account_info(),
        a.vault_b.to_account_info(),
        a.creator.to_account_info(),
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
    for (destination, amount) in [
        (a.creator_lp.to_account_info(), shares),
        (
            a.locked_lp.to_account_info(),
            math::liquidity::MINIMUM_LIQUIDITY,
        ),
    ] {
        token::mint_to(
            CpiContext::new(
                a.token_program.key(),
                MintTo {
                    mint: a.lp_mint.to_account_info(),
                    to: destination,
                    authority: p.to_account_info(),
                },
            )
            .with_signer(&[seeds]),
            amount,
        )?;
    }
    emit!(LiquidityChanged {
        pool: p.key(),
        user: a.creator.key(),
        adding: true,
        amount_a,
        amount_b,
        shares
    });
    Ok(())
}
