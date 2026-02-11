use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::GigCancelled;
use crate::state::{GigEscrow, GigStatus};
use crate::utils;

#[derive(Accounts)]
pub struct CancelGig<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, gig_escrow.gig_id.as_bytes()],
        bump = gig_escrow.bump,
        close = poster
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    #[account(mut)]
    pub poster: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CancelGig>) -> Result<()> {
    let gig_escrow = &ctx.accounts.gig_escrow;

    // Only poster can cancel
    require!(
        gig_escrow.is_poster(&ctx.accounts.poster.key()),
        TendaError::NotPoster
    );

    // Can only cancel if status is Open
    require!(
        gig_escrow.status.can_cancel(),
        TendaError::InvalidGigStatus
    );

    // Emit event before closing account
    emit!(GigCancelled {
        gig_id: gig_escrow.gig_id.clone(),
        poster: ctx.accounts.poster.key(),
        refund_amount: gig_escrow.total_locked,
        timestamp: utils::current_timestamp()?,
    });

    msg!(
        "Gig cancelled, {} lamports refunded to poster",
        gig_escrow.total_locked
    );

    // Account closure and rent refund handled automatically by Anchor
    Ok(())
}
