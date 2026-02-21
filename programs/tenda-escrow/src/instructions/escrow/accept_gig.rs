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

    require!(
        gig_escrow.status.can_accept(),
        TendaError::InvalidGigStatus
    );

    require!(
        !gig_escrow.is_poster(&ctx.accounts.worker.key()),
        TendaError::CannotAcceptOwnGig
    );

    let current_time = utils::current_timestamp()?;

    // Enforce accept_deadline if set
    if let Some(deadline) = gig_escrow.accept_deadline {
        require!(
            current_time <= deadline,
            TendaError::AcceptDeadlinePassed
        );
    }

    // Compute completion_deadline = now + duration
    let completion_deadline = current_time
        .checked_add(gig_escrow.completion_duration_seconds as i64)
        .ok_or(TendaError::ArithmeticOverflow)?;

    gig_escrow.worker               = Some(ctx.accounts.worker.key());
    gig_escrow.accepted_at          = Some(current_time);
    gig_escrow.completion_deadline  = Some(completion_deadline);
    gig_escrow.status               = GigStatus::Accepted;

    emit!(GigAccepted {
        gig_id: gig_escrow.gig_id.clone(),
        poster: gig_escrow.poster,
        worker: ctx.accounts.worker.key(),
        completion_deadline,
        timestamp: current_time,
    });

    msg!(
        "Gig {} accepted by {}, must complete by {}",
        gig_escrow.gig_id,
        ctx.accounts.worker.key(),
        completion_deadline,
    );

    Ok(())
}
