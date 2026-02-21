use anchor_lang::prelude::*;

// ==================== PLATFORM EVENTS ====================

#[event]
pub struct PlatformInitialized {
    pub admin: Pubkey,
    pub platform_fee_bps: u16,
    pub grace_period_seconds: i64,
    pub timestamp: i64,
}

// ==================== USER EVENTS ====================

#[event]
pub struct UserAccountCreated {
    pub wallet: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct GasSubsidyAirdropped {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct EarningsWithdrawn {
    pub user: Pubkey,
    pub amount: u64,
    pub remaining_balance: u64,
    pub timestamp: i64,
}

// ==================== ESCROW EVENTS ====================

#[event]
pub struct GigCreated {
    pub gig_id: String,
    pub poster: Pubkey,
    pub payment_amount: u64,
    pub platform_fee: u64,
    pub completion_duration_seconds: u64,
    pub accept_deadline: Option<i64>,
    pub timestamp: i64,
}

#[event]
pub struct GigCancelled {
    pub gig_id: String,
    pub poster: Pubkey,
    pub refund_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct GigAccepted {
    pub gig_id: String,
    pub poster: Pubkey,
    pub worker: Pubkey,
    pub completion_deadline: i64,
    pub timestamp: i64,
}

#[event]
pub struct ProofSubmitted {
    pub gig_id: String,
    pub worker: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct GigCompleted {
    pub gig_id: String,
    pub poster: Pubkey,
    pub worker: Pubkey,
    pub payment_amount: u64,
    pub platform_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct GigExpired {
    pub gig_id: String,
    pub poster: Pubkey,
    pub refund_amount: u64,
    pub timestamp: i64,
}

// ==================== DISPUTE EVENTS ====================

#[event]
pub struct DisputeOpened {
    pub gig_id: String,
    pub initiator: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

#[event]
pub struct DisputeResolved {
    pub gig_id: String,
    pub winner: String, // "Poster", "Worker", or "Split"
    pub poster_payout: u64,
    pub worker_payout: u64,
    pub platform_fee: u64,
    pub timestamp: i64,
}
