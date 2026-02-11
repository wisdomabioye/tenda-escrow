use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::GigAccepted;
use crate::state::{GigEscrow, GigStatus, UserAccount};
use crate::utils;

#[derive(Accounts)]
pub struct AcceptGig<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, gig_escrow.gig_id.as_bytes()],
        bump = gig_escrow.bump
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    #[account(
        seeds = [USER_SEED, worker.key().as_ref()],
        bump
    )]
    pub worker_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub worker: Signer<'info>,
}

pub fn handler(ctx: Context<AcceptGig>) -> Result<()> {
    let gig_escrow = &mut ctx.accounts.gig_escrow;

    // Check gig status
    require!(
        gig_escrow.status.can_accept(),
        TendaError::InvalidGigStatus
    );

    // Cannot accept own gig
    require!(
        !gig_escrow.is_poster(&ctx.accounts.worker.key()),
        TendaError::CannotAcceptOwnGig
    );

    // Update escrow
    gig_escrow.worker = Some(ctx.accounts.worker.key());
    gig_escrow.accepted_at = Some(utils::current_timestamp()?);
    gig_escrow.status = GigStatus::Accepted;

    emit!(GigAccepted {
        gig_id: gig_escrow.gig_id.clone(),
        poster: gig_escrow.poster,
        worker: ctx.accounts.worker.key(),
        timestamp: gig_escrow.accepted_at.unwrap(),
    });

    msg!(
        "Gig {} accepted by worker {}",
        gig_escrow.gig_id,
        ctx.accounts.worker.key()
    );

    Ok(())
}
