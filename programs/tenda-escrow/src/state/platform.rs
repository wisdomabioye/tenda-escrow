use anchor_lang::prelude::*;

#[account]
pub struct PlatformState {
    /// Platform admin wallet
    pub admin: Pubkey,

    /// Standard platform fee in basis points (e.g., 250 = 2.5%)
    pub platform_fee_bps: u16,

    /// Reduced fee for Seeker device users in basis points (e.g., 100 = 1%)
    pub seeker_fee_bps: u16,

    /// Platform treasury for collecting fees
    pub treasury: Pubkey,

    /// Total number of gigs created
    pub total_gigs: u64,

    /// Total volume processed in lamports
    pub total_volume: u64,

    /// Grace period after deadline in seconds (default: 86400 = 24h)
    pub grace_period_seconds: i64,
}

impl PlatformState {
    pub const LEN: usize = 8 + // discriminator
        32 +  // admin
        2 +   // platform_fee_bps
        2 +   // seeker_fee_bps
        32 +  // treasury
        8 +   // total_gigs
        8 +   // total_volume
        8;    // grace_period_seconds

    /// Returns the effective fee bps for a given poster.
    pub fn effective_fee_bps(&self, is_seeker: bool) -> u16 {
        if is_seeker { self.seeker_fee_bps } else { self.platform_fee_bps }
    }
}
