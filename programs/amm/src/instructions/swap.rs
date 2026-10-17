use crate::*;
#[derive(Accounts)]
pub struct Swap<'info> {
    pub user: Signer<'info>,
    #[account(mut, seeds = [b"pool", pool.config.as_ref(), pool.mint_a.as_ref(), pool.mint_b.as_ref()], bump = pool.bump, has_one = config, has_one = mint_a, has_one = mint_b, has_one = vault_a, has_one = vault_b)]
    pub pool: Box<Account<'info, Pool>>,
    pub config: Box<Account<'info, Config>>,
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(mut, token::mint = mint_a, token::authority = pool)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = pool)]
    pub vault_b: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_a, token::authority = user)]
    pub user_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, token::mint = mint_b, token::authority = user)]
    pub user_b: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
}
pub fn handle_swap(
    ctx: Context<Swap>,
    a_to_b: bool,
    input: u64,
    min_output: u64,
    deadline: i64,
) -> Result<()> {
    check_deadline(deadline)?;
    let a = ctx.accounts;
    let ra = math::fees::reserves(a.vault_a.amount, a.pool.protocol_fees_a)?;
    let rb = math::fees::reserves(a.vault_b.amount, a.pool.protocol_fees_b)?;
    let (ri, ro) = if a_to_b { (ra, rb) } else { (rb, ra) };
    let (output, protocol_fee) = math::constant_product::quote_swap(
        input,
        ri,
        ro,
        a.config.fee_bps,
        a.config.protocol_share_bps,
    )?;
    require!(output >= min_output, AmmError::Slippage);
    if a_to_b {
        a.pool.protocol_fees_a = a
            .pool
            .protocol_fees_a
            .checked_add(protocol_fee)
            .ok_or(AmmError::Math)?;
    } else {
        a.pool.protocol_fees_b = a
            .pool
            .protocol_fees_b
            .checked_add(protocol_fee)
            .ok_or(AmmError::Math)?;
    }
    let (source, input_vault, output_vault, destination, input_mint, output_mint) = if a_to_b {
        (
            &a.user_a, &a.vault_a, &a.vault_b, &a.user_b, &a.mint_a, &a.mint_b,
        )
    } else {
        (
            &a.user_b, &a.vault_b, &a.vault_a, &a.user_a, &a.mint_b, &a.mint_a,
        )
    };
    transfer(
        &a.token_program,
        source.to_account_info(),
        input_vault.to_account_info(),
        a.user.to_account_info(),
        input_mint,
        input,
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
    transfer(
        &a.token_program,
        output_vault.to_account_info(),
        destination.to_account_info(),
        p.to_account_info(),
        output_mint,
        output,
        &[seeds],
    )?;
    emit!(Swapped {
        pool: p.key(),
        user: a.user.key(),
        a_to_b,
        input,
        output,
        protocol_fee
    });
    Ok(())
}
