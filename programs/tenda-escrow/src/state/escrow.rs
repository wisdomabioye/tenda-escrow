use anchor_lang::prelude::*;

#[account]
pub struct GigEscrow {
    /// Unique gig identifier (UUID from backend)
    pub gig_id: String,

    /// Poster wallet address
    pub poster: Pubkey,

    /// Worker wallet address (None until accepted)
    pub worker: Option<Pubkey>,

    // ==================== PAYMENT DETAILS ====================

    /// Gig payment in lamports
    pub payment_amount: u64,

    /// Platform fee in lamports
    pub platform_fee: u64,

    /// Total locked in escrow (payment + fee)
    pub total_locked: u64,

    // ==================== DEADLINES ====================

    /// Optional hard cutoff for worker acceptance (Unix timestamp).
    /// None means the gig is indefinitely open until poster cancels.
    pub accept_deadline: Option<i64>,

    /// How long (in seconds) the worker has to complete after accepting.
    pub completion_duration_seconds: u64,

    /// Computed at acceptance: accepted_at + completion_duration_seconds.
    /// None until a worker accepts the gig.
    pub completion_deadline: Option<i64>,

    // ==================== TIMESTAMPS ====================

    /// When gig escrow was created (gig published)
    pub created_at: i64,

    /// When worker accepted gig
    pub accepted_at: Option<i64>,

    /// When proof was submitted
    pub submitted_at: Option<i64>,

    /// When payment was released
    pub completed_at: Option<i64>,

    // ==================== STATUS ====================

    /// Current gig status
    pub status: GigStatus,

    /// PDA bump seed
    pub bump: u8,
}

impl GigEscrow {
    pub const LEN: usize = 8 +          // discriminator
        (4 + 32) +                      // gig_id (String, max 32 chars — UUID without hyphens)
        32 +                            // poster
        (1 + 32) +                      // worker (Option<Pubkey>)
        8 +                             // payment_amount
        8 +                             // platform_fee
        8 +                             // total_locked
        (1 + 8) +                       // accept_deadline (Option<i64>)
        8 +                             // completion_duration_seconds
        (1 + 8) +                       // completion_deadline (Option<i64>)
        8 +                             // created_at
        (1 + 8) +                       // accepted_at (Option<i64>)
        (1 + 8) +                       // submitted_at (Option<i64>)
        (1 + 8) +                       // completed_at (Option<i64>)
        1 +                             // status (enum as u8)
        1;                              // bump

    /// Whether this gig can be refunded due to expiry.
    /// - Open gig: accept_deadline must be set and passed
    /// - Accepted gig: completion_deadline must be set and past grace period
    pub fn is_expired(&self, current_time: i64, grace_period: i64) -> bool {
        match self.status {
            GigStatus::Open => {
                match self.accept_deadline {
                    Some(deadline) => current_time > deadline,
                    None => false, // indefinitely open — poster must cancel manually
                }
            }
            GigStatus::Accepted => {
                match self.completion_deadline {
                    Some(deadline) => current_time > deadline + grace_period,
                    None => false,
                }
            }
            _ => false,
        }
    }

    /// Whether the submission window is still open (within grace period)
    pub fn is_in_grace_period(&self, current_time: i64, grace_period: i64) -> bool {
        match self.completion_deadline {
            Some(deadline) => current_time > deadline && current_time <= deadline + grace_period,
            None => false,
        }
    }

    pub fn is_poster(&self, caller: &Pubkey) -> bool {
        &self.poster == caller
    }

    pub fn is_worker(&self, caller: &Pubkey) -> bool {
        match &self.worker {
            Some(worker) => worker == caller,
            None => false,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GigStatus {
    /// Gig posted and funded, awaiting worker
    Open,

    /// Worker accepted, task in progress
    Accepted,

    /// Worker submitted proof, awaiting poster approval
    Submitted,

    /// Poster approved, payment released
    Completed,

    /// Either party opened dispute
    Disputed,

    /// Dispute resolved by admin
    Resolved,

    /// Poster cancelled before acceptance (full refund)
    Cancelled,

    /// Deadline passed without completion (full refund to poster)
    Expired,
}

impl GigStatus {
    pub fn can_cancel(&self) -> bool {
        matches!(self, GigStatus::Open)
    }

    pub fn can_accept(&self) -> bool {
        matches!(self, GigStatus::Open)
    }

    pub fn can_submit(&self) -> bool {
        matches!(self, GigStatus::Accepted)
    }

    pub fn can_approve(&self) -> bool {
        matches!(self, GigStatus::Submitted)
    }

    pub fn can_dispute(&self) -> bool {
        matches!(self, GigStatus::Accepted | GigStatus::Submitted)
    }

    pub fn can_refund(&self) -> bool {
        matches!(self, GigStatus::Open | GigStatus::Accepted)
    }
}
