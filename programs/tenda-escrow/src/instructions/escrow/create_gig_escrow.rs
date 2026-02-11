use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::GigCreated;
use crate::state::{GigEscrow, GigStatus, PlatformState};
use crate::utils;

#[derive(Accounts)]
#[instruction(gig_id: String)]
pub struct CreateGigEscrow<'info> {
    #[account(
        init,
        payer = poster,
        space = GigEscrow::LEN,
        seeds = [ESCROW_SEED, gig_id.as_bytes()],
        bump
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    #[account(
        mut,
        seeds = [PLATFORM_SEED],
        bump
    )]
    pub platform_state: Account<'info, PlatformState>,

    #[account(mut)]
    pub poster: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<CreateGigEscrow>,
    gig_id: String,
    payment_amount: u64,
    deadline: i64,
) -> Result<()> {
    // Validate inputs
    require!(
        gig_id.len() <= MAX_GIG_ID_LEN,
        TendaError::GigIdTooLong
    );

    require!(
        payment_amount >= MIN_PAYMENT,
        TendaError::PaymentTooLow
    );

    let current_time = utils::current_timestamp()?;
    require!(
        deadline > current_time,
        TendaError::InvalidDeadline
    );

    // Calculate platform fee
    let platform_fee = utils::calculate_platform_fee(
        payment_amount,
        ctx.accounts.platform_state.platform_fee_bps,
    )?;

    let total_locked = payment_amount
        .checked_add(platform_fee)
        .ok_or(TendaError::ArithmeticOverflow)?;

    // Transfer SOL from poster to escrow PDA
    utils::transfer_sol(
        &ctx.accounts.poster.to_account_info(),
        &ctx.accounts.gig_escrow.to_account_info(),
        total_locked,
        &ctx.accounts.system_program.to_account_info(),
    )?;

    // Initialize escrow account
    let gig_escrow = &mut ctx.accounts.gig_escrow;
    gig_escrow.gig_id = gig_id.clone();
    gig_escrow.poster = ctx.accounts.poster.key();
    gig_escrow.worker = None;
    gig_escrow.payment_amount = payment_amount;
    gig_escrow.platform_fee = platform_fee;
    gig_escrow.total_locked = total_locked;
    gig_escrow.created_at = current_time;
    gig_escrow.deadline = deadline;
    gig_escrow.accepted_at = None;
    gig_escrow.submitted_at = None;
    gig_escrow.completed_at = None;
    gig_escrow.status = GigStatus::Open;
    gig_escrow.bump = ctx.bumps.gig_escrow;

    // Update platform stats
    let platform_state = &mut ctx.accounts.platform_state;
    platform_state.total_gigs = platform_state.total_gigs
        .checked_add(1)
        .ok_or(TendaError::ArithmeticOverflow)?;

    emit!(GigCreated {
        gig_id,
        poster: ctx.accounts.poster.key(),
        payment_amount,
        platform_fee,
        deadline,
        timestamp: current_time,
    });

    msg!(
        "Gig created with {} lamports locked in escrow",
        total_locked
    );

    Ok(())
}
