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
    
    // ==================== TIMESTAMPS ====================
    
    /// When gig was created
    pub created_at: i64,
    
    /// Work completion deadline (Unix timestamp)
    pub deadline: i64,
    
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
    pub const LEN: usize = 8 + // discriminator
        (4 + 36) + // gig_id (String with max 36 chars for UUID)
        32 +       // poster
        (1 + 32) + // worker (Option<Pubkey>)
        8 +        // payment_amount
        8 +        // platform_fee
        8 +        // total_locked
        8 +        // created_at
        8 +        // deadline
        (1 + 8) +  // accepted_at (Option<i64>)
        (1 + 8) +  // submitted_at (Option<i64>)
        (1 + 8) +  // completed_at (Option<i64>)
        1 +        // status (enum as u8)
        1;         // bump

    /// Check if gig has expired (deadline + grace period passed)
    pub fn is_expired(&self, current_time: i64, grace_period: i64) -> bool {
        current_time > self.deadline + grace_period
    }

    /// Check if within grace period
    pub fn is_in_grace_period(&self, current_time: i64, grace_period: i64) -> bool {
        current_time > self.deadline && current_time <= self.deadline + grace_period
    }

    /// Check if caller is poster
    pub fn is_poster(&self, caller: &Pubkey) -> bool {
        &self.poster == caller
    }

    /// Check if caller is worker
    pub fn is_worker(&self, caller: &Pubkey) -> bool {
        match &self.worker {
            Some(worker) => worker == caller,
            None => false,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum GigStatus {
    /// Gig posted, awaiting worker
    Open,
    
    /// Worker accepted, task in progress
    Accepted,
    
    /// Worker submitted proof, awaiting approval
    Submitted,
    
    /// Poster approved, payment released
    Completed,
    
    /// Either party opened dispute
    Disputed,
    
    /// Poster cancelled before acceptance
    Cancelled,
    
    /// Gig timed out, refunded
    Expired,
}

impl GigStatus {
    /// Check if gig can be cancelled
    pub fn can_cancel(&self) -> bool {
        matches!(self, GigStatus::Open)
    }

    /// Check if gig can be accepted
    pub fn can_accept(&self) -> bool {
        matches!(self, GigStatus::Open)
    }

    /// Check if proof can be submitted
    pub fn can_submit(&self) -> bool {
        matches!(self, GigStatus::Accepted)
    }

    /// Check if gig can be approved
    pub fn can_approve(&self) -> bool {
        matches!(self, GigStatus::Submitted)
    }

    /// Check if gig can be disputed
    pub fn can_dispute(&self) -> bool {
        matches!(self, GigStatus::Accepted | GigStatus::Submitted)
    }

    /// Check if gig can be refunded (expired)
    pub fn can_refund(&self) -> bool {
        matches!(self, GigStatus::Open | GigStatus::Accepted)
    }
}
