use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::TendaError;
use crate::events::DisputeResolved;
use crate::state::{GigEscrow, GigStatus, PlatformState, UserAccount};
use crate::utils;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DisputeWinner {
    Poster,
    Worker,
    Split,
}

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(
        mut,
        seeds = [ESCROW_SEED, gig_escrow.gig_id.as_bytes()],
        bump = gig_escrow.bump,
        close = poster
    )]
    pub gig_escrow: Account<'info, GigEscrow>,

    #[account(
        seeds = [PLATFORM_SEED],
        bump,
        constraint = platform_state.admin == admin.key() @ TendaError::NotAdmin
    )]
    pub platform_state: Account<'info, PlatformState>,

    #[account(
        mut,
        seeds = [USER_SEED, worker.key().as_ref()],
        bump
    )]
    pub worker_account: Account<'info, UserAccount>,

    pub admin: Signer<'info>,

    /// CHECK: Poster receiving potential refund
    #[account(
        mut,
        constraint = poster.key() == gig_escrow.poster
    )]
    pub poster: UncheckedAccount<'info>,

    /// CHECK: Worker receiving potential payment
    #[account(
        mut,
        constraint = Some(worker.key()) == gig_escrow.worker
    )]
    pub worker: UncheckedAccount<'info>,

    /// CHECK: Platform treasury
    #[account(
        mut,
        constraint = treasury.key() == platform_state.treasury
    )]
    pub treasury: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ResolveDispute>, winner: DisputeWinner) -> Result<()> {
    let gig_escrow = &ctx.accounts.gig_escrow;

    // Check if gig is disputed
    require!(
        gig_escrow.status == GigStatus::Disputed,
        TendaError::GigNotDisputed
    );

    let payment_amount = gig_escrow.payment_amount;
    let platform_fee = gig_escrow.platform_fee;
    let current_time = utils::current_timestamp()?;

    let (poster_payout, worker_payout) = match winner {
        DisputeWinner::Worker => {
            // Worker wins - full payment + platform keeps fee
            **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= payment_amount;
            **ctx.accounts.worker.try_borrow_mut_lamports()? += payment_amount;

            **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= platform_fee;
            **ctx.accounts.treasury.try_borrow_mut_lamports()? += platform_fee;

            // Update worker account
            ctx.accounts.worker_account.add_earnings(payment_amount)?;
            ctx.accounts.worker_account.increment_completed_gigs();

            (0, payment_amount)
        }
        DisputeWinner::Poster => {
            // Poster wins - payment refunded; platform keeps the fee.
            // The platform provided a real service (escrow, marketplace, dispute resolution)
            // regardless of outcome. Keeping the fee in all three outcomes also makes the
            // admin financially indifferent between winner choices, removing any incentive
            // to favour the worker.
            **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= payment_amount;
            **ctx.accounts.poster.try_borrow_mut_lamports()? += payment_amount;

            **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= platform_fee;
            **ctx.accounts.treasury.try_borrow_mut_lamports()? += platform_fee;

            (payment_amount, 0)
        }
        DisputeWinner::Split => {
            // 50/50 split of payment, platform keeps fee.
            // If payment_amount is odd, the remainder (1 lamport) goes to poster via `close = poster`.
            let half_payment = payment_amount / 2;

            **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= half_payment;
            **ctx.accounts.worker.try_borrow_mut_lamports()? += half_payment;

            **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= half_payment;
            **ctx.accounts.poster.try_borrow_mut_lamports()? += half_payment;

            **ctx.accounts.gig_escrow.to_account_info().try_borrow_mut_lamports()? -= platform_fee;
            **ctx.accounts.treasury.try_borrow_mut_lamports()? += platform_fee;

            // Update worker account with half
            ctx.accounts.worker_account.add_earnings(half_payment)?;
            ctx.accounts.worker_account.increment_completed_gigs();

            (half_payment, half_payment)
        }
    };

    emit!(DisputeResolved {
        gig_id: gig_escrow.gig_id.clone(),
        winner: format!("{:?}", winner),
        poster_payout,
        worker_payout,
        platform_fee,
        timestamp: current_time,
    });

    msg!(
        "Dispute resolved for gig {}, winner: {:?}",
        gig_escrow.gig_id,
        winner
    );

    // Account closure handled automatically by Anchor
    Ok(())
}
