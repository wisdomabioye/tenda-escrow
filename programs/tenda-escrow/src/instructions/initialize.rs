use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::PlatformInitialized;
use crate::state::PlatformState;
use crate::utils;

#[derive(Accounts)]
pub struct InitializePlatform<'info> {
    #[account(
        init,
        payer = admin,
        space = PlatformState::LEN,
        seeds = [PLATFORM_SEED],
        bump
    )]
    pub platform_state: Account<'info, PlatformState>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Treasury can be any account
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializePlatform>,
    platform_fee_bps: u16,
    grace_period_seconds: i64,
) -> Result<()> {
    // Validate platform fee
    require!(
        platform_fee_bps <= MAX_PLATFORM_FEE_BPS,
        TendaError::PlatformFeeTooHigh
    );

    let platform_state = &mut ctx.accounts.platform_state;

    platform_state.admin = ctx.accounts.admin.key();
    platform_state.platform_fee_bps = platform_fee_bps;
    platform_state.treasury = ctx.accounts.treasury.key();
    platform_state.total_gigs = 0;
    platform_state.total_volume = 0;
    platform_state.grace_period_seconds = grace_period_seconds;

    emit!(PlatformInitialized {
        admin: ctx.accounts.admin.key(),
        platform_fee_bps,
        grace_period_seconds,
        timestamp: utils::current_timestamp()?,
    });

    msg!("Platform initialized with fee: {} bps", platform_fee_bps);

    Ok(())
}
