use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("7H6AAoghUCPAVA1WTEwpSmkiRfPHWrgFidZQPzbXzkes");

#[program]
pub mod tenda_escrow {
    use super::*;

    // ==================== PLATFORM INITIALIZATION ====================
    
    pub fn initialize_platform(
        ctx: Context<InitializePlatform>,
        platform_fee_bps: u16,
        grace_period_seconds: i64,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, platform_fee_bps, grace_period_seconds)
    }

    // ==================== USER MANAGEMENT ====================
    
    pub fn create_user_account(ctx: Context<CreateUserAccount>) -> Result<()> {
        instructions::user::create_user_account::handler(ctx)
    }

    pub fn airdrop_gas_subsidy(
        ctx: Context<AirdropGasSubsidy>,
        amount: u64,
    ) -> Result<()> {
        instructions::user::airdrop_gas_subsidy::handler(ctx, amount)
    }

    pub fn withdraw_earnings(
        ctx: Context<WithdrawEarnings>,
        amount: u64,
    ) -> Result<()> {
        instructions::user::withdraw_earnings::handler(ctx, amount)
    }

    // ==================== ESCROW LIFECYCLE ====================
    
    pub fn create_gig_escrow(
        ctx: Context<CreateGigEscrow>,
        gig_id: String,
        payment_amount: u64,
        completion_duration_seconds: u64,
        accept_deadline: Option<i64>,
    ) -> Result<()> {
        instructions::escrow::create_gig_escrow::handler(
            ctx,
            gig_id,
            payment_amount,
            completion_duration_seconds,
            accept_deadline,
        )
    }

    pub fn cancel_gig(ctx: Context<CancelGig>) -> Result<()> {
        instructions::escrow::cancel_gig::handler(ctx)
    }

    pub fn accept_gig(ctx: Context<AcceptGig>) -> Result<()> {
        instructions::escrow::accept_gig::handler(ctx)
    }

    pub fn submit_proof(ctx: Context<SubmitProof>) -> Result<()> {
        instructions::escrow::submit_proof::handler(ctx)
    }

    pub fn approve_completion(ctx: Context<ApproveCompletion>) -> Result<()> {
        instructions::escrow::approve_completion::handler(ctx)
    }

    pub fn refund_expired(ctx: Context<RefundExpired>) -> Result<()> {
        instructions::escrow::refund_expired::handler(ctx)
    }

    // ==================== DISPUTE HANDLING ====================
    
    pub fn dispute_gig(
        ctx: Context<DisputeGig>,
        reason: String,
    ) -> Result<()> {
        instructions::dispute::dispute_gig::handler(ctx, reason)
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        winner: DisputeWinner,
    ) -> Result<()> {
        instructions::dispute::resolve_dispute::handler(ctx, winner)
    }
}
