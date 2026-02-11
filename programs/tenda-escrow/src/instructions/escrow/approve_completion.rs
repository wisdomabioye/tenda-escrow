use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::GigCompleted;
use crate::state::{GigEscrow, GigStatus, PlatformState, UserAccount};
use crate::utils;

#[derive(Accounts)]
pub struct ApproveCompletion<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, gig_escrow.gig_id.as_bytes()],
        bump = gig_escrow.bump,
        close = poster
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump
    )]
    pub platform_state: Account<'info, PlatformState>,

    #[account(
        mut,
        seeds = [USER_SEED, worker.key().as_ref()],
        bump
    )]
    pub worker_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub poster: Signer<'info>,

    /// CHECK: Worker receiving payment
    #[account(mut)]
    pub worker: UncheckedAccount<'info>,

    /// CHECK: Platform treasury
    #[account(
        mut,
        constraint = treasury.key() == platform_state.treasury
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ApproveCompletion>) -> Result<()> {
    let gig_escrow = &ctx.accounts.gig_escrow;

    // Only poster can approve
    require!(
        gig_escrow.is_poster(&ctx.accounts.poster.key()),
        TendaError::NotPoster
    );

    // Check status
    require!(
        gig_escrow.status.can_approve(),
        TendaError::InvalidGigStatus
    );

    // Verify worker matches
    require!(
        gig_escrow.worker == Some(ctx.accounts.worker.key()),
        TendaError::NotWorker
    );

    let payment_amount = gig_escrow.payment_amount;
    let platform_fee = gig_escrow.platform_fee;
    let current_time = utils::current_timestamp()?;

    // Transfer payment to worker
    **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= payment_amount;
    **ctx.accounts.worker.try_borrow_mut_lamports()? += payment_amount;

    // Transfer platform fee to treasury
    **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= platform_fee;
    **ctx.accounts.treasury.try_borrow_mut_lamports()? += platform_fee;

    // Update worker account
    let worker_account = &mut ctx.accounts.worker_account;
    worker_account.add_earnings(payment_amount)?;
    worker_account.increment_completed_gigs();

    // Update platform stats
    let platform_state = &mut ctx.accounts.platform_state;
    platform_state.total_volume = platform_state.total_volume
        .checked_add(payment_amount)
        .ok_or(TendaError::ArithmeticOverflow)?;

    emit!(GigCompleted {
        gig_id: gig_escrow.gig_id.clone(),
        poster: ctx.accounts.poster.key(),
        worker: ctx.accounts.worker.key(),
        payment_amount,
        platform_fee,
        timestamp: current_time,
    });

    msg!(
        "Gig {} completed, {} lamports paid to worker, {} fee to treasury",
        gig_escrow.gig_id,
        payment_amount,
        platform_fee
    );

    // Account closure handled automatically by Anchor
    Ok(())
}
