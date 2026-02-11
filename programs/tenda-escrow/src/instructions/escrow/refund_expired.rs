use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::GigExpired;
use crate::state::{GigEscrow, PlatformState};
use crate::utils;

#[derive(Accounts)]
pub struct RefundExpired<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, gig_escrow.gig_id.as_bytes()],
        bump = gig_escrow.bump,
        close = poster
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    #[account(
        seeds = [PLATFORM_SEED],
        bump
    )]
    pub platform_state: Account<'info, PlatformState>,

    /// CHECK: Poster receiving refund
    #[account(
        mut,
        constraint = poster.key() == gig_escrow.poster
    )]
    pub poster: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RefundExpired>) -> Result<()> {
    let gig_escrow = &ctx.accounts.gig_escrow;

    // Check if gig can be refunded (Open or Accepted status only)
    require!(
        gig_escrow.status.can_refund(),
        TendaError::InvalidGigStatus
    );

    // Cannot refund if proof has been submitted
    require!(
        gig_escrow.submitted_at.is_none(),
        TendaError::CannotRefundWithProof
    );

    // Check if grace period has passed
    let current_time = utils::current_timestamp()?;
    let grace_period = ctx.accounts.platform_state.grace_period_seconds;
    
    require!(
        gig_escrow.is_expired(current_time, grace_period),
        TendaError::GigNotExpired
    );

    let refund_amount = gig_escrow.total_locked;

    // Refund locked SOL to poster
    **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= refund_amount;
    **ctx.accounts.poster.try_borrow_mut_lamports()? += refund_amount;

    emit!(GigExpired {
        gig_id: gig_escrow.gig_id.clone(),
        poster: ctx.accounts.poster.key(),
        refund_amount,
        timestamp: current_time,
    });

    msg!(
        "Gig {} expired, {} lamports refunded to poster",
        gig_escrow.gig_id,
        refund_amount
    );

    // Account closure handled automatically by Anchor
    Ok(())
}
