use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::ProofSubmitted;
use crate::state::{GigEscrow, GigStatus, PlatformState};
use crate::utils;

#[derive(Accounts)]
pub struct SubmitProof<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, gig_escrow.gig_id.as_bytes()],
        bump = gig_escrow.bump
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    #[account(
        seeds = [PLATFORM_SEED],
        bump
    )]
    pub platform_state: Account<'info, PlatformState>,

    #[account(mut)]
    pub worker: Signer<'info>,
}

pub fn handler(ctx: Context<SubmitProof>) -> Result<()> {
    let gig_escrow = &mut ctx.accounts.gig_escrow;

    require!(
        gig_escrow.is_worker(&ctx.accounts.worker.key()),
        TendaError::NotWorker
    );

    require!(
        gig_escrow.status.can_submit(),
        TendaError::InvalidGigStatus
    );

    // completion_deadline is always Some once accepted — safe to unwrap
    let completion_deadline = gig_escrow.completion_deadline
        .ok_or(TendaError::InvalidGigStatus)?;

    let current_time = utils::current_timestamp()?;
    let grace_period = ctx.accounts.platform_state.grace_period_seconds;

    require!(
        current_time <= completion_deadline + grace_period,
        TendaError::SubmissionDeadlinePassed
    );

    gig_escrow.submitted_at = Some(current_time);
    gig_escrow.status       = GigStatus::Submitted;

    emit!(ProofSubmitted {
        gig_id: gig_escrow.gig_id.clone(),
        worker: ctx.accounts.worker.key(),
        timestamp: current_time,
    });

    msg!(
        "Proof submitted for gig {} by worker {}",
        gig_escrow.gig_id,
        ctx.accounts.worker.key()
    );

    Ok(())
}
