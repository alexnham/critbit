use crate::*;
#[derive(Accounts)]
pub struct CollectProtocolFees<'info> {
    pub admin: Signer<'info>,
    #[account(has_one = admin)]
    pub config: Box<Account<'info, Config>>,
    #[account(mut, seeds = [b"pool", pool.config.as_ref(), pool.mint_a.as_ref(), pool.mint_b.as_ref()], bump = pool.bump, has_one = config, has_one = mint_a, has_one = mint_b, has_one = vault_a, has_one = vault_b)]
    pub pool: Box<Account<'info, Pool>>,
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint_a, token::authority = pool)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = pool)]
    pub vault_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_a, token::authority = admin)]
    pub destination_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = admin)]
    pub destination_b: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
pub fn handle_collect_protocol_fees(ctx: Context<CollectProtocolFees>) -> Result<()> {
    let a = ctx.accounts;
    let p = &a.pool;
    let seeds: &[&[u8]] = &[
        b"pool",
        p.config.as_ref(),
        p.mint_a.as_ref(),
        p.mint_b.as_ref(),
        &[p.bump],
    ];
    transfer(
        &a.token_program,
        a.vault_a.to_account_info(),
        a.destination_a.to_account_info(),
        p.to_account_info(),
        &a.mint_a,
        p.protocol_fees_a,
        &[seeds],
    )?;
    transfer(
        &a.token_program,
        a.vault_b.to_account_info(),
        a.destination_b.to_account_info(),
        p.to_account_info(),
        &a.mint_b,
        p.protocol_fees_b,
        &[seeds],
    )?;
    a.pool.protocol_fees_a = 0;
    a.pool.protocol_fees_b = 0;
    Ok(())
}
