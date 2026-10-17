use crate::*;
#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 8 + Config::INIT_SPACE, seeds = [b"config", admin.key().as_ref()], bump)]
    pub config: Box<Account<'info, Config>>,
    pub system_program: Program<'info, System>,
}
pub fn handle_initialize_config(
    ctx: Context<InitializeConfig>,
    fee_bps: u16,
    protocol_share_bps: u16,
) -> Result<()> {
    require!(
        fee_bps < 10_000 && protocol_share_bps <= 10_000,
        AmmError::InvalidFee
    );
    ctx.accounts.config.set_inner(Config {
        admin: ctx.accounts.admin.key(),
        fee_bps,
        protocol_share_bps,
        bump: ctx.bumps.config,
    });
    Ok(())
}
