use anchor_lang::prelude::*;

#[account]
pub struct UserAccount {
    /// User wallet address
    pub wallet: Pubkey,
    
    /// Locked airdrop SOL (unlocks after 1 completed gig)
    pub airdrop_sol: u64,
    
    /// Lifetime earnings tracker (paid directly to wallet on gig completion — not held here)
    pub earned_sol: u64,
    
    /// Total completed gigs
    pub completed_gigs: u32,
    
    /// Phone verification status
    pub phone_verified: bool,
    
    /// Account creation timestamp
    pub created_at: i64,
}

impl UserAccount {
    pub const LEN: usize = 8 + // discriminator
        32 +    // wallet
        8 +     // airdrop_sol
        8 +     // earned_sol
        4 +     // completed_gigs
        1 +     // phone_verified
        8;      // created_at

    /// Calculate total withdrawable balance.
    /// earned_sol is tracking-only (paid directly to wallet on approval).
    /// Only airdrop_sol is held in this PDA and withdrawable here.
    pub fn withdrawable_balance(&self) -> u64 {
        if self.completed_gigs >= 1 {
            // Airdrop unlocked after 1 completed gig
            self.airdrop_sol
        } else {
            0
        }
    }

    /// Check if user has completed at least one gig
    pub fn has_completed_gig(&self) -> bool {
        self.completed_gigs >= 1
    }

    /// Increment completed gigs counter
    pub fn increment_completed_gigs(&mut self) {
        self.completed_gigs = self.completed_gigs.saturating_add(1);
    }

    /// Add earnings to user account
    pub fn add_earnings(&mut self, amount: u64) -> Result<()> {
        self.earned_sol = self.earned_sol
            .checked_add(amount)
            .ok_or(error!(crate::errors::TendaError::ArithmeticOverflow))?;
        Ok(())
    }

    /// Deduct from airdrop_sol (the only lamports held in this PDA).
    /// earned_sol is tracking-only and is never deducted here.
    pub fn deduct_withdrawal(&mut self, amount: u64) -> Result<()> {
        self.airdrop_sol = self.airdrop_sol
            .checked_sub(amount)
            .ok_or(error!(crate::errors::TendaError::ArithmeticUnderflow))?;
        Ok(())
    }
}
