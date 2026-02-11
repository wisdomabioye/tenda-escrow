use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::DisputeOpened;
use crate::state::{GigEscrow, GigStatus};
use crate::utils;

#[derive(Accounts)]
pub struct DisputeGig<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, gig_escrow.gig_id.as_bytes()],
        bump = gig_escrow.bump
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    pub initiator: Signer<'info>,
}

pub fn handler(ctx: Context<DisputeGig>, reason: String) -> Result<()> {
    let gig_escrow = &mut ctx.accounts.gig_escrow;

    // Validate reason length
    require!(
        reason.len() <= MAX_DISPUTE_REASON_LEN,
        TendaError::DisputeReasonTooLong
    );

    // Check gig status - can only dispute Accepted or Submitted
    require!(
        gig_escrow.status.can_dispute(),
        TendaError::CannotDispute
    );

    // Check if initiator is poster or worker
    let is_poster = gig_escrow.is_poster(&ctx.accounts.initiator.key());
    let is_worker = gig_escrow.is_worker(&ctx.accounts.initiator.key());

    require!(
        is_poster || is_worker,
        TendaError::NotAuthorizedToDispute
    );

    // Update status
    gig_escrow.status = GigStatus::Disputed;

    emit!(DisputeOpened {
        gig_id: gig_escrow.gig_id.clone(),
        initiator: ctx.accounts.initiator.key(),
        reason: reason.clone(),
        timestamp: utils::current_timestamp()?,
    });

    msg!(
        "Dispute opened for gig {} by {}",
        gig_escrow.gig_id,
        ctx.accounts.initiator.key()
    );

    Ok(())
}
